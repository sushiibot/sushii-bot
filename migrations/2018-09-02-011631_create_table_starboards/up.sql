CREATE TABLE starboards (
    guild_id BIGINT PRIMARY KEY,
    channel BIGINT NOT NULL,
    emoji TEXT NOT NULL DEFAULT '🍣',
    emoji_id BIGINT,
    minimum INT NOT NULL DEFAULT 1
)
