use serenity::framework::standard::CommandError;
use serenity::utils::parse_channel;
use utils::config::*;
use std::fmt::Write;

command!(disable_channel(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(&ctx, &pool, guild_id.0));
        
        config.disabled_channels = if let Some(mut disabled_channels) = config.disabled_channels {
            // check if already disabled
            if disabled_channels.contains(&(channel as i64)) {
                return Err(CommandError::from(get_msg!("error/channel_already_disabled")));
            }

            disabled_channels.push(channel as i64);

            Some(disabled_channels)
        } else {
            Some(vec![channel as i64])
        };

        update_config(&ctx, &pool, &config);

        let s = get_msg!("info/channel_disabled", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(enable_channel(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(&ctx, &pool, guild_id.0));
        
        config.disabled_channels = if let Some(mut disabled_channels) = config.disabled_channels {
            if let Some(index) = disabled_channels.iter().position(|x| *x == channel as i64) {
                disabled_channels.remove(index);
            } else {
                return Err(CommandError::from(get_msg!("error/channel_not_disabled")));
            }
            Some(disabled_channels)
        } else {
            return Err(CommandError::from(get_msg!("error/channel_not_disabled")));
        };

        update_config(&ctx, &pool, &config);

        let s = get_msg!("info/channel_enabled", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(list_disabled_channels(ctx, msg, _args) {
    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(&ctx, &pool, guild_id.0));
        
        if let Some(channels) = config.disabled_channels {
            if channels.is_empty() {
                return Err(CommandError::from(get_msg!("error/channel_none_disabled")));
            }

            let mut s = String::new();
            for chan in &channels {
                let _ = write!(s, "<#{}>\n", chan);
            }

            let _ = msg.channel_id.send_message(|m|
                m.embed(|e| e
                    .author(|a| a
                        .name("Disabled Channels")
                    )
                    .color(0x2ecc71)
                    .description(&s)
                )
            );
        } else {
            return Err(CommandError::from(get_msg!("error/channel_none_disabled")));
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});
