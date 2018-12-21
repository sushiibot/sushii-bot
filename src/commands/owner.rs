use std::error::Error;
use std::io::Read;
use std::fmt::Write;
use serenity::http;
use serenity::utils::parse_channel;
use serenity::model::id::ChannelId;
use serenity::framework::standard::CommandError;
use serenity::CACHE;
use std::process::Command;
use serde_json::map::Map;
use serde_json::value::Value;
use base64;

use SerenityShardManager;
use utils;
use utils::config::*;

use commands::tags::split_message;

command!(quit(ctx, msg, _args) {
    let _ = msg.channel_id.say("Shutting down all shards");

    let data = ctx.data.lock();
    let close_handle = match data.get::<SerenityShardManager>() {
        Some(v) => v,
        None => return Err(CommandError::from("There was a problem getting the shard manager")),
    };
    
    close_handle.lock().shutdown_all();
});

command!(username(_ctx, msg, args) {
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => {
            return Err(CommandError("Missing argument".to_owned()));
        },
    };

    let mut m = Map::new();
    let n = Value::String(name.clone());
    m.insert("username".into(), n);

    match http::edit_profile(&m) {
        Ok(_) => {
            let _ = msg.channel_id.say(&format!("Changed my username to {}", &name));
        },
        Err(e) => return Err(CommandError(e.description().to_owned())),
    }
});


command!(set_avatar(ctx, msg, args) {
    let url = match args.single::<String>().ok().or_else(|| msg.attachments.get(0).map(|x| x.url.clone())) {
        Some(val) => val,
        None => {
            return Err(CommandError::from(get_msg!("error/no_url_or_attachment_given")));
        },
    };

    let client = get_reqwest_client(&ctx);
    let mut resp = match client.get(&url).send() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_url_request"))),
    };

    let mut buf = vec![];

    match resp.read_to_end(&mut buf) {
        Ok(_) => {},
        Err(e) => return Err(CommandError::from(e)),
    }

    let b64 = base64::encode(&buf);

    let ext = if url.ends_with("png") {
        "png"
    } else {
        "jpg"
    };

    match CACHE.write().user.edit(|p| p.avatar(Some(&format!("data:image/{};base64,{}", ext, b64)))) {
        Ok(_) => {
            let _ = msg.channel_id.say("Updated avatar.");
        },
        Err(e) => return Err(CommandError::from(e)),
    }
});

command!(patron(ctx, msg, args) {
    let id = match args.single::<String>().ok().and_then(|x| utils::user::get_id(&x)) {
        Some(id) => id,
        None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
    };

    if let Ok(status) = args.single::<String>() {
        let pool = get_pool(ctx);

        if status == "add" {
            if pool.set_patron(id, true) {
                let _ = msg.channel_id.say(get_msg!("info/patron_added"));
                return Ok(());
            }
        } else if status == "remove" {
            if pool.set_patron(id, false) {
                let _ = msg.channel_id.say(get_msg!("info/patron_removed"));
                return Ok(());
            }
        } else {
            return Err(CommandError::from(get_msg!("error/invalid_add_remove")));
        };

        return Err(CommandError::from(get_msg!("error/unknown_error")));
    } else {
        return Err(CommandError::from(get_msg!("error/invalid_add_remove")));
    }
});

command!(patron_emoji(ctx, msg, args) {
    let id = match args.single::<String>().ok().and_then(|x| utils::user::get_id(&x)) {
        Some(id) => id,
        None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
    };

    if let Ok(emoji) = args.single::<String>() {
        let pool = get_pool(ctx);
        if pool.set_patron_emoji(id, &emoji) {
            let _ = msg.channel_id.say(get_msg!("info/patron_emoji_set", emoji));
            return Ok(());
        } else {
            return Err(CommandError::from(get_msg!("error/unknown_error")));
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_patron_emoji_given")));
    }
});

command!(listservers(_ctx, msg, _args) {
    let guilds = &CACHE.read().guilds;

    let mut s = String::new();
    for guild in guilds.values() {
        let guild = guild.read();

        let owner_tag = guild.owner_id.to_user().map(|x| x.tag()).unwrap_or_else(|_| "N/A".to_owned());

        let _ = write!(s, "{} ({}) - Owner: {} ({}) - Members: {}\n", 
            guild.name, guild.id.0, owner_tag, guild.owner_id.0, guild.member_count);
    }

    let messages = split_message(&s, Some("Server List"), true);

    for message in messages {
        let _ = msg.channel_id.say(&message);
    }
});

command!(say(_ctx, msg, args) {
    let discord_channel = match args.single::<String>() {
        Ok(val) => val.parse::<u64>().ok().or(parse_channel(&val)).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if discord_channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    let content = args.rest();

    if content.is_empty() {
        return Err(CommandError::from(get_msg!("owner/say/empty_content")))
    }

    ChannelId(discord_channel).say(&content)?;
    let _ = msg.react("✅");
});

command!(exec(_ctx, msg, args) {
    let cmd = args.full().split(" ").collect::<Vec<&str>>();
    if cmd.is_empty() {
        return Err(CommandError::from(get_msg!("owner/exec/empty_command")));
    }

    let cmd_binary = cmd[0];
    let mut child_proc = Command::new(cmd_binary);

    if cmd.len() > 1 {
        child_proc.args(&cmd[1..]);
    }

    let output = child_proc.output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let status_code = output.status.code();

    let mut s = String::new();

    if !stdout.is_empty() {
        s.push_str(&format!("STDOUT: ```{}```", stdout));
    }

    if !stderr.is_empty() {
        s.push_str(&format!("\nSTDERR: ```{}```", stderr));
    }

    if let Some(code) = status_code {
        s.push_str(&format!("\nExit code: `{}`", code));
    }

    if s.is_empty() {
        s = "No output.".into();
    }

    let _ = msg.channel_id.say(&s);
});
