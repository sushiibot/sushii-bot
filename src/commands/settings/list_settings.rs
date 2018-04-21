use serenity::framework::standard::CommandError;

use std::env;
use utils::config::get_pool;
use utils::config::get_config;
use utils::config::update_config;

command!(settings(ctx, msg, _args) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let pool = get_pool(ctx);
        
        let config = check_res_msg!(get_config(ctx, &pool, guild.id.0));

        let default_prefix = env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.");


        let msg_channel = if let Some(msg_channel) = config.msg_channel {
            format!("<#{}>", msg_channel)
        } else {
            "N/A".to_owned()
        };

        let role_channel = if let Some(role_channel) = config.role_channel {
            format!("<#{}>", role_channel)
        } else {
            "N/A".to_owned()
        };

        let log_msg = if let Some(log_msg) = config.log_msg {
            format!("<#{}>", log_msg)
        } else {
            "N/A".to_owned()
        };

        let log_mod = if let Some(log_mod) = config.log_mod {
            format!("<#{}>", log_mod)
        } else {
            "N/A".to_owned()
        };

        let log_member = if let Some(log_member) = config.log_member {
            format!("<#{}>", log_member)
        } else {
            "N/A".to_owned()
        };

        let mute_role = if let Some(mute_role) = config.mute_role {
            format!("<@&{}>", mute_role)
        } else {
            "N/A".to_owned()
        };

        let disabled_channels = if let Some(ref disabled_channels) = config.disabled_channels {
            disabled_channels.iter().map(|x| format!("<#{}>", x)).collect::<Vec<String>>().join(", ")
        } else {
            "N/A".to_owned()
        };


        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .author(|a| a
                    .name(&format!("{} - Guild Settings", guild.name))
                    .icon_url(&guild.icon_url().unwrap_or_else(|| "N/A".to_owned()))
                )
                .color(0x3498db)
                .field("join_msg", config.join_msg.unwrap_or_else(|| "N/A".to_owned()), true)
                .field("join_react", config.join_react.unwrap_or_else(|| "N/A".to_owned()), true)
                .field("leave_msg", config.leave_msg.unwrap_or_else(|| "N/A".to_owned()), true)
                .field("msg_channel", msg_channel, true)
                .field("role_channel", role_channel, true)
                .field("invite_guard", config.invite_guard.unwrap_or_else(|| false), true)
                .field("log_msg", log_msg, true)
                .field("log_mod", log_mod, true)
                .field("log_member", log_member, true)
                .field("mute_role", mute_role, true)
                .field("prefix", config.prefix.unwrap_or_else(|| default_prefix), true)
                .field("max_mention", config.max_mention, true)
                .field("disabled_channels", disabled_channels, false)
            )
        );
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});


command!(clear_setting(ctx, msg, args) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let pool = get_pool(ctx);
        
        let mut config = check_res_msg!(get_config(ctx, &pool, guild.id.0));

        let setting = match args.single::<String>() {
            Ok(val) => val,
            Err(_) => return Err(CommandError::from(get_msg!("error/clear_setting_not_given")))
        };

        match setting.as_ref() {
            "joinmsg" | "join_msg" => {
                config.join_msg = None
            },
            "joinreact" | "join_react" => {
                config.join_react = None
            },
            "leavemsg" | "leave_msg" => {
                config.leave_msg = None
            },
            "msgchannel" | "msg_channel" => {
                config.msg_channel = None
            },
            "rolechannel" | "role_channel" => {
                config.role_channel = None
            },
            "inviteguard" | "invite_guard" => {
                config.invite_guard = None
            },
            "msglog" | "log_msg" => {
                config.log_msg = None
            },
            "modlog" | "log_mod" => {
                config.log_mod = None
            },
            "memberlog" | "log_member" => {
                config.log_member = None
            },
            "muterole" | "mute_role" => {
                config.mute_role = None
            },
            "prefix" => {
                config.prefix = None
            },
            "maxmention" | "max_mention" => {
                return Err(CommandError::from(get_msg!("error/clear_setting_cannot_clear")))
            },
            "disabledchannels" | "disabled_channels" => {
                config.disabled_channels = None
            },
            _ => {
                return Err(CommandError::from(get_msg!("error/clear_setting_invalid")));
            }
        }

        update_config(ctx, &pool, &config);

        let _ = msg.channel_id.say(&get_msg!("info/setting_cleared", setting));
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")))
    }
});
