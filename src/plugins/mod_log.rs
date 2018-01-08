use serenity::model::ChannelId;
use serenity::model::GuildId;
use serenity::model::UserId;
use serenity::model::Message;
use serenity::model::Member;
use serenity::model::User;
use serenity::prelude::Context;
use serenity::CACHE;
use serenity::Error;

use std::env;

use utils::config::get_pool;
use utils::time::now_utc;

use models::ModAction;
use models::GuildConfig;

use inflector::Inflector;


pub fn on_guild_ban_addition(ctx: &Context, guild: &GuildId, user: &User) {
    let pool = get_pool(&ctx);

    // check if a ban command was used instead of discord right click ban
    // add the action to the database if not pendings
    let mut db_entry = match pool.get_pending_mod_actions("ban", guild.0, user.id.0) {
        Some(val) => val,
        None => pool.add_mod_action("ban", guild.0, user, None, false, None),
    };

    let (tag, face) = get_user_tag_face(&db_entry);

    let config = pool.get_guild_config(guild.0);
    let reason = get_reason(&config, &db_entry);

    if let Some(channel) = config.log_mod {
        if let Ok(msg) = send_mod_action_msg(channel, &tag, &face, &user, "Ban", &reason, db_entry.case_id) {
            // edit the mod entry to have the mod log message id if successfull msg send
            db_entry.msg_id = Some(msg.id.0 as i64);
        }
        // if failed to send the message, it should be already set to None
    }

    db_entry.pending = false;

    pool.update_mod_action(db_entry);
}

// pub fn on_guild_ban_removal(ctx: &Context, guild: &GuildId, _: &User) {
//     let config = get_config_from_context(&ctx, guild.id.0);
//     let audits = match guild.audit_logs() {
//         Ok()
//     };
// 
// }

// handle mutes
pub fn on_guild_member_update(ctx: &Context, member_before: &Option<Member>, member: &Member) {
    let pool = get_pool(&ctx);

    let config = pool.get_guild_config(member.guild_id.0);

    // check if there is a mute role
    let mute_role = match config.mute_role {
        Some(val) => val,
        None => return,
    };

    let action;

    // check if there is a before member model, otherwise this is kind of useless
    let member_before = match member_before {
        &Some(ref val) => val,
        &None => return,
    };

    // check if a mute was added
    if let Some(_) = member.roles.iter().find(|&x| x.0 == mute_role as u64) {
        // found a mute role, let's check previous to see if the mute role caused the update
        if let None = (member_before).roles.iter().find(|&x| x.0 == mute_role as u64) {
            // previous member has no mute role, current does, so this is a mute action
            action = "mute";
        } else {
            return;
        }
    } else {
        // current has no mute role
        if let Some(_) = member_before.roles.iter().find(|&x| x.0 == mute_role as u64) {
            // previous member has mute role, this was an unmute action
            action = "unmute";
        } else {
            return;
        }
    }


    let user = member.user.read().unwrap();

    // check for pending mutes (automated or command mutes)
    let mut db_entry = match pool.get_pending_mod_actions(action, member.guild_id.0, user.id.0) {
        Some(val) => val,
        None => pool.add_mod_action(action, member.guild_id.0, &user, None, false, None),
    };

    let (tag, face) = get_user_tag_face(&db_entry);

    let config = pool.get_guild_config(member.guild_id.0);
    let reason = get_reason(&config, &db_entry);

    if let Some(channel) = config.log_mod {
        if let Ok(msg) = send_mod_action_msg(channel, &tag, &face, &user, &action.to_sentence_case(), &reason, db_entry.case_id) {
            // edit the mod entry to have the mod log message id if successfull msg send
            db_entry.msg_id = Some(msg.id.0 as i64);
        }
        // if failed to send the message, it should be already set to None
    }

    db_entry.pending = false;

    pool.update_mod_action(db_entry);
}


// get the tag and face of the executor if it exists,
// if getting the user fails, just fall back to the bot's tag / id
fn get_user_tag_face(db_entry: &ModAction) -> (String, String) {
    // get the tag and face of the executor if it exists,
    // if getting the user fails, just fall back to the bot's tag / id
    if let Some(executor) = db_entry.executor_id {
        if let Ok(user) = UserId(executor as u64).get() {
            (user.tag(), user.face())
        } else {
            let c = &CACHE.read().unwrap().user;

            (c.tag(), c.face())
        }
    } else {
        let c = &CACHE.read().unwrap().user;

        (c.tag(), c.face())
    }
}

// gets the reason or creates a default reason
fn get_reason(config: &GuildConfig, db_entry: &ModAction) -> String {
    let prefix = match config.prefix.clone() {
        Some(prefix) => prefix,
        None => env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.")
    };

    match db_entry.reason.clone() {
        Some(val) => val,
        None => format!("Responsible moderator: Please use `{}reason {} [reason]` to set a reason for this case.", prefix, db_entry.case_id)
    }
}

fn send_mod_action_msg(channel: i64, tag: &str, face: &str, user: &User, 
        action: &str, reason: &str, case_id: i32) -> Result<Message, Error> {

    ChannelId(channel as u64).send_message(|m| m
       .embed(|e| e
           .author(|a| a
               .name(tag)
               .icon_url(face)
           )
           .color(0xe74c3c)
           .field(|f| f
               .name("User")
               .value(format!("{} ({})", user.tag(), user.id.0))
               .inline(false)
           )
           .field(|f| f
               .name("Action")
               .value(action)
               .inline(false)
           )
           .field(|f| f
               .name("Reason")
               .value(reason)
               .inline(false)
           )
           .footer(|ft| ft
               .text(format!("Case #{}", case_id))
           )
           .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
       )
    )
}