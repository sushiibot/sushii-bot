use serenity::framework::standard::CommandError;
use serenity::utils::parse_channel;
use utils::config::get_pool;

use regex::Regex;
use std::fmt::Write;

command!(gallery_add(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(https://discordapp\.com/api/webhooks/\d*/\w*)").unwrap();
    }

    let webhook_url = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/no_webhook_given"))),
    };

    // validate webhook url 
    if !RE.is_match(&webhook_url) {
        return Err(CommandError::from(get_msg!("error/invalid_webhook")));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        pool.add_gallery(channel, guild_id.0, &webhook_url);

        let s = get_msg!("info/gallery_added", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(gallery_list(ctx, msg, _args) {
    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        let s = if let Some(galleries) = pool.list_galleries(guild_id.0) {
            if galleries.is_empty() {
                get_msg!("info/galleries_none")
            } else {
                let mut s = "This guild's gallery channels:\n".to_owned();
                let mut counter = 1;

                for gallery in galleries {
                    lazy_static! {
                        static ref RE: Regex = Regex::new(r"(\d{17,18})").unwrap();
                    }


                    let target = RE.find(&gallery.webhook_url).map_or("N/A", |x| x.as_str());
                    let _ = write!(s, "`[{:02}]` <#{}> :arrow_right: {} (webhook ID)\n", counter, gallery.watch_channel, target);
                    counter += 1;
                }

                s
            }
        } else {
            get_msg!("info/galleries_none")
        };

        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(gallery_delete(ctx, msg, args) {
    let gallery_id = match args.single::<i32>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/invalid_gallery_id"))),
    };

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        if pool.delete_gallery(guild_id.0, gallery_id) {
            let _ = msg.channel_id.say(get_msg!("info/gallery_deleted"));
        } else {
            return Err(CommandError::from(get_msg!("info/gallery_failed_delete")));
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});