use serenity::model::channel::Message;
use serenity::model::id::RoleId;
use serenity::prelude::Context;

use std::vec::Vec;
use std::collections::HashMap;
use std::fmt::Write;

use regex::{Regex, RegexBuilder};

use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    // ignore self and bots
    if msg.author.bot {
        return;
    }

    let guild = match msg.guild() {
        Some(val) => val.read().clone(),
        None => return,
    };

    // get configs
    let config = check_res!(pool.get_guild_config(guild.id.0));
    let role_config = check_opt!(config.role_config);
    let role_channel = check_opt!(config.role_channel);

    // check if in correct channel
    if msg.channel_id.0 != role_channel as u64 {
        return;
    }

    // searching for multiple role assignments
    lazy_static! {
        static ref RE: Regex = RegexBuilder::new(r"(-|\+)([\w ]*)").case_insensitive(true).build().unwrap();
    }

    // actions and role search strings for further parsing
    let mut to_modify = Vec::new();

    // check if there are any matches or no
    let mut found = false;
    for caps in RE.captures_iter(&msg.content) {
        found = true;
        let action = caps.get(1).map_or("", |m| m.as_str());
        let target = caps.get(2).map_or("", |m| m.as_str());

        if action.is_empty() {
            // dont think this will ever be empty as if it isn't + or - then
            // the regex won't match?
            let _ = msg.channel_id.say(get_msg!("error/roles_invalid_action"));
            return;
        }

        if target.is_empty() {
            // don't think this will be empty either
            let _ = msg.channel_id.say(get_msg!("error/roles_invalid_target"));
            return;
        }

        // make individual cases for each word in each capture
        // Example: +role1 role2 role3
        // Each gets it's own separate entry as to preserve order
        let split = target.split(' ');

        for value in split {
            // to add / remove, string to search, position in message
            to_modify.push((action, value));
        }
    }

    // check if user wants to reset roles
    let should_reset = msg.content.to_lowercase() == "reset";

    // none found, exit
    if !found && !should_reset {
        return;
    }

    let member = match guild.member(msg.author.id) {
        Ok(val) => val,
        Err(e) => {
            warn!("[PLUGIN:roles] Failed to fetch guild member: {}", e);
            return;
        },
    };

    let role_config = check_opt!(role_config.as_object());


    // roles of the member, this is modified on role add / remove
    let mut current_roles = member.roles.iter().map(|x| x.0).collect::<Vec<u64>>();

    // hashmap grouped into category and roles,
    // this is used just for keeping track of roles in each category
    let mut member_roles: HashMap<String, Vec<u64>> = HashMap::new();

    // add the current member's roles
    for (cat_name, cat_data) in role_config.iter() {
        let roles = check_opt!(cat_data.get("roles").and_then(|x| x.as_object()));

        member_roles.insert(cat_name.to_string(), vec![]);

        for (_role_name, role_data) in roles.iter() {
            let primary = check_opt!(role_data.get("primary").and_then(|x| x.as_u64()));
            let secondary = check_opt!(role_data.get("secondary").and_then(|x| x.as_u64()));

            if current_roles.contains(&primary) {
                (*member_roles.get_mut(cat_name).unwrap()).push(primary);
            }

            if current_roles.contains(&secondary) {
                (*member_roles.get_mut(cat_name).unwrap()).push(secondary);
            }

            if should_reset {
                // remove the role
                if let Some(role_index) = current_roles.iter().position(|x| *x == primary || *x == secondary) {
                    current_roles.remove(role_index);
                }
            }
        }
    }


    let mut added_names = Vec::new();
    let mut removed_names = Vec::new();
    let mut errors = Vec::new();

    // check each role +/- action if we should use this
    // role
    for modify in &to_modify {
        // loop through each role
        'category: for (cat_name, cat_data) in role_config.iter() {
            let limit = check_opt!(cat_data.get("limit").and_then(|x| x.as_u64()));
            let roles = check_opt!(cat_data.get("roles").and_then(|x| x.as_object()));

            for (role_name, role_data) in roles.iter() {
                let search = check_opt!(role_data.get("search").and_then(|x| x.as_str()));
                let primary = check_opt!(role_data.get("primary").and_then(|x| x.as_u64()));
                let secondary = check_opt!(role_data.get("secondary").and_then(|x| x.as_u64()));

                
                // compile regex for search
                let re = match RegexBuilder::new(search).case_insensitive(true).build() {
                    Ok(val) => val,
                    Err(e) => {
                        let s = format!("Regex compile error for `{}` in `{}`: {}\nPlease fix in role config.", role_name, cat_name, e);
                        let _ = msg.channel_id.say(&s);
                        return;
                    }
                };

                if !re.is_match(modify.1) {
                    continue;
                }

                // if adding
                if modify.0 == "+" {
                    // check if over limit
                    if member_roles[cat_name].len() >= limit as usize && limit > 0 {
                        errors.push(format!("You can't add anymore roles in the `{}` category. (Limit: `{}`)", cat_name, limit));
                        continue 'category;
                    }

                    // check if already has the role
                    if member_roles[cat_name].iter().any(|x| *x == primary || *x == secondary) {
                        errors.push(format!("You already have the `{}` role.", role_name));
                        continue;
                    }

                    // add the role
                    if member_roles[cat_name].is_empty() || secondary == 0 {
                        // add primary role if it's first in it's category
                        // OR if secondary is set to 0, which is disabled
                        (*member_roles.get_mut(cat_name).unwrap()).push(primary);
                        // add to actual roles
                        current_roles.push(primary);
                        added_names.push(role_name.clone());
                    } else {
                        // add secondary if there's already existing roles in this category
                        (*member_roles.get_mut(cat_name).unwrap()).push(secondary);
                        current_roles.push(secondary);
                        added_names.push(role_name.clone());
                    }
                } else if let Some(index) = member_roles[cat_name].iter().position(|x| *x == primary || *x == secondary) {
                    // remove a role if member already has it
                    (*member_roles.get_mut(cat_name).unwrap()).remove(index);

                    // remove from actual roles
                    if let Some(role_index) = current_roles.iter().position(|x| *x == primary || *x == secondary) {
                        current_roles.remove(role_index);
                    }

                    removed_names.push(role_name.clone());
                } else {
                    errors.push(format!("You don't have the `{}` role.", role_name));
                }
            }
        }
    }

    let mut s = String::new();

    if !added_names.is_empty() {
        let _ = write!(s, "Added roles: {}\n", 
            added_names
            .iter()
            .map(|x| format!("`{}`", x))
            .collect::<Vec<String>>()
            .join(", ")
        );
    }
    
    if !removed_names.is_empty() {
        let _ = write!(s, "Removed roles: {}\n", 
            removed_names
            .iter()
            .map(|x| format!("`{}`", x))
            .collect::<Vec<String>>()
            .join(", ")
        );
    }

    if should_reset {
        s = "Your roles have been reset.".to_owned();
    } else if added_names.is_empty() && removed_names.is_empty() && errors.is_empty() {
        s = "Couldn't modify your roles\n".to_owned();
    }

    if !errors.is_empty() {
        let _ = write!(s, "There were errors while updating your roles:\n{}", errors.join(",\n"));
    }


    if let Err(e) = member.edit(|m| m.roles(current_roles.iter().map(|x| RoleId(*x)).collect::<Vec<RoleId>>())) {
        let _ = msg.channel_id.say(format!("Failed to edit your roles: {}", e));
    } else {
        let _ = msg.channel_id.say(&s);
    }
}
