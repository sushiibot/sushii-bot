use serenity::framework::standard::CommandError;

use std::env;
use utils::config::get_pool;

command!(settings(ctx, msg, _args) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let pool = get_pool(&ctx);
        
        let config = check_res_msg!(pool.get_guild_config(guild.id.0));

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
                    .icon_url(&guild.icon_url().unwrap_or("N/A".to_owned()))
                )
                .color(0x3498db)
                .field("join_msg", config.join_msg.unwrap_or("N/A".to_owned()), true)
                .field("join_react", config.join_react.unwrap_or("N/A".to_owned()), true)
                .field("leave_msg", config.leave_msg.unwrap_or("N/A".to_owned()), true)
                .field("msg_channel", msg_channel, true)
                .field("role_channel", role_channel, true)
                .field("invite_guard", config.invite_guard.unwrap_or(false), true)
                .field("log_msg", log_msg, true)
                .field("log_mod", log_mod, true)
                .field("log_member", log_member, true)
                .field("mute_role", mute_role, true)
                .field("prefix", config.prefix.unwrap_or(default_prefix), true)
                .field("max_mention", config.max_mention, true)
                .field("disabled_channels", disabled_channels, false)
            )
        );
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});