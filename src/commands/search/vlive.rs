use serenity::framework::standard::CommandError;
use reqwest::Client;
use vlive::ReqwestVLiveRequester;
use utils::numbers::comma_number;
use regex::Regex;

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
            // channel list, maybe lazy_static this?
            let channels = match client.get_channel_list() {
                Ok(val) => val,
                Err(why) => {
                    warn_discord!("Err searching vlive '{}': {:?}", query, why);

                    return Err(CommandError::from(get_msg!("vlive/error/failed_fetch_data")));
                },
            };
            
            // search channel in list
            let channel = match channels.find_channel(query) {
                Some(val) => val,
                None => return Err(CommandError::from(get_msg!("vlive/error/no_search_results"))),
            };

            // get channel code
            let channel_code = match channel.code {
                Some(val) => val,
                None => return Err(CommandError::from(get_msg!("vlive/error/invalid_channel"))),
            };

            // fetch decoded channel code
            let channel_code = match client.decode_channel_code(channel_code) {
                Ok(val) => val,
                Err(e) => {
                    warn_discord!("Error decoding channel: {}", e);

                    return Err(CommandError::from(get_msg!("vlive/error/failed_fetch_data")));
                }
            };

            // get channel videos
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
        },
        "video" => {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"vlive\.tv/video/(\d+)").unwrap();
            }

            let video_seq = match RE.captures(&query)
                .and_then(|caps| caps.get(1))
                .map(|cap| cap.as_str())
                .and_then(|num| num.parse::<u32>().ok()) {
                Some(val) => val,
                None => return Err(CommandError::from(get_msg!("vlive/error/invalid_video"))),
            };

            let video = match client.get_video(video_seq) {
                Ok(val) => val,
                Err(why) => {
                    warn_discord!("Err searching vlive '{}': {:?}", query, why);

                    return Err(CommandError::from(get_msg!("vlive/error/failed_fetch_or_not_vod")));
                },
            };

            let best_video = match video.videos.list.first() {
                Some(val) => val,
                None => return Err(CommandError::from(get_msg!("vlive/error/no_videos"))),
            };

            let duration = {
                let minutes = best_video.duration as u64 / 60;
                let seconds = best_video.duration as u64 % 60;

                format!("{}min {}sec", minutes, seconds)
            };

            let eng_sub = video.captions.list
                .iter()
                .find(|x| x.language == "en")
                .map(|x| x.source.clone());
            
            let direct_links = if let Some(sub) = eng_sub {
                format!("[.mp4 Video]({}) - [English .vtt Subtitle]({})",
                    best_video.source, sub)
            } else {
                format!("[.mp4 Video]({}) (No English subtitles)", best_video.source)
            };

            let _ = msg.channel_id.send_message(|m| m
                .embed(|e| e
                    .title(&video.meta.subject)
                    .url(&video.meta.url)
                    .image(&video.meta.cover.source)
                    .field("Duration", &duration, true)
                    .field("Resolution", &best_video.encoding_option.name, true)
                    .field("File Size", &format!("{}MB", best_video.size / 1048576), true)
                    .field("Bitrate", &format!("Video: {}kbps\nAudio: {}kbps",
                        best_video.bitrate.video, best_video.bitrate.audio), true)
                    .field("Direct Links", &direct_links, false)
                )
            );
        },
        _ => {
            return Err(CommandError::from(get_msg!("vlive/error/missing_or_invalid_subcommand")));
        }
    }
});
