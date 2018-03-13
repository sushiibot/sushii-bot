use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use serenity::model::channel::Message;

use serde_json::value::Value;
use serde_json::map::Map;
use reqwest;

use regex::Regex;
use std::collections::HashMap;

use utils;
use utils::user::*;
use utils::config::get_pool;
use utils::html::escape_html;

use models::{User, UserLevelRanked};

use num_traits::cast::ToPrimitive;

const PROFILE_HTML: &'static str = include_str!("../../../assets/html/profile.html");

command!(profile(ctx, msg, args) {
    let pool = get_pool(&ctx);

    let action = match args.single_n::<String>() {
        Ok(val) => {
            let subcommands = vec!["background", "bg", "bio", "bgdarkness", "contentcolor", "contentopacity", "textcolor", "accentcolor"];

            if !subcommands.contains(&val.as_ref()) {
                "profile".to_owned()
            } else {
                val
            }
        },
        Err(_) => "profile".to_owned(),
    };

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let id = if action == "profile" {
        match args.single::<String>() {
            Ok(val) => {
                match utils::user::get_id(&val) {
                    Some(id) => id,
                    None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
                }
            },
            Err(_) => msg.author.id.0,
        }
    } else {
        msg.author.id.0
    };

    let mut user_data = match pool.get_user(id) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/profile_user_not_found"))),
    };

    let mut profile_options = user_data.profile_options
        .clone()
        .and_then(|x| x.as_object().cloned())
        .unwrap_or(Map::new());
    
    let mut s = None;

    match action.as_ref() {
        "background" | "bg" => {
            return Err(CommandError::from("uhh not yet"));

            let _ = args.skip();
            let bg = match args.single::<String>() {
                Ok(val) => val,
                Err(_) => return Err(CommandError::from(get_msg!("error/profile_background_not_given"))),
            };
        },
        "bio" => {
            let _ = args.skip();
            let bio = args.full();

            if bio.is_empty() {
                return Err(CommandError::from(get_msg!("error/profile_bio_not_given")));
            }
            
            profile_options.insert("bio".to_owned(), json!(&bio));
            s = Some(get_msg!("info/profile_bio_set", bio));
        },
        "bgdarkness" => {
            let _ = args.skip();
            let darkness = match args.single::<f32>() {
                Ok(val) => val,
                Err(_) => return Err(CommandError::from(get_msg!("error/profile_invalid_opacity"))),
            };
            
            // check if in range
            if darkness < 0.0 || darkness > 1.0 {
                return Err(CommandError::from(get_msg!("error/profile_invalid_opacity")));
            }

            profile_options.insert("bg_darken".to_owned(), json!(darkness.to_string()));
            s = Some(get_msg!("info/profile_bg_darken_set", darkness));
        },
        "contentcolor" => {
            let _ = args.skip();
            let color = args.full();

            if color.is_empty() {
                return Err(CommandError::from(get_msg!("error/profile_contentcolor_not_given")));
            }

            let color = parse_number(&color, "rgb");

            if let Some(color) = color {
                profile_options.insert("content_color".to_owned(), json!(color.clone()));

                s = Some(get_msg!("info/profile_content_color_set", color));
            } else {
                return Err(CommandError::from(get_msg!("error/profile_invalid_color")));
            }
        },
        "contentopacity" => {
            let _ = args.skip();
            let opacity = match args.single::<f32>() {
                Ok(val) => val,
                Err(_) => return Err(CommandError::from(get_msg!("error/profile_invalid_opacity"))),
            };
            
            // check if in range
            if opacity < 0.0 || opacity > 1.0 {
                return Err(CommandError::from(get_msg!("error/profile_invalid_opacity")));
            }

            profile_options.insert("content_opacity".to_owned(), json!(opacity.to_string()));

            s = Some(get_msg!("info/profile_content_opacity_set", opacity));
        },
        "textcolor" => {
            let _ = args.skip();
            let color = args.full();

            if color.is_empty() {
                return Err(CommandError::from(get_msg!("error/profile_textcolor_not_given")));
            }

            let color = parse_number(&color, "hex");

            if let Some(color) = color {
                profile_options.insert("text_color".to_owned(), json!(color.clone()));

                s = Some(get_msg!("info/profile_text_color_set", color));
            } else {
                return Err(CommandError::from(get_msg!("error/profile_invalid_color")));
            }
        },
        "accentcolor" => {
            let _ = args.skip();

            let color = args.full();
            
            if color.is_empty() {
                return Err(CommandError::from(get_msg!("error/profile_accentcolor_not_given")));
            }

            let color = parse_number(&color, "hex");

            if let Some(color) = color {
                profile_options.insert("accent_color".to_owned(), json!(color.clone()));

                s = Some(get_msg!("info/profile_accent_color_set", color));
            } else {
                return Err(CommandError::from(get_msg!("error/profile_invalid_color")));
            }
        },
        _ => {},
    };

    user_data.profile_options = Some(Value::Object(profile_options));
    pool.save_user(&user_data);

    // doesn't match any subcommands, just look up profile
    let level_data = match pool.get_level(id, guild_id) {
        Some(level_data) => level_data,
        None => return Err(CommandError::from(get_msg!("error/level_no_data"))),
    };

    let global_xp = pool.get_global_xp(id).and_then(|x| x.to_i64()).unwrap_or(0);

    generate_profile(&msg, id, &user_data, &level_data, global_xp, s)?;
    pool.update_stat("profile", "profiles_generated", 1);
});

fn parse_number(val: &str, format: &str) -> Option<String> {
    if format == "rgb" {
        let (r, g, b) = if let Some(rgb) = parse_rgba(&val) {
            rgb
        } else if let Some(rgb) = hex_to_rgba(&val) {
            rgb
        } else {
            return None;
        };

        return Some(format!("{}, {}, {}", r, g, b));
    } else if format == "hex" {
        let hex = if let Some(hex) = parse_rgba(&val).and_then(|x| Some(rgba_to_hex(x))) {
            hex
        } else if let Some(hex) = parse_hex(&val) {
            hex
        } else {
            return None;
        };

        return Some(hex);
    }

    None
}

fn parse_rgba(val: &str) -> Option<(u32, u32, u32)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d{1,3}), ?(\d{1,3}), ?(\d{1,3})").unwrap();
    }

    if let Some(caps) = RE.captures(&val) {
        let r = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let g = caps.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let b = caps.get(3).unwrap().as_str().parse::<u32>().unwrap();

        // numbers given out of range
        if !in_range(r) || !in_range(g) || !in_range(b) {
            return None;
        }

        Some((r, g, b))
    } else {
        None
    }
}

fn parse_hex(val: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:[0-9a-fA-F]{3}){1,2}").unwrap();
    }

    RE.find(&val).and_then(|x| Some(x.as_str().to_string()))
}

fn in_range(num: u32) -> bool {
    num < 256
}

fn hex_to_rgba(val: &str) -> Option<(u32, u32, u32)> {
    // skip the first char if #
    let mut pos = if val.starts_with("#") {
        1
    } else {
        0
    };

    let r = u32::from_str_radix(&val[pos..pos + 2], 16).ok()?;
    pos += 2;
    let g = u32::from_str_radix(&val[pos..pos + 2], 16).ok()?;
    pos += 2;
    let b = u32::from_str_radix(&val[pos..pos + 2], 16).ok()?;

    Some((r, g, b))
}

fn rgba_to_hex(val: (u32, u32, u32)) -> String {
    format!("{:x}{:x}{:x}", val.0, val.1, val.2)
}

fn generate_profile(msg: &Message, id: u64, user_data: &User,   
        level_data: &UserLevelRanked, global_xp: i64, message: Option<String>) -> Result<(), CommandError> {

    let user_rep = user_data.rep.clone();
    let is_patron = user_data.is_patron.clone();
    let patron_emoji = user_data.patron_emoji.clone();
    let fishies = user_data.fishies.clone();

    // profiles
    let profile_options = user_data.profile_options
        .clone()
        .and_then(|x| x.as_object().cloned())
        .unwrap_or(Map::new());

    let background_url = profile_options.get("background_url").and_then(|x| x.as_str())
        .unwrap_or("https://cdn.discordapp.com/attachments/166974040798396416/420180917009645597/image.jpg");
    let bio = profile_options.get("bio").and_then(|x| x.as_str())
        .unwrap_or("Hey hey heyy");
    let bg_darken = profile_options.get("bg_darken").and_then(|x| x.as_str())
        .unwrap_or("0");
    
    // content color has to be rgba for transparency
    let content_color = profile_options.get("content_color").and_then(|x| x.as_str())
        .unwrap_or("73, 186, 255");
    let content_opacity = profile_options.get("content_opacity").and_then(|x| x.as_str())
        .unwrap_or("0.9");
    let text_color = profile_options.get("text_color").and_then(|x| x.as_str())
        .unwrap_or("ffffff");
    let accent_color = profile_options.get("accent_color").and_then(|x| x.as_str())
        .unwrap_or("ffffff");

    

    let user = match UserId(id).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };

    let _ = msg.channel_id.broadcast_typing();

    let mut html = PROFILE_HTML.to_owned();

    html = html.replace("{USERNAME}", &escape_html(&user.tag()));
    html = html.replace("{AVATAR_URL}", &user.face().replace("gif", "jpg"));
    html = html.replace("{BACKGROUND_URL}", &escape_html(&background_url));
    html = html.replace("{BIO}", &escape_html(&bio));
    html = html.replace("{DAILY}", &format_rank(&level_data.msg_day_rank, &level_data.msg_day_total));
    html = html.replace("{REP}", &user_rep.to_string());
    html = html.replace("{FISHIES}", &fishies.to_string());

    html = html.replace("{BACKGROUND_URL}", &background_url);
    html = html.replace("{BIO}", &bio);
    html = html.replace("{BG_DARKEN}", &bg_darken);
    html = html.replace("{CONTENT_COLOR}", &content_color);
    html = html.replace("{CONTENT_OPACITY}", &content_opacity);
    html = html.replace("{TEXT_COLOR}", &text_color);
    html = html.replace("{ACCENT_COLOR}", &accent_color);


    let global_level = get_level(global_xp);
    let level = get_level(level_data.msg_all_time);
    let last_level_total_xp_required = next_level(level);
    let next_level_total_xp_required = next_level(level + 1);
    
    let next_level_xp_required = next_level_total_xp_required - last_level_total_xp_required;
    let next_level_xp_remaining = next_level_total_xp_required - level_data.msg_all_time;
    let next_level_xp_progress = next_level_xp_required - next_level_xp_remaining;

    let xp_percentage = next_level_xp_progress as f64 / next_level_xp_required as f64;

    let xp_percentage = if xp_percentage > 1.0 {
        0.0
    } else {
        xp_percentage
    };

    html = html.replace("{LEVEL}", &level.to_string());
    html = html.replace("{GLOBAL_LEVEL}", &global_level.to_string());
    html = html.replace("{XP_PROGRESS}", &(xp_percentage * 326.72).to_string());


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
    let mut img = match client.post("http://127.0.0.1:3000/html").json(&json).send() {
        Ok(val) => val,
        Err(_) => {
           return Err(CommandError::from(get_msg!("error/profile_image_server_failed")))
        }
    };

    let mut buf: Vec<u8> = vec![];
    img.copy_to(&mut buf)?;

    let files = vec![(&buf[..], "profile.png")];

    let _ = msg.channel_id.send_files(files, |m| m.content(message.unwrap_or("".into())));

    Ok(())
}
