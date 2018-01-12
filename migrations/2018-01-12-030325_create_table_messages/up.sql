-- Your SQL goes here
CREATE TABLE messages (
    id BIGINT PRIMARY KEY,
    author BIGINT NOT NULL,
    tag TEXT NOT NULL,
    channel BIGINT NOT NULL,
    guild BIGINT,
    created TIMESTAMP NOT NULL,
    content TEXT NOT NULL
)