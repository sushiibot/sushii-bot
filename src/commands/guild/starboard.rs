use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use diesel::result::Error;
use models::Starboard;
use utils::config::get_pool;
use utils::arg_types;
use regex::Regex;

command!(starboard_channel(ctx, msg, args) {
    if msg.guild_id.is_none() {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }

    let pool = get_pool(&ctx);

    let mut starboard = match pool.get_starboard(msg.guild_id.unwrap().0) {
        Ok(s) => s,
        Err(e) => {
            match default_starboard(e, &msg) {
                Ok(s) => s,
                Err(e) => return Err(e),
            }
        }
    };

    if args.is_empty() {
        let _ = msg.channel_id.say(get_msg!("guild/starboard/info/current_channel", starboard.channel));
        return Ok(());
    }

    let channel = arg_types::ChannelArg::new()
        .args(&mut args)
        .guild(msg.guild())
        .error(get_msg!("guild/starboard/error/invalid_channel"))
        .get()?;

    starboard.channel = channel as i64;
    
    if let Err(err) = pool.update_starboard(&starboard) {
        warn_discord!(format!("[STARBOARD] Failed to update starboard channel: {:?}", err));
        let _ = msg.channel_id.say(get_msg!("guild/starboard/error/failed_update"));
        return Ok(());
    }

    let _ = msg.channel_id.say(get_msg!("guild/starboard/info/updated_channel", channel));
});

command!(starboard_number(ctx, msg, args) {
    if msg.guild_id.is_none() {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }

    let pool = get_pool(&ctx);

    let mut starboard = match pool.get_starboard(msg.guild_id.unwrap().0) {
        Ok(s) => s,
        Err(e) => {
            match default_starboard(e, &msg) {
                Ok(s) => s,
                Err(e) => return Err(e),
            }
        }
    };

    if args.is_empty() {
        let _ = msg.channel_id.say(get_msg!("guild/starboard/info/current_number", starboard.minimum));
        return Ok(());
    }

    let number = args.single::<i32>()?;
    starboard.minimum = number;

    if let Err(err) = pool.update_starboard(&starboard) {
        warn_discord!(format!("[STARBOARD] Failed to update starboard number: {:?}", err));
        let _ = msg.channel_id.say(get_msg!("guild/starboard/error/failed_update"));
        return Ok(());
    }

    let _ = msg.channel_id.say(get_msg!("guild/starboard/info/updated_number", number));
});

command!(starboard_emoji(ctx, msg, args) {
    if msg.guild_id.is_none() {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }

    let pool = get_pool(&ctx);

    let mut starboard = match pool.get_starboard(msg.guild_id.unwrap().0) {
        Ok(s) => s,
        Err(e) => {
            match default_starboard(e, &msg) {
                Ok(s) => s,
                Err(e) => return Err(e),
            }
        }
    };

    if args.is_empty() {
        let _ = msg.channel_id.say(get_msg!("guild/starboard/info/current_emoji", starboard.emoji));
        return Ok(());
    }

    let emoji = args.single::<String>()?;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"<:[^:]*:(\d+)>").unwrap();
    }

    let emoji_id = RE
        .captures(&emoji)
        .and_then(|cap| cap.get(1)) // first capture group
        .map(|x| x.as_str().to_string())
        .and_then(|x| x.parse::<i64>().ok());

    println!("emoji: {}, id: {:?}", &emoji, &emoji_id);
    starboard.emoji = emoji.clone();
    starboard.emoji_id = emoji_id;

    if let Err(err) = pool.update_starboard(&starboard) {
        warn_discord!(format!("[STARBOARD] Failed to update starboard emoji: {:?}", err));
        let _ = msg.channel_id.say(get_msg!("guild/starboard/error/failed_update"));
        return Ok(());
    }

    let _ = msg.channel_id.say(get_msg!("guild/starboard/info/updated_emoji", emoji));
});

fn default_starboard(err: Error, msg: &Message) -> Result<Starboard, CommandError> {
    let starboard = match err {
        Error::NotFound => {
            Starboard {
                guild_id: msg.guild_id.unwrap().0 as i64,
                channel: msg.channel_id.0 as i64,
                emoji: "🍣".into(),
                emoji_id: None,
                minimum: 2,
            }
        },
        _ => return Err(CommandError::from(get_msg!("guild/starboard/error/failed_fetch"))),
    };

    Ok(starboard)
}
