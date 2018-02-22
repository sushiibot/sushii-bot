use serenity::framework::standard::CommandError;
use chrono::Duration;
use chrono_humanize::HumanTime;

use utils::config::get_pool;
use utils::time::now_utc;

command!(fishy(ctx, msg, _args) {
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

    let num_fishies = pool.get_fishies(msg.author.id.0);
    pool.update_stat("fishies", "fishies_given");

    let _ = msg.channel_id.say(get_msg!("info/fishies_received", num_fishies));
});
