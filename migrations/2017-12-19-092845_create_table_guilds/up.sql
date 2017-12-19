-- Your SQL goes here
CREATE TABLE guilds (
    id BIGINT PRIMARY KEY,
    name TEXT,
    join_msg TEXT,
    join_react TEXT,
    leave_msg TEXT,
    msg_channel BIGINT,
    invite_guard BOOLEAN,
    log_msg BIGINT,
    log_mod BIGINT,
    log_member BIGINT,
    mute_role BIGINT,
    prefix TEXT
)
