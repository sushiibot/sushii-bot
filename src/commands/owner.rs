use std::error::Error;
use serenity::framework::standard::CommandError;

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
