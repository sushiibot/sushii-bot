use schema::*;

use chrono::naive::NaiveDateTime;
use diesel::sql_types::*;
use serde_json;

#[derive(Queryable, AsChangeset, Clone)]
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
}

#[derive(Insertable)]
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
}

#[derive(Queryable)]
pub struct EventCounter {
    pub name: String,
    pub count: i64,
}

#[derive(Insertable)]
#[table_name = "events"]
pub struct NewEventCounter<'a> {
    pub name: &'a str,
    pub count: i64,
}

#[derive(Queryable, Clone)]
pub struct UserLevel {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub msg_all_time: i64,
    pub msg_month: i64,
    pub msg_week: i64,
    pub msg_day: i64,
    pub last_msg: NaiveDateTime,
}

#[derive(QueryableByName, Clone)]
pub struct UserLevelRanked {
    #[sql_type = "Integer"]
    pub id: i32,

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

#[derive(Insertable)]
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
pub struct TopLevels {
    pub day: Option<Vec<UserLevel>>,
    pub week: Option<Vec<UserLevel>>,
    pub month: Option<Vec<UserLevel>>,
    pub all_time: Option<Vec<UserLevel>>,
}

#[derive(Queryable)]
pub struct Reminder {
    pub id: i32,
    pub user_id: i64,
    pub description: String,
    pub time_set: NaiveDateTime,
    pub time_to_remind: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "reminders"]
pub struct NewReminder<'a> {
    pub user_id: i64,
    pub description: &'a str,
    pub time_set: &'a NaiveDateTime,
    pub time_to_remind: &'a NaiveDateTime,
}

#[derive(Queryable)]
pub struct Notification {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub keyword: String,
}

#[derive(Insertable)]
#[table_name = "notifications"]
pub struct NewNotification<'a> {
    pub user_id: i64,
    pub guild_id: i64,
    pub keyword: &'a str,
}

#[derive(Queryable, Clone)]
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
}

#[derive(Insertable)]
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
}

#[derive(Queryable, AsChangeset, Clone)]
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

#[derive(Insertable)]
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

#[derive(Queryable, Clone)]
pub struct Message {
    pub id: i64,
    pub author: i64,
    pub tag: String,
    pub channel: i64,
    pub guild: Option<i64>,
    pub created: NaiveDateTime,
    pub content: String,
}

#[derive(Insertable)]
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

#[derive(Queryable, Clone)]
pub struct Mute {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
}

#[derive(Insertable)]
#[table_name = "mutes"]
pub struct NewMute {
    pub user_id: i64,
    pub guild_id: i64,
}

#[derive(Queryable, Clone)]
pub struct Gallery {
    pub id: i32,
    pub watch_channel: i64,
    pub webhook_url: String,
    pub guild_id: i64,
}

#[derive(Insertable)]
#[table_name = "galleries"]
pub struct NewGallery<'a> {
    pub watch_channel: i64,
    pub webhook_url: &'a str,
    pub guild_id: i64,
}

#[derive(Queryable, Clone)]
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

#[derive(Insertable)]
#[table_name = "tags"]
pub struct NewTag<'a> {
    pub owner_id: i64,
    pub guild_id: i64,
    pub tag_name: &'a str,
    pub content: &'a str,
    pub count: i32,
    pub created: &'a NaiveDateTime,
}


#[derive(Queryable, Clone)]
pub struct MemberEvent {
    pub id: i32,
    pub guild_id: i64,
    pub user_id: i64,
    pub event_name: String,
    pub event_time: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "member_events"]
pub struct NewMemberEvent<'a> {
    pub guild_id: i64,
    pub user_id: i64,
    pub event_name: &'a str,
    pub event_time: &'a NaiveDateTime,
}