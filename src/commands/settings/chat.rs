use serenity::framework::standard::CommandError;
use utils::config::*;


command!(inviteguard(ctx, msg, args) {
    let status_str = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/invalid_option_enable_disable"))),
    };

    let mut status;
    let mut s;

    if status_str == "enable" {
        status = true;
        s = "Invite guard has been enabled.";
    } else if status_str == "disable" {
        status = false;
        s = "Invite guard has been disabled.";
    } else {
        return Err(CommandError::from(get_msg!("error/invalid_option_enable_disable")));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(&ctx);

        let mut config = pool.get_guild_config(guild_id.0);

        config.invite_guard = Some(status);

        pool.save_guild_config(&config);

        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});


command!(max_mentions(ctx, msg, args) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let max_mention = match args.single::<i32>() {
            Ok(val) => val,
            Err(_) => return Err(CommandError::from(get_msg!("error/required_number"))),
        };

        let pool = get_pool(&ctx);

        let mut config = pool.get_guild_config(guild.id.0);
        config.max_mention = max_mention;

        pool.save_guild_config(&config);

        let s = get_msg!("info/max_mention_set", max_mention);
        let _ = msg.channel_id.say(&s);
    }    
});
