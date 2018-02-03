use serenity::framework::standard::CommandError;
use reqwest;
use serde_json::Value;

use chrono::naive::NaiveDateTime;

use env;

use utils::config::get_pool;


const FM_RECENT_TRACKS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.getRecentTracks&user={USER}&api_key={KEY}&format=json";


command!(fm(ctx, msg, args) {
    let fm_key = env::var("LASTFM_KEY").expect("Expected LASTFM_KEY to be set in environment");
    let pool = get_pool(&ctx);

    let _ = msg.channel_id.broadcast_typing();

    let username_or_set = args.full();
    let mut saved = false;

    let username = if username_or_set.starts_with("set ") {
        saved = true;
        // save to db
        pool.set_lastfm_username(msg.author.id.0, &username_or_set.replace("set ", ""));

        // remove the set arg
        username_or_set.replace("set ", "")
    } else {
        match pool.get_lastfm_username(msg.author.id.0) {
            Some(val) => val,
            None => username_or_set.to_owned(),
        }
    };

    if username.is_empty() {
        return Err(CommandError::from(get_msg!("error/fm_no_username")));
    }

    let url = FM_RECENT_TRACKS_URL.replace("{USER}", &username)
        .replace("{KEY}", &fm_key);
    
    // fetch data
    let data: Value = match reqwest::get(&url).and_then(|mut x| x.json()) {
        Ok(val) => val,
        Err(e) => {
            warn_discord!("[CMD:fm] Failed to fetch last.fm data: {}", e);
            return Err(CommandError::from("error/fm_fetch_error"))
        },
    };

    let username = data.pointer("/recenttracks/@attr/user").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_artist = data.pointer("/recenttracks/track/0/artist/#text").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_name = data.pointer("/recenttracks/track/0/name").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_album = data.pointer("/recenttracks/track/0/album/#text").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_url = data.pointer("/recenttracks/track/0/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");
    // default blank image for fallback
    let last_track_image = {
        let img = data.pointer("/recenttracks/track/0/image/2/#text").and_then(|x| x.as_str()).unwrap_or("");

        if img.is_empty() {
            "https://i.imgur.com/oYm77EU.jpg"
        } else {
            img
        }
    };

    let last_track_time = data.pointer("/recenttracks/track/0/date/uts").and_then(|x| x.as_i64()).unwrap_or(0);
    let last_track_timestamp = NaiveDateTime::from_timestamp(last_track_time, 0).format("%Y-%m-%dT%H:%M:%S");

    let _ = msg.channel_id.send_message(|m| {
        let mut m = m;

        if saved {
            m = m.content(get_msg!("info/fm_saved_username"));
        }

        m.embed(|e| {
            let mut e = e;

            e = e.author(|a| a
                .name(username)
                .url(&format!("https://www.last.fm/user/{}", username))
                .icon_url("https://i.imgur.com/C7u8gqg.jpg")
            )
            .color(0xb90000)
            .field("Song", format!("[{}]({})", last_track_name, last_track_url), true)
            .field("Album", last_track_album, true)
            .field("Artist", last_track_artist, true)
            .thumbnail(last_track_image);

            if last_track_time != 0 {
                e = e.timestamp(last_track_timestamp.to_string());
            }

            e
        })
    });
});