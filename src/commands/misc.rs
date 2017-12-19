use serenity::framework::standard::CommandError;
use reqwest;
use reqwest::header::ContentType;
use regex::Regex;
use std::fmt::Write;

use chrono::{DateTime, Utc, Duration};
use chrono_humanize::HumanTime;
use database;

#[derive(Deserialize)]
struct Response {
    stderr: String,
    stdout: String,
}

command!(play(_ctx, msg, args) {
    let mut code = args.full();

    // check if using code block
    if !code.starts_with("```") || !code.ends_with("```") {
        return Err(CommandError("Missing code block".to_owned()));
    }

    let _ = msg.react("👌");

    // clean up input
    code = code.replace("```rust", "");
    code = code.replacen("```", "", 2); // 2 in case rust in top of code block isn't used
    code = code.replace("\"", "\\\"");  // escape quotes
    code = code.replace("\n", "\\n");   // escape new lines

    // create json data
    let mut data = r#"{"channel":"stable","mode":"debug","crateType":"bin","tests":false,"code": "{CODE}"}"#.to_owned();
    data = data.replace("{CODE}", &code);

    // send data
    let client = reqwest::Client::new();
    let res = client.post("http://play.integer32.com/execute")
        .body(data)
        .header(ContentType::json())
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
    let time = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError("Not enough arguments".to_owned())),
    };

    // parse durations for each
    let re = Regex::new(r"(\d+)\s*d").unwrap();
    let day = match re.find(&time) {
        Some(val) => val.as_str().replace("d", "").parse::<i64>().unwrap(),
        None => 0
    };
    
    let re = Regex::new(r"(\d+)\s*h").unwrap();
    let hour = match re.find(&time){
        Some(val) => val.as_str().replace("h", "").parse::<i64>().unwrap(),
        None => 0
    };
    
    let re = Regex::new(r"(\d+)\s*m").unwrap();
    let min = match re.find(&time) {
        Some(val) => val.as_str().replace("m", "").parse::<i64>().unwrap(),
        None => 0
    };
    
    let re = Regex::new(r"(\d+)\s*s").unwrap();
    let sec = match re.find(&time) {
        Some(val) => val.as_str().replace("s", "").parse::<i64>().unwrap(),
        None => 0
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

    let content = args.full();

    // throw reminder into the database
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    pool.add_reminder(msg.author.id.0, &content, &remind_date);

    let now = now.naive_utc();
    let since = remind_date.signed_duration_since(
        now,
    );

    let ht = HumanTime::from(since);

    let s = format!("I'll remind you at `{}` ({}) to `{}`", remind_date.format("%Y-%m-%d %H:%M:%S UTC"), ht, content);
    let _ = msg.channel_id.say(&s);
});

command!(reminders(ctx, msg, _args) {
    // throw reminder into the database
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let reminders = pool.get_reminders(msg.author.id.0);

    if let Some(reminders) = reminders {
        let mut s = format!("You have {} reminders:\n```rust\n", reminders.len());

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        for reminder in reminders {
            let since = reminder.time_to_remind.signed_duration_since(
                now
            );

            let ht = HumanTime::from(since);
            let _ = write!(s, "{} ({})\n    {}\n", reminder.time_to_remind.format("%Y-%m-%d %H:%M:%S UTC"), ht, reminder.description);
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
