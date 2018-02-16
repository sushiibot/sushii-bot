-- Your SQL goes here
CREATE TABLE galleries (
    id SERIAL PRIMARY KEY,
    watch_channel BIGINT NOT NULL,
    webhook_url TEXT NOT NULL,
    guild_id BIGINT NOT NULL
)
