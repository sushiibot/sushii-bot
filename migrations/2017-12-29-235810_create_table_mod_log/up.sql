-- Your SQL goes here
CREATE TABLE mod_log (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL,
    guild_id BIGINT NOT NULL,
    executor_id BIGINT,
    user_id BIGINT NOT NULL,
    user_tag TEXT NOT NULL,
    action TEXT NOT NULL,
    reason TEXT,
    action_time TIMESTAMP NOT NULL,
    msg_id BIGINT,
    pending BOOLEAN NOT NULL
)