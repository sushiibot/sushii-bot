use std::error::Error;
use serenity::framework::standard::CommandError;

use SerenityShardManager;

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
