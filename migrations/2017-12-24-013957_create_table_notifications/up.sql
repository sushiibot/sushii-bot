-- Your SQL goes here
CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    keyword TEXT NOT NULL
)