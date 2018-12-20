use serenity::prelude::Context;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use reqwest::Client;
use vlive::ReqwestVLiveRequester;
use vlive::model::channel::ChannelType;
use utils::numbers::comma_number;
use models::VliveChannel;
use chrono::Utc;

use std::{thread, time, vec::Vec};
use std::sync::{Once, ONCE_INIT};

use database;

static INIT: Once = ONCE_INIT;

pub fn on_ready(ctx: &Context, _: &Ready) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap().clone();

    INIT.call_once(|| {
        debug!("Spawning vlive thread");
        thread::spawn(move || loop {
            let start = Utc::now();

            let channels = match pool.get_vlive_channels() {
                Ok(val) => val,
                Err(e) => {
                    warn_discord!("[DB:get_vlive_channels] Failed to get vlive channels: {}", e);
                    continue;
                },
            };

            let mut vlive_channels: Vec<i32> = channels
                .iter()
                .map(|x| x.channel_seq)
                .collect();
            
            // dedupe channels, requires vec to be sorted first
            vlive_channels.sort();
            vlive_channels.dedup();

            let client = Client::new();

            // fetch videos / upcoming videos for each channel
            for &channel_seq in &vlive_channels {
                // get channel videos
                let channel_data = match client.get_channel_video_list(channel_seq as u32, 10, 1) {
                    Ok(val) => val,
                    Err(e) => {
                        warn!("Error decoding channel: {}", e);
                        continue; // skip errored
                    }
                };

                let is_channel_plus = match channel_data.channel_info.channel_plus_type {
                    ChannelType::PREMIUM => true,
                    _ => false,
                };

                let old_videos = match pool.get_vlive_videos(channel_seq) {
                    Ok(val) => val,
                    Err(e) => {
                        warn_discord!("Error fetching vlive channel videos: {}", e);
                        continue;
                    }
                };                

                let old_video_seqs: Vec<i32> = old_videos.iter().map(|x| x.video_seq).collect();

                let mut new_videos = Vec::new();
                // check for new videos
                for video_data in &channel_data.video_list {
                    if !old_video_seqs.contains(&(video_data.video_seq as i32)) {
                        new_videos.push(video_data);
                    }
                }

                // get the discord channels to send new videos to
                let target_channels: Vec<&VliveChannel> = channels
                    .iter()
                    .filter(|x| x.channel_seq == channel_seq)
                    .collect();
                
                let channel_color = u64::from_str_radix(&channel_data.channel_info.representative_color.replace("#", ""), 16);
                
                for video in new_videos {
                    // save video to db
                    // saving channel_seq kind of useless? could just save
                    // video_seq since they're unique anyways
                    pool.add_vlive_video(channel_seq, video.video_seq as i32);

                    // ignore non channel+ videos for channel+ channels
                    if is_channel_plus && !video.channel_plus_public_yn {
                        continue;
                    }

                    // also ignore channel+ videos for regular channels
                    if !is_channel_plus && video.channel_plus_public_yn {
                        continue;
                    }

                    // ignore v pick
                    if video.title.starts_with(&format!("[{}]", channel_data.channel_info.channel_name)) {
                        continue;
                    }

                    let mut video_data_res = client.get_video(video.video_seq);

                    let mut is_subbed = None;
                    let mut highest_resolution = None;

                    if let Ok(mut video_data) = video_data_res {
                        if let Some(ref captions) = video_data.captions {
                            // check if eng subbed
                            let subbed = captions.list.iter().any(|x| x.language == "en");

                            if subbed {
                                is_subbed = Some(":white_check_mark: **Yes**");
                            } else {
                                is_subbed = Some(":x: No");
                            }
                        } else {
                            is_subbed = Some(":x: No");
                        }

                        // sort videos by size
                        video_data.videos.list.sort_by(|a, b| 
                            b.size.cmp(&a.size)
                        );

                        let best = video_data.videos.list.first();
                        if let Some(best_video) = best {
                            highest_resolution = Some(best_video.encoding_option.name.clone());
                        }
                    }

                    // send messages
                    for channel in &target_channels {
                        let mention = if let Some(role) = channel.mention_role {
                            format!("<@&{}>", role)
                        } else {
                            "".into()
                        };

                        let live_emoji_or_vod = if video.video_type == "LIVE" {
                            "<:live:441734958025801730>"
                        } else {
                            "[VOD]"
                        };

                        if let Err(e) = ChannelId(channel.discord_channel as u64).send_message(|m| m
                            .content(&mention)
                            .embed(|e| {
                                let mut e = e
                                    .author(|a| a
                                        .name(&format!("{} - New VLive", channel_data.channel_info.channel_name))
                                        .icon_url("https://i.imgur.com/NzGrmho.jpg")
                                        .url(&channel_data.channel_info.url())
                                    )
                                    .title(&format!("{} {}", live_emoji_or_vod, video.title))
                                    .url(&video.url())
                                    .thumbnail(&channel_data.channel_info.channel_profile_image)
                                    .image(&video.thumbnail)
                                    .field("Plays", &comma_number(video.play_count.into()), true)
                                    .field("Hearts", &comma_number(video.like_count.into()), true)
                                    .field("Comments", &comma_number(video.comment_count.into()), true)
                                    .timestamp(video.on_air_start_at.to_rfc3339());
                                
                                if let Ok(color) = channel_color {
                                    e = e.colour(color);
                                }
                                
                                if let Some(is_subbed) = is_subbed {
                                    e = e.field("English Subtitles", &is_subbed, true);
                                }

                                if let Some(ref res) = highest_resolution {
                                    e = e.field("Resolution", res, true);
                                }

                                if video.channel_plus_public_yn {
                                    e = e.description("<:channel_plus:441720556212060160> **Requires CHANNEL+ subscription**");
                                }

                                e
                            })
                        ) {
                            warn_discord!(format!("[VLIVE] Failed to send VLive notification: {:?}", e));
                        }
                    }
                }
            }

            let end = Utc::now();
            let ms = {
                let end_ms = i64::from(end.timestamp_subsec_millis());
                let start_ms = i64::from(start.timestamp_subsec_millis());

                end_ms - start_ms
            };
            let diff = ((end.timestamp() - start.timestamp()) * 1000) + ms;
            debug!("Updated {} VLive channels, took {} ms", vlive_channels.len(), diff);

            // calculate sleep time, makes sure these updates are fetched every 30 seconds,
            // when the time reaches exactly xx:00 or xx:30
            let interval_secs = 30;
            let now_secs = end.timestamp();
            let delay_secs = interval_secs - now_secs % interval_secs;
            let delay = time::Duration::from_secs(delay_secs as u64);

            debug!("Current time: {} ({}), new delay is {} seconds", start.to_rfc3339(), now_secs, delay_secs);
            thread::sleep(delay);
        });
    });
}
