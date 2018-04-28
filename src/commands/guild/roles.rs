use serenity::framework::standard::CommandError;
use serenity::model::id::RoleId;
use std::collections::HashMap;
use std::fmt::Write;


command!(roleinfo(_ctx, msg, args) {
    // get the guild
    let guild = match msg.guild() {
        Some(val) => val.read().clone(),
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let search = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("guild/roles/error/no_role_name"))),
    };

    let role = match guild.roles.values().find(|ref x| x.name.to_lowercase() == search.to_lowercase()) {
        Some(val) => val,
        None => return Err(CommandError::from(get_msg!("guild/roles/error/no_role_found")))
    };

    let hoisted = if role.hoist {
        "Yes"
    } else {
        "No"
    };

    let mentionable = if role.mentionable {
        "Yes"
    } else {
        "No"
    };

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title(format!("Role info for {} ({})", role.name, role.id))
            .colour(role.colour)
            .field("Hoisted", hoisted, true)
            .field("Mentionable", mentionable, true)
            .field("Position", &role.position.to_string(), true)
            .field("Color", &format!("#{:X}", role.colour.0), true)
            .field("Permissions", &role.permissions.bits().to_string(), true)
        )
    );
});

command!(rolestats(_ctx, msg, _args) {
    let guild = match msg.guild() {
        Some(val) => val.read().clone(),
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let mut roles_map = HashMap::new();

    // count each role
    for role in guild.members.values().flat_map(|x| x.roles.iter()) {
        if let Some(roleid) = roles_map.get_mut(&role) {
            *roleid += 1;
            continue;
        }

        roles_map.insert(role, 1);
    }

    // convert hashmap to a vec
    let mut roles_vec: Vec<(RoleId, u64)> = roles_map
        .iter()
        .map(|(&&id, &count)| (id.clone(), count) )
        .collect();

    roles_vec.sort_by(|a, b| b.1.cmp(&a.1)); // sort by count
    roles_vec.truncate(10); // limit to 10 roles

    let mut s = String::new();

    for (roleid, count) in roles_vec {
        if let Some(role) = guild.roles.get(&roleid) {
            let _ = write!(s, "`{}` - {}\n", count, role.name);
        }
    }

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title(format!("Role stats for {}", guild.name))
            .description(&s)
        )
    );
});
