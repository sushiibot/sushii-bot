use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use serenity::model::channel::Message;
use serenity::CACHE;
use serde_json;
use std::collections::HashMap;
use std::fmt::Write;
use utils::config::get_pool;

use CommandsList;

command!(tag_info(ctx, msg, args) {
    let tag_name = match args.single::<String>() {
        Ok(val) => val.to_lowercase(),
        Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_name_given"))),
    };

    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let found_tag = match pool.get_tag(guild_id.0, &tag_name) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("tags/error/not_found", tag_name))),
        };

        let (user_tag, user_face) = match UserId(found_tag.owner_id as u64).to_user() {
            Ok(val) => {
                (val.tag(), val.face())
            },
            Err(_) => {
                // fallback to default
                let c = &CACHE.read().user;

                (c.tag(), c.face())
            }
        };

        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .author(|a| a
                    .name(&user_tag)
                    .icon_url(&user_face)
                )
                .field("Name", &found_tag.tag_name, true)
                .field("Content", &found_tag.content, true)
                .field("Use Count", &found_tag.count.to_string(), true)
                .field("Owner", &format!("<@{}>", found_tag.owner_id), true)
                .footer(|f| f
                    .text("Created on")
                )
                .timestamp(found_tag.created.format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_add(ctx, msg, args) {
    let tag_name = match args.single::<String>() {
        Ok(val) => val.to_lowercase(),
        Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_name_given"))),
    };

    // check if conflicts with core commands
    {
        let data = ctx.data.lock();
        let cmds = data.get::<CommandsList>().unwrap();

        if cmds.contains_key(&tag_name) {
            return Err(CommandError::from(get_msg!("tags/error/command_conflict")));
        }
    }

    let mut tag_content = args.rest();

    // check if tag content is given or no
    if tag_content.is_empty() {
        if let Some(attachment) = msg.attachments.first() {
            // empty content but attached image
            tag_content = &attachment.url;
        } else {
            // no content
            return Err(CommandError::from(get_msg!("tags/error/no_content_given")));
        }
    }

    let pool = get_pool(ctx);

    // if in guild
    if let Some(guild_id) = msg.guild_id {
        // check if tag exists
        if pool.get_tag(guild_id.0, &tag_name).is_some() {
            // theres already a tag with this name found
            return Err(CommandError::from(get_msg!("tags/error/already_exists")));
        } else if pool.add_tag(msg.author.id.0, guild_id.0, &tag_name, tag_content) {
            let _ = msg.channel_id.say(get_msg!("tags/added", &tag_name, &tag_content));
        } else {
            return Err(CommandError::from(get_msg!("error/unknown_error")));
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_list(ctx, msg, _args) {
    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);
        let tags = match pool.get_tags(guild_id.0) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("error/tags_not_found"))),
        };

        if tags.is_empty() {
            return Err(CommandError::from(get_msg!("tags/error/none")))
        }

        let mut contents = String::new();
        for tg in tags {
	        let _ = write!(contents, "{}\n", tg.tag_name);
	    }
	
	    let dm = match msg.author.create_dm_channel() {
	        Ok(val) => val,
	        Err(e) => {
                warn_discord!(format!("[TAGS] Failed to send DM: {:?}", e));
	            let _ = msg.channel_id.say(get_msg!("error/failed_dm"));
	            return Ok(());
	        }
	    };
	
	    let messages = split_message(&contents, Some("Available Tags:"), true);
	
	    for msg in messages {
	        let _ = dm.say(&msg);
	    }
	
	    if !msg.is_private() {
	        let _ = msg.channel_id.say(get_msg!("tags/list"));
	    }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});


command!(tag_top(ctx, msg, _args) {
    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let top_tags = match pool.get_tags_top(guild_id.0) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("tags/error/no_tags"))),
        };

        if top_tags.is_empty() {
            return Err(CommandError::from(get_msg!("tags/error/none")))
        }

        let mut s = String::new();

        for tg in top_tags {
            let _ = write!(s, "{} - {}\n", tg.count, tg.tag_name);
        }

        let _ = msg.channel_id.send_message(|m| m
            .embed(|e| e
                .author(|a| a
                    .name("Most Used Tags")
                )
                .description(&s)
                .footer(|f| f
                    .text("Use Count - Tag Name")
                )
            )
        );

    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_search(ctx, msg, args) {
    if let Some(guild_id) = msg.guild_id {
        let search = match args.single::<String>() {
            Ok(val) => val.to_lowercase(),
            Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_search_given"))),
        };

        if search.len() < 3 {
            return Err(CommandError::from(get_msg!("tags/error/search_query_too_short")));
        }

        let pool = get_pool(ctx);

        if let Some(results) = pool.search_tag(guild_id.0, &search) {
            let _ = msg.channel_id.send_message(|m| m
                .embed(|e| {
                    let mut e = e;

                    e = e.author(|a| a
                        .name("Tag Search (Limited to 10)")
                    );

                    for tg in &results {
                        e = e.field(&tg.tag_name, &tg.count.to_string(), false);
                    }

                    e
                })
            );
        } else {
            return Err(CommandError::from(get_msg!("tags/error/no_tags")));
        }

        
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_delete(ctx, msg, args) {
    if let Some(guild_id) = msg.guild_id {
        let tag_name = match args.single::<String>() {
            Ok(val) => val.to_lowercase(),
            Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_name_given"))),
        };

        let pool = get_pool(ctx);

        // get the current tag to check owner
        let current = match pool.get_tag(guild_id.0, &tag_name) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("tags/error/not_found", tag_name))),
        };

        // check if user owns the tag or has mod perms
        if !current.is_owner(msg.author.id.0) && !has_permission(msg) {
            return Err(CommandError::from(get_msg!("tags/error/no_permission")))
        }

        if pool.delete_tag(guild_id.0, &tag_name) {
            let _ = msg.channel_id.say(get_msg!("tags/deleted", tag_name));
        } else {
            return Err(CommandError::from(get_msg!("tags/error/not_found_or_not_owner")));
        }

    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_rename(ctx, msg, args) {
    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let tag_name = match args.single::<String>() {
            Ok(val) => val.to_lowercase(),
            Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_name_given"))),
        };

        let tag_new_name = match args.single::<String>() {
            Ok(val) => val.to_lowercase(),
            Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_new_name_given"))),
        };

        if tag_name == tag_new_name {
            return Err(CommandError::from(get_msg!("tags/error/name_unchanged")));
        }

        // get the current tag to check owner
        let current = match pool.get_tag(guild_id.0, &tag_name) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("tags/error/not_found", tag_name)))
        };

        // check if new name exists
        if pool.get_tag(guild_id.0, &tag_new_name).is_some() {
            return Err(CommandError::from(get_msg!("tags/error/already_exists")));
        }

        // check if user owns the tag or has mod perms
        if !current.is_owner(msg.author.id.0) && !has_permission(msg) {
            return Err(CommandError::from(get_msg!("tags/error/no_permission")))
        }

        // edit only name
        if pool.edit_tag(guild_id.0, &tag_name, &tag_new_name, &current.content) {
            let _ = msg.channel_id.say(get_msg!("tags/renamed", &tag_name, &tag_new_name));
        } else {
            return Err(CommandError::from(get_msg!("tags/error/not_found_or_not_owner")));
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_edit(ctx, msg, args) {
    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let tag_name = match args.single::<String>() {
            Ok(val) => val.to_lowercase(),
            Err(_) => return Err(CommandError::from(get_msg!("tags/error/no_name_given"))),
        };

        let tag_content = args.rest();

        // check if tag content is given or no
        if tag_content.is_empty() {
            return Err(CommandError::from(get_msg!("tags/error/no_content_given")));
        }

        // get the current tag to check owner
        let current = match pool.get_tag(guild_id.0, &tag_name) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("tags/error/not_found", tag_name)))
        };

        // check if content was changed
        if current.content == tag_content {
            return Err(CommandError::from(get_msg!("tags/error/content_unchanged")));
        }

        // check if user owns the tag or has mod perms
        if !current.is_owner(msg.author.id.0) && !has_permission(msg) {
            return Err(CommandError::from(get_msg!("tags/error/no_permission")));
        }

        if pool.edit_tag(guild_id.0, &tag_name, &tag_name, tag_content) {
            let _ = msg.channel_id.say(get_msg!("tags/edited_content", &tag_name, &tag_content));
        } else {
            return Err(CommandError::from(get_msg!("tags/error/not_found_or_not_owner")));
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_random(ctx, msg, _args) {
    if let Some(guild_id) = msg.guild_id {
        let pool = get_pool(ctx);

        let found_tag = match pool.get_random_tag(guild_id.0) {
            Some(val) => val,
            None => return Err(CommandError::from(get_msg!("tags/error/none"))),
        };

        let _ = msg.channel_id.say(&format!("\u{200b}{}: {}", found_tag.tag_name, found_tag.content));
        // update the counter
        pool.increment_tag(guild_id.0, &found_tag.tag_name);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(tag_import(ctx, msg, args) {
    let guild_id = match msg.guild_id {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let mut raw_json = args.rest().to_string();

    // try reading file
    if raw_json.is_empty() && msg.attachments.len() > 0 {
        let json_bytes = msg.attachments[0].download()?;
        raw_json = String::from_utf8(json_bytes)?;
    }

    let imported: HashMap<String, String> = serde_json::from_str(&raw_json)?;

    let pool = get_pool(ctx);

    let _ = msg.react("👌");

    let _ = msg.channel_id.say(get_msg!("tags/import_started"));
    let mut existing = 0;
    let mut existing_tags = vec![];

    for (tag_name, tag_content) in imported.iter() {
        if !pool.get_tag(guild_id.0, &tag_name).is_some() {
            pool.add_tag(msg.author.id.0, guild_id.0, &tag_name, &tag_content);
        } else {
            existing += 1;
            existing_tags.push(tag_name.clone());
        }
    }

    if existing > 0 {
        let _ = msg.channel_id.say(get_msg!("tags/import_existing", existing_tags.len(), existing_tags.join(", ")));
    } else {
        let _ = msg.channel_id.say(get_msg!("tags/import_finished"));
    }
});

// splits a string that might be too long
pub fn split_message(msg: &str, prepend: Option<&str>, with_code_block: bool) -> Vec<String> {
    let split = msg.split('\n');
    let mut vec = Vec::new();
    let mut single_msg = String::new();

    // add text in beginning before code blocks
    if let Some(val) = prepend {
        single_msg.push_str(val);
    }

    if with_code_block {
        single_msg.push_str("\n```"); // add starting code block
    }

    for s in split {
        single_msg.push_str("\n"); // re-add new line at end

        // will overflow, move to next msg (in bytes) + 6 just in case?
        if single_msg.len() + s.len() + 6 > 4000 {
            // add closing code block
            if with_code_block {
                single_msg.push_str("```");
            }

            vec.push(single_msg.clone());   // push the full string to vec
            single_msg.clear();     // reset single message

            // start new code block 
            if with_code_block {
                single_msg.push_str("```\n");
            }
        }

        // append the next line
        single_msg.push_str(s);
    }

    // push the remaining string
    if !single_msg.is_empty() {
        if with_code_block {
            single_msg.push_str("```"); // add closing code block
        }

        vec.push(single_msg);
    }

    vec
}

fn has_permission(msg: &Message) -> bool {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => {
            warn!("Couldn't get message guild!");

            return false;
        }
    };
    let guild = guild.read();

    // fetch member
    let member = match guild.members.get(&msg.author.id) {
        Some(member) => member,
        None => return false
    };
    // check if has perm
    if let Ok(permissions) = member.permissions() {
        return permissions.manage_guild();
    } else {
        return false;
    }
}