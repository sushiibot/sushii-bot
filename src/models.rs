use schema::*;

use chrono::naive::NaiveDateTime;
use diesel::sql_types::*;
use bigdecimal::BigDecimal;
use serde_json;

#[derive(Queryable, AsChangeset, Clone, Debug)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "guilds"]
pub struct GuildConfig {
    pub id: i64,
    pub name: Option<String>,
    pub join_msg: Option<String>,
    pub join_react: Option<String>,
    pub leave_msg: Option<String>,
    pub msg_channel: Option<i64>,
    pub role_channel: Option<i64>,
    pub role_config: Option<serde_json::Value>,
    pub invite_guard: Option<bool>,
    pub log_msg: Option<i64>,
    pub log_mod: Option<i64>,
    pub log_member: Option<i64>,
    pub mute_role: Option<i64>,
    pub prefix: Option<String>,
    pub max_mention: i32,
    pub disabled_channels: Option<Vec<i64>>,
}

#[derive(Insertable, Debug)]
#[table_name = "guilds"]
pub struct NewGuildConfig {
    pub id: i64,
    pub name: Option<String>,
    pub join_msg: Option<String>,
    pub join_react: Option<String>,
    pub leave_msg: Option<String>,
    pub msg_channel: Option<i64>,
    pub role_channel: Option<i64>,
    pub role_config: Option<serde_json::Value>,
    pub invite_guard: Option<bool>,
    pub log_msg: Option<i64>,
    pub log_mod: Option<i64>,
    pub log_member: Option<i64>,
    pub mute_role: Option<i64>,
    pub prefix: Option<String>,
    pub max_mention: i32,
    pub disabled_channels: Option<Vec<i64>>,
}

#[derive(Queryable, Debug)]
pub struct EventCounter {
    pub name: String,
    pub count: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "events"]
pub struct NewEventCounter<'a> {
    pub name: &'a str,
    pub count: i64,
}

#[derive(QueryableByName, Queryable, Clone, Debug)]
pub struct UserLevel {
    #[sql_type = "BigInt"]
    pub user_id: i64,

    #[sql_type = "BigInt"]
    pub guild_id: i64,

    #[sql_type = "BigInt"]
    pub msg_all_time: i64,

    #[sql_type = "BigInt"]
    pub msg_month: i64,

    #[sql_type = "BigInt"]
    pub msg_week: i64,

    #[sql_type = "BigInt"]
    pub msg_day: i64,

    #[sql_type = "Timestamp"]
    pub last_msg: NaiveDateTime,
}

// used for global ranks
#[derive(QueryableByName, Clone, Debug)]
pub struct UserLevelAllTime {
    #[sql_type = "BigInt"]
    pub user_id: i64,

    #[sql_type = "Numeric"]
    pub xp: BigDecimal,
}

#[derive(QueryableByName, Clone, Debug)]
pub struct UserLevelRanked {
    #[sql_type = "BigInt"]
    pub user_id: i64,

    #[sql_type = "BigInt"]
    pub guild_id: i64,

    #[sql_type = "BigInt"]
    pub msg_all_time: i64,

    #[sql_type = "BigInt"]
    pub msg_month: i64,

    #[sql_type = "BigInt"]
    pub msg_week: i64,

    #[sql_type = "BigInt"]
    pub msg_day: i64,

    #[sql_type = "Timestamp"]
    pub last_msg: NaiveDateTime,

    #[sql_type = "BigInt"]
    pub msg_day_rank: i64,
    #[sql_type = "BigInt"]
    pub msg_day_total: i64,

    #[sql_type = "BigInt"]
    pub msg_week_rank: i64,
    #[sql_type = "BigInt"]
    pub msg_week_total: i64,

    #[sql_type = "BigInt"]
    pub msg_month_rank: i64,
    #[sql_type = "BigInt"]
    pub msg_month_total: i64,

    #[sql_type = "BigInt"]
    pub msg_all_time_rank: i64,
    #[sql_type = "BigInt"]
    pub msg_all_time_total: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "levels"]
pub struct NewUserLevel<'a> {
    pub user_id: i64,
    pub guild_id: i64,
    pub msg_all_time: i64,
    pub msg_month: i64,
    pub msg_week: i64,
    pub msg_day: i64,
    pub last_msg: &'a NaiveDateTime,
}

// for leaderboards
#[derive(Debug)]
pub struct TopLevels {
    pub day: Option<Vec<UserLevel>>,
    pub week: Option<Vec<UserLevel>>,
    pub month: Option<Vec<UserLevel>>,
    pub all_time: Option<Vec<UserLevel>>,
}

#[derive(Queryable, Debug)]
pub struct Reminder {
    pub id: i32,
    pub user_id: i64,
    pub description: String,
    pub time_set: NaiveDateTime,
    pub time_to_remind: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "reminders"]
pub struct NewReminder<'a> {
    pub user_id: i64,
    pub description: &'a str,
    pub time_set: &'a NaiveDateTime,
    pub time_to_remind: &'a NaiveDateTime,
}

#[derive(Queryable, Debug)]
pub struct Notification {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub keyword: String,
}

#[derive(Insertable, Debug)]
#[table_name = "notifications"]
pub struct NewNotification<'a> {
    pub user_id: i64,
    pub guild_id: i64,
    pub keyword: &'a str,
}

#[derive(Queryable, AsChangeset, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub last_msg: NaiveDateTime,
    pub msg_activity: Vec<i32>,
    pub rep: i32,
    pub last_rep: Option<NaiveDateTime>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub lastfm: Option<String>,
    pub is_patron: bool,
    pub fishies: i64,
    pub last_fishies: Option<NaiveDateTime>,
    pub patron_emoji: Option<String>,
    pub profile_background_url: Option<String>,
    pub profile_bio: Option<String>,
    pub profile_bg_darken: Option<String>,
    pub profile_content_color: Option<String>,
    pub profile_content_opacity: Option<String>,
    pub profile_text_color: Option<String>,
    pub profile_accent_color: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: i64,
    pub last_msg: &'a NaiveDateTime,
    pub msg_activity: &'a Vec<i32>,
    pub rep: i32,
    pub last_rep: Option<&'a NaiveDateTime>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<&'a str>,
    pub lastfm: Option<&'a str>,
    pub is_patron: bool,
    pub fishies: i64,
    pub last_fishies: Option<&'a NaiveDateTime>,
    pub patron_emoji: Option<&'a str>,
}

#[derive(Queryable, AsChangeset, Clone, Debug)]
#[table_name = "mod_log"]
pub struct ModAction {
    pub id: i32,
    pub case_id: i32,
    pub guild_id: i64,
    pub executor_id: Option<i64>,
    pub user_id: i64,
    pub user_tag: String,
    pub action: String,
    pub reason: Option<String>,
    pub action_time: NaiveDateTime,
    pub msg_id: Option<i64>,
    pub pending: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "mod_log"]
pub struct NewModAction<'a> {
    pub case_id: i32,
    pub guild_id: i64,
    pub executor_id: Option<i64>,
    pub user_id: i64,
    pub user_tag: &'a str,
    pub action: &'a str,
    pub reason: Option<&'a str>,
    pub action_time: &'a NaiveDateTime,
    pub msg_id: Option<i64>,
    pub pending: bool,
}

#[derive(Queryable, Clone, Debug)]
pub struct Message {
    pub id: i64,
    pub author: i64,
    pub tag: String,
    pub channel: i64,
    pub guild: Option<i64>,
    pub created: NaiveDateTime,
    pub content: String,
}

#[derive(Insertable, Debug)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub id: i64,
    pub author: i64,
    pub tag: &'a str,
    pub channel: i64,
    pub guild: Option<i64>,
    pub created: NaiveDateTime,
    pub content: &'a str,
}

#[derive(Queryable, Clone, Debug)]
pub struct Mute {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "mutes"]
pub struct NewMute {
    pub user_id: i64,
    pub guild_id: i64,
}

#[derive(Queryable, Clone, Debug)]
pub struct Gallery {
    pub id: i32,
    pub watch_channel: i64,
    pub webhook_url: String,
    pub guild_id: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "galleries"]
pub struct NewGallery<'a> {
    pub watch_channel: i64,
    pub webhook_url: &'a str,
    pub guild_id: i64,
}

#[derive(Queryable, Clone, Debug)]
pub struct Tag {
    pub id: i32,
    pub owner_id: i64,
    pub guild_id: i64,
    pub tag_name: String,
    pub content: String,
    pub count: i32,
    pub created: NaiveDateTime,
}

impl Tag {
    pub fn is_owner(&self, id: u64) -> bool {
        id == self.owner_id as u64
    }
}

#[derive(Insertable, Debug)]
#[table_name = "tags"]
pub struct NewTag<'a> {
    pub owner_id: i64,
    pub guild_id: i64,
    pub tag_name: &'a str,
    pub content: &'a str,
    pub count: i32,
    pub created: &'a NaiveDateTime,
}


#[derive(Queryable, Clone, Debug)]
pub struct MemberEvent {
    pub id: i32,
    pub guild_id: i64,
    pub user_id: i64,
    pub event_name: String,
    pub event_time: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "member_events"]
pub struct NewMemberEvent<'a> {
    pub guild_id: i64,
    pub user_id: i64,
    pub event_name: &'a str,
    pub event_time: &'a NaiveDateTime,
}

// cache
#[derive(Queryable, Clone, Debug)]
pub struct CachedUser {
    pub id: i64,
    pub avatar: String,
    pub user_name: String,
    pub discriminator: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "cache_users"]
pub struct NewCachedUser<'a> {
    pub id: i64,
    pub avatar: &'a str,
    pub user_name: &'a str,
    pub discriminator: i32,
}

#[derive(Queryable, Clone, Debug)]
pub struct CachedChannel {
    pub id: i64,
    pub category_id: Option<i64>,
    pub guild_id: i64,
    pub kind: String,
    pub channel_name: String,
    pub position: i64,
    pub topic: Option<String>,
    pub nsfw: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "cache_channels"]
pub struct NewCachedChannel<'a> {
    pub id: i64,
    pub category_id: Option<i64>,
    pub guild_id: i64,
    pub kind: &'a str,
    pub channel_name: &'a str,
    pub position: i64,
    pub topic: Option<&'a str>,
    pub nsfw: bool,
}

#[derive(Queryable, Clone, Debug)]
pub struct CachedGuild {
    pub id: i64,
    pub guild_name: String,
    pub icon: Option<String>,
    pub member_count: i64,
    pub owner_id: i64,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "cache_guilds"]
pub struct NewCachedGuild<'a> {
    pub id: i64,
    pub guild_name: &'a str,
    pub icon: Option<&'a str>,
    pub member_count: i64,
    pub owner_id: i64,
}

#[derive(Queryable, Clone, Debug)]
pub struct CachedMember {
    pub user_id: i64,
    pub guild_id: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "cache_members"]
pub struct NewCachedMember {
    pub user_id: i64,
    pub guild_id: i64,
}

#[derive(Queryable, Clone, Debug)]
pub struct Stat {
    pub stat_name: String,
    pub count: i64,
    pub category: String,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "stats"]
pub struct NewStat<'a> {
    pub stat_name: &'a str,
    pub count: i64,
    pub category: &'a str,
}