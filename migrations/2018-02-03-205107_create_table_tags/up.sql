-- Your SQL goes here
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    tag_name TEXT NOT NULL,
    content TEXT NOT NULL,
    count INTEGER NOT NULL,
    created TIMESTAMP NOT NULL
)
