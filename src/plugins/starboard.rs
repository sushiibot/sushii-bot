use serenity::model::channel::Reaction;
use serenity::model::channel::ReactionType;
use serenity::builder::CreateEmbed;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;
use models::StarredMessage;
use database::ConnectionPool;


pub fn on_reaction_add(_ctx: &Context, pool: &ConnectionPool, reaction: &Reaction) {
    // checks cache first, so minimal api requests
    let guild_id = match reaction.channel_id
        .to_channel()
        .ok()
        .and_then(|channel| channel.guild())
        .map(|guild_channel| guild_channel
            .read()
            .guild_id.0
        )
         {
        Some(id) => id,
        None => return,
    };

    // get starboard, db call cheaper than rest api call to get message
    let starboard = match pool.get_starboard(guild_id) {
        Ok(s) => s,
        Err(_) => return, // silent return, don't want to error on reacts when some guilds don't have it set
    };

    let message = match reaction.message() {
        Ok(m) => m,
        Err(e) => {
            warn_discord!(format!("[STARBOARD] Failed to fetch reaction message: {:?}", e));
            return;
        }
    };

    // ignore messages by bots
    if message.author.bot {
        return;
    }

    // reaction doesn't match, some other emoji
    match reaction.emoji {
        ReactionType::Custom {id, ..} => {
            if let Some(emoji_id) = starboard.emoji_id {
                if id.0 as i64 != emoji_id {
                    return;
                }
            } else {
                return;
            }
        },
        ReactionType::Unicode(ref emoji) => {
            if *emoji != starboard.emoji {
                return;
            }
        },
    };

    let count = match message.reactions
        .iter()
        .find(|reaction| {
            match reaction.reaction_type {
                ReactionType::Custom {id, ..} => id.0 == starboard.emoji_id.unwrap_or(0) as u64,
                ReactionType::Unicode(ref emoji) => *emoji == starboard.emoji,
            }
        })
        .map(|reaction| reaction.count) {
            Some(count) => count,
            None => {
                warn_discord!("[STARBOARD] Couldn't find matching reaction");
                return;
            }
        };
    
    if count < starboard.minimum as u64 {
        return;
    }

    let mut starred_message = match pool.get_starred_message(reaction.message_id.0) {
        Some(m) => m,
        None => {
            StarredMessage {
                orig_message_id: message.id.0 as i64, // user message
                message_id: 0, // starboard embed message
                author_id: message.author.id.0 as i64,
                guild_id: guild_id as i64,
                channel_id: message.channel_id.0 as i64,
                created: message.timestamp.naive_utc(),
                count: 0,
            }
        }
    };

    // no change, possibly same user removed and re-added so ignore
    if starred_message.count == count as i64 && count != 0 {
        return;
    }

    starred_message.count = count as i64;

    // starboard embed not sent yet
    if starred_message.message_id == 0 {
        let sent_starred_message = match ChannelId(starboard.channel as u64).send_message(|m| m
            .embed(|e| {
                let mut e = e
                .author(|a| a
                    .name(&message.author.tag())
                    .icon_url(&message.author.face())
                )
                .color(0xffc938)
                .description(&message.content)
                .timestamp(message.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string())
                .field(
                    "\u{200B}", // zws
                    &format!(
                        "{} {} <#{}> Jump to message:\nhttp://discordapp.com/channels/{}/{}/{}", // guild, channel, message
                        starboard.emoji, starred_message.count, // emoji, count

                        message.channel_id.0, // channel mention

                        starboard.guild_id,
                        message.channel_id.0,
                        message.id.0,
                    ),
                    true
                )
                .footer(|f| f
                    .text(&format!("ID {}", message.id.0))
                );

                if !message.attachments.is_empty() {
                    e = e
                        .image(message.attachments
                            .first()
                            .map(|attachment| attachment.url.clone())
                            .unwrap() // checked if empty so should be fine to unwrap?
                        );
                }

                e
            })
        ) {
            Ok(m) => m,
            Err(e) => {
                warn_discord!(format!("[STARBOARD] Failed to send starred message: {:?}", e));
                return;
            }
        };

        // set the starboard message 
        starred_message.message_id = sent_starred_message.id.0 as i64;
    } else {
        // already an embed sent, edit previous one
        let mut starboard_message = match ChannelId(starboard.channel as u64).message(starred_message.message_id as u64) {
            Ok(msg) => msg,
            Err(e) => {
                warn_discord!(format!("[STARBOARD] Failed to fetch starred message: {:?}", e));
                return;
            }
        };

        let mut embed = match starboard_message.embeds.get(0) {
            Some(val) => val.clone(),
            None => return, // shouldn't really be empty but oh well?
        };

        // edit star count in footer
        /*
        embed.footer = Some(EmbedFooter {
            text: format!("{} {}", starboard.emoji, starred_message.count),
            icon_url: None,
            proxy_icon_url: None,
        });
        */

        // edit star count in field
        if let Some(field) = embed.fields.first_mut() {
            field.value = format!(
                "{} {} <#{}> Jump to message:\nhttp://discordapp.com/channels/{}/{}/{}", // guild, channel, message
                starboard.emoji, starred_message.count, // emoji, count

                message.channel_id.0, // channel mention

                starboard.guild_id,
                message.channel_id.0,
                message.id.0,
            );
        }

        // edit the starboarded message embed
        let _ = starboard_message.edit(|m| m.embed(|_| CreateEmbed::from(embed.clone())));
    }

    // TODO: fix "race" condition here, in case fast reacts, starboard message won't be saved in the > min reacts
    // sends duplicate starboard messages but newest works fine. 
    // Can't just move to before embed send since it needs the embed message ID
    if let Err(e) = pool.update_starred_message(&starred_message) {
        warn_discord!(format!("[STARBOARD] Failed to update starred message: {:?}", e));
    }
}

/*
pub fn on_reaction_remove(ctx: &Context, pool: &ConnectionPool, reaction: &Reaction) {

}
*/