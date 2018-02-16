-- Your SQL goes here
CREATE TABLE cache_users (
    id BIGINT PRIMARY KEY,
    avatar TEXT NOT NULL,
    user_name TEXT NOT NULL,
    discriminator INTEGER NOT NULL
)
