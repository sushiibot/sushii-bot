-- Your SQL goes here

CREATE TABLE guilds (
    id bigint PRIMARY KEY,
    name TEXT NOT NULL,
    join_msg TEXT,
    leave_msg TEXT,
    invite_guard BOOLEAN,
    log_msg bigint,
    log_mod bigint
)