-- Your SQL goes here
CREATE TABLE member_events (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    event_name TEXT NOT NULL,
    event_time TIMESTAMP NOT NULL
)