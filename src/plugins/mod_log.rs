use serenity::model::ChannelId;
use serenity::model::GuildId;
use serenity::model::User;
use serenity::prelude::Context;
use serenity::CACHE;

use std::env;

use utils::config::get_pool;
use utils::time::now_utc;


pub fn on_guild_ban_addition(ctx: &Context, guild: &GuildId, user: &User) {
    let pool = get_pool(&ctx);

    // check if a ban command was used instead of discord right click ban
    // add the action to the database if not pendings
    let mut db_entry = match pool.get_pending_mod_actions("ban", guild.0, user.id.0) {
        Some(val) => val,
        None => pool.add_mod_action("ban", guild.0, user, None, false),
    };

    let current_user = &CACHE.read().unwrap().user;

    // send a mod action message to the mod_log in discord chat,
    // if there is a channel set in the settings
    let config = pool.get_guild_config(guild.0);
    let prefix = match pool.get_prefix(guild.0) {
        Some(prefix) => prefix,
        None => {
            env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment.")
        }
    };

    if let Some(channel) = config.log_mod {
        if let Ok(msg) = ChannelId(channel as u64).send_message(|m| m
            .embed(|e| e
                .author(|a| a
                    .name(&current_user.tag())
                    .icon_url(&current_user.face())
                )
                .color(0xe74c3c)
                .field(|f| f
                    .name("User")
                    .value(format!("{} ({})", user.tag(), user.id.0))
                    .inline(false)
                )
                .field(|f| f
                    .name("Action")
                    .value("Ban")
                    .inline(false)
                )
                .field(|f| f
                    .name("Reason")
                    .value(format!("Responsible moderator: Please use `{}reason {} [reason]` to set a reason for this case.", prefix, db_entry.case_id))
                    .inline(false)
                )
                .footer(|ft| ft
                    .text(format!("Case #{}", db_entry.case_id))
                )
                .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        ) {
            // edit the mod entry to have the mod log message id if successfull msg send
            db_entry.msg_id = Some(msg.id.0 as i64);
        }
        // if failed to send the message, it should be already set to None
    }

    

    pool.update_mod_action(db_entry);
}

// pub fn on_guild_ban_removal(ctx: &Context, guild: &GuildId, _: &User) {
//     let config = get_config_from_context(&ctx, guild.id.0);
//     let audits = match guild.audit_logs() {
//         Ok()
//     };
// 
// }
