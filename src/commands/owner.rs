use std::error::Error;
use serenity::framework::standard::CommandError;

use SerenityShardManager;

command!(quit(ctx, msg, _args) {
    let _ = msg.channel_id.say("cya");

    let data = ctx.data.lock();
    let close_handle = data.get::<SerenityShardManager>()
        .unwrap();
    
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
