-- Your SQL goes here
CREATE TABLE guilds (
    id bigint PRIMARY KEY,
    name TEXT,
    join_msg TEXT,
    join_react TEXT,
    leave_msg TEXT,
    invite_guard BOOLEAN,
    log_msg bigint,
    log_mod bigint,
    prefix TEXT
)
