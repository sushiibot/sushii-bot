use diesel;
use diesel::result::Error;
use diesel::pg::PgConnection;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::env;
use std::sync::Arc;

use serenity::model::Guild;
use models::NewGuildConfig;
use models::EventCounter;
use models::NewEventCounter;
use models::UserLevel;
use models::NewUserLevel;

use chrono::{DateTime, Utc, Datelike};

pub struct ConnectionPool {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

pub fn init() -> ConnectionPool {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment.");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = Pool::builder().build(manager).expect(
        "Failed to create pool.",
    );

    ConnectionPool { pool: Arc::new(pool) }
}


impl ConnectionPool {
    /// Creates a new config for a guild,
    /// ie when the bot joins a new guild.
    pub fn new_guild<'a>(&self, guild: &'a Guild) {
        use schema::guilds;

        let new_guild_obj = NewGuildConfig {
            id: guild.id.0 as i64,
            name: &guild.name,
            join_msg: None,
            leave_msg: None,
            invite_guard: false,
            log_msg: None,
            log_mod: None,
        };

        // get a connection from the pool
        let conn = (*&self.pool).get().unwrap();

        diesel::insert_into(guilds::table)
            .values(&new_guild_obj)
            .execute(&*conn)
            .expect("Error saving new guild.");
    }

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

    // check if day is different and month (could possible be same day 1 year apart but unlikey)
    if now.day() != last_msg.day() && now.month() != last_msg.month() {
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
