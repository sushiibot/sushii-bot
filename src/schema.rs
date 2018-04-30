table! {
    cache_channels (id) {
        id -> Int8,
        category_id -> Nullable<Int8>,
        guild_id -> Nullable<Int8>,
        kind -> Text,
        channel_name -> Text,
        position -> Int8,
        topic -> Nullable<Text>,
        nsfw -> Bool,
    }
}

table! {
    cache_guilds (id) {
        id -> Int8,
        guild_name -> Text,
        icon -> Nullable<Text>,
        member_count -> Int8,
        owner_id -> Int8,
    }
}

table! {
    cache_members (user_id, guild_id) {
        user_id -> Int8,
        guild_id -> Int8,
    }
}

table! {
    cache_users (id) {
        id -> Int8,
        avatar -> Text,
        user_name -> Text,
        discriminator -> Int4,
    }
}

table! {
    events (name) {
        name -> Text,
        count -> Int8,
    }
}

table! {
    galleries (id) {
        id -> Int4,
        watch_channel -> Int8,
        webhook_url -> Text,
        guild_id -> Int8,
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
        disabled_channels -> Nullable<Array<Int8>>,
    }
}

table! {
    levels (user_id, guild_id) {
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
    member_events (id) {
        id -> Int4,
        guild_id -> Int8,
        user_id -> Int8,
        event_name -> Text,
        event_time -> Timestamp,
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
    mutes (id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
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
    stats (stat_name) {
        stat_name -> Text,
        count -> Int8,
        category -> Text,
    }
}

table! {
    tags (id) {
        id -> Int4,
        owner_id -> Int8,
        guild_id -> Int8,
        tag_name -> Text,
        content -> Text,
        count -> Int4,
        created -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int8,
        last_msg -> Timestamp,
        msg_activity -> Array<Int4>,
        rep -> Int4,
        last_rep -> Nullable<Timestamp>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        address -> Nullable<Text>,
        lastfm -> Nullable<Text>,
        is_patron -> Bool,
        fishies -> Int8,
        last_fishies -> Nullable<Timestamp>,
        patron_emoji -> Nullable<Text>,
        profile_options -> Nullable<Jsonb>,
    }
}

table! {
    vlive_channels (channel_seq, discord_channel) {
        channel_seq -> Int4,
        channel_code -> Text,
        channel_name -> Text,
        guild_id -> Int8,
        discord_channel -> Int8,
    }
}

table! {
    vlive_videos (channel_seq, video_seq) {
        channel_seq -> Int4,
        video_seq -> Int4,
    }
}

joinable!(cache_channels -> cache_guilds (guild_id));

allow_tables_to_appear_in_same_query!(
    cache_channels,
    cache_guilds,
    cache_members,
    cache_users,
    events,
    galleries,
    guilds,
    levels,
    member_events,
    messages,
    mod_log,
    mutes,
    notifications,
    reminders,
    stats,
    tags,
    users,
    vlive_channels,
    vlive_videos,
);
