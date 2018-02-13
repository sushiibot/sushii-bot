use std::error::Error;
use std::io::Read;
use serenity::framework::standard::CommandError;
use serenity::CACHE;
use reqwest;
use base64;

use SerenityShardManager;
use utils;
use utils::config::get_pool;

command!(quit(ctx, msg, _args) {
    let _ = msg.channel_id.say("Shutting down all shards");

    let data = ctx.data.lock();
    let close_handle = match data.get::<SerenityShardManager>() {
        Some(v) => v,
        None => return Err(CommandError::from("There was a problem getting the shard manager")),
    };
    
    close_handle.lock().shutdown_all();
});

command!(username(ctx, msg, args) {
    let name = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => {
            return Err(CommandError("Missing argument".to_owned()));
        },
    };

    match ctx.edit_profile(|e| e.username(&name)) {
        Ok(_) => {
            let _ = msg.channel_id.say(&format!("Changed my username to {}", &name));
        },
        Err(e) => return Err(CommandError(e.description().to_owned())),
    }
});


command!(set_avatar(_ctx, msg, args) {
    let url = match args.single::<String>().ok().or_else(|| msg.attachments.get(0).map(|x| x.url.clone())) {
        Some(val) => val,
        None => {
            return Err(CommandError::from(get_msg!("error/no_url_or_attachment_given")));
        },
    };

    let mut resp = match reqwest::get(&url) {
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
        let pool = get_pool(&ctx);

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
