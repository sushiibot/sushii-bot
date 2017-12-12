use schema::guilds;
use schema::events;

#[derive(Queryable)]
pub struct GuildConfig {
    pub id: i64,
    pub name: String,
    pub join_msg: String,
    pub leave_msg: String,
    pub invite_guard: bool,
    pub log_msg: i64,
    pub log_mod: i64,
}

#[derive(Insertable)]
#[table_name = "guilds"]
pub struct NewGuildConfig<'a> {
    pub id: i64,
    pub name: &'a str,
    pub join_msg: Option<String>,
    pub leave_msg: Option<String>,
    pub invite_guard: bool,
    pub log_msg: Option<i64>,
    pub log_mod: Option<i64>,
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
