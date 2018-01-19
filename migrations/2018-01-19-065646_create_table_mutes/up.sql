-- Your SQL goes here
CREATE TABLE mutes (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL
)
