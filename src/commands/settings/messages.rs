use serenity::framework::standard::CommandError;
use serenity::utils::parse_channel;
use utils::config::get_pool;

command!(joinmsg(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let message = args.full().to_owned();

    if let Some(guild_id) = msg.guild_id() {
        let guild_id = guild_id.0;
        let config = check_res_msg!(pool.get_guild_config(guild_id));

        // no message given, just print out the current message
        if args.len() == 0 {
            if let Some(current_message) = config.join_msg {
                let s = get_msg!("info/join_message_current", current_message);
                let _ = msg.channel_id.say(&s);
            } else {
                let _ = msg.channel_id.say(get_msg!("info/join_message_none"));
            }
        } else {
            let mut config = config;

            if message == "off" {
                config.join_msg = None;

                let _ = msg.channel_id.say(get_msg!("info/join_message_disable"));
            } else {
                config.join_msg = Some(message.to_owned());

                let s = get_msg!("info/join_message_set", message);
                let _ = msg.channel_id.say(&s);
            }

            pool.save_guild_config(&config);
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(leavemsg(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let message = args.full().to_owned();

    if let Some(guild_id) = msg.guild_id() {
        let guild_id = guild_id.0;
        let config = check_res_msg!(pool.get_guild_config(guild_id));

        // no message given, just print out the current message
        if args.len() == 0 {
            if let Some(current_message) = config.leave_msg {
                let s = get_msg!("info/leave_message_current", current_message);
                let _ = msg.channel_id.say(&s);
            } else {
                let _ = msg.channel_id.say(get_msg!("info/leave_message_none"));
            }
        } else {
            let mut config = config;

            if message == "off" {
                config.leave_msg = None;

                let _ = msg.channel_id.say(get_msg!("info/leave_message_disable"));
            } else {
                config.leave_msg = Some(message.to_owned());

                let s = get_msg!("info/leave_message_set", message);
                let _ = msg.channel_id.say(&s);
            }

            pool.save_guild_config(&config);
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(msg_channel(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(&ctx);

        let mut config = check_res_msg!(pool.get_guild_config(guild_id.0));

        config.msg_channel = Some(channel as i64);

        pool.save_guild_config(&config);

        let s = get_msg!("info/msg_channel_set", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});
