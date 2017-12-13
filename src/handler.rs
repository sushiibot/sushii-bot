use serenity::model::event::ResumedEvent;
use serenity::model::Ready;
use serenity::model::Message;
use serenity::prelude::*;

use database;

pub struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.tag());
    }

    fn on_resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    // fn on_channel_create(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}

    // fn on_category_create(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}

    // fn on_category_delete(&self, _: Context, _: Arc<RwLock<ChannelCategory>>) {}

    // fn on_private_channel_create(&self, _: Context, _: Arc<RwLock<PrivateChannel>>) {}

    // fn on_channel_delete(&self, _: Context, _: Arc<RwLock<GuildChannel>>) {}

    // fn on_channel_pins_update(&self, _: Context, _: ChannelPinsUpdateEvent) {}

    // fn on_channel_recipient_addition(&self, _: Context, _: ChannelId, _: User) {}

    // fn on_channel_recipient_removal(&self, _: Context, _: ChannelId, _: User) {}

    // fn on_channel_update(&self, _: Context, _: Option<Channel>, _: Channel) {}

    // fn on_guild_ban_addition(&self, _: Context, _: GuildId, _: User) {}

    // fn on_guild_ban_removal(&self, _: Context, _: GuildId, _: User) {}

    // fn on_guild_create(&self, _: Context, _: Guild, _: bool) {}

    // fn on_guild_delete(&self, _: Context, _: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {}

    // fn on_guild_emojis_update(&self, _: Context, _: GuildId, _: HashMap<EmojiId, Emoji>) {}

    // fn on_guild_integrations_update(&self, _: Context, _: GuildId) {}

    // fn on_guild_member_addition(&self, _: Context, _: GuildId, _: Member) {}

    // fn on_guild_member_removal(&self, _: Context, _: GuildId, _: User, _: Option<Member>) {}

    // fn on_guild_member_update(&self, _: Context, _: Option<Member>, _: Member) {}

    // fn on_guild_members_chunk(&self, _: Context, _: GuildId, _: HashMap<UserId, Member>) {}

    // fn on_guild_role_create(&self, _: Context, _: GuildId, _: Role) {}

    // fn on_guild_role_delete(&self, _: Context, _: GuildId, _: RoleId, _: Option<Role>) {}

    // fn on_guild_role_update(&self, _: Context, _: GuildId, _: Option<Role>, _: Role) {}

    // fn on_guild_unavailable(&self, _: Context, _: GuildId) {}

    // fn on_guild_update(&self, _: Context, _: Option<Arc<RwLock<Guild>>>, _: PartialGuild) {}

    fn on_message(&self, ctx: Context, _: Message) {
        let mut data = ctx.data.lock();
        let pool = data.get_mut::<database::ConnectionPool>().unwrap();

        pool.log_event("MESSAGE_CREATE");
    }

    // fn on_message_delete(&self, _: Context, _: ChannelId, _: MessageId) {}

    // fn on_message_delete_bulk(&self, _: Context, _: ChannelId, _: Vec<MessageId>) {}

    // fn on_reaction_add(&self, _: Context, _: Reaction) {}

    // fn on_reaction_remove(&self, _: Context, _: Reaction) {}

    // fn on_reaction_remove_all(&self, _: Context, _: ChannelId, _: MessageId) {}

    // fn on_message_update(&self, _: Context, _: MessageUpdateEvent) {}

    // fn on_presence_replace(&self, _: Context, _: Vec<Presence>) {}

    // fn on_presence_update(&self, _: Context, _: PresenceUpdateEvent) {}

    // fn on_typing_start(&self, _: Context, _: TypingStartEvent) {}

    // fn on_unknown(&self, _: Context, _: String, _: Value) {}

    // fn on_user_update(&self, _: Context, _: CurrentUser, _: CurrentUser) {}

    // fn on_voice_server_update(&self, _: Context, _: VoiceServerUpdateEvent) {}

    // fn on_voice_state_update(&self, _: Context, _: Option<GuildId>, _: VoiceState) {}

    // fn on_webhook_update(&self, _: Context, _: GuildId, _: ChannelId) {}
}
