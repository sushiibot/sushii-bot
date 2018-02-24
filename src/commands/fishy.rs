use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use chrono::Duration;
use chrono_humanize::HumanTime;

use utils::config::get_pool;
use utils::time::now_utc;
use utils::user::get_id;

command!(fishy(ctx, msg, args) {
    let pool = get_pool(&ctx);

    if let Some(last_fishy) = pool.get_last_fishies(msg.author.id.0) {
        let now = now_utc();
        let next_rep = last_fishy + Duration::hours(12);

        let diff = next_rep.signed_duration_since(now);
        // precise humanized time 
        let ht = format!("{:#}", HumanTime::from(diff));

        if next_rep > now {
            return Err(CommandError::from(get_msg!("error/fishy_too_soon", ht)))
        }
    };

    let mut fishies_self = false;

    let target = if !args.is_empty() {
        // fishies for someone else
        match args.single::<String>().ok().and_then(|x| get_id(&x)) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
        }
    } else {
        // fishies for self
        fishies_self = true;
        msg.author.id.0
    };

    if target == msg.author.id.0 {
        fishies_self = true
    }

    let target_user = match UserId(target).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };


    let num_fishies = pool.get_fishies(msg.author.id.0, target, fishies_self);
    pool.update_stat("fishies", "fishies_given", num_fishies);

    let _ = if fishies_self {
        msg.channel_id.say(get_msg!("info/fishies_received", num_fishies))
    } else {
        msg.channel_id.say(get_msg!("info/fishies_given", num_fishies, target_user.tag()))
    };
});
