CREATE TABLE starboarded (
    orig_message_id BIGINT PRIMARY KEY,
    message_id BIGINT NOT NULL,
    author_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    created TIMESTAMP NOT NULL,
    count BIGINT NOT NULL
)
