use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use gel_errors::SHOULD_RETRY;
use gel_protocol::model::Uuid;

const SEED: &str = "delete Cell; \
                    insert Cell { name := 'a', color := 'black' }; \
                    insert Cell { name := 'b', color := 'white' };";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = gel_tokio::create_client().await?;
    client.execute(SEED, &()).await?;

    let attempts = Arc::new(AtomicU32::new(0));

    // Sibling client used to run the conflicting transaction T2 without
    // retries — we want any error from T2 to be visible and not muddy the
    // outer attempt counter.
    let conflicting_client = client.with_retry_options(
        gel_tokio::RetryOptions::default().new(1, |_| Duration::from_millis(0)),
    );

    let result = client
        .with_retry_options(
            gel_tokio::RetryOptions::default().new(10, |_| Duration::from_millis(50)),
        )
        .transaction({
            let attempts = attempts.clone();
            move |mut tx| {
                let attempts = attempts.clone();
                let conflicting_client = conflicting_client.clone();
                async move {
                    let n = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    println!("[tx] attempt {n}");

                    // T1: predicate read of black.
                    let _: Vec<Uuid> = tx
                        .query("select Cell.id filter Cell.color = 'black'", &())
                        .await?;

                    // T1: flip 'a' black->white. Body should complete cleanly.
                    tx.execute(
                        "update Cell filter .name = 'a' set { color := 'white' }",
                        &(),
                    )
                    .await?;

                    // T2: sibling transaction (predicate-read white + write 'b' black).
                    // Run AFTER T1's UPDATE so the rw-cycle isn't visible to PG until
                    // T1 reaches COMMIT.
                    conflicting_client
                        .transaction(|mut tx2| async move {
                            let _: Vec<Uuid> = tx2
                                .query("select Cell.id filter Cell.color = 'white'", &())
                                .await?;
                            tx2.execute(
                                "update Cell filter .name = 'b' set { color := 'black' }",
                                &(),
                            )
                            .await.unwrap();
                            Ok::<_, gel_tokio::Error>(())
                        })
                        .await?;

                    println!("[tx] closure returning Ok");
                    Ok::<_, gel_tokio::Error>(())
                }
            }
        })
        .await;

    let n = attempts.load(Ordering::SeqCst);
    println!();
    println!("=== summary ===");
    println!("[main] transaction result: {result:?}");
    println!("[main] closure invocations: {n}");

    if let Err(e) = &result {
        let retryable = e.has_tag(SHOULD_RETRY);
        println!("[main] surfaced error has_tag(SHOULD_RETRY) = {retryable}");
        if retryable && n == 1 {
            println!(
                "BUG CONFIRMED: closure invoked exactly once, but the surfaced \
                 error is SHOULD_RETRY-tagged. gel-tokio's commit path did not \
                 consult the retry rule."
            );
        }
    }

    Ok(())
}
