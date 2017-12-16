use schema::guilds;
use schema::events;
use schema::levels;

use chrono::naive::NaiveDateTime;

#[derive(Queryable, Clone)]
pub struct GuildConfig {
    pub id: i64,
    pub name: Option<String>,
    pub join_msg: Option<String>,
    pub join_react: Option<String>,
    pub leave_msg: Option<String>,
    pub invite_guard: Option<bool>,
    pub log_msg: Option<i64>,
    pub log_mod: Option<i64>,
    pub prefix: Option<String>,
}

#[derive(Insertable)]
#[table_name = "guilds"]
pub struct NewGuildConfig {
    pub id: i64,
    pub name: Option<String>,
    pub join_msg: Option<String>,
    pub join_react: Option<String>,
    pub leave_msg: Option<String>,
    pub invite_guard: Option<bool>,
    pub log_msg: Option<i64>,
    pub log_mod: Option<i64>,
    pub prefix: Option<String>,
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
