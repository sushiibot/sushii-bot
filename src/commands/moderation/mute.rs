use utils::config::get_pool;
use utils::user::get_id;
use serenity::framework::standard::CommandError;

command!(mute(ctx, msg, args) {
    // get the target
    let user_str = args.single::<String>()?;

    let user_id = match get_id(&user_str) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
    };

    // get the guild
    let guild = match msg.guild() {
        Some(val) => val.read().clone(),
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    // get the reason
    let reason_raw = args.full();
    let reason;

    if reason_raw.len() > 0 {
        reason = Some(&reason_raw[..]);
    } else {
        reason = None;
    }

    let pool = get_pool(&ctx);
    // get the mute role, return if there isn't one set
    let mute_role = match check_res_msg!(pool.get_guild_config(guild.id.0)).mute_role {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/no_mute_role"))),
    };

    let mut member = match guild.member(user_id) {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_member"))),
    };

    // check if user is already muted
    if let Some(_) = member.roles.iter().find(|x| x.0 == mute_role as u64) {
        return Err(CommandError::from(get_msg!("error/already_muted")));
    }

    let user = member.user.read().clone();

    // add a pending case, remove if ban errored
    let case_id = check_res_msg!(pool.add_mod_action("mute", guild.id.0, &user, reason, true, Some(msg.author.id.0))).case_id;

    if let Err(_) = member.add_role(mute_role as u64) {
        // remove failed mod entry
        pool.remove_mod_action(guild.id.0, &user, case_id);
        // return error
        return Err(CommandError::from(get_msg!("error/failed_mute_member")));
    }
    
    let s = if let Some(reason) = reason {
        get_msg!("info/member_muted_with_reason", user.tag(), user.id.0, reason)
    } else {
        get_msg!("info/member_muted", user.tag(), user.id.0)
    };
    
    let _ = msg.channel_id.say(&s);
});
