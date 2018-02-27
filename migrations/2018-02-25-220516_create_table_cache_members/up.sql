-- Your SQL goes here
CREATE TABLE cache_members (
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    PRIMARY KEY(user_id, guild_id)
)