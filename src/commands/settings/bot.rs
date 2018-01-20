use serenity::framework::standard::CommandError;

use std::env;
use database;

command!(prefix(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    // check for MANAGE_SERVER permissions

    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let pref = match args.single::<String>() {
            Ok(val) => val,
            Err(_) => {
                // no prefix argument, set the prefix
                match pool.get_prefix(guild.id.0) {
                    Some(pref) => {
                        let _ = msg.channel_id.say(&get_msg!("info/prefix_current", &pref));
                        return Ok(());
                    },
                    None => {
                        let pref = env::var("DEFAULT_PREFIX").expect(&get_msg!("error/prefix_no_default"));
                        let _ = msg.channel_id.say(get_msg!("info/prefix_current", &pref));
                        return Ok(());
                    }
                }
            },
        };

        let has_manage_guild = guild.member_permissions(msg.author.id).manage_guild();

        if has_manage_guild {
            let success = pool.set_prefix(guild.id.0, &pref);

            if success {
                let _ = msg.channel_id.say(get_msg!("info/prefix_set", &pref));
            } else {
                let _ = msg.channel_id.say(get_msg!("info/prefix_existing", &pref));
            }
        } else {
            return Err(CommandError::from("error/prefix_no_perms"));
        }
        
    } else {
        // no guild found, probably in DMs
        let pref = env::var("DEFAULT_PREFIX").expect(&get_msg!("error/prefix_no_default"));
        let _ = msg.channel_id.say(get_msg!("info/prefix_default", &pref));
    }
});