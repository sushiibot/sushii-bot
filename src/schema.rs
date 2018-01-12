table! {
    events (name) {
        name -> Text,
        count -> Int8,
    }
}

table! {
    guilds (id) {
        id -> Int8,
        name -> Nullable<Text>,
        join_msg -> Nullable<Text>,
        join_react -> Nullable<Text>,
        leave_msg -> Nullable<Text>,
        msg_channel -> Nullable<Int8>,
        role_channel -> Nullable<Int8>,
        role_config -> Nullable<Jsonb>,
        invite_guard -> Nullable<Bool>,
        log_msg -> Nullable<Int8>,
        log_mod -> Nullable<Int8>,
        log_member -> Nullable<Int8>,
        mute_role -> Nullable<Int8>,
        prefix -> Nullable<Text>,
        max_mention -> Int4,
    }
}

table! {
    levels (id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
        msg_all_time -> Int8,
        msg_month -> Int8,
        msg_week -> Int8,
        msg_day -> Int8,
        last_msg -> Timestamp,
    }
}

table! {
    messages (id) {
        id -> Int8,
        author -> Int8,
        tag -> Text,
        channel -> Int8,
        guild -> Nullable<Int8>,
        created -> Timestamp,
        content -> Text,
    }
}

table! {
    mod_log (id) {
        id -> Int4,
        case_id -> Int4,
        guild_id -> Int8,
        executor_id -> Nullable<Int8>,
        user_id -> Int8,
        user_tag -> Text,
        action -> Text,
        reason -> Nullable<Text>,
        action_time -> Timestamp,
        msg_id -> Nullable<Int8>,
        pending -> Bool,
    }
}

table! {
    notifications (id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
        keyword -> Text,
    }
}

table! {
    reminders (id) {
        id -> Int4,
        user_id -> Int8,
        description -> Text,
        time_set -> Timestamp,
        time_to_remind -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int8,
        last_msg -> Timestamp,
        msg_activity -> Array<Int4>,
        rep -> Int4,
        last_rep -> Nullable<Timestamp>,
    }
}

allow_tables_to_appear_in_same_query!(
    events,
    guilds,
    levels,
    messages,
    mod_log,
    notifications,
    reminders,
    users,
);
