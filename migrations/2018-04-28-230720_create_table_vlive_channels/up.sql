CREATE TABLE vlive_channels (
    channel_seq INTEGER NOT NULL,
    channel_code TEXT NOT NULL,
    channel_name TEXT NOT NULL,
    guild_id BIGINT NOT NULL,
    discord_channel BIGINT NOT NULL,
    mention_role BIGINT,
    PRIMARY KEY (channel_seq, discord_channel)
)
