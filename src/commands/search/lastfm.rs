use serenity::framework::standard::CommandError;
use reqwest;
use serde_json::Value;
use chrono::Utc;
use chrono::naive::NaiveDateTime;
use utils::config::get_pool;
use utils::user::get_id;
use env;


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
    } else if let Some(user_mention) = get_id(username_or_set) {
        // check if @mention someone, then look up if they have a saved username
        // fall back to just use the args as a username
        match pool.get_lastfm_username(user_mention) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/fm_no_username_mentioned"))),
        }
    } else if !username_or_set.is_empty() {
        username_or_set.to_owned()
    } else {
        match pool.get_lastfm_username(msg.author.id.0) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/fm_no_username"))),
        }
    };

    let url = FM_RECENT_TRACKS_URL.replace("{USER}", &username)
        .replace("{KEY}", &fm_key);
    
    // fetch data
    let data: Value = match reqwest::get(&url).and_then(|mut x| x.json()) {
        Ok(val) => val,
        Err(e) => {
            warn_discord!("[CMD:fm] Failed to fetch last.fm data: {}", e);
            return Err(CommandError::from(get_msg!("error/fm_fetch_error")))
        },
    };

    let username = data.pointer("/recenttracks/@attr/user").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_artist = data.pointer("/recenttracks/track/0/artist/#text").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_name = data.pointer("/recenttracks/track/0/name").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_album = data.pointer("/recenttracks/track/0/album/#text").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_url = data.pointer("/recenttracks/track/0/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");

    // check for empty values that break embeds
    let username = if username.is_empty() {
        "N/A"
    } else {
        username
    };

    let last_track_artist = if last_track_artist.is_empty() {
        "N/A"
    } else {
        last_track_artist
    };

    let last_track_name = if last_track_name.is_empty() {
        "N/A"
    } else {
        last_track_name
    };

    let last_track_album = if last_track_album.is_empty() {
        "N/A"
    } else {
        last_track_album
    };


    // default blank image for fallback
    let last_track_image = {
        let img = data.pointer("/recenttracks/track/0/image/2/#text").and_then(|x| x.as_str()).unwrap_or("");

        if img.is_empty() {
            "https://i.imgur.com/oYm77EU.jpg"
        } else {
            img
        }
    };

    // get the last track timestamp,
    // if it's currently playing, use now timestamp
    let last_track_timestamp = data.pointer("/recenttracks/track/0/date/uts")
        .and_then(|x| x.as_str())
        .and_then(|x| x.parse::<i64>().ok())
        .and_then(|x| Some(NaiveDateTime::from_timestamp(x, 0)))
        .unwrap_or(Utc::now().naive_utc())
        .format("%Y-%m-%dT%H:%M:%S");

    let last_track_status = if let Some(nowplaying) = data.pointer("/recenttracks/track/0/@attr/nowplaying").and_then(|x| x.as_str()) {
        if nowplaying == "true" {
            "Now Playing"
        } else {
            "Last Track"
        }
    } else {
        "Last Track"
    };

    let total_tracks = data.pointer("/recenttracks/@attr/total").and_then(|x| x.as_str()).unwrap_or("N/A");

    let _ = msg.channel_id.send_message(|m| {
        let mut m = m;

        if saved {
            m = m.content(get_msg!("info/fm_saved_username"));
        }

        m.embed(|e| e
            .author(|a| a
                .name(&format!("{} - {}", username, last_track_status))
                .url(&format!("https://www.last.fm/user/{}", username))
                .icon_url("https://i.imgur.com/C7u8gqg.jpg")
            )
            .color(0xb90000)
            .field("Artist - Song", format!("{} - [{}]({})", last_track_artist, last_track_name, last_track_url), true)
            .field("Album", last_track_album, true)
            .thumbnail(last_track_image)
            .footer(|f| f
                .text(format!("Total Tracks: {}", total_tracks))
            )
            .timestamp(last_track_timestamp.to_string())
        )
    });
});