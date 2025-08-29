CREATE TABLE IF NOT EXISTS clipboard
(
    id integer PRIMARY KEY,
    content blob NOT NULL UNIQUE,
    last_updated integer NOT NULL
) STRICT ;
