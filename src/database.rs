use diesel;
use diesel::pg::PgConnection;
use diesel::RunQueryDsl;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::env;
use std::sync::Arc;

use serenity::model::Guild;
use models::NewGuild;

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
    pub fn new_guild<'a>(&self, guild: &'a Guild) {
        use schema::guilds;

        let new_guild_obj = NewGuild {
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
}
