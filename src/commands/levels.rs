use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use reqwest;
use std::collections::HashMap;
use std::fmt::Write;
use utils;
use utils::config::get_pool;
use utils::time::now_utc;

use regex::Regex;

use chrono::Duration;
use chrono_humanize::HumanTime;

const LEVEL_HTML: &'static str = include_str!("../../assets/html/rank.html");

command!(profile(ctx, msg, args) {
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

    let (user_rep, activity) = match pool.get_user(id) {
        Some(val) => (val.rep, val.msg_activity),
        None => (0, vec![0; 24]),
    };

    let user = match UserId(id).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError("Could not fetch user.".to_owned())),
    };

    let _ = msg.channel_id.broadcast_typing();

    println!("started typing");

    let mut html = LEVEL_HTML.clone();

    let html = html.replace("{USERNAME}", &user.tag());
    let html = html.replace("{AVATAR_URL}", &user.face());
    let html = html.replace("{DAILY}", &format!("{}/{}", level_data.msg_day_rank, level_data.msg_day_total));
    let html = html.replace("{WEEKLY}", &format!("{}/{}", level_data.msg_week_rank, level_data.msg_week_total));
    let html = html.replace("{MONTHLY}", &format!("{}/{}", level_data.msg_month_rank, level_data.msg_month_total));
    let html = html.replace("{ALL}", &format!("{}/{}", level_data.msg_all_time_rank, level_data.msg_all_time_total));
    let html = html.replace("{REP_EMOJI}", &get_rep_emoji_level(user_rep));
    let html = html.replace("{REP}", &user_rep.to_string());
    let html = html.replace("{LAST_MESSAGE}", &level_data.last_msg.format("%Y-%m-%d %H:%M:%S UTC").to_string());
    let html = html.replace("{ACTIVITY_DATA}", &format!("{:?}", &activity));

    println!("created html");

    let mut json = HashMap::new();
    json.insert("html", html);
    json.insert("width", "500".to_owned());
    json.insert("height", "350".to_owned());

    println!("created json");


    let client = reqwest::Client::new();
    println!("created reqwest client");
    let res = match client.post("http://127.0.0.1:3000/html").json(&json).send() {
        Ok(val) => val.error_for_status(),
        Err(_) => {
            let _ = msg.channel_id.send_message(|m|
                m.embed(|e| e
                    .author(|a| a
                        .name(&format!("{} [{} {} rep]", &user.tag(), get_rep_emoji_plain(user_rep), user_rep))
                        .icon_url(&user.face())
                    )
                    .color(0x2ecc71)
                    .field("Daily", &format!("{}/{}", level_data.msg_day_rank, level_data.msg_day_total), true)
                    .field("Weekly", &format!("{}/{}", level_data.msg_week_rank, level_data.msg_week_total), true)
                    .field("Monthly", &format!("{}/{}", level_data.msg_month_rank, level_data.msg_month_total), true)
                    .field("All Time", &format!("{}/{}", level_data.msg_all_time_rank, level_data.msg_all_time_total), true)
                    .field("24 Hour Activity", get_activity_plain_graph(&activity), false)
                    .thumbnail(&user.face())
                )
            );
            return Ok(());
        }
    };

    println!("got response");

    let mut img = match res {
        Ok(val) => val,
        Err(_) => {
            // in case webserver down or something?
            // fallback to text
            let _ = msg.channel_id.send_message(|m|
                m.embed(|e| e
                    .author(|a| a
                        .name(&format!("{} [{} {} rep]", &user.tag(), get_rep_emoji_plain(user_rep), user_rep))
                        .icon_url(&user.face())
                    )
                    .color(0x2ecc71)
                    .field("Daily", &format!("{}/{}", level_data.msg_day_rank, level_data.msg_day_total), true)
                    .field("Weekly", &format!("{}/{}", level_data.msg_week_rank, level_data.msg_week_total), true)
                    .field("Monthly", &format!("{}/{}", level_data.msg_month_rank, level_data.msg_month_total), true)
                    .field("All Time", &format!("{}/{}", level_data.msg_all_time_rank, level_data.msg_all_time_total), true)
                    .field("24 Hour Activity", get_activity_plain_graph(&activity), false)
                    .thumbnail(&user.face())
                )
            );
            return Ok(());
        },
    };

    println!("got image");

    let mut buf: Vec<u8> = vec![];
    println!("created buffer");
    img.copy_to(&mut buf)?;

    println!("copied to buffer");

    let files = vec![(&buf[..], "level.png")];
    println!("created files");

    let _ = msg.channel_id.send_files(files, |m| m.content(""));
});

fn get_rep_emoji_level(user_rep: i32) -> String {
    let num = match user_rep {
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

fn get_rep_emoji_plain(user_rep: i32) -> String {
    match user_rep {
        n if n >= 150 => "😀",
        n if n >= 100 => "😄",
        n if n >= 50  => "😊",
        n if n >= 10  => "🙂",
        n if n >=  0  => "😶",
        n if n >= -5  => "😨",
        n if n >= -10 => "🤒",
        n if n >= -20 => "😦",
        n if n >= -30 => "☹",
        n if n >= -40 => "😠",
        _ => "😡",
    }.to_owned()
}

fn get_activity_plain_graph(activity: &Vec<i32>) -> String {
    let max = activity.iter().max().unwrap_or(&0);
    let min = activity.iter().min().unwrap_or(&0);
    let range = max - min;
    let chunk = range / 8;

    let mut s = "```0  ".to_owned();

    for msgs in activity.iter() {
        let val = match msgs {
            x if x > &(chunk * 7) => "█",
            x if x > &(chunk * 6) => "▇",
            x if x > &(chunk * 5) => "▆",
            x if x > &(chunk * 4) => "▅",
            x if x > &(chunk * 3) => "▄",
            x if x > &(chunk * 2) => "▃",
            x if x > &(chunk * 1) => "▂",
            x if x > &(chunk * 0) => "▁",
            _ => "_",
        };

        let _ = write!(s, "{}", val);
    }

    let _ = write!(s, " 24\n```");

    s
}

command!(rep(ctx, msg, args) {
    let pool = get_pool(&ctx);

    // print next rep time 
    if args.is_empty() {
        if let Some(last_rep) = pool.get_last_rep(msg.author.id.0) {
            let now = now_utc();
            let next_rep = last_rep + Duration::hours(24);

            let diff = next_rep.signed_duration_since(now);
            // precise humanized time 
            let ht = format!("{:#}", HumanTime::from(diff));

            if next_rep > now {
                let _ = msg.channel_id.say(&get_msg!("error/rep_too_soon", ht));
                return Ok(());
            }
        }

        // check if can rep and args empty 
        return Err(CommandError::from(get_msg!("error/rep_no_args")));
    }


    let action_target = args.full();

    let action = if action_target.contains("+") {
        "+"
    } else if action_target.contains("-") {
        "-"
    } else {
        return Err(CommandError::from(get_msg!("error/invalid_rep_option")));
    };

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d{17,18})").unwrap();
    }

    let target = match RE.find(&action_target).and_then(|x| x.as_str().parse::<u64>().ok()) {
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