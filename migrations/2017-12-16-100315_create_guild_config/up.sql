-- Your SQL goes here
CREATE TABLE guilds (
    id BIGINT PRIMARY KEY,
    name TEXT,
    join_msg TEXT,
    join_react TEXT,
    leave_msg TEXT,
    invite_guard BOOLEAN,
    log_msg BIGINT,
    log_mod BIGINT,
    prefix TEXT
)
