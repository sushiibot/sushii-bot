use serenity::framework::standard::CommandError;
use serenity::utils::parse_channel;
use utils::config::*;

command!(modlog(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(ctx, &pool, guild_id.0));

        config.log_mod = Some(channel as i64);

        update_config(ctx, &pool, &config);

        let s = get_msg!("info/mod_log_set", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(msglog(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(ctx, &pool, guild_id.0));

        config.log_msg = Some(channel as i64);

        update_config(ctx, &pool, &config);

        let s = get_msg!("info/message_log_set", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(memberlog(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(ctx, &pool, guild_id.0));

        config.log_member = Some(channel as i64);

        update_config(ctx, &pool, &config);

        let s = get_msg!("info/member_log_set", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});
