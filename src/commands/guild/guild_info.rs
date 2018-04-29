use serenity::framework::standard::CommandError;
use serenity::model::user::OnlineStatus;

command!(guild_info(_ctx, msg, args) {
    // get the guild
    let guild = match msg.guild() {
        Some(val) => val.read().clone(),
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let owner_tag = match guild.owner_id.get() {
        Ok(val) => val.tag(),
        Err(_) => "N/A".into(),
    };

    let guild_presences = {
        //Online, Idle, DoNotDisturb, Offline
        let presences = guild.presences.values().fold((0, 0, 0), |mut acc, x| {
            match x.status {
                OnlineStatus::Online => acc.0 += 1,
                OnlineStatus::Idle => acc.1 += 1,
                OnlineStatus::DoNotDisturb => acc.2 += 1,
                _ => {},
            }

            acc
        });

        format!("{} online / {} idle / {} dnd",
            presences.0, presences.1, presences.2)
    };

    let voice_states = {
        // self_deaf, self_mute
        let states = guild.voice_states.values().fold((0, 0), |mut acc, x| {
            if x.self_deaf {
                acc.0 += 1;
            }

            if x.self_mute {
                acc.1 += 1;
            }

            acc
        });

        format!("{} total / {} deafened / {} muted",
            guild.voice_states.len(), states.0, states.1)
    };

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .author(|a| a
                .name(&guild.name)
                .url(&guild.icon_url().unwrap_or_else(|| "N/A".into()))
                .icon_url(&guild.icon_url().unwrap_or_else(|| "N/A".into()))
            )
            .thumbnail(&guild.icon_url().unwrap_or_else(|| "N/A".into()))
            .field("Owner", &format!("{} ({})", owner_tag, guild.owner_id), true)
            .field("Channels", &guild.channels.len().to_string(), true)
            .field("Emojis", &guild.emojis.len().to_string(), true)
            .field("Explicit Content Filter", &format!("{:?}", guild.explicit_content_filter), true)
            .field("Member Count", &guild.member_count.to_string(), true)
            .field("Region", &guild.region, true)
            .field("Roles", &guild.roles.len().to_string(), true)
            .field("Verification Level", &format!("{:?}", guild.verification_level), true)
            .field("Presences", &guild_presences, false)
            .field("Voice States", &voice_states, false)
            .footer(|f| f
                .text(&format!("Guild ID: {}", &guild.id.0))
            )
        )
    );
});
