-- Your SQL goes here
CREATE TABLE reminders (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    description TEXT NOT NULL,
    time_set TIMESTAMP NOT NULL,
    time_to_remind TIMESTAMP NOT NULL
)