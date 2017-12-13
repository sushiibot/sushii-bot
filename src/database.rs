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
            diesel::update(events)
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

        let results = events.load::<EventCounter>(&*conn).expect(
            "Error loading events",
        );

        Ok(results)
    }
}
