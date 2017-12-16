use std::error::Error;
use serenity::framework::standard::CommandError;

command!(quit(ctx, msg, _args) {
    match ctx.quit() {
        Ok(()) => {
            let _ = msg.reply("Shutting down!");
        },
        Err(why) => {
            let _ = msg.reply(&format!("Failed to shutdown: {:?}", why));
        },
    }
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
