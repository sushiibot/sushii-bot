use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use serenity::framework::standard::CommandError;
use serenity::model::user::OnlineStatus;

command!(guild_info(_ctx, msg, _args) {
    // get the guild
    let guild = match msg.guild() {
        Some(g) => g,
        None => return Err(CommandError::from(get_msg!("error/no_guild"))),
    };

    let owner_tag = match guild.read().owner_id.to_user() {
        Ok(val) => val.tag(),
        Err(_) => "N/A".into(),
    };

    let guild_presences = {
        //Online, Idle, DoNotDisturb, Offline
        let presences = guild.read().presences.values().fold((0, 0, 0), |mut acc, x| {
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
        let states = guild.read().voice_states.values().fold((0, 0), |mut acc, x| {
            if x.self_deaf {
                acc.0 += 1;
            }

            if x.self_mute {
                acc.1 += 1;
            }

            acc
        });

        format!("{} total / {} deafened / {} muted",
            guild.read().voice_states.len(), states.0, states.1)
    };

    let current_time = Utc::now();
    let created_at = guild.read().id.created_at();
    let created_duration = current_time.signed_duration_since(DateTime::<Utc>::from_utc(created_at, Utc));
    let created_humanized = format!("{:#}", HumanTime::from(created_duration)).replace("in ", "");

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .author(|a| a
                .name(&guild.read().name)
                .url(&guild.read().icon_url().unwrap_or_else(|| "N/A".into()))
                .icon_url(&guild.read().icon_url().unwrap_or_else(|| "N/A".into()))
            )
            .thumbnail(&guild.read().icon_url().unwrap_or_else(|| "N/A".into()))
            .field("Owner", &format!("{} ({})", owner_tag, guild.read().owner_id), true)
            .field("Channels", &guild.read().channels.len().to_string(), true)
            .field("Emojis", &guild.read().emojis.len().to_string(), true)
            .field("Explicit Content Filter", &format!("{:?}", guild.read().explicit_content_filter), true)
            .field("Member Count", &guild.read().member_count.to_string(), true)
            .field("Region", &guild.read().region, true)
            .field("Roles", &guild.read().roles.len().to_string(), true)
            .field("Verification Level", &format!("{:?}", guild.read().verification_level), true)
            .field("Presences", &guild_presences, false)
            .field("Voice States", &voice_states, false)
            .field("Created At", &format!("{}\n{}", created_at.format("%Y-%m-%dT%H:%M:%S"), created_humanized), false)
            .footer(|f| f
                .text(&format!("Guild ID: {}", &guild.read().id.0))
            )
        )
    );
});
