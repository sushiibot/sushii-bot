use serenity::framework::standard::CommandError;
use serenity::utils::parse_channel;

use serde_json;

use std::env;
use database;
use utils::config::get_pool;

command!(prefix(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    // check for MANAGE_SERVER permissions

    if let Some(guild) = msg.guild() {
        let guild = guild.read().unwrap();

        let prefix = match args.single::<String>() {
            Ok(val) => val,
            Err(_) => {
                // no prefix argument, set the prefix
                match pool.get_prefix(guild.id.0) {
                    Some(prefix) => {
                        let _ = msg.channel_id.say(&format!("The prefix in this guild is set to: `{}`", prefix));
                        return Ok(());
                    },
                    None => {
                        let prefix = env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.");
                        let _ = msg.channel_id.say(format!("The prefix in this guild is set to: `{}`", prefix));
                        return Ok(());
                    }
                }
            },
        };

        let has_manage_guild = guild.member_permissions(msg.author.id).manage_guild();

        if has_manage_guild {
            let success = pool.set_prefix(guild.id.0, &prefix);

            if success {
                let _ = msg.channel_id.say(format!("The prefix for this server has been set to: `{}`", prefix));
            } else {
                let _ = msg.channel_id.say(format!("The prefix for this server is already: `{}`", prefix));
            }
        } else {
            return Err(CommandError("You need `MANAGE_GUILD` permissions to set the prefix.".to_owned()));
        }
        
    } else {
        // no guild found, probably in DMs
        let prefix = env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.");
        let _ = msg.channel_id.say(format!("The default prefix is set to `{}`", prefix));
    }
});

command!(joinmsg(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let message = args.full();

    if let Some(guild_id) = msg.guild_id() {
        let guild_id = guild_id.0;
        let config = pool.get_guild_config(guild_id);

        // no message given, just print out the current message
        if args.len() == 0 {
            if let Some(current_message) = config.join_msg {
                let s = format!("The current join message is: {}", current_message);
                let _ = msg.channel_id.say(&s);
            } else {
                let _ = msg.channel_id.say("There is no join message set.  \
                    You can set one with the placeholders <mention>, <username>, <server>.");
            }
        } else {
            let mut config = config;

            if message == "off" {
                config.join_msg = None;

                let _ = msg.channel_id.say("Join messages have been disabled.");
            } else {
                config.join_msg = Some(message.clone());

                let s = format!("The current join message has been set to: {}", message);
                let _ = msg.channel_id.say(&s);
            }

            pool.save_guild_config(&config);
        }
    } else {
        return Err(CommandError("No guild found.".to_owned()));
    }
});

command!(leavemsg(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let message = args.full();

    if let Some(guild_id) = msg.guild_id() {
        let guild_id = guild_id.0;
        let config = pool.get_guild_config(guild_id);

        // no message given, just print out the current message
        if args.len() == 0 {
            if let Some(current_message) = config.leave_msg {
                let s = format!("The current leave message is: {}", current_message);
                let _ = msg.channel_id.say(&s);
            } else {
                let _ = msg.channel_id.say("There is no leave message set.  \
                    You can set one with the placeholders <mention>, <username>, <server>.");
            }
        } else {
            let mut config = config;

            if message == "off" {
                config.leave_msg = None;

                let _ = msg.channel_id.say("Leave messages have been disabled.");
            } else {
                config.leave_msg = Some(message.clone());

                let s = format!("The current leave message has been set to: {}", message);
                let _ = msg.channel_id.say(&s);
            }

            pool.save_guild_config(&config);
        }
    } else {
        return Err(CommandError("No guild found.".to_owned()));
    }
});

command!(modlog(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError("No channel given.".to_owned())),
    };

    if channel == 0 {
        return Err(CommandError("Invalid channel.".to_owned()));
    }

    if let Some(guild_id) = msg.guild_id() {
        let mut data = ctx.data.lock();
        let pool = data.get_mut::<database::ConnectionPool>().unwrap();

        let mut config = pool.get_guild_config(guild_id.0);

        config.log_mod = Some(channel as i64);

        pool.save_guild_config(&config);

        let s = format!("The moderation log channel has been set to: <#{}>", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("No guild found.".to_owned()));
    }
});

command!(msglog(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError("No channel given.".to_owned())),
    };

    if channel == 0 {
        return Err(CommandError("Invalid channel.".to_owned()));
    }

    if let Some(guild_id) = msg.guild_id() {
        let mut data = ctx.data.lock();
        let pool = data.get_mut::<database::ConnectionPool>().unwrap();

        let mut config = pool.get_guild_config(guild_id.0);

        config.log_msg = Some(channel as i64);

        pool.save_guild_config(&config);

        let s = format!("The message log channel has been set to: <#{}>", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("No guild found.".to_owned()));
    }
});

command!(memberlog(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError("No channel given.".to_owned())),
    };

    if channel == 0 {
        return Err(CommandError("Invalid channel.".to_owned()));
    }

    if let Some(guild_id) = msg.guild_id() {
        let mut data = ctx.data.lock();
        let pool = data.get_mut::<database::ConnectionPool>().unwrap();

        let mut config = pool.get_guild_config(guild_id.0);

        config.log_member = Some(channel as i64);

        pool.save_guild_config(&config);

        let s = format!("The member log channel has been set to: <#{}>", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("No guild found.".to_owned()));
    }
});

command!(inviteguard(ctx, msg, args) {
    let status_str = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from("No option given.  Use `enable` or `disable`")),
    };

    let mut status;
    let mut s;

    if status_str == "enable" {
        status = true;
        s = "Invite guard has been enabled.";
    } else if status_str == "disable" {
        status = false;
        s = "Invite guard has been disabled.";
    } else {
        return Err(CommandError::from("Invalid option.  Use `enable` or `disable`"));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(&ctx);

        let mut config = pool.get_guild_config(guild_id.0);

        config.invite_guard = Some(status);

        pool.save_guild_config(&config);

        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError("No guild found.".to_owned()));
    }
});

command!(set_roles(ctx, msg, args) {
    let mut raw_json = args.full();

    if raw_json.is_empty() && msg.attachments.len() > 0 {
        let bytes = match msg.attachments[0].download() {
            Ok(content) => content,
            Err(e) => return Err(CommandError::from(e)),
        };

        raw_json = match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(e) => return Err(CommandError::from(e)),
        };
    } else if raw_json.is_empty() && msg.attachments.is_empty() {
        // no message or attachment 
        return Err(CommandError::from("No configuration provided."));
    }

    let role_config: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(&raw_json) {
        Ok(val) => val,
        Err(e) => return Err(CommandError::from(e)),
    };

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(&ctx);

        let mut config = pool.get_guild_config(guild_id.0);        
        config.role_config = Some(serde_json::Value::from(role_config));

        pool.save_guild_config(&config);

        let _ = msg.channel_id.say("Updated the guild role config.");
    } else {
        return Err(CommandError::from("No guild."));
    }
});