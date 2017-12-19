use diesel;
use diesel::result::Error;
use diesel::pg::PgConnection;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::env;

use models::*;

use chrono::{DateTime, Utc, Datelike};
use chrono::naive::NaiveDateTime;

#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

pub fn init() -> ConnectionPool {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment.");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = Pool::builder().build(manager).expect(
        "Failed to create pool.",
    );

    ConnectionPool { pool }
}


impl ConnectionPool {
    /// Creates a new config for a guild,
    /// ie when the bot joins a new guild.
    pub fn new_guild(&self, guild_id: u64) -> GuildConfig {
        use schema::guilds;

        let new_guild_obj = NewGuildConfig {
            id: guild_id as i64,
            name: None,
            join_msg: None,
            join_react: None,
            leave_msg: None,
            invite_guard: Some(false),
            log_msg: None,
            log_mod: None,
            prefix: None,
        };

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        diesel::insert_into(guilds::table)
            .values(&new_guild_obj)
            .execute(&*conn)
            .expect("Error saving new guild.");

        new_guild_to_guild_config(new_guild_obj)
    }

    // gets a guild's config if it exists, or create one if it doesn't
    pub fn get_guild_config(&self, guild_id: u64) -> GuildConfig {
        use schema::guilds::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        let rows = guilds
            .filter(id.eq(guild_id as i64))
            .load::<GuildConfig>(&*conn)
            .expect("Error loading guild config");

        if rows.len() == 1 {
            rows[0].clone()
        } else {
            self.new_guild(guild_id)
        }
    }

    pub fn save_guild_config(&self, config: &GuildConfig) {
        use schema::guilds;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        diesel::update(guilds::table)
            .set(config)
            .execute(&*conn)
            .expect("Error updating guild.");
    }

    /// PREFIX

    /// Shortcut function to get the prefix for a guild
    pub fn get_prefix(&self, guild_id: u64) -> Option<String> {
        let guild = self.get_guild_config(guild_id);

        guild.prefix
    }

    // sets the prefix for a guild
    pub fn set_prefix(&self, guild_id: u64, new_prefix: &str) -> bool {
        use schema::guilds::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        // fetch event
        let rows = guilds
            .filter(id.eq(guild_id as i64))
            .load::<GuildConfig>(&*conn)
            .expect("Error loading guild");

        // check if this is a new guild
        if rows.len() == 0 {
            self.new_guild(guild_id);
        } else {
            let guild = rows[0].clone();
            // check if guild has same prefix
            if let Some(existing_prefix) = guild.prefix {
                if new_prefix == existing_prefix {
                    return false;
                }
            }
        }

        // update the guild row
        diesel::update(guilds.filter(id.eq(guild_id as i64)))
            .set(prefix.eq(new_prefix))
            .execute(&*conn)
            .expect("Failed to update the guild prefix.");

        true
    }

    /// EVENTS

    /// Logs a counter for each event that is handled
    pub fn log_event(&self, event_name: &str) {
        use schema::events;
        use schema::events::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        // fetch event
        let rows = events
            .filter(name.eq(&event_name))
            .load::<EventCounter>(&*conn)
            .expect("Error loading event");

        // check if a row was found
        if rows.len() == 1 {
            // increment the counter
            diesel::update(events.filter(name.eq(&event_name)))
                .set(count.eq(rows[0].count + 1))
                .execute(&*conn)
                .expect("Failed to update the event.");
        } else {
            let new_event_obj = NewEventCounter {
                name: &event_name,
                count: 1,
            };

            diesel::insert_into(events::table)
                .values(&new_event_obj)
                .execute(&*conn)
                .expect("Error saving new event.");
        }
    }

    /// Gets the counters for events handled
    pub fn get_events(&self) -> Result<Vec<EventCounter>, Error> {
        use schema::events::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        let results = events
            .order(name.asc())
            .load::<EventCounter>(&*conn)
            .expect("Error loading events");

        Ok(results)
    }

    pub fn reset_events(&self) -> Result<(), Error> {
        use schema::events::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        diesel::update(events)
            .set(count.eq(0))
            .execute(&*conn)
            .expect("Failed to reset the events.");

        Ok(())
    }

    /// LEVELS

    pub fn update_level(&self, id_user: u64, id_guild: u64) -> Result<(), Error> {
        use schema::levels;
        use schema::levels::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        let user = levels
            .filter(user_id.eq(id_user as i64))
            .filter(guild_id.eq(id_guild as i64))
            .load::<UserLevel>(&*conn)
            .expect("Error loading user's level.");

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        if user.len() == 1 {
            let new_interval_user = level_interval(&user[0]);

            // found a user object
            diesel::update(levels.filter(user_id.eq(id_user as i64)).filter(
                guild_id.eq(
                    id_guild as
                        i64,
                ),
            )).set((
                msg_all_time.eq(user[0].msg_all_time + 1),
                msg_month.eq(new_interval_user.msg_month + 1),
                msg_week.eq(new_interval_user.msg_week + 1),
                msg_day.eq(new_interval_user.msg_day + 1),
                last_msg.eq(now),
            ))
                .execute(&*conn)
                .expect("Failed to update level row.");
        } else {

            // create a new level row for the user + guild
            let new_level_obj = NewUserLevel {
                user_id: id_user as i64,
                guild_id: id_guild as i64,
                msg_all_time: 1,
                msg_month: 1,
                msg_week: 1,
                msg_day: 1,
                last_msg: &now,
            };

            diesel::insert_into(levels::table)
                .values(&new_level_obj)
                .execute(&*conn)
                .expect("Failed to insert new user level row.");
        }

        Ok(())
    }

    pub fn get_level(&self, id_user: u64, id_guild: u64) -> Option<UserLevel> {
        use schema::levels::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        let results = levels
            .filter(user_id.eq(id_user as i64))
            .filter(guild_id.eq(id_guild as i64))
            .load::<UserLevel>(&*conn)
            .expect("Error loading level");

        if results.len() == 1 {
            return Some(results[0].clone());
        } else {
            return None;
        }
    }

    /// REMINDERS

    pub fn add_reminder(&self, id_user: u64, content: &str, time: &NaiveDateTime) {
        use schema::reminders;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        let new_reminder_obj = NewReminder {
            user_id: id_user as i64,
            description: content,
            time_set: &now,
            time_to_remind: time,
        };

        diesel::insert_into(reminders::table)
            .values(&new_reminder_obj)
            .execute(&*conn)
            .expect("Failed to insert new reminder.");
    }

    pub fn get_overdue_reminders(&self) -> Option<Vec<Reminder>> {
        use schema::reminders::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        let rows = reminders
            .filter(time_to_remind.lt(now))
            .load::<Reminder>(&*conn)
            .expect("Error loading reminders");

        if rows.len() == 0 {
            return None;
        } else {
            return Some(rows);
        }
    }

    pub fn remove_reminder(&self, reminder_id: i32) {
        use schema::reminders::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        diesel::delete(reminders.filter(id.eq(reminder_id)))
            .execute(&*conn)
            .expect("Error deleting reminder.");
    }

    pub fn get_reminders(&self, id_user: u64) -> Option<Vec<Reminder>> {
        use schema::reminders::dsl::*;

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        let rows = reminders
            .filter(user_id.eq(id_user as i64))
            .load::<Reminder>(&*conn)
            .expect("Error loading reminders.");

        if rows.len() == 0 {
            return None;
        } else {
            return Some(rows);
        }
    }
}

pub struct UserLevelInterval {
    pub msg_month: i64,
    pub msg_week: i64,
    pub msg_day: i64,
}

/// checks if a new interval has passed and reset message counts accordingly
pub fn level_interval(user_level: &UserLevel) -> UserLevelInterval {
    let utc: DateTime<Utc> = Utc::now();
    let now = utc.naive_utc();

    let last_msg = user_level.last_msg;
    let mut msg_day = user_level.msg_day;
    let mut msg_week = user_level.msg_week;
    let mut msg_month = user_level.msg_month;

    // check if new day (could possible be same day 1 year apart but unlikey)
    if now.ordinal() != last_msg.ordinal() {
        msg_day = 0;
    }

    // check if new week
    if now.iso_week() != last_msg.iso_week() {
        msg_week = 0;
    }

    // check if new month
    if now.month() != last_msg.month() {
        msg_month = 0;
    }

    UserLevelInterval {
        msg_month,
        msg_week,
        msg_day,
    }
}

fn new_guild_to_guild_config(config: NewGuildConfig) -> GuildConfig {
    GuildConfig {
        id: config.id,
        name: config.name,
        join_msg: config.join_msg,
        join_react: config.join_react,
        leave_msg: config.leave_msg,
        invite_guard: config.invite_guard,
        log_msg: config.log_msg,
        log_mod: config.log_mod,
        prefix: config.prefix,
    }
}
