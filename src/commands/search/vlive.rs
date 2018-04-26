use serenity::framework::standard::CommandError;
use reqwest::Client;
use vlive::ReqwestVLiveRequester;
use utils::numbers::comma_number;

command!(vlive(_ctx, msg, args) {
    let subcommand = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("vlive/error/missing_or_invalid_subcommand"))),
    };

    let query = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("vlive/error/missing_query"))),
    };

    let _ = msg.channel_id.broadcast_typing();

    let client = Client::new();

    match subcommand.as_ref() {
        "search" => {
            let channels = match client.get_channel_list() {
                Ok(val) => val,
                Err(why) => {
                    warn_discord!("Err searching vlive '{}': {:?}", query, why);

                    return Err(CommandError::from(get_msg!("vlive/error/failed_fetch_data")));
                },
            };

            let channel = match channels.find_channel(query) {
                Some(val) => val,
                None => return Err(CommandError::from(get_msg!("vlive/error/no_search_results"))),
            };

            let channel_code = match channel.code {
                Some(val) => val,
                None => return Err(CommandError::from(get_msg!("vlive/error/invalid_channel"))),
            };

            let channel_code = match client.decode_channel_code(channel_code) {
                Ok(val) => val,
                Err(e) => {
                    warn_discord!("Error decoding channel: {}", e);

                    return Err(CommandError::from(get_msg!("vlive/error/failed_fetch_data")));
                }
            };

            let channel_data = match client.get_channel_video_list(channel_code as u32, 10, 1) {
                Ok(val) => val,
                Err(e) => {
                    warn_discord!("Error decoding channel: {}", e);

                    return Err(CommandError::from(get_msg!("vlive/error/failed_fetch_data")));
                }
            };

            let channel_color = u64::from_str_radix(&channel_data.channel_info.representative_color.replace("#", ""), 16);

            let _ = msg.channel_id.send_message(|m| m
                .embed(|e| {
                    let mut e = e
                        .title(&format!("{}", channel_data.channel_info.channel_name))
                        .url(&channel_data.channel_info.url())
                        .thumbnail(&channel_data.channel_info.channel_profile_image)
                        .footer(|f| f
                            .text(&format!("{} channel fans", comma_number(channel_data.channel_info.fan_count.into())))
                        );
                    
                    if let Ok(color) = channel_color {
                        e = e.colour(color);
                    }

                    if let Some(video) = channel_data.video_list.first() {
                        e = e
                            .image(&video.thumbnail)
                            .field("Latest Video", &format!("**[{}]({})**", video.title, video.url()), false)
                            .field("Plays", &comma_number(video.play_count.into()), true)
                            .field("Hearts", &comma_number(video.like_count.into()), true)
                            .field("Comments", &comma_number(video.comment_count.into()), true)
                            .timestamp(video.on_air_start_at.to_rfc3339());
                    }

                    e
                })
            );
        },/*
        "upcoming" => {
            let mut response = match client.definitions(&query[..]) {
                Ok(response) => response,
                Err(why) => {
                    warn_discord!("Err retrieving word '{}': {:?}", query, why);

                    return Err(CommandError::from(get_msg!("error/urban_failed_retrieve")));
                },
            };

        },
        "latest" => {
            let mut response = match client.definitions(&query[..]) {
                Ok(response) => response,
                Err(why) => {
                    warn_discord!("Err retrieving word '{}': {:?}", query, why);

                    return Err(CommandError::from(get_msg!("error/urban_failed_retrieve")));
                },
            };

        },
        "video" => {
            let mut response = match client.definitions(&query[..]) {
                Ok(response) => response,
                Err(why) => {
                    warn_discord!("Err retrieving word '{}': {:?}", query, why);

                    return Err(CommandError::from(get_msg!("error/urban_failed_retrieve")));
                },
            };

        },*/
        _ => {
            return Err(CommandError::from(get_msg!("vlive/error/missing_or_invalid_subcommand")));
        }
    }
});
