/// macro to run multiple plugins in a loop
macro_rules! exec_on_message {
    ( [$ctx:expr, $msg:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_message($ctx, $msg);
        )*
    }
}


macro_rules! exec_on_ready {
    ( [$ctx:expr, $ready:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_ready($ctx, $ready);
        )*
    }
}

macro_rules! exec_on_guild_member_addition {
    ( [$ctx:expr, $GuildId:expr, $member:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_member_addition($ctx, $GuildId, $member);
        )*
    }
}


macro_rules! exec_on_guild_member_removal {
    ( [$ctx:expr, $GuildId:expr, $user:expr, $member:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_member_removal($ctx, $GuildId, $user, $member);
        )*
    }
}

macro_rules! exec_on_guild_ban_addition {
    ( [$ctx:expr, $GuildId:expr, $user:expr], $( $plugin:ident ),* ) => {
        $(
            $plugin::on_guild_ban_addition($ctx, $GuildId, $user);
        )*
    }
}
