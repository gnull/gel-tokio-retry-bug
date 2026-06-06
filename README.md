1. Init the Gel project.
2. Run rust code with `cargo run`.

The output confirming no retries have fired:

> [tx] attempt 1
> [tx] closure returning Ok
>
> === summary ===
> [main] transaction result: Err(Error(Inner { code: 84082945, messages: ["could not serialize access due to read/write dependencies among transactions"], error: None, headers: {257: b"edb.errors.TransactionSerializationError: could not serialize access due to read/write dependencies among transactions\n"}, annotations: {}, fields: {} }))
> [main] closure invocations: 1
> [main] surfaced error has_tag(SHOULD_RETRY) = true
> BUG CONFIRMED: closure invoked exactly once, but the surfaced error is SHOULD_RETRY-tagged. gel-tokio's commit path did not consult the retry rule.
