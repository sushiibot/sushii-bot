use serenity::framework::standard::CommandError;
use serenity::framework::standard::Args;
use serenity::model::channel::Message;
use serenity::prelude::Context;

use std::fmt::Write;
use reqwest;
use serde_json::Value;
use chrono::Utc;
use chrono::naive::NaiveDateTime;
use utils::config::get_pool;
use utils::user::get_id;
use utils::html::clean_url;
use env;


const FM_RECENT_TRACKS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.getRecentTracks&user={USER}&api_key={KEY}&format=json";
const FM_TOP_TRACKS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.gettoptracks&user={USER}&api_key={KEY}&format=json&limit=10&period={PERIOD}";
const FM_TOP_ARTISTS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.gettopartists&user={USER}&api_key={KEY}&format=json&limit=10&period={PERIOD}";
const FM_TOP_ALBUMS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.gettopalbums&user={USER}&api_key={KEY}&format=json&limit=10&period={PERIOD}";
const FM_LOVED_TRACKS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.getlovedtracks&user={USER}&api_key={KEY}&format=json&limit=10";


command!(fm(ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();

    let sub_command = match args.single_n::<String>() {
        Ok(val) => val,
        Err(_) => "nowplaying".to_owned(),
    };

    match sub_command.as_ref() {
        "help" => {
            let _ = msg.channel_id.say(get_msg!("info/fm_help"));
            return Ok(());
        },
        "toptracks" | "topsongs" => {
            let _ = args.skip();
            let period = args.single::<String>().unwrap_or_else(|_| "overall".to_owned());
            let username = get_username(ctx, msg.author.id.0)?;

            if !is_valid_period(&period) {
                return Err(CommandError::from(get_msg!("error/fm_invalid_period")));
            }

            let data = get_data(FM_TOP_TRACKS_URL, &username, &period)?;

            top_tracks(msg, &data, &period);
        },
        "topartists" => {
            let _ = args.skip();
            let period = args.single::<String>().unwrap_or_else(|_| "overall".to_owned());
            let username = get_username(ctx, msg.author.id.0)?;

            if !is_valid_period(&period) {
                return Err(CommandError::from(get_msg!("error/fm_invalid_period")));
            }

            let data = get_data(FM_TOP_ARTISTS_URL, &username, &period)?;

            top_artists(msg, &data, &period);
        },
        "topalbums" => {
            let _ = args.skip();
            let period = args.single::<String>().unwrap_or_else(|_| "overall".to_owned());
            let username = get_username(ctx, msg.author.id.0)?;

            if !is_valid_period(&period) {
                return Err(CommandError::from(get_msg!("error/fm_invalid_period")));
            }

            let data = get_data(FM_TOP_ALBUMS_URL, &username, &period)?;

            top_albums(msg, &data, &period);
        },
        "loved" => {
            let _ = args.skip();
            let username = get_username(ctx, msg.author.id.0)?;
            let data = get_data(FM_LOVED_TRACKS_URL, &username, "ayy lmao")?;

            loved_tracks(msg, &data);
        },
        // no matches would equal just -fm, show now playing / last track
        "nowplaying" | _ => {
            let (username, saved) = set_or_get_username(ctx, msg.author.id.0, &mut args)?;
            let data = get_data(FM_RECENT_TRACKS_URL, &username, "yep lol")?;
            recent_tracks(msg, &data, saved)?;
        }
    };
});

/// Check if a time period is valid for last.fm
fn is_valid_period(period: &str) -> bool {
    let valid_periods = vec!["overall", "7day", "1month", "3month", "6month", "12month"];
    valid_periods.contains(&period)
}

fn top_tracks(msg: &Message, data: &Value, period: &str) {
    let username = data.pointer("/toptracks/@attr/user").and_then(|x| x.as_str()).unwrap_or("N/A");
    let default_vec = vec![];
    let tracks = data.pointer("/toptracks/track").and_then(|x| x.as_array()).unwrap_or(&default_vec);

    if tracks.is_empty() {
        let _ = msg.channel_id.say(get_msg!("info/fm_no_top_tracks"));
        return;
    }

    let mut s = String::new();

    let first_image = tracks.first().and_then(|x| x.pointer("/image/2/#text")).and_then(|x| x.as_str()).unwrap_or("N/A");

    for (i, track) in tracks.iter().enumerate() {
        let playcount = track.pointer("/playcount").and_then(|x| x.as_str()).unwrap_or("N/A");
        let title = track.pointer("/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let url = track.pointer("/url").and_then(|x| x.as_str()).unwrap_or("N/A");
        let artist = track.pointer("/artist/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let artist_url = track.pointer("/artist/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");

        let play_plural = if playcount == "1" {
            "play"
        } else {
            "plays"
        };

        let _ = write!(s, "`[{:02}] {}` {} - **[{}]({})** by [{}]({})\n",
            i + 1, playcount, play_plural, title, clean_url(url), artist, clean_url(artist_url));
    }

    send_embed(msg, None, &format!("{}'s Top Tracks - {}", username, period), username, &s, first_image);
}

fn top_artists(msg: &Message, data: &Value, period: &str) {
    let username = data.pointer("/topartists/@attr/user").and_then(|x| x.as_str()).unwrap_or("N/A");
    let default_vec = vec![];
    let artists = data.pointer("/topartists/artist").and_then(|x| x.as_array()).unwrap_or(&default_vec);

    if artists.is_empty() {
        let _ = msg.channel_id.say(get_msg!("info/fm_no_top_artists"));
        return;
    }

    let mut s = String::new();

    let first_image = artists.first().and_then(|x| x.pointer("/image/2/#text")).and_then(|x| x.as_str()).unwrap_or("N/A");

    for (i, artist) in artists.iter().enumerate() {
        let name = artist.pointer("/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let playcount = artist.pointer("/playcount").and_then(|x| x.as_str()).unwrap_or("N/A");
        let url = artist.pointer("/url").and_then(|x| x.as_str()).unwrap_or("N/A");

        let play_plural = if playcount == "1" {
            "play"
        } else {
            "plays"
        };

        let _ = write!(s, "`[{:02}] {}` {} - [{}]({})\n", i + 1, playcount, play_plural, name, clean_url(url));
    }

    send_embed(msg, None, &format!("{}'s Top Artists - {}", username, period), username, &s, first_image);
}

fn top_albums(msg: &Message, data: &Value, period: &str) {
    let username = data.pointer("/topalbums/@attr/user").and_then(|x| x.as_str()).unwrap_or("N/A");
    let default_vec = vec![];
    let albums = data.pointer("/topalbums/album").and_then(|x| x.as_array()).unwrap_or(&default_vec);

    if albums.is_empty() {
        let _ = msg.channel_id.say(get_msg!("info/fm_no_top_albums"));
        return;
    }

    let mut s = String::new();

    let first_image = albums.first().and_then(|x| x.pointer("/image/2/#text")).and_then(|x| x.as_str()).unwrap_or("N/A");

    for (i, album) in albums.iter().enumerate() {
        let name = album.pointer("/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let playcount = album.pointer("/playcount").and_then(|x| x.as_str()).unwrap_or("N/A");
        let url = album.pointer("/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");
        let artist = album.pointer("/artist/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let artist_url = album.pointer("/artist/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");

        let play_plural = if playcount == "1" {
            "play"
        } else {
            "plays"
        };

        let _ = write!(s, "`[{:02}] {}` {} - **[{}]({})** by [{}]({})\n", 
            i + 1, playcount, play_plural, name, clean_url(url), artist, clean_url(artist_url));
    }

    send_embed(msg, None, &format!("{}'s Top Albums - {}", username, period), username, &s, first_image);
}

fn loved_tracks(msg: &Message, data: &Value) {
    let username = data.pointer("/lovedtracks/@attr/user").and_then(|x| x.as_str()).unwrap_or("N/A");
    let default_vec = vec![];
    let tracks = data.pointer("/lovedtracks/track").and_then(|x| x.as_array()).unwrap_or(&default_vec);

    if tracks.is_empty() {
        let _ = msg.channel_id.say(get_msg!("info/fm_no_loved_tracks"));
        return;
    }

    let mut s = String::new();

    let first_image = tracks.first().and_then(|x| x.pointer("/image/2/#text")).and_then(|x| x.as_str()).unwrap_or("N/A");

    for (i, track) in tracks.iter().enumerate() {
        let title = track.pointer("/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let url = track.pointer("/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");
        let artist = track.pointer("/artist/name").and_then(|x| x.as_str()).unwrap_or("N/A");
        let artist_url = track.pointer("/artist/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");

        let _ = write!(s, "`[{:02}]` **[{}]({})** by [{}]({})\n",
            i + 1, title, clean_url(url), artist, clean_url(artist_url));
    }

    send_embed(msg, None, &format!("{}'s Recently Loved Tracks", username), username, &s, first_image);
}

fn recent_tracks(msg: &Message, data: &Value, saved: bool) -> Result<(), CommandError> {
    let username = if let Some(username) = data.pointer("/recenttracks/@attr/user").and_then(|x| x.as_str()) {
        username
    } else {
        return Err(CommandError::from(get_msg!("error/fm_user_not_found")));
    };

    let last_track_artist = data.pointer("/recenttracks/track/0/artist/#text").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_name = data.pointer("/recenttracks/track/0/name").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_album = data.pointer("/recenttracks/track/0/album/#text").and_then(|x| x.as_str()).unwrap_or("N/A");
    let last_track_url = data.pointer("/recenttracks/track/0/url").and_then(|x| x.as_str()).unwrap_or("https://www.last.fm");

    // urlencode parenthesis
    let last_track_url = clean_url(last_track_url);

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
        .unwrap_or_else(|| Utc::now().naive_utc())
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
            .field("Artist - Song", format!("{} - [{}]({})", last_track_artist, last_track_name, last_track_url), false)
            .field("Album", last_track_album, true)
            .thumbnail(last_track_image)
            .footer(|f| f
                .text(format!("Total Tracks: {}", total_tracks))
            )
            .timestamp(last_track_timestamp.to_string())
        )
    });

    Ok(())
}

fn get_username(ctx: &Context, user: u64) -> Result<String, CommandError> {
    let pool = get_pool(ctx);
    pool.get_lastfm_username(user).ok_or_else(|| CommandError::from(get_msg!("error/fm_no_username")))
}

fn set_or_get_username(ctx: &Context, user: u64, args: &mut Args) -> Result<(String, bool), CommandError> {
    let pool = get_pool(ctx);

    let username_or_set = args.rest();
    let mut saved = false;

    let username = if username_or_set.starts_with("set ") {
        saved = true;
        // save to db
        pool.set_lastfm_username(user, &username_or_set.replace("set ", ""));

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
        match pool.get_lastfm_username(user) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/fm_no_username"))),
        }
    };

    Ok((username, saved))
}

fn get_data(url: &str, username: &str, period: &str) -> Result<Value, CommandError> {
    let fm_key = env::var("LASTFM_KEY").expect("Expected LASTFM_KEY to be set in environment");
    let url = url
        .replace("{USER}", username)
        .replace("{KEY}", &fm_key)
        .replace("{PERIOD}", period);

    // fetch data
    match reqwest::get(&url).and_then(|mut x| x.json()) {
        Ok(val) => Ok(val),
        Err(e) => {
            warn_discord!("[CMD:fm] Failed to fetch last.fm data: {}", e);
        
            Err(CommandError::from(get_msg!("error/fm_fetch_error")))
        }
    }
}

fn send_embed(msg: &Message, content: Option<&str>, title: &str, username: &str, desc: &str, thumbnail: &str) {
    let split_desc = desc.split("\n");
    let mut count = 0;
    let mut truncated_desc = String::new();

    for line in split_desc {
        if count + line.len() >= 2000 {
            break;
        }
        
        truncated_desc = format!("{}\n{}", truncated_desc, line);
        count = truncated_desc.len();
    }

    let _ = msg.channel_id.send_message(|m| {
        let mut m = m;

        if let Some(content) = content {
            m = m.content(content);
        }

        m.embed(|e| e
            .author(|a| a
                .name(title)
                .url(&format!("https://www.last.fm/user/{}", username))
                .icon_url("https://i.imgur.com/C7u8gqg.jpg")
            )
            .color(0xb90000)
            .description(&truncated_desc)
            .thumbnail(thumbnail)
        )
    });
}
