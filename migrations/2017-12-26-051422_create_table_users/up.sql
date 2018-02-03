-- Your SQL goes here
CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    last_msg TIMESTAMP NOT NULL,
    msg_activity INTEGER[] NOT NULL,
    rep INTEGER NOT NULL,
    last_rep TIMESTAMP,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    address TEXT,
    lastfm TEXT
)