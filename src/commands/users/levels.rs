use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use reqwest;
use std::collections::HashMap;
use std::fmt::Write;
use utils;
use utils::user::*;
use utils::config::get_pool;
use utils::time::now_utc;
use utils::html::escape_html;

use serde_json::map::Map;
use num_traits::cast::ToPrimitive;
use chrono::Duration;
use chrono_humanize::HumanTime;

const LEVEL_HTML: &'static str = include_str!("../../../assets/html/rank.html");

command!(rank(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let id = match args.single::<String>() {
        Ok(val) => {
            match utils::user::get_id(&val) {
                Some(id) => id,
                None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
            }
        },
        Err(_) => msg.author.id.0,
    };

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let level_data = match pool.get_level(id, guild_id) {
        Some(level_data) => level_data,
        None => return Err(CommandError::from(get_msg!("error/level_no_data"))),
    };

    let global_xp = pool.get_global_xp(id).and_then(|x| x.to_i64()).unwrap_or(0);

    let user_data = match pool.get_user(id) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/level_no_data"))),
    };
    
    let user_rep = user_data.rep;
    let activity = user_data.msg_activity;
    let is_patron = user_data.is_patron;
    let patron_emoji = user_data.patron_emoji;
    let fishies = user_data.fishies;

    let profile_options = user_data.profile_options
        .clone()
        .and_then(|x| x.as_object().cloned())
        .unwrap_or(Map::new());

    let content_color = profile_options.get("content_color").and_then(|x| x.as_str())
        .unwrap_or("73, 186, 255");
    let content_opacity = profile_options.get("content_opacity").and_then(|x| x.as_str())
        .unwrap_or("1");

    

    let user = match UserId(id).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };

    let _ = msg.channel_id.broadcast_typing();

    let mut html = LEVEL_HTML.to_owned();

    html = html.replace("{USERNAME}", &escape_html(&user.tag()));
    html = html.replace("{AVATAR_URL}", &user.face());
    html = html.replace("{DAILY}", &format_rank(&level_data.msg_day_rank, &level_data.msg_day_total));
    html = html.replace("{WEEKLY}", &format_rank(&level_data.msg_week_rank, &level_data.msg_week_total));
    html = html.replace("{MONTHLY}", &format_rank(&level_data.msg_month_rank, &level_data.msg_month_total));
    html = html.replace("{ALL}", &format_rank(&level_data.msg_all_time_rank, &level_data.msg_all_time_total));
    html = html.replace("{REP_EMOJI}", &get_rep_emoji_level(user_rep));
    html = html.replace("{REP}", &user_rep.to_string());
    html = html.replace("{FISHIES}", &fishies.to_string());
    html = html.replace("{LAST_MESSAGE}", &level_data.last_msg.format("%Y-%m-%d %H:%M:%S UTC").to_string());
    html = html.replace("{ACTIVITY_DATA}", &format!("{:?}", &activity));

    html = html.replace("{CONTENT_COLOR}", &content_color);
    html = html.replace("{CONTENT_OPACITY}", &content_opacity);

    let global_level = get_level(global_xp);
    let level = get_level(level_data.msg_all_time);
    let last_level_total_xp_required = next_level(level);
    let next_level_total_xp_required = next_level(level + 1);
    
    let next_level_xp_required = next_level_total_xp_required - last_level_total_xp_required;
    let next_level_xp_remaining = next_level_total_xp_required - level_data.msg_all_time;
    let next_level_xp_progress = next_level_xp_required - next_level_xp_remaining;

    let xp_percentage = ((next_level_xp_progress as f64 / next_level_xp_required as f64) * 100.0) as u64;

    let xp_percentage = if xp_percentage > 100 {
        0
    } else {
        xp_percentage
    };

    html = html.replace("{LEVEL}", &level.to_string());
    html = html.replace("{GLOBAL_LEVEL}", &global_level.to_string());
    html = html.replace("{XP_PROGRESS}", &xp_percentage.to_string());
    html = html.replace("{CURR_LEVEL_XP}", &next_level_xp_progress.to_string());
    html = html.replace("{LEVEL_XP_REQ}", &next_level_xp_required.to_string());


    // check if patron, add a heart
    if is_patron {
        html = html.replace("style=\"display:none;\"", "");

        // check if has custom emoji
        if let Some(emoji) = patron_emoji {
            html = html.replace("{PATRON_EMOJI}", &emoji);
        } else {
            // default heart
            html = html.replace("{PATRON_EMOJI}", "heart");
        }
    }

    let mut json = HashMap::new();
    json.insert("html", html);
    json.insert("width", "500".to_owned());
    json.insert("height", "400".to_owned());


    let client = reqwest::Client::new();
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
                    .field("Daily", &format_rank(&level_data.msg_day_rank, &level_data.msg_day_total), true)
                    .field("Weekly", &format_rank(&level_data.msg_week_rank, &level_data.msg_week_total), true)
                    .field("Monthly", &format_rank(&level_data.msg_month_rank, &level_data.msg_month_total), true)
                    .field("All Time", &format_rank(&level_data.msg_all_time_rank, &level_data.msg_all_time_total), true)
                    .field("24 Hour Activity", get_activity_plain_graph(&activity), false)
                    .thumbnail(&user.face())
                )
            );
            return Ok(());
        }
    };

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
                    .field("Daily", &format_rank(&level_data.msg_day_rank, &level_data.msg_day_total), true)
                    .field("Weekly", &format_rank(&level_data.msg_week_rank, &level_data.msg_week_total), true)
                    .field("Monthly", &format_rank(&level_data.msg_month_rank, &level_data.msg_month_total), true)
                    .field("All Time", &format_rank(&level_data.msg_all_time_rank, &level_data.msg_all_time_total), true)
                    .field("24 Hour Activity", get_activity_plain_graph(&activity), false)
                    .thumbnail(&user.face())
                )
            );
            return Ok(());
        },
    };

    let mut buf: Vec<u8> = vec![];
    img.copy_to(&mut buf)?;

    let files = vec![(&buf[..], "level.png")];

    let _ = msg.channel_id.send_files(files, |m| m.content(""));
    pool.update_stat("profile", "levels_generated", 1);
});

fn get_rep_emoji_level(user_rep: i32) -> String {
    let num = match user_rep {
        n if n >= 200 => 11,
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
            let next_rep = last_rep + Duration::hours(12);

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

    let target = match args.single::<String>().ok().and_then(|x| utils::user::get_id(&x)) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
    };

    // check if repping self
    if target == msg.author.id.0 {
        return Err(CommandError::from(get_msg!("error/rep_self")));
    }

    let target_user = match UserId(target).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };

    if target_user.bot {
        return Err(CommandError::from(get_msg!("error/rep_bot")));
    }

    if let Some(last_rep) = pool.get_last_rep(msg.author.id.0) {
        let now = now_utc();
        let next_rep = last_rep + Duration::hours(12);

        let diff = next_rep.signed_duration_since(now);
        // precise humanized time 
        let ht = format!("{:#}", HumanTime::from(diff));

        if next_rep > now {
            return Err(CommandError::from(get_msg!("error/rep_too_soon", ht)))
        }
    };

    pool.rep_user(msg.author.id.0, target);
    pool.update_stat("rep", "rep_given", 1);

    let _ = msg.channel_id.say(get_msg!("info/rep_given", &target_user.tag()));
});


command!(top_levels(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let is_global = if Some("global".to_owned()) == args.single::<String>().ok() {
        true
    } else {
        false
    };

    if is_global {
        if let Some(levels) = pool.get_global_levels() {
            let mut s = String::new();

            let width = levels.first().map(|x| get_level(x.xp.to_i64().unwrap_or(0)).to_string().len()).unwrap_or(1);

            for (i, user) in levels.iter().enumerate() {
                let _ = write!(s, "{} `Level {:0width$}` - <@{}>\n", get_pos_emoji(i as i64),
                    get_level(user.xp.to_i64().unwrap_or(0)), user.user_id,
                    width = width);
            }

            let _ = msg.channel_id.send_message(|m|
                m.embed(|e| e
                    .author(|a| a
                        .name("Top Levels - Global")
                    )
                    .color(0x2ecc71)
                    .description(&s)
                )
            );

            // exit
            return Ok(());
        }
    }

    if let Some(guild_id) = msg.guild_id() {
        
        let top = pool.get_top_levels(guild_id.0);

        let daily = if let Some(daily) = top.day {
            let mut s = String::new();
            for (i, user) in daily.iter().enumerate() {
                let lvl_change = get_level(user.msg_all_time) - get_level(user.msg_all_time - user.msg_day);

                let _ = if lvl_change > 1 {
                    write!(s, "{} <@{}> (Gained {} levels)\n", get_pos_emoji(i as i64),
                        user.user_id, lvl_change)
                } else if lvl_change > 0 {
                    write!(s, "{} <@{}> (Gained {} level)\n", get_pos_emoji(i as i64),
                        user.user_id, lvl_change)
                } else {
                    write!(s, "{} <@{}>\n", get_pos_emoji(i as i64), user.user_id)
                };
            }

            s
        } else {
            "N/A".to_owned()
        };

        let weekly = if let Some(weekly) = top.week {
            let mut s = String::new();
            for (i, user) in weekly.iter().enumerate() {
                let lvl_change = get_level(user.msg_all_time) - get_level(user.msg_all_time - user.msg_day);

                let _ = if lvl_change > 1 {
                    write!(s, "{} <@{}> (Gained {} levels)\n", get_pos_emoji(i as i64),
                        user.user_id, lvl_change)
                } else if lvl_change > 0 {
                    write!(s, "{} <@{}> (Gained {} level)\n", get_pos_emoji(i as i64),
                        user.user_id, lvl_change)
                } else {
                    write!(s, "{} <@{}>\n", get_pos_emoji(i as i64), user.user_id)
                };
            }

            s
        } else {
            "N/A".to_owned()
        };

        let monthly = if let Some(monthly) = top.month {
            let mut s = String::new();
            for (i, user) in monthly.iter().enumerate() {
                let lvl_change = get_level(user.msg_all_time) - get_level(user.msg_all_time - user.msg_day);

                let _ = if lvl_change > 1 {
                    write!(s, "{} <@{}> (Gained {} levels)\n", get_pos_emoji(i as i64),
                        user.user_id, lvl_change)
                } else if lvl_change > 0 {
                    write!(s, "{} <@{}> (Gained {} level)\n", get_pos_emoji(i as i64),
                        user.user_id, lvl_change)
                } else {
                    write!(s, "{} <@{}>\n", get_pos_emoji(i as i64), user.user_id)
                };
            }

            s
        } else {
            "N/A".to_owned()
        };

        let all_time = if let Some(all_time) = top.all_time {
            let mut s = String::new();

            let width = all_time.first().map(|x| get_level(x.msg_all_time).to_string().len()).unwrap_or(1);

            for (i, user) in all_time.iter().enumerate() {
                let _ = write!(s, "{} `Level {:0width$}` - <@{}>\n", get_pos_emoji(i as i64),
                    get_level(user.msg_all_time), user.user_id, width = width);
            }

            s
        } else {
            "N/A".to_owned()
        };

        let _ = msg.channel_id.send_message(|m|
                m.embed(|e| e
                    .author(|a| a
                        .name("Top Levels")
                    )
                    .color(0x2ecc71)
                    .field("Daily", &daily, true)
                    .field("Weekly", &weekly, true)
                    .field("Monthly", &monthly, true)
                    .field("All Time", &all_time, true)
                )
            );

    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});


command!(top_reps(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let guild_id = match msg.guild_id() {
        Some(val) => val.0,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let mut is_global = false;

    let top_users_reps = if Some("global".to_owned()) == args.single::<String>().ok() {
        is_global = true;
        pool.get_top_reps_global()
    } else {
        pool.get_top_reps(guild_id)
    };

    let title = if is_global {
        "Top Reps - Global"
    } else {
        "Top Reps"
    };

    if let Some(reps) = top_users_reps {
        let mut s = String::new();

        let width = reps.first().map(|x| x.1.to_string().len()).unwrap_or(1);

        for (i, user) in reps.iter().enumerate() {
            let _ = write!(s, "{} `{:0width$} rep` - <@{}>\n",
                get_pos_emoji(i as i64), user.1, user.0, width = width);
        }

        let _ = msg.channel_id.send_message(|m|
            m.embed(|e| e
                .author(|a| a
                    .name(title)
                )
                .color(0x2ecc71)
                .description(&s)
            )
        );
    }
});
