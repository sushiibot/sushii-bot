
use serenity::framework::standard::CommandError;

// use std::fmt::Write;


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
