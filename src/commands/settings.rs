use serenity::framework::standard::CommandError;
use serenity::model::permissions::Permissions;

use std::env;
use database;

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
