use serenity::model::prelude::*;
use serenity::prelude::Context;
use serenity::prelude::EventHandler;

use serenity::prelude::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

use plugins::*;
use tasks::*;

use serde_json::Value;

use utils::config::get_pool;
pub struct Handler;

/*
pub struct Modules(Vec<Box<EventHandler>>);

impl Modules {
    pub fn new() -> Self {
        Modules(Vec::new())
    }

    pub fn add_module<T: EventHandler + 'static>(&mut self, module: T) {
        self.0.push(Box::new(module))
    }

    fn propagate<F: Fn(&EventHandler)>(&self, name: &str, f: F) {
        debug!("Passing {}", name);
        for handler in &self.0 {
            f(handler);
        }
    }
}

impl EventHandler for Modules {
    fn ready(&self, ctx: Context, ready: Ready) {
        self.propagate("ready", |h| h.ready(ctx, ready));
    }
}

pub struct TestEcho;
impl EventHandler for TestEcho {
    fn message(&self, _ctx: Context, msg: Message) {
        let _ = msg.channel_id.say("hello");
    }
}*/

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        info_discord!(format!("READY: Connected as {}", ready.user.tag()));

        exec_on_ready!([&ctx, &ready], reminders, offline_check, bot_stats, bot_game, vlive);

        update_event(&ctx, "READY");
    }

    fn resume(&self, ctx: Context, resume: ResumedEvent) {
        info_discord!(format!("RESUMED: {:?}", resume));

        update_event(&ctx, "RESUMED");
    }

    fn channel_create(&self, ctx: Context, _: Arc<RwLock<GuildChannel>>) {
        update_event(&ctx, "CHANNEL_CREATE");
    }

    fn category_create(&self, ctx: Context, _: Arc<RwLock<ChannelCategory>>) {
        update_event(&ctx, "CATEGORY_CREATE")
    }

    fn category_delete(&self, ctx: Context, _: Arc<RwLock<ChannelCategory>>) {
        update_event(&ctx, "CATEGORY_DELETE")
    }

    fn private_channel_create(&self, ctx: Context, _: Arc<RwLock<PrivateChannel>>) {
        update_event(&ctx, "PRIVATE_CHANNEL_CREATE")
    }

    fn channel_delete(&self, ctx: Context, _: Arc<RwLock<GuildChannel>>) {
        update_event(&ctx, "CHANNEL_DELETE");
    }

    fn channel_pins_update(&self, ctx: Context, _: ChannelPinsUpdateEvent) {
        update_event(&ctx, "CHANNEL_PINS_UPDATE");
    }

    fn channel_recipient_addition(&self, ctx: Context, _: ChannelId, _: User) {
        update_event(&ctx, "CHANNEL_RECIPIENT_ADDITION")
    }

    fn channel_recipient_removal(&self, ctx: Context, _: ChannelId, _: User) {
        update_event(&ctx, "CHANNEL_RECIPIENT_REMOVAL")
    }

    fn channel_update(&self, ctx: Context, _: Option<Channel>, _: Channel) {
        update_event(&ctx, "CHANNEL_UPDATE");
    }

    fn guild_ban_addition(&self, ctx: Context, guild: GuildId, user: User) {
        exec_on_guild_ban_addition!([&ctx, &guild, &user], mod_log);

        {
            let pool = get_pool(&ctx);
            pool.update_stat("users", "users_banned", Some(1), None);
        }
        update_event(&ctx, "GUILD_BAN_ADD");
    }

    fn guild_ban_removal(&self, ctx: Context, guild: GuildId, user: User) {
        exec_on_guild_ban_removal!([&ctx, &guild, &user], mod_log);

        {
            let pool = get_pool(&ctx);
            pool.update_stat("users", "users_unbanned", Some(1), None);
        }
        update_event(&ctx, "GUILD_BAN_REMOVE");
    }

    fn guild_create(&self, ctx: Context, guild: Guild, is_new_guild: bool) {
        exec_on_guild_create!([&ctx, &guild, is_new_guild], db_cache);
        if is_new_guild {
            let owner_tag = match guild.owner_id.to_user() {
                Ok(user) => user.tag(),
                Err(_) => format!("<@{}>", guild.owner_id.0),
            };

            info_discord!("Joined new guild: {} - {} users - Owner: {} ({})", 
                guild.name, guild.member_count, owner_tag, guild.owner_id.0);
            
            {
                let pool = get_pool(&ctx);
                pool.update_stat("guilds", "guilds_joined", Some(1), None);
            }
        }

        update_event(&ctx, "GUILD_CREATE");
    }

    fn guild_delete(&self, ctx: Context, _: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {
        update_event(&ctx, "GUILD_DELETE");
    }

    fn guild_emojis_update(&self, ctx: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {
        update_event(&ctx, "GUILD_EMOJIS_UPDATE");
    }

    fn guild_integrations_update(&self, ctx: Context, _: GuildId) {
        update_event(&ctx, "GUILD_INTEGRATIONS_UPDATE");
    }

    fn guild_member_addition(&self, ctx: Context, guild: GuildId, mut member: Member) {
        exec_on_guild_member_addition!(
            [&ctx, &guild, &mut member],
            join_leave_message,
            member_log,
            mute_evasion,
            db_cache
        );

        {
            let pool = get_pool(&ctx);
            pool.update_stat("users", "users_added", Some(1), None);
        }
        update_event(&ctx, "GUILD_MEMBER_ADD");
    }

    fn guild_member_removal(
        &self,
        ctx: Context,
        guild: GuildId,
        user: User,
        member: Option<Member>,
    ) {
        exec_on_guild_member_removal!(
            [&ctx, &guild, &user, &member],
            join_leave_message,
            member_log,
            mute_evasion
        );

        {
            let pool = get_pool(&ctx);
            pool.update_stat("users", "users_removed", Some(1), None);
        }
        update_event(&ctx, "GUILD_MEMBER_REMOVE");
    }

    fn guild_member_update(&self, ctx: Context, prev_member: Option<Member>, curr_member: Member) {
        exec_on_guild_member_update!([&ctx, &prev_member, &curr_member], mod_log);
        update_event(&ctx, "GUILD_MEMBER_UPDATE");
    }

    fn guild_members_chunk(&self, ctx: Context, guild_id: GuildId, members: HashMap<UserId, Member>) {
        exec_on_guild_members_chunk!([&ctx, &guild_id, &members], db_cache);

        update_event(&ctx, "GUILD_MEMBERS_CHUNK")
    }

    fn guild_role_create(&self, ctx: Context, _: GuildId, _: Role) {
        update_event(&ctx, "GUILD_ROLE_CREATE");
    }

    fn guild_role_delete(&self, ctx: Context, _: GuildId, _: RoleId, _: Option<Role>) {
        update_event(&ctx, "GUILD_ROLE_DELETE");
    }

    fn guild_role_update(&self, ctx: Context, _: GuildId, _: Option<Role>, _: Role) {
        update_event(&ctx, "GUILD_ROLE_UPDATE");
    }

    fn guild_unavailable(&self, ctx: Context, _: GuildId) {
        update_event(&ctx, "GUILD_UNAVAILABLE")
    }

    fn guild_update(&self, ctx: Context, guild: Option<Arc<RwLock<Guild>>>, partial_guild: PartialGuild) {
        exec_on_guild_update!([&ctx, &guild, &partial_guild], db_cache);
        update_event(&ctx, "GUILD_UPDATE");
    }

    fn message(&self, ctx: Context, msg: Message) {
        update_event(&ctx, "MESSAGE_CREATE");
        
        {
            let pool = get_pool(&ctx);

            if msg.is_own() {
                pool.update_stat("messages", "messages_sent", Some(1), None);
            } else {
                pool.update_stat("messages", "messages_recieved", Some(1), None);
            }
        }

        exec_on_message!(
            [&ctx, &msg],
            user_info_activity,
            levels,
            random_hi,
            invite_guard,
            anti_spam,
            message_log,
            notifications,
            tags,
            gallery,
            roles,
            dm,
            pinged
        );
    }

    fn message_delete(&self, ctx: Context, channel_id: ChannelId, msg_id: MessageId) {
        exec_on_message_delete!([&ctx, &channel_id, &msg_id], message_log);
        update_event(&ctx, "MESSAGE_DELETE");
    }

    fn message_delete_bulk(&self, ctx: Context, _: ChannelId, _: Vec<MessageId>) {
        update_event(&ctx, "MESSAGE_DELETE_BULK");
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        update_event(&ctx, "MESSAGE_REACTION_ADD");

        exec_on_reaction_add!(
            [&ctx, &reaction],
            starboard
        );
    }

    fn reaction_remove(&self, ctx: Context, _reaction: Reaction) {
        update_event(&ctx, "MESSAGE_REACTION_REMOVE");
    }

    fn reaction_remove_all(&self, ctx: Context, _: ChannelId, _: MessageId) {
        update_event(&ctx, "MESSAGE_REACTION_REMOVE_ALL");
    }

    fn message_update(&self, ctx: Context, _old: Option<Message>, new: Message) {
        exec_on_message_update!([&ctx, &new], message_log);
        update_event(&ctx, "MESSAGE_UPDATE");
    }

    fn presence_replace(&self, ctx: Context, _: Vec<Presence>) {
        update_event(&ctx, "PRESENCE_REPLACE")
    }

    fn presence_update(&self, ctx: Context, presence_event: PresenceUpdateEvent) {
        exec_on_presence_update!([&ctx, &presence_event], db_cache);
        update_event(&ctx, "PRESENCE_UPDATE");
    }

    fn typing_start(&self, ctx: Context, _: TypingStartEvent) {
        update_event(&ctx, "TYPING_START");
    }

    fn unknown(&self, ctx: Context, _: String, _: Value) {
        update_event(&ctx, "UNKNOWN")
    }

    fn user_update(&self, ctx: Context, _: CurrentUser, _: CurrentUser) {
        update_event(&ctx, "USER_UPDATE")
    }

    fn voice_server_update(&self, ctx: Context, _: VoiceServerUpdateEvent) {
        update_event(&ctx, "VOICE_SERVER_UPDATE");
    }

    fn voice_state_update(&self, ctx: Context, _: Option<GuildId>, _: VoiceState) {
        update_event(&ctx, "VOICE_STATE_UPDATE");
    }

    fn webhook_update(&self, ctx: Context, _: GuildId, _: ChannelId) {
        update_event(&ctx, "WEBHOOK_UPDATE");
    }
}

/// Updates a counter for each event
fn update_event(ctx: &Context, event_name: &str) {
    let pool = get_pool(ctx);

    if let Err(e) = pool.log_event(event_name) {
        warn_discord!("Failed to log event: {}", e);
    }
}
