use serenity::framework::standard::CommandError;
use serenity::CACHE;

use std::fmt::Write;
use models::Notification;
use database;
use utils::config::get_pool;

command!(add_notification(ctx, msg, args) {
    let keyword = args.full();

    if keyword.is_empty() {
        return Err(CommandError("Missing keyword".to_owned()));
    }

    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let guild_id = match msg.guild_id() {
        Some(val) => val.0,
        None => return Err(CommandError("No guild".to_owned())),
    };

    pool.new_notification(msg.author.id.0, guild_id, &keyword);

    let s = format!("Added a new notification with keyword `{}`", keyword);
    let _ = msg.channel_id.say(&s);
});

command!(list_notifications(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();
    let mut notifications = match pool.list_notifications(msg.author.id.0) {
        Some(val) => val,
        None => {
            let _ = msg.channel_id.say("You have no notifications set.");
            return Ok(());
        }
    };
    let cache = CACHE.read()?;

    if let Some(guild_id) = msg.guild_id() {
        // get the notifications in this server
        let notifications_server: Vec<&Notification> = notifications.iter().filter(|x| guild_id.0 == x.guild_id as u64).collect();
        let mut s = String::new();
        if notifications_server.len() == 0 {
            let _ = write!(s, "You have no notifications set in this server.");
        } else {
            let _ = write!(s, "Your notifications in this server:\n```\n");

            for noti in notifications_server {
                let _ = write!(s, "{}\n", noti.keyword);
            }

            let _ = write!(s, "```");
        }

        let mut notifications_else: Vec<&Notification> = notifications.iter().filter(|x| guild_id.0 != x.guild_id as u64).collect();
        notifications_else.sort_by(|a, b| a.guild_id.cmp(&b.guild_id));

        if notifications_else.len() > 0 {
            let _ = write!(s, "\nYour notifications in other servers:\n```\n");

            for noti in notifications_else {
                let guild_name = match cache.guild(noti.guild_id as u64) {
                    Some(val) => val.read().unwrap().name.clone(),
                    None => noti.guild_id.to_string(),
                };

                let _ = write!(s, "{} ({})\n", noti.keyword, guild_name);
            }

            let _ = write!(s, "```");
        }
        
        let _ = msg.channel_id.say(&s);
    } else {
        notifications.sort_by(|a, b| a.guild_id.cmp(&b.guild_id));
        let mut s = "Your notifications in all servers:\n```".to_owned();
        for noti in notifications {
            let guild_name = match cache.guild(noti.guild_id as u64) {
                Some(val) => val.read().unwrap().name.clone(),
                None => noti.guild_id.to_string(),
            };

            let _ = write!(s, "{} ({})\n", noti.keyword, guild_name);
        }
        let _ = write!(s, "```");
        let _ = msg.channel_id.say(&s);
    }
});

command!(delete_notification(ctx, msg, args) {
    let keyword = args.full();

    if keyword.is_empty() {
        return Err(CommandError::from("Missing keyword."));
    }

    let guild_id = match msg.guild_id() {
        Some(val) => val.0,
        None => return Err(CommandError::from("No guild, try this in a server.")),
    };

    let pool = get_pool(&ctx);
    let result = pool.delete_notification(msg.author.id.0, guild_id, &keyword);

    if result {
        let s = format!("Deleted the keyword `{}`.", keyword);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from("You don't have that keyword set."));
    }
});