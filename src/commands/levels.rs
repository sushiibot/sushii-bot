use serenity::framework::standard::CommandError;
use serenity::model::UserId;
use reqwest;
use std::fmt::Write;
use std::collections::HashMap;

use utils;
use utils::num::format_percentile;
use utils::config::get_pool;
use utils::time::now_utc;

use chrono::Duration;
use chrono_humanize::HumanTime;

const LEVEL_HTML: &'static str = include_str!("../../assets/html/rank.html");

command!(rank(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let id = match args.single::<String>() {
        Ok(val) => {
            match utils::user::get_id(&val) {
                Some(id) => id,
                None => return Err(CommandError("Invalid mention.".to_owned())),
            }
        },
        Err(_) => msg.author.id.0,
    };

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return Err(CommandError("No guild found.".to_owned())),
    };

    let level_data = match pool.get_level(id, guild_id) {
        Some(level_data) => level_data,
        None => return Err(CommandError("No level data found.".to_owned())),
    };

    let (rep, activity) = match pool.get_user(id) {
        Some(val) => (val.rep, val.msg_activity),
        None => (0, vec![0; 24]),
    };

    let user = match UserId(id).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError("Could not fetch user.".to_owned())),
    };

    let _ = msg.channel_id.broadcast_typing();

    let mut s = "```ruby\nMessage Count\n".to_owned();
    let _ = write!(s, "Month: {}\n", level_data.msg_month);
    let _ = write!(s, "Week: {}\n", level_data.msg_week);
    let _ = write!(s, "Day: {}\n", level_data.msg_day);
    let _ = write!(s, "All: {}\n\n", level_data.msg_all_time);
    let _ = write!(s, "Last Message: {}\n", level_data.last_msg.format("%Y-%m-%d %H:%M:%S UTC"));
    let _ = write!(s, "```");

    let mut html = LEVEL_HTML.clone();

    let html = html.replace("{USERNAME}", &user.tag());
    let html = html.replace("{AVATAR_URL}", &user.face());
    let html = html.replace("{DAILY}", &format_percentile(level_data.msg_day_rank));
    let html = html.replace("{WEEKLY}", &format_percentile(level_data.msg_week_rank));
    let html = html.replace("{MONTHLY}", &format_percentile(level_data.msg_month_rank));
    let html = html.replace("{ALL}", &format_percentile(level_data.msg_all_time_rank));
    let html = html.replace("{REP_EMOJI}", &get_rep_emoji_level(rep));
    let html = html.replace("{REP}", &rep.to_string());
    let html = html.replace("{LAST_MESSAGE}", &level_data.last_msg.format("%Y-%m-%d %H:%M:%S UTC").to_string());
    let html = html.replace("{ACTIVITY_DATA}", &format!("{:?}", activity));

    let mut json = HashMap::new();
    json.insert("html", html);
    json.insert("width", "500".to_owned());
    json.insert("height", "350".to_owned());

    let client = reqwest::Client::new();
    let res = match client.post("http://127.0.0.1:3000/html").json(&json).send() {
        Ok(val) => val.error_for_status(),
        Err(_) => {
            let _ = msg.channel_id.say(&s);
            return Ok(());
        }
    };

    let mut img = match res {
        Ok(val) => val,
        Err(_) => {
            // in case webserver down or something?
            // fallback to text
            let _ = msg.channel_id.say(&s);
            return Ok(());
        },
    };

    let mut buf: Vec<u8> = vec![];
    img.copy_to(&mut buf)?;

    let files = vec![(&buf[..], "level.png")];

    let _ = msg.channel_id.send_files(files, |m| m.content(""));
});

fn get_rep_emoji_level(rep: i32) -> String {
    let num = match rep {
        n if n >= 150 => 11,
        n if n >= 100 => 10,
        n if n >= 50  => 9,
        n if n >= 10  => 8,
        n if n >=  0  => 7,
        n if n >= -5  => 6,
        n if n >= -10 => 5,
        n if n >= -20 => 4,
        n if n >= -30 => 3,
        n if n >= -40 => 2,
        _ => 1,
    };

    format!("{:02}", num)
}


command!(rep(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let action = if let Ok(action) = args.single::<String>() {
        if action != "+" && action != "-" {
            return Err(CommandError::from(get_msg!("error/invalid_rep_option")));
        }

        action
    } else {
        return Err(CommandError::from(get_msg!("error/invalid_rep_option")));
    };

    let target = match args.single::<String>().ok().and_then(|x| utils::user::get_id(&x)) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/no_user_given"))),
    };

    // check if repping self
    if target == msg.author.id.0 {
        return Err(CommandError::from(get_msg!("error/rep_self")));
    }

    let target_user = match UserId(target).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };

    if let Some(last_rep) = pool.get_last_rep(msg.author.id.0) {
        let now = now_utc();
        let next_rep = last_rep + Duration::hours(24);

        let diff = next_rep.signed_duration_since(now);
        // precise humanized time 
        let ht = format!("{:#}", HumanTime::from(diff));

        if next_rep > now {
            return Err(CommandError::from(get_msg!("error/rep_too_soon", ht)))
        }
    };

    pool.rep_user(msg.author.id.0, target, &action);

    let _ = msg.channel_id.say(get_msg!("info/rep_given", &target_user.tag(), &action));
});