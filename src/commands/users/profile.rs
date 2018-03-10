use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use serenity::model::channel::Message;
use reqwest;
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
        Ok(val) => val,
        Err(_) => "profile".to_owned(),
    };

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    match action.as_ref() {
        "background" => {},
        "bio" => {},
        "bg_darken" => {
            let _ = args.skip();
            let value = match args.single::<String>() {
                Ok(val) => val,
                Err(_) => return Err(CommandError::from(get_msg!("error/invalid_bg_darkness"))),
            };
        },
        "content_color" => {},
        "content_opacity" => {},
        "text_color" => {},
        "accent_color" => {},
        "profile" | _ => {
            let id = match args.single::<String>() {
                Ok(val) => {
                    match utils::user::get_id(&val) {
                        Some(id) => id,
                        None => return Err(CommandError::from(get_msg!("error/invalid_user"))),
                    }
                },
                Err(_) => msg.author.id.0,
            };

            let user_data = pool.get_user(id);
            let level_data = match pool.get_level(id, guild_id) {
                Some(level_data) => level_data,
                None => return Err(CommandError::from(get_msg!("error/level_no_data"))),
            };

            let global_xp = pool.get_global_xp(id).and_then(|x| x.to_i64()).unwrap_or(0);

            generate_profile(&msg, id, &user_data, &level_data, global_xp)?;
            pool.update_stat("profile", "profiles_generated", 1);
        }
    };
});

// fn hex_to_rgba(val: &str) -> String {
//     
// }

fn generate_profile(msg: &Message, id: u64, user_data: &Option<User>,   
        level_data: &UserLevelRanked, global_xp: i64) -> Result<(), CommandError> {

    let user_rep;
    let is_patron;
    let patron_emoji;
    let fishies;

    let background_url;
    let bio;
    let bg_darken;
    let content_color;
    let content_opacity;
    let text_color;
    let accent_color;

    if let &Some(ref val) = user_data {
        user_rep = val.rep.clone();
        is_patron = val.is_patron.clone();
        patron_emoji = val.patron_emoji.clone();
        fishies = val.fishies.clone();
        // profiles
        background_url = val.profile_background_url.clone()
            .unwrap_or("https://cdn.discordapp.com/attachments/166974040798396416/420180917009645597/image.jpg".to_owned());
        bio = val.profile_bio.clone()
            .unwrap_or("Hey hey heyy".to_owned());
        bg_darken = val.profile_bg_darken.clone()
            .unwrap_or("0".to_owned());
        
        // content color has to be rgba for transparency
        content_color = val.profile_content_color.clone()
            .unwrap_or("73, 186, 255".to_owned());
        content_opacity = val.profile_content_opacity.clone()
            .unwrap_or("0.9".to_owned());
        text_color = val.profile_text_color.clone()
            .unwrap_or("ffffff".to_owned());
        accent_color = val.profile_accent_color.clone()
            .unwrap_or("ffffff".to_owned());
    } else {
        return Err(CommandError::from(get_msg!("error/profile_user_not_found")))
    }

    let user = match UserId(id).get() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_user"))),
    };

    let _ = msg.channel_id.broadcast_typing();

    let mut html = PROFILE_HTML.to_owned();

    html = html.replace("{USERNAME}", &escape_html(&user.tag()));
    html = html.replace("{AVATAR_URL}", &user.face());
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

    let xp_percentage = ((next_level_xp_progress as f64 / next_level_xp_required as f64) * 100.0) as u64;

    let xp_percentage = if xp_percentage > 100 {
        0
    } else {
        xp_percentage
    };

    html = html.replace("{LEVEL}", &level.to_string());
    html = html.replace("{GLOBAL_LEVEL}", &global_level.to_string());
    html = html.replace("{XP_PROGRESS}", &xp_percentage.to_string());


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

    let _ = msg.channel_id.send_files(files, |m| m.content(""));

    Ok(())
}
