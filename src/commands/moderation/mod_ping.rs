use rand::{self, Rng};
use serenity::model::Permissions;
use serenity::model::user::OnlineStatus;
use serenity::model::misc::Mentionable;
use serenity::model::guild::{Guild, Member};

// mostly from
// https://github.com/zeyla/nanobot/blob/master/src/commands/conversation.rs
command!(modping(_ctx, msg, _args) {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => return Ok(()),
    };

    let guild = guild.read();

    let found_mod = find_by_status(&*guild, OnlineStatus::Online)
        .or_else(|| find_by_status(&*guild, OnlineStatus::DoNotDisturb))
        .or_else(|| find_by_status(&*guild, OnlineStatus::Idle));

    let chosen_mod = match found_mod {
        Some(chosen_mod) => chosen_mod,
        None => {
            let _ = msg.channel_id.say("There are no online mods to ping.");

            return Ok(());
        },
    };

    let content = format!("{}, you were pinged for a mod action by **{}**.",
                        chosen_mod.mention(),
                        msg.author.tag());
    let _ = msg.channel_id.say(&content);
});

fn find_by_status(guild: &Guild, status: OnlineStatus) -> Option<&Member> {
    let required_perms = Permissions::BAN_MEMBERS
        | Permissions::KICK_MEMBERS
        | Permissions::MANAGE_MESSAGES;

    let mut members = guild.members.iter().filter(|&(user_id, member)| {
        if member.user.read().bot {
            return false;
        }

        if let Some(presence) = guild.presences.get(user_id) {
            if presence.status != status {
                return false;
            }
        } else {
            return false;
        }

        // Check if the member has at least one of the required permissions.
        match member.permissions() {
            Ok(perms) if perms.contains(required_perms) => true,
            _ => false,
        }
    })
        .map(|x| x.1)
        .collect::<Vec<_>>();

    rand::thread_rng().shuffle(&mut members[..]);

    if members.is_empty() {
        None
    } else {
        Some(members.remove(0))
    }
}