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
    ( [$ctx:expr, $msg_update:expr], $( $plugin:ident ),* ) => {{
            use utils::config::get_pool;
            let pool = get_pool(&$ctx);
    
            $(
                $plugin::on_message_update($ctx, &pool, $msg_update);
            )*
        }}
}


macro_rules! exec_on_ready {
    ( [$ctx:expr, $ready:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_ready($ctx, $ready);
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
