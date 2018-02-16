-- Your SQL goes here
CREATE TABLE cache_guilds (
    id BIGINT PRIMARY KEY,
    guild_name TEXT NOT NULL,
    icon TEXT,
    member_count BIGINT NOT NULL,
    owner_id BIGINT NOT NULL
)
