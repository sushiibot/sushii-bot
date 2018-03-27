use serenity::framework::standard::CommandError;

use std::env;
use utils::config::get_pool;
use utils::config::get_config;
use utils::config::update_config;

command!(prefix(ctx, msg, args) {
    let pool = get_pool(ctx);

    // check for MANAGE_SERVER permissions

    if let Some(guild) = msg.guild() {
        let guild = guild.read();
        let mut config = check_res_msg!(get_config(ctx, &pool, guild.id.0));

        let pref = match args.single::<String>() {
            Ok(val) => val,
            Err(_) => {
                // no prefix argument, set the prefix
                match config.prefix.clone() {
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

            if Some(&pref) == config.prefix.as_ref() {
                let _ = msg.channel_id.say(get_msg!("info/prefix_existing", &pref));
                return Ok(());
            }

            config.prefix = Some(pref.clone());

            update_config(ctx, &pool, &config);

            let _ = msg.channel_id.say(get_msg!("info/prefix_set", &pref));
        } else {
            return Err(CommandError::from(get_msg!("error/prefix_no_perms")));
        }
        
    } else {
        // no guild found, probably in DMs
        let pref = env::var("DEFAULT_PREFIX").expect(&get_msg!("error/prefix_no_default"));
        let _ = msg.channel_id.say(get_msg!("info/prefix_default", &pref));
    }
});