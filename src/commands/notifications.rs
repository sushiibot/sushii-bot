use serenity::framework::standard::CommandError;
use serenity::CACHE;

use std::fmt::Write;
use database;
use utils::config::get_pool;

command!(add_notification(ctx, msg, args) {
    let mut keyword = args.full().to_lowercase();

    if keyword.is_empty() {
        return Err(CommandError("Missing keyword".to_owned()));
    }

    let _ = msg.delete();

    let guild_id = if keyword.starts_with("global ") {
        keyword = keyword.replace("global ", "");
        0
    } else {
        msg.guild_id().map_or(0, |x| x.0)
    };

    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    pool.new_notification(msg.author.id.0, guild_id, &keyword);

    let s = if guild_id == 0 {
        get_msg!("info/notification_added_global")
    } else {
        get_msg!("info/notification_added")
    };

    let _ = msg.channel_id.say(&s);
});

command!(list_notifications(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();
    let mut notifications = match pool.list_notifications(msg.author.id.0) {
        Some(val) => val,
        None => {
            let _ = msg.channel_id.say("You have no notifications set.");
            return Ok(());
        }
    };
    let cache = CACHE.read();

    notifications.sort_by(|a, b| a.keyword.cmp(&b.keyword));
    let mut s = String::new();

    if notifications.len() == 0 {
        let _ = msg.channel_id.say("You have no notifications set.");
    } else {
        let _ = write!(s, "Your notifications:\n```\n");
        let mut counter = 1;

        for noti in notifications {
            let noti_scope = if noti.guild_id == 0 {
                "Global".to_owned()
            } else {
                match cache.guild(noti.guild_id as u64) {
                    Some(val) => val.read().name.clone(),
                    None => noti.guild_id.to_string(),
                }
            };

            let _ = write!(s, "[{:02}] {} ({})\n", counter, noti.keyword, noti_scope);
            
            counter += 1;
        }

        let _ = write!(s, "```");
    }

    if let Err(_) = msg.author.direct_message(|m| m.content(&s)) {
        let _ = msg.channel_id.say(get_msg!("error/failed_dm"));
    } else {
        let _ = msg.channel_id.say(get_msg!("info/notification_sent_dm"));
    }

    /*

    if let Some(guild_id) = msg.guild_id() {
        // get the notifications in this server
        
        let notifications_server: Vec<&Notification> = notifications.iter().filter(|x| guild_id.0 == x.guild_id as u64).collect();
        let mut s = String::new();
        if notifications_server.len() == 0 {
            let _ = write!(s, "You have no notifications set in this server.");
        } else {
            let _ = write!(s, "Your notifications in this server:\n```\n");

            for noti in notifications_server {
                let _ = write!(s, "[{:02}] {}\n", noti.notification_id, noti.keyword);
            }

            let _ = write!(s, "```");
        }

        let mut notifications_else: Vec<&Notification> = notifications.iter().filter(|x| guild_id.0 != x.guild_id as u64).collect();
        notifications_else.sort_by(|a, b| a.guild_id.cmp(&b.guild_id));

        if notifications_else.len() > 0 {
            let _ = write!(s, "\nYour notifications in other servers:\n```\n");

            for noti in notifications_else {
                let guild_name = match cache.guild(noti.guild_id as u64) {
                    Some(val) => val.read().name.clone(),
                    None => noti.guild_id.to_string(),
                };

                let _ = write!(s, "[{:02}] {} ({})\n", noti.notification_id, noti.keyword, guild_name);
            }

            let _ = write!(s, "```");
        }
        
        let _ = msg.channel_id.say(&s);
    } else {
        notifications.sort_by(|a, b| a.guild_id.cmp(&b.guild_id));
        let mut s = "Your notifications in all servers:\n```".to_owned();
        for noti in notifications {
            let guild_name = match cache.guild(noti.guild_id as u64) {
                Some(val) => val.read().name.clone(),
                None => noti.guild_id.to_string(),
            };

            let _ = write!(s, "[{:02}] {} ({})\n", noti.notification_id, noti.keyword, guild_name);
        }
        let _ = write!(s, "```");
        let _ = msg.channel_id.say(&s);
    }

    */
});

command!(delete_notification(ctx, msg, args) {
    let keyword_or_id = args.full();

    if keyword_or_id.is_empty() {
        return Err(CommandError::from("Missing keyword or ID."));
    }

    // is keyword or id?
    let mut is_keyword;

    let notification_id = match keyword_or_id.parse::<i32>() {
        Ok(val) => {
            is_keyword = false;
            val
        },
        Err(_) => {
            is_keyword = true;
            0
        }
    };

    let pool = get_pool(&ctx);

    let guild_id = msg.guild_id().map(|x| x.0);

    let result = if is_keyword {
        if guild_id.is_none() {
            return Err(CommandError::from(get_msg!("error/notification_delete_by_keyword")));
        }

        pool.delete_notification(msg.author.id.0, guild_id, Some(&keyword_or_id.to_lowercase()), None)
    } else {
        pool.delete_notification(msg.author.id.0, None, None, Some(notification_id))
    };

    if let Some(deleted) = result {
        let s = format!("Deleted the keyword `{}`.", deleted.keyword);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/notification_delete_failed")));
    }
});