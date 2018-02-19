-- Your SQL goes here
CREATE TABLE stats (
    stat_name TEXT PRIMARY KEY,
    count BIGINT NOT NULL,
    category TEXT NOT NULL
)