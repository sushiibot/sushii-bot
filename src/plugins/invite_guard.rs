use serenity::model::Message;
use serenity::prelude::Context;
use serenity::CACHE;

use utils::config::get_config_from_context;
use regex::Regex;

pub fn on_message(ctx: &Context, msg: &Message) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read().unwrap();

        let current_user_id = CACHE.read().unwrap().user.id;

        // return if bot doesn't have delete perms
        if !guild.member_permissions(current_user_id).manage_messages() {
            return;
        }

        // return if bot sent the message, not sure why this would happen
        if msg.author.id == current_user_id {
            return;
        }

        // check the guild config if inviteguard is enabled
        let invite_guard = match get_config_from_context(&ctx, guild.id.0).invite_guard {
            Some(val) => val,
            None => return,
        };

        if invite_guard {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(discord(\.gg/|app\.com/invite/))").unwrap();
            }

            // allow those with perms to bypass
            if guild.member_permissions(msg.author.id).manage_guild() {
                return;
            }

            if RE.is_match(&msg.content) {
                if let Err(e) = msg.delete() {
                    error!("Error while deleting invite, {}", e);
                }
            }
        }
    }
}
