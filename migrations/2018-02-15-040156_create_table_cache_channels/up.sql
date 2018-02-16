-- Your SQL goes here
CREATE TABLE cache_channels (
    id BIGINT PRIMARY KEY,
    category_id BIGINT,
    guild_id BIGINT REFERENCES cache_guilds(id),
    kind TEXT NOT NULL,
    channel_name TEXT NOT NULL,
    position BIGINT NOT NULL,
    topic TEXT,
    nsfw BOOLEAN NOT NULL
)
