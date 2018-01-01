use utils::config::get_pool;
use utils::user::get_id;
use serenity::framework::standard::CommandError;
use serenity::model::UserId;
use serenity::Error;
use serenity::model::ModelError::InvalidPermissions;
use serenity::model::ModelError::DeleteMessageDaysAmount;

command!(ban(ctx, msg, args) {
    // get the target
    let user = args.single::<String>()?;
    let user_id = match get_id(&user) {
        Some(val) => val,
        None => return Err(CommandError::from("Missing mention or ID.")),
    };

    // get the guild
    let guild = match msg.guild() {
        Some(val) => val.read().unwrap().clone(),
        None => return Err(CommandError::from("No guild.")),
    };

    // get the reason
    let reason_raw = args.full();
    let reason;

    if reason_raw.is_empty() {
        reason = None;
    } else {
        reason = Some(&reason_raw[..]);
    }

    // fetch the user for tag
    let user = match UserId(user_id).get() {
        Ok(val) => val,
        Err(e) => return Err(CommandError::from(format!("Failed to fetch user: {}", e))),
    };

    // ban the user
    let _ = match guild.ban(user_id, 7) {
        Err(Error::Model(InvalidPermissions(permissions))) => 
            return Err(CommandError::from(format!("I don't have permission to ban this user, requires: `{:?}`.", permissions))),
        Err(Error::Model(DeleteMessageDaysAmount(num))) => 
            return Err(CommandError::from(format!("The number of days worth of messages to delete is over the maximum: ({}).", num))),
        Err(_) => return Err(CommandError::from("There was an error trying to ban this user.")),
        _ => {},
    };

    // log the ban in the database
    let pool = get_pool(&ctx);
    let _ = pool.add_mod_action("ban", guild.id.0, &user, reason, true);

    let s = format!("Banned user {} ({}).", user.tag(), user.id.0);
    let _ = msg.channel_id.say(&s);
});
