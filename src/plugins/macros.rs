/// macro to run multiple plugins in a loop
macro_rules! exec_on_message {
    ( [$ctx:expr, $msg:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_message($ctx, &pool, $msg);
        )*
    }}
}

macro_rules! exec_on_message_update {
    ( [$ctx:expr, $event:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_message_update($ctx, &pool, $event);
        )*
    }}
}

macro_rules! exec_on_message_delete {
    ( [$ctx:expr, $channel_id:expr, $msg_id:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_message_delete($ctx, &pool, $channel_id, $msg_id);
        )*
    }}
}

macro_rules! exec_on_reaction_add {
    ( [$ctx:expr, $reaction:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_reaction_add($ctx, &pool, $reaction);
        )*
    }}
}

/*
macro_rules! exec_on_reaction_remove {
    ( [$ctx:expr, $reaction:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_reaction_remove($ctx, &pool, $reaction);
        )*
    }}
}
*/

macro_rules! exec_on_ready {
    ( [$ctx:expr, $ready:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_ready($ctx, $ready);
        )*
    }
}

macro_rules! exec_on_presence_update {
    ( [$ctx:expr, $presenceupdateevent:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_presence_update($ctx, $presenceupdateevent);
        )*
    }
}

macro_rules! exec_on_guild_member_addition {
    ( [$ctx:expr, $GuildId:expr, $member:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_guild_member_addition($ctx, &pool, $GuildId, $member);
        )*
    }}
}


macro_rules! exec_on_guild_member_removal {
    ( [$ctx:expr, $GuildId:expr, $user:expr, $member:expr], $( $plugin:ident ),* ) => {{
        use utils::config::get_pool;
        let pool = get_pool(&$ctx);

        $(
            $plugin::on_guild_member_removal($ctx, &pool, $GuildId, $user, $member);
        )*
    }}
}

macro_rules! exec_on_guild_ban_addition {
    ( [$ctx:expr, $GuildId:expr, $user:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_ban_addition($ctx, $GuildId, $user);
        )*
    }
}

macro_rules! exec_on_guild_ban_removal {
    ( [$ctx:expr, $GuildId:expr, $user:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_ban_removal($ctx, $GuildId, $user);
        )*
    }
}

macro_rules! exec_on_guild_member_update {
    ( [$ctx:expr, $prev_member:expr, $curr_member:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_member_update($ctx, $prev_member, $curr_member);
        )*
    }
}

macro_rules! exec_on_guild_create {
    ( [$ctx:expr, $guild:expr, $if_joined:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_create($ctx, $guild, $if_joined);
        )*
    }
}

macro_rules! exec_on_guild_members_chunk {
    ( [$ctx:expr, $guild_id:expr, $members:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_members_chunk($ctx, $guild_id, $members);
        )*
    }
}

macro_rules! exec_on_guild_update {
    ( [$ctx:expr, $guild:expr, $partial_guild:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_update($ctx, $guild, $partial_guild);
        )*
    }
}


