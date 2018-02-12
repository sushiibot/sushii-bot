use serenity::framework::standard::CommandError;
use reqwest;
use regex::Regex;
use std::fmt::Write;

use serde_json::value::Value;

use chrono::{DateTime, Utc, Duration};
use chrono_humanize::HumanTime;
use database;

#[derive(Deserialize)]
struct Response {
    stderr: String,
    stdout: String,
}

command!(play(_ctx, msg, args) {
    let mut code = args.full().to_owned();

    // check if using code block
    if !code.starts_with("```") || !code.ends_with("```") {
        return Err(CommandError("Missing code block".to_owned()));
    }

    let _ = msg.react("👌");

    // clean up input
    code = code.replace("```rust", "");
    code = code.replacen("```", "", 2); // 2 in case rust in top of code block isn't used

    let mut json = json!({
        "channel": "stable",
        "mode": "debug",
        "crateType": "bin",
        "tests": false,
        "code": "",
    });

    *json.get_mut("code").unwrap() = Value::String(code);

    // send data
    let client = reqwest::Client::new();
    let res = client.post("http://play.integer32.com/execute")
        .json(&json)
        .send()?.error_for_status();

    // check response
    match res {
        Ok(mut val) => {
            let res_obj: Response = val.json()?;

            let mut clean = res_obj.stdout.replace("@", "@\u{200B}"); // add zws to possible mentions
            clean = clean.replace("`", "'");                          // replace comment ticks to single quotes

            let _ = msg.channel_id.say(format!("```rust\n{}\n{}\n```", res_obj.stderr, clean));
        },
        Err(e) => {
            let _ = msg.channel_id.say(format!("Error: {}", e));
        }
    }
});


command!(reminder(ctx, msg, args) {
    let mut full_msg = args.full().to_owned();


    if full_msg.is_empty() {
        return Err(CommandError::from(get_msg!("error/no_reminder_given")));
    }

    let mut end_pos = 0;

    // parse durations for each
    let re = Regex::new(r"(\d+)\s*d\w*").unwrap();
    let day = if let Some(caps) = re.captures(&full_msg) {
        end_pos = caps.get(0).unwrap().end();
        
        let start = caps.get(1).unwrap().start();
        let end = caps.get(1).unwrap().end();
        full_msg[start..end].parse::<i64>().unwrap()
    } else {
        0
    };
    
    let re = Regex::new(r"(\d+)\s*h\w*").unwrap();
    let hour = if let Some(caps) = re.captures(&full_msg){
        let caps_full_end = caps.get(0).unwrap().end();
        if caps_full_end > end_pos {
            end_pos = caps_full_end
        }

        let start = caps.get(1).unwrap().start();
        let end = caps.get(1).unwrap().end();
        full_msg[start..end].parse::<i64>().unwrap()
    } else {
        0
    };
    
    let re = Regex::new(r"(\d+)\s*m\w*").unwrap();
    let min = if let Some(caps) = re.captures(&full_msg) {
        let caps_full_end = caps.get(0).unwrap().end();
        if caps_full_end > end_pos {
            end_pos = caps_full_end
        }

        let start = caps.get(1).unwrap().start();
        let end = caps.get(1).unwrap().end();
        full_msg[start..end].parse::<i64>().unwrap()
    } else {
        0
    };
    
    let re = Regex::new(r"(\d+)\s*s\w*").unwrap();
    let sec = if let Some(caps) = re.captures(&full_msg) {
        let caps_full_end = caps.get(0).unwrap().end();
        if caps_full_end > end_pos {
            end_pos = caps_full_end
        }

        let start = caps.get(1).unwrap().start();
        let end = caps.get(1).unwrap().end();
        full_msg[start..end].parse::<i64>().unwrap()
    } else {
        0
    };

    let reminder_content = if let Some(pos) = full_msg.rfind("to ") {
        &full_msg[pos + 3..]
    } else if end_pos == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_reminder")));
    } else {
        &full_msg[end_pos..]
    };

    // get current time and add up the offsets
    let now: DateTime<Utc> = Utc::now();
    let offset = Duration::days(day) + Duration::hours(hour) + Duration::minutes(min) + Duration::seconds(sec);

    // get the reminder time, current time + offset time
    let remind_date = now + offset;
    let remind_date = remind_date.naive_utc();

    // check if offset is great enough
    if offset.num_seconds() < 5 {
        return Err(CommandError("Reminder must be at least 5 seconds from now".to_owned()));
    }

    if reminder_content.is_empty() {
        return Err(CommandError::from(get_msg!("error/reminder_not_given")))
    }

    // throw reminder into the database
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    pool.add_reminder(msg.author.id.0, &reminder_content, &remind_date);

    let now = now.naive_utc();
    let since = remind_date.signed_duration_since(
        now,
    );

    let ht = HumanTime::from(since);

    let s = format!("I'll remind you {:#} (`{}`) to `{}`", ht, remind_date.format("%Y-%m-%d %H:%M:%S UTC"), reminder_content);
    let _ = msg.channel_id.say(&s);
});

command!(reminders(ctx, msg, _args) {
    // throw reminder into the database
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let current_reminders = pool.get_reminders(msg.author.id.0);

    if let Some(current_reminders) = current_reminders {
        let mut s = format!("You have {} reminders:\n```rust\n", current_reminders.len());

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        for remind in current_reminders {
            let since = remind.time_to_remind.signed_duration_since(
                now
            );

            let ht = HumanTime::from(since);
            let _ = write!(s, "{} ({:#})\n    {}\n", remind.time_to_remind.format("%Y-%m-%d %H:%M:%S UTC"), ht, remind.description);
        }

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        let _ = write!(s, "\nCurrent time: {}\n", now.format("%Y-%m-%d %H:%M:%S UTC"));

        let _ = write!(s, "```");

        let _ = msg.channel_id.say(&s);
    } else {
        let _ = msg.channel_id.say("You have no reminders set.");
    }
});
