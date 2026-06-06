CREATE MIGRATION m1gd6fqk2q54fsasv47ahflla3c5v7jf4nkjvwl5o2gmnkd7q3tthq
    ONTO initial
{
  CREATE FUTURE no_linkful_computed_splats;
  CREATE TYPE default::Cell {
      CREATE REQUIRED PROPERTY color: std::str;
      CREATE REQUIRED PROPERTY name: std::str {
          CREATE CONSTRAINT std::exclusive;
      };
  };
};
