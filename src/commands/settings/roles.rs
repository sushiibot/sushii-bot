use serenity::framework::standard::CommandError;
use serenity::utils::parse_channel;
use serenity::utils::parse_role;
use serenity::model::guild::Role;

use serde_json;

use std::fmt::Write;
use std::collections::HashSet;
use utils::config::*;

fn validate_roles_config(cfg: &serde_json::Map<String, serde_json::Value>, guild_roles: HashSet<u64>) -> String {
    let mut s = String::new();
    for (cat_name, cat_data) in cfg.iter() {
        // check if there's a roles field
        if let Some(lim) = cat_data.get("limit") {
            if !lim.is_number() {
                let _ = write!(s, "Category limit for `{}` has to be a number\n", cat_name);
            }
        } else {
            let _ = write!(s, "Missing category limit for `{}`, set to 0 to disable\n", cat_name);
        }
        // check if there is a roles field
        if let Some(roles) = cat_data.get("roles") {
            // check if roles is an object
            if let Some(obj) = roles.as_object() {
                // check if roles object is empty
                if obj.is_empty() {
                    let _ = write!(s, "Roles for `{}` cannot be empty\n", cat_name); 
                }
                // check if each role has correct properties
                for (role_name, role_data) in obj.iter() {
                    // search
                    if let Some(val) = role_data.get("searches") {
                        if !val.is_array() {
                            let _ = write!(s, "Field `searches` for role `{}` in category `{}` must be an array of strings\n", 
                                role_name, cat_name);
                        } else {
                            if let Some(arr) = val.as_array() {
                                if arr.is_empty() {
                                    let _ = write!(s, "Field `searches` for role `{}` in category `{}` cannot be empty\n", 
                                        role_name, cat_name);
                                } else {
                                    if arr.first().and_then(|x| x.as_str()).is_none() {
                                        let _ = write!(s, "Field `searches` for role `{}` in category `{}` must be an array of strings\n", 
                                            role_name, cat_name);
                                    }
                                }
                            }
                        }
                    } else {
                        let _ = write!(s, "Role `{}` in category `{}` is missing field `search`\n", 
                            role_name, cat_name);
                    }

                    // primary
                    if let Some(val) = role_data.get("primary") {
                        if let Some(id) = val.as_u64() {
                            if !guild_roles.contains(&id) && id != 0 {
                                let _ = write!(s, "Field `primary` for role `{}` in category `{}` is not a valid role ID\n", 
                                    role_name, cat_name);
                            }
                        } else {
                            let _ = write!(s, "Field `primary` for role `{}` in category `{}` has to be a number (Role ID)\n", 
                                role_name, cat_name);
                        }
                    } else {
                        let _ = write!(s, "Role `{}` in category `{}` is missing field `primary`\n", 
                            role_name, cat_name);
                    }

                    // secondary
                    if let Some(val) = role_data.get("secondary") {
                        if let Some(id) = val.as_u64() {
                            if !guild_roles.contains(&id) && id != 0 {
                                let _ = write!(s, "Field `secondary` for role `{}` in category `{}` is not a valid role ID\n", 
                                    role_name, cat_name);
                            }
                        } else {
                            let _ = write!(s, "Field `secondary` for role `{}` in category `{}` has to be a number (Role ID)\n", 
                                role_name, cat_name);
                        }
                    } else {
                        let _ = write!(s, "Role `{}` in category `{}` is missing field `secondary`\n", 
                            role_name, cat_name);
                    }
                }
            } else {
                let _ = write!(s, "Roles in category `{}` are not configured properly as an object\n", cat_name);
            }
        } else {
            let _ = write!(s, "Missing roles for category `{}`\n", cat_name);
        }
    }

    s
}

command!(roles_set(ctx, msg, args) {
    let mut raw_json = args.rest().to_owned();
    let guild = match msg.guild() {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    // check if it starts with a code block
    if raw_json.starts_with("```") && raw_json.ends_with("```") {
        // remove code block from string
        raw_json = raw_json.replace("```json", "");
        raw_json = raw_json.replacen("```", "", 2);
    }

    if raw_json.is_empty() && !msg.attachments.is_empty() {
        let bytes = match msg.attachments[0].download() {
            Ok(content) => content,
            Err(e) => return Err(CommandError::from(e)),
        };

        raw_json = match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(e) => return Err(CommandError::from(e)),
        };
    } else if raw_json.is_empty() && msg.attachments.is_empty() {
        // no message or attachment 
        return Err(CommandError::from(get_msg!("error/no_config_given")));
    }

    let role_config: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(&raw_json) {
        Ok(val) => val,
        Err(e) => return Err(CommandError::from(e)),
    };

    let guild_roles: HashSet<u64> = guild
        .read()
        .roles
        .keys()
        .map(|role_id| role_id.0)
        .collect();

    let validated = validate_roles_config(&role_config, guild_roles);
    let errs = if validated.len() > 1950 {
        format!("{}\n... and more", &validated[..1950])
    } else {
        validated
    };

    if !errs.is_empty() {
        return Err(CommandError::from(errs));
    }

    let pool = get_pool(ctx);

    let mut config = check_res_msg!(get_config(ctx, &pool, guild.read().id.0));
    config.role_config = Some(serde_json::Value::from(role_config));

    update_config(ctx, &pool, &config);

    let _ = msg.channel_id.say(get_msg!("info/role_config_set"));
});

command!(roles_channel(ctx, msg, args) {
    let channel = match args.single::<String>() {
        Ok(val) => parse_channel(&val).unwrap_or(0),
        Err(_) => return Err(CommandError::from(get_msg!("error/no_channel_given"))),
    };

    if channel == 0 {
        return Err(CommandError::from(get_msg!("error/invalid_channel")));
    }

    if let Some(guild_id) = msg.guild_id() {
        let pool = get_pool(ctx);

        let mut config = check_res_msg!(get_config(&ctx, &pool, guild_id.0));

        config.role_channel = Some(channel as i64);

        update_config(ctx, &pool, &config);

        let s = get_msg!("info/role_channel_set", channel);
        let _ = msg.channel_id.say(&s);
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});

command!(roles_get(ctx, msg, _args) {
    if let Some(guild_id) = msg.guild_id() {
        let config = check_res_msg!(get_config_from_context(ctx, guild_id.0));

        if let Some(role_config) = config.role_config {
            let roles_pretty = match serde_json::to_string_pretty(&role_config) {
                Ok(val) => val,
                Err(e) => return Err(CommandError::from(e)),
            };

            let s = format!("```json\n{}\n```", roles_pretty);
            let _ = msg.channel_id.say(&s);
        } else {
            return Err(CommandError::from(get_msg!("error/no_role_config")))
        }
    }
});


command!(mute_role(ctx, msg, args) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let role = match args.single::<String>() {
            Ok(val) => val,
            Err(e) => return Err(CommandError::from(e)),
        };

        let role_id = parse_role(&role)
            .or_else(|| guild.roles.values().find(|&x| x.name == role).map(|x| x.id.0));

        if let Some(id) = role_id {
            let pool = get_pool(ctx);

            let mut config = check_res_msg!(get_config(ctx, &pool, guild.id.0));
            config.mute_role = Some(id as i64);

            update_config(ctx, &pool, &config);

            let s = get_msg!("info/mute_role_set", id);
            let _ = msg.channel_id.say(&s);
        } else {
            return Err(CommandError::from(get_msg!("error/invalid_role")));
        }
    }    
});

command!(list_ids(_ctx, msg, _args) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let mut roles_text = String::new();

        let mut roles = guild.roles.values().collect::<Vec<&Role>>();
        roles.sort_by(|&a, &b| b.position.cmp(&a.position));

        for role in &roles {
            let _ = write!(roles_text, "[{:02}] {} - {}\n", role.position, role.id.0, role.name);
        }

        // check if over limit, send a text file instead
        if roles_text.len() >= 2000 {
            let files = vec![(roles_text.as_bytes(), "roles.txt")];
            
            let _ = msg.channel_id.send_files(files, |m| m.content(get_msg!("info/list_ids_attached")));
        } else {
            let s = format!("Server roles:\n```ruby\n{}```", roles_text);

            let _ = msg.channel_id.say(&s);
        }
    } else {
        return Err(CommandError::from(get_msg!("error/no_guild")));
    }
});