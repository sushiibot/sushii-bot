use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::CACHE;

use regex::Regex;
use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        let current_user_id = CACHE.read().user.id;

        // return if bot doesn't have delete perms
        if !guild.member_permissions(current_user_id).manage_messages() {
            return;
        }

        // return if bot sent the message, not sure why this would happen
        if msg.author.id == current_user_id {
            return;
        }

        // check the guild config if inviteguard is enabled
        let invite_guard = match check_res!(pool.get_guild_config(guild.id.0)).invite_guard {
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
