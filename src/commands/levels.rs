use serenity::framework::standard::CommandError;
use reqwest;
use reqwest::header::ContentType;
use std::fmt::Write;
use std::collections::HashMap;
use database;

use util;

const LEVEL_HTML: &'static str = include_str!("../../html/rank.html");

command!(rank(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let id = match args.single::<String>() {
        Ok(val) => {
            match util::get_id(&val) {
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

    let mut s = "```ruby\nMessage Count\n".to_owned();
    let _ = write!(s, "Month: {}\n", level_data.msg_month);
    let _ = write!(s, "Week: {}\n", level_data.msg_week);
    let _ = write!(s, "Day: {}\n", level_data.msg_day);
    let _ = write!(s, "All: {}\n\n", level_data.msg_all_time);
    let _ = write!(s, "Last Message: {}\n", level_data.last_msg.format("%Y-%m-%d %H:%M:%S UTC"));
    let _ = write!(s, "```");

    let mut html = LEVEL_HTML.clone();

    let html = html.replace("{DAILY}", &level_data.msg_day.to_string());
    let html = html.replace("{WEEKLY}", &level_data.msg_week.to_string());
    let html = html.replace("{MONTHLY}", &level_data.msg_month.to_string());
    let html = html.replace("{ALL}", &level_data.msg_all_time.to_string());
    let html = html.replace("{LAST_MESSAGE}", &level_data.last_msg.format("%Y-%m-%d %H:%M:%S UTC").to_string());


    let mut json = HashMap::new();
    json.insert("html", html);
    json.insert("width", "400".to_owned());
    json.insert("height", "300".to_owned());

    let client = reqwest::Client::new();
    let res = client.post("http://127.0.0.1:3000/html")
        .json(&json)
        .send()?.error_for_status();

    let mut img = match res {
        Ok(val) => val,
        Err(e) => return Err(CommandError(format!("Error: {}", e))),
    };

    let mut buf: Vec<u8> = vec![];
    img.copy_to(&mut buf)?;

    let files = vec![(&buf[..], "level.png")];

    let _ = msg.channel_id.send_files(files, |m| m.content(&s));
});
