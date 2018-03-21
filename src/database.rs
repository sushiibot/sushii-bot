use diesel;
use diesel::QueryDsl;
use diesel::dsl::max;
use diesel::dsl::sum;
use diesel::JoinOnDsl;
use diesel::RunQueryDsl;
use diesel::result::Error;
use diesel::pg::PgConnection;
use diesel::sql_types::BigInt;
use diesel::ExpressionMethods;
use diesel::BoolExpressionMethods;
use diesel::TextExpressionMethods;

use r2d2::Pool;
use r2d2::PooledConnection;
use diesel::r2d2::ConnectionManager;

use bigdecimal::BigDecimal;

use serenity;
use serenity::model::guild::Guild;
use std;
use std::env;

use models::*;

use chrono::{DateTime, Utc, Datelike, Timelike, Duration};
use chrono::naive::NaiveDateTime;

use utils::time::now_utc;


#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

embed_migrations!("./migrations");

impl Default for ConnectionPool {
    fn default() -> Self {
        ConnectionPool::new()
    }
}

impl ConnectionPool {
    pub fn new() -> ConnectionPool {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in the environment.");
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        let pool = Pool::builder().build(manager)
            .expect("Failed to create pool.");

        // run pending (embedded) migrations 
        info!("Running pending migrations...");
        let conn = (&pool).get().unwrap();
        if let Err(e) = embedded_migrations::run_with_output(&conn, &mut std::io::stdout()) {
            warn_discord!("[DB:embedded_migrations] Error while running pending migrations: {}", e);
        };

        ConnectionPool { pool }
    }

    pub fn connection(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        // get a connection from the pool
        self.pool.get().unwrap()
    }

    /// Creates a new config for a guild,
    /// ie when the bot joins a new guild.
    pub fn new_guild(&self, guild_id: u64) -> Result<GuildConfig, Error> {
        use schema::guilds;

        let new_guild_obj = NewGuildConfig {
            id: guild_id as i64,
            name: None,
            join_msg: None,
            join_react: None,
            leave_msg: None,
            msg_channel: None,
            role_channel: None,
            role_config: None,
            invite_guard: Some(false),
            log_msg: None,
            log_mod: None,
            log_member: None,
            mute_role: None,
            prefix: None,
            max_mention: 10,
            disabled_channels: None,
        };

        let conn = self.connection();

        diesel::insert_into(guilds::table)
            .values(&new_guild_obj)
            .get_result::<GuildConfig>(&conn)
    }

    // gets a guild's config if it exists, or create one if it doesn't
    pub fn get_guild_config(&self, guild_id: u64) -> Result<GuildConfig, Error> {
        use schema::guilds::dsl::*;

        let conn = self.connection();

        let config = guilds
            .filter(id.eq(guild_id as i64))
            .first::<GuildConfig>(&conn);

        if let Err(e) = config {
            match e {
                Error::NotFound => {
                    return self.new_guild(guild_id)
                },
                _ => {
                    warn_discord!("[DB:get_guild_config] Error while fetching guild_config: {}", e);
                    return Err(e);
                }
            }
        } else {
            return config;
        }
    }

    pub fn save_guild_config(&self, config: &GuildConfig) {
        use schema::guilds;
        use schema::guilds::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(guilds::table)
            .filter(id.eq(config.id))
            .set(config)
            .execute(&conn) {
                
                warn_discord!("[DB:save_guild_config] Error while updating a guild: {}", e)
        }
    }

    /// PREFIX

    /// Shortcut function to get the prefix for a guild
    pub fn get_prefix(&self, guild_id: u64) -> Option<String> {
        match self.get_guild_config(guild_id) {
            Ok(val) => val.prefix,
            Err(_) => None,
        }
    }

    // sets the prefix for a guild
    pub fn set_prefix(&self, guild_id: u64, new_prefix: &str) -> bool {
        use schema::guilds::dsl::*;

        let conn = self.connection();

        // fetch event
        let guild = guilds
            .filter(id.eq(guild_id as i64))
            .first::<GuildConfig>(&conn)
            .ok();


        if let Some(guild) = guild {
            let guild = guild.clone();
            // check if guild has same prefix
            if let Some(existing_prefix) = guild.prefix {
                if new_prefix == existing_prefix {
                    return false;
                }
            }
        // check if this is a new guild,
        // make a new config if it is
        } else if self.new_guild(guild_id).is_err() {
                return false;
            }

        // update the guild row
        if let Err(e) = diesel::update(guilds)
            .filter(id.eq(guild_id as i64))
            .set(prefix.eq(new_prefix))
            .execute(&conn) {
            
                warn_discord!("[DB:set_prefix] Error while setting a guild prefix: {}", e);
                return false;
        }

        true
    }

    /// EVENTS

    /// Logs a counter for each event that is handled
    pub fn log_event(&self, event_name: &str) -> Result<(), Error> {
        use schema::events;
        use schema::events::dsl::*;

        let conn = self.connection();

        // increment the counter
        if diesel::update(events)
            .filter(name.eq(&event_name))
            .set(count.eq(count + 1))
            .execute(&conn).is_err() {

            let new_event = NewEventCounter {
                name: event_name,
                count: 1,
            };
            
            // error when incremeneting, maybe the row doesn't exist
            // so create a row
            diesel::insert_into(events::table)
                .values(&new_event)
                .execute(&conn)?;
        }

        Ok(())
    }

    /// Gets the counters for events handled
    pub fn get_events(&self) -> Result<Vec<EventCounter>, Error> {
        use schema::events::dsl::*;

        let conn = self.connection();

        let results = events
            .order(name.asc())
            .load::<EventCounter>(&conn)?;

        Ok(results)
    }

    pub fn reset_events(&self) -> Result<(), Error> {
        use schema::events::dsl::*;

        let conn = self.connection();

        diesel::update(events)
            .set(count.eq(0))
            .execute(&conn)?;

        Ok(())
    }

    /// LEVELS

    pub fn update_level(&self, id_user: u64, id_guild: u64) -> Result<(), Error> {
        use schema::levels;
        use schema::levels::dsl::*;

        let conn = self.connection();

        let user = levels
            .filter(user_id.eq(id_user as i64))
            .filter(guild_id.eq(id_guild as i64))
            .first::<UserLevel>(&conn);

        let now = now_utc();

        if let Ok(user) = user {
            let next_level_update = user.last_msg + Duration::minutes(1);

            // has not been 1 minute since last "message"
            // last message will always have inaccuracy
            // +/- 1 minute because of this check
            // could potentially create another user field
            // to have last_level_update or something
            if next_level_update > now {
                return Ok(());
            }

            let new_interval_user = level_interval(&user);

            // found a user object
            diesel::update(levels)
                .filter(user_id.eq(id_user as i64))
                .filter(guild_id.eq(id_guild as i64))
                .set((
                    msg_all_time.eq(user.msg_all_time + 1),
                    msg_month.eq(new_interval_user.msg_month + 1),
                    msg_week.eq(new_interval_user.msg_week + 1),
                    msg_day.eq(new_interval_user.msg_day + 1),
                    last_msg.eq(now),
                ))
                .execute(&conn)?;
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
                .execute(&conn)?;
        }

        Ok(())
    }

    pub fn get_level(&self, id_user: u64, id_guild: u64) -> Option<UserLevelRanked> {
        let conn = self.connection();

        // get ranks
        match diesel::sql_query(r#"
            SELECT * 
                FROM (
                    SELECT *,
                        ROW_NUMBER() OVER(PARTITION BY EXTRACT(DOY FROM last_msg) ORDER BY msg_day DESC) AS msg_day_rank,
                        COUNT(*) OVER(PARTITION BY EXTRACT(DOY FROM last_msg)) AS msg_day_total,

                        ROW_NUMBER() OVER(PARTITION BY EXTRACT(WEEK FROM last_msg) ORDER BY msg_week DESC) AS msg_week_rank,
                        COUNT(*) OVER(PARTITION BY EXTRACT(WEEK FROM last_msg)) AS msg_week_total,

                        ROW_NUMBER() OVER(PARTITION BY EXTRACT(MONTH FROM last_msg) ORDER BY msg_month DESC) AS msg_month_rank,
                        COUNT(*) OVER(PARTITION BY EXTRACT(MONTH FROM last_msg)) AS msg_month_total,

                        ROW_NUMBER() OVER(ORDER BY msg_all_time DESC) AS msg_all_time_rank,
                        COUNT(*) OVER() AS msg_all_time_total
                    FROM levels WHERE guild_id = $1 
                ) t
            WHERE t.user_id = $2
        "#)
            .bind::<BigInt, i64>(id_guild as i64)
            .bind::<BigInt, i64>(id_user as i64)
            .load(&conn) {

            Ok(val) => val.get(0).map(|x| level_interval_ranked(x)),
            Err(e) => {
                warn_discord!("[DB:get_level] Error while getting level: {}", e);
                None
            },
        }
    }

    pub fn get_global_levels(&self) -> Option<Vec<UserLevelAllTime>> {
        let conn = self.connection();

        match diesel::sql_query(r#"
            SELECT
                t.user_id,
                t.xp
            FROM (
                SELECT user_id, SUM(msg_all_time) AS xp
                FROM levels
                GROUP BY user_id
            ) t
            JOIN levels l ON l.user_id = t.user_id
            GROUP BY t.user_id, t.xp
            ORDER BY t.xp DESC
            LIMIT 10
        "#)
            .load::<UserLevelAllTime>(&conn) {

            Ok(val) => Some(val),
            Err(e) => {
                warn_discord!("[DB:get_global_levels] Error while getting global levels: {}", e);
                None
            }
        }
    }

    pub fn get_global_xp(&self, id_user: u64) -> Option<BigDecimal> {
        use schema::levels::dsl::*;

        let conn = self.connection();

        levels
            .filter(user_id.eq(id_user as i64))
            .select(sum(msg_all_time))
            .first::<Option<BigDecimal>>(&conn)
            .ok()
            .unwrap_or(None)
    }

    // fetch top 10 for each category
    pub fn get_top_levels(&self, id_guild: u64) -> TopLevels {
        use schema::levels::dsl::*;

        let conn = self.connection();

        // daily
        let day = diesel::sql_query(r#"
            SELECT *,
                ROW_NUMBER() OVER(PARTITION BY EXTRACT(DOY FROM last_msg) ORDER BY msg_day DESC)
            FROM levels WHERE guild_id = $1 
                AND EXTRACT(DOY FROM last_msg) = EXTRACT(DOY FROM NOW())
                LIMIT 5
        "#)
            .bind::<BigInt, i64>(id_guild as i64)
            .load::<UserLevel>(&conn)
            .ok();
        
        let week = diesel::sql_query(r#"
            SELECT *,
                ROW_NUMBER() OVER(PARTITION BY EXTRACT(WEEK FROM last_msg) ORDER BY msg_week DESC)
            FROM levels WHERE guild_id = $1 
                AND EXTRACT(WEEK FROM last_msg) = EXTRACT(WEEK FROM NOW())
                LIMIT 5
        "#)
            .bind::<BigInt, i64>(id_guild as i64)
            .load::<UserLevel>(&conn)
            .ok();
        
        let month = diesel::sql_query(r#"
            SELECT *,
                ROW_NUMBER() OVER(PARTITION BY EXTRACT(MONTH FROM last_msg) ORDER BY msg_month DESC)
            FROM levels WHERE guild_id = $1 
                AND EXTRACT(MONTH FROM last_msg) = EXTRACT(MONTH FROM NOW())
                LIMIT 5
        "#)
            .bind::<BigInt, i64>(id_guild as i64)
            .load::<UserLevel>(&conn)
            .ok();
        
        let all_time = levels
            .filter(guild_id.eq(id_guild as i64))
            .order(msg_all_time.desc())
            .limit(5)
            .load::<UserLevel>(&conn)
            .ok();
        

        TopLevels {
            day,
            week,
            month,
            all_time,
        }
    }

    pub fn update_user_activity_message(&self, id_user: u64) {
        use schema::users;
        use schema::users::dsl::*;

        let conn = self.connection();

        let user = users
            .filter(id.eq(id_user as i64))
            .first::<User>(&conn);

        // get current timestamp
        let now = now_utc();
        let hour = now.hour();

        match user {
            Ok(user) => {
                // check if should upate
                let next_activity_update = user.last_msg + Duration::minutes(1);

                // has not been 1 minute since last "message"
                // last message will always have inaccuracy
                // +/- 1 minute because of this check
                if next_activity_update > now {
                    return;
                }

                // update the useractivity
                let mut updated_activity = user.msg_activity.clone();
                if let Some(elem) = updated_activity.get_mut(hour as usize) {
                    *elem += 1;
                } else {
                    warn_discord!("[DB:update_user_activity_message] Error incrementing user {} activity", id_user);
                }
                // update the user
                // does not modify last_message, this plugin
                // executes after the level plugin which updates the
                // last message timestamp.  If last_message is updated here
                // the level plugin will never execute (needs 1 min since last msg)
                if let Err(e) = diesel::update(users)
                    .filter(id.eq(id_user as i64))
                    .set(msg_activity.eq(updated_activity))
                    .execute(&conn) {

                    warn_discord!("[DB:update_user_activity_message] Error while updating user activity: {}", e);
                }
            },
            Err(e) => {
                match e {
                    // if it's not found, create new user
                    Error::NotFound => {
                        // create vector of 24 0's
                        let init_activity = vec![0; 24];
                        // create new user
                        let new_user = NewUser {
                            id: id_user as i64,
                            last_msg: &now,
                            msg_activity: &init_activity,
                            rep: 0,
                            last_rep: None,
                            latitude: None,
                            longitude: None,
                            address: None,
                            lastfm: None,
                            is_patron: false,
                            fishies: 0,
                            last_fishies: None,
                            patron_emoji: None,
                        };

                        if let Err(e) = diesel::insert_into(users::table)
                            .values(&new_user)
                            .execute(&conn) {

                            warn_discord!(format!("[DB:update_user_activity_message] Failed to insert new user row: {}\nData: {:?}", e, &new_user));
                        }
                    }
                    _ => {
                        warn_discord!(format!("[DB:update_user_activity_message] Error fetching user row: {}", e));
                    }
                }
            }
        }
    }

    pub fn get_user(&self, id_user: u64) -> Option<User> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .first::<User>(&conn)
            .ok()
    }

    pub fn save_user(&self, new_user_data: &User) {
        use schema::users::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(users)
            .filter(id.eq(new_user_data.id))
            .set(new_user_data)
            .execute(&conn) {
                
                warn_discord!("[DB:save_user] Error while updating a user: {}", e);
        }
    }

    pub fn get_user_last_message(&self, id_user: u64) -> Option<NaiveDateTime> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .select(last_msg)
            .first(&conn)
            .ok()
    }

    pub fn set_patron(&self, id_user: u64, status: bool) -> bool {
        use schema::users::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(users)
                .filter(id.eq(id_user as i64))
                .set(is_patron.eq(status))
                .execute(&conn) {
            
            warn_discord!("[DB:set_patron] Error while adding a patron: {}", e);
            false
        } else {
            true
        }
    }

    pub fn set_patron_emoji(&self, id_user: u64, emoji: &str) -> bool {
        use schema::users::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(users)
                .filter(id.eq(id_user as i64))
                .set(patron_emoji.eq(emoji))
                .execute(&conn) {
            
            warn_discord!("[DB:set_patron_emoji] Error while setting patron emoji: {}", e);
            false
        } else {
            true
        }
    }

    /// REP
    pub fn get_last_rep(&self, id_user: u64) -> Option<NaiveDateTime> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .select(last_rep)
            .first::<Option<NaiveDateTime>>(&conn)
            .unwrap_or(None)
    }

    pub fn rep_user(&self, id_user: u64, id_target: u64) {
        use schema::users::dsl::*;

        let conn = self.connection();

        let now = now_utc();

        // update last_rep timestamp
        if let Err(e) = diesel::update(users)
            .filter(id.eq(id_user as i64))
            .set(last_rep.eq(now))
            .execute(&conn) {
                warn_discord!("[DB:rep_user] Error when updating last rep: {}", e);
            }
        
        if let Err(e) = diesel::update(users)
                .filter(id.eq(id_target as i64))
                .set(rep.eq(rep + 1))
                .execute(&conn) {
            warn_discord!("[DB:rep_user] Error when updating rep: {}", e);
        }
    }

    pub fn get_top_reps(&self, guild: u64) -> Option<Vec<(i64, i32)>> {
        use schema::users::dsl::*;
        use schema::cache_members::dsl::*;

        let conn = self.connection();

        users
            .inner_join(cache_members.on(user_id.eq(id)))
            .select((id, rep))
            .filter(rep.gt(0))
            .filter(guild_id.eq(guild as i64))
            .order(rep.desc())
            .limit(10)
            .load(&conn)
            .ok()
    }

    pub fn get_top_reps_global(&self) -> Option<Vec<(i64, i32)>> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .select((id, rep))
            .filter(rep.gt(0))
            .order(rep.desc())
            .limit(10)
            .load(&conn)
            .ok()
    }

    // DAILY FISHIES
    pub fn get_last_fishies(&self, id_user: u64) -> Option<NaiveDateTime> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .select(last_fishies)
            .first::<Option<NaiveDateTime>>(&conn)
            .unwrap_or(None)
    }

    pub fn get_fishies(&self, id_user: u64, target: u64, is_self: bool) -> (i64, bool) {
        use schema::users::dsl::*;
        use rand::thread_rng;
        use rand::distributions::{IndependentSample, Range};

        let conn = self.connection();

        let now = now_utc();
        let mut rng = thread_rng();

        // 1% chance of getting golden fishy == 80-150 fishies?
        let golden_range = Range::new(1, 1000);
        let is_golden = golden_range.ind_sample(&mut rng) == 1;

        let new_fishies: i64 = if is_golden {
            let between = Range::new(80, 150);
            between.ind_sample(&mut rng)
        } else if is_self {
            let between = Range::new(5, 20);
            between.ind_sample(&mut rng)
        } else {
            let between = Range::new(15, 30);
            between.ind_sample(&mut rng)
        };

        // update last_fishies timestamp
        if let Err(e) = diesel::update(users)
            .filter(id.eq(id_user as i64))
            .set(last_fishies.eq(now))
            .execute(&conn) {
                warn_discord!("[DB:get_fishies] Error when updating last fishies: {}", e);
            }
        
        if let Err(e) = diesel::update(users)
                .filter(id.eq(target as i64))
                .set(fishies.eq(fishies + new_fishies))
                .execute(&conn) {
            warn_discord!("[DB:get_fishies] Error when updating fishies: {}", e);
        }

        (new_fishies, is_golden)
    }

    pub fn get_top_fishies(&self, guild: u64) -> Option<Vec<(i64, i64)>> {
        use schema::users::dsl::*;
        use schema::cache_members::dsl::*;

        let conn = self.connection();

        users
            .inner_join(cache_members.on(user_id.eq(id)))
            .select((id, fishies))
            .filter(fishies.gt(0))
            .filter(guild_id.eq(guild as i64))
            .order(fishies.desc())
            .limit(10)
            .load(&conn)
            .ok()
    }

    pub fn get_top_fishies_global(&self) -> Option<Vec<(i64, i64)>> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .select((id, fishies))
            .filter(fishies.gt(0))
            .order(fishies.desc())
            .limit(10)
            .load(&conn)
            .ok()
    }

    /// REMINDERS

    pub fn add_reminder(&self, id_user: u64, content: &str, time: &NaiveDateTime) {
        use schema::reminders;

        let conn = self.connection();

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        let new_reminder_obj = NewReminder {
            user_id: id_user as i64,
            description: content,
            time_set: &now,
            time_to_remind: time,
        };

        if let Err(e) = diesel::insert_into(reminders::table)
            .values(&new_reminder_obj)
            .execute(&conn) {

            warn_discord!("[DB:add_reminder] Failed to insert new reminder: {}", e);
        }
    }

    pub fn get_overdue_reminders(&self) -> Option<Vec<Reminder>> {
        use schema::reminders::dsl::*;

        let conn = self.connection();

        // get current timestamp
        let utc: DateTime<Utc> = Utc::now();
        let now = utc.naive_utc();

        reminders
            .filter(time_to_remind.lt(now))
            .load::<Reminder>(&conn)
            .ok()
    }

    pub fn remove_reminder(&self, reminder_id: i32) {
        use schema::reminders::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::delete(reminders.filter(id.eq(reminder_id)))
            .execute(&conn) {

            warn_discord!("[DB:remove_reminder] Failed to remove reminder: {}", e);
        }
    }

    pub fn get_reminders(&self, id_user: u64) -> Option<Vec<Reminder>> {
        use schema::reminders::dsl::*;

        let conn = self.connection();

        reminders
            .filter(user_id.eq(id_user as i64))
            .load::<Reminder>(&conn)
            .ok()
    }

    /// Creates a new notification
    pub fn new_notification(&self, user: u64, guild: u64, kw: &str) {
        use schema::notifications;

        let conn = self.connection();

        let new_notification = NewNotification {
            user_id: user as i64,
            guild_id: guild as i64,
            keyword: kw,
        };

        if let Err(e) = diesel::insert_into(notifications::table)
            .values(&new_notification)
            .execute(&conn) {

            warn_discord!("[DB:new_notification] Failed to insert new notification.", e);
        }
    }

    /// Gets notifications that have been triggered
    pub fn get_notifications(&self, msg: &str, guild: u64) -> Option<Vec<Notification>> {
        use schema::notifications::dsl::*;

        sql_function!(strpos, strpos_t, (string: diesel::sql_types::Text, substring: diesel::sql_types::Text) -> diesel::sql_types::Integer);

        let conn = self.connection();

        notifications
            .filter(guild_id.eq(guild as i64).or(guild_id.eq(0)))
            .filter(strpos(msg, keyword).gt(0))
            .load::<Notification>(&conn)
            .ok()
    }

    /// Lists the notification's of a user
    pub fn list_notifications(&self, user: u64) -> Option<Vec<Notification>> {
        use schema::notifications::dsl::*;

        let conn = self.connection();

        notifications
            .filter(user_id.eq(user as i64))
            .load::<Notification>(&conn)
            .ok()
    }

    pub fn delete_notification(&self, user: u64, guild: Option<u64>,
            kw: Option<&str>, notification_id: Option<i32>) -> Option<Notification> {

        use schema::notifications::dsl::*;

        let conn = self.connection();

        if let Some(notification_id) = notification_id {
            // delete with a #
            if let Some(mut v) = self.list_notifications(user) {
                v.sort_by(|a, b| a.keyword.cmp(&b.keyword));

                let target = match v.get((notification_id - 1) as usize) {
                    Some(val) => val,
                    None => return None,
                };

                diesel::delete(
                    notifications
                        .filter(user_id.eq(user as i64))
                        .filter(guild_id.eq(target.guild_id))
                        .filter(keyword.eq(&target.keyword))
                    )
                    .get_result::<Notification>(&conn)
                    .ok()
            } else {
                return None;
            }

           
        } else if let Some(guild) = guild {
            if let Some(kw) = kw {
                // delete with keyword in a guild
                diesel::delete(
                    notifications
                        .filter(user_id.eq(user as i64))
                        .filter(guild_id.eq(guild as i64))
                        .filter(keyword.eq(kw))
                    )
                    .get_result::<Notification>(&conn)
                    .ok()
            } else {
                None
            }
        } else if let Some(kw) = kw {
            // delete all of a keyword
            diesel::delete(
                notifications
                    .filter(user_id.eq(user as i64))
                    .filter(keyword.eq(kw))
                )
                .get_result::<Notification>(&conn)
                .ok()
        } else {
            None
        }
    }

    /// MOD ACTIONS
    pub fn add_mod_action(&self, mod_action: &str, guild: u64, user: &serenity::model::user::User,
            action_reason: Option<&str>, is_pending: bool, executor: Option<u64>) -> Result<ModAction, Error> {
        use schema::mod_log;
        use schema::mod_log::dsl::*;
        
        let now = now_utc();

        let conn = self.connection();

        // get a new case id
        let new_case_id = mod_log
            .select(max(case_id))
            .filter(guild_id.eq(guild as i64))
            .first::<Option<i32>>(&conn)?
            .unwrap_or(0) + 1;

        // create new mod action 
        let new_action = NewModAction {
            case_id: new_case_id,
            guild_id: guild as i64,
            executor_id: executor.map(|x| x as i64),
            user_id: user.id.0 as i64,
            user_tag: &user.tag(),
            action: mod_action,
            reason: action_reason,
            action_time: &now,
            msg_id: None,
            pending: is_pending,
        };

        // add the action and return the new action
        let result = diesel::insert_into(mod_log::table)
            .values(&new_action)
            .get_result::<ModAction>(&conn);

        if let Err(ref e) = result {
            warn_discord!("[DB:add_mod_action] Error adding mod action: {}", e);
        }

        result
    }

    pub fn remove_mod_action(&self, guild: u64, user: &serenity::model::user::User, case: i32) {
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::delete(
            mod_log
                .filter(user_id.eq(user.id.0 as i64))
                .filter(guild_id.eq(guild as i64))
                .filter(case_id.eq(case))
            )
            .execute(&conn) {
                warn_discord!("[DB:remove_mod_action] Error while removing a mod action due to failed ban: {}", e);
        }
    }

    pub fn update_mod_action(&self, entry: &ModAction) {
        use schema::mod_log;
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        // id/entry_id is the unique global serial id, not the case id
        if let Err(e) = diesel::update(mod_log::table)
            .filter(id.eq(entry.id))
            .set(entry)
            .execute(&conn) {

                warn_discord!("[DB:update_mod_action] Error while updating mod action: {}", e);
            }
    }

    pub fn get_latest_mod_action(&self, guild: u64) -> i32 {
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        mod_log
            .select(case_id)
            .filter(guild_id.eq(guild as i64))
            .order(case_id.desc())
            .first(&conn)
            .unwrap_or(0)
    }

    pub fn fetch_mod_actions(&self, guild: u64, lower: i32, upper: i32) -> Option<Vec<ModAction>> {
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        mod_log
            .filter(guild_id.eq(guild as i64))
            .filter(case_id.between(lower, upper))
            .order(case_id.asc())
            .load::<ModAction>(&conn)
            .ok()
    }

    pub fn get_mod_action_user_history(&self, guild: u64, target: u64) -> Option<Vec<ModAction>>{
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        mod_log
            .filter(guild_id.eq(guild as i64))
            .filter(user_id.eq(target as i64))
            .order(case_id.desc())
            .load::<ModAction>(&conn)
            .ok()
    }

    pub fn get_pending_mod_actions(&self, mod_action: &str, guild: u64, user: u64) -> Option<ModAction> {
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        mod_log
            .filter(guild_id.eq(guild as i64))
            .filter(action.eq(mod_action))
            .filter(user_id.eq(user as i64))
            .filter(pending.eq(true))
            .first::<ModAction>(&conn)
            .ok()
    }

    pub fn log_message(&self, msg: &serenity::model::channel::Message) {
        use schema::messages;

        let conn = self.connection();

        let new_message = NewMessage {
            id: msg.id.0 as i64,
            author: msg.author.id.0 as i64,
            tag: &msg.author.tag(),
            channel: msg.channel_id.0 as i64,
            guild: msg.guild_id().map(|x| x.0 as i64),
            created: msg.timestamp.naive_utc(),
            content: &msg.content,
        };

        if let Err(e) = diesel::insert_into(messages::table)
            .values(&new_message)
            .execute(&conn) {
                warn_discord!("[DB:log_message] Error while logging new message: {}", e);
            }
    }

    pub fn update_message(&self, msg_id: u64, new_content: &str) {
        use schema::messages;
        use schema::messages::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(messages::table)
            .filter(id.eq(msg_id as i64))
            .set(content.eq(new_content))
            .execute(&conn) {
                warn_discord!("[DB:update_message] Error updating message: {}", e);
        }
    }

    pub fn get_messages(&self, channel_id: u64, limit: i64) -> Option<Vec<Message>> {
        use schema::messages::dsl::*;

        let conn = self.connection();

        messages
            .filter(channel.eq(channel_id as i64))
            .order(created.desc())
            .limit(limit)
            .load(&conn)
            .ok()
    }

    pub fn save_weather_location(&self, id_user: u64, lat: f64, lng: f64, loc: &str) {
        use schema::users;
        use schema::users::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(users::table)
            .filter(id.eq(id_user as i64))
            .set((
                latitude.eq(Some(lat)),
                longitude.eq(Some(lng)),
                address.eq(Some(loc))
            ))
            .execute(&conn) {

                warn_discord!("[DB:save_weather_location] Error while updating a user weather location: {}", e);
        };
    }

    pub fn get_weather_location(&self, id_user: u64) -> Option<(Option<f64>, Option<f64>, Option<String>)> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .select((latitude, longitude, address))
            .first(&conn)
            .ok()
    }


    /// MUTES
    pub fn add_mute(&self, id_user: u64, id_guild: u64) {
        use schema::mutes;

        let conn = self.connection();

        let new_mute = NewMute {
            user_id: id_user as i64,
            guild_id: id_guild as i64,
        };

        if let Err(e) = diesel::insert_into(mutes::table)
            .values(&new_mute)
            .execute(&conn) {
                warn_discord!("[DB:add_mute] Error while adding new mute: {}", e);
            }
    }

    pub fn should_mute(&self, id_user: u64, id_guild: u64) -> bool {
        use schema::mutes::dsl::*;

        let conn = self.connection();

        mutes
            .filter(user_id.eq(id_user as i64))
            .filter(guild_id.eq(id_guild as i64))
            .first::<Mute>(&conn).is_ok()
    }

    pub fn delete_mute(&self, id_user: u64, id_guild: u64) {
        use schema::mutes::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::delete(mutes
                .filter(user_id.eq(id_user as i64))
                .filter(guild_id.eq(id_guild as i64))
            )
            .execute(&conn) {
                warn_discord!("[DB:delete_mute] Error while deleting mute: {}", e);
        }
    }

    /// GALLERY
    
    pub fn add_gallery(&self, channel: u64, guild: u64, webhook: &str) {
        use schema::galleries;

        let conn = self.connection();


        let new_gallery = NewGallery {
            watch_channel: channel as i64,
            webhook_url: webhook,
            guild_id: guild as i64,
        };

        if let Err(e) = diesel::insert_into(galleries::table)
            .values(&new_gallery)
            .execute(&conn) {
                warn_discord!("[DB:add_gallery] Error while adding new gallery: {}", e);
            }
    }

    pub fn get_gallery_webhook(&self, channel: u64) -> Option<Vec<String>> {
        use schema::galleries::dsl::*;

        let conn = self.connection();

        galleries
            .select(webhook_url)
            .filter(watch_channel.eq(channel as i64))
            .load(&conn)
            .ok()
    }

    pub fn list_galleries(&self, guild: u64) -> Option<Vec<Gallery>> {
        use schema::galleries::dsl::*;

        let conn = self.connection();

        galleries
            .filter(guild_id.eq(guild as i64))
            .load::<Gallery>(&conn)
            .ok()
    }

    pub fn delete_gallery(&self, guild: u64, id_gallery: i32) -> bool {
        use schema::galleries::dsl::*;

        let conn = self.connection();

        let mut current = match self.list_galleries(guild) {
            Some(val) => val,
            None => return false,
        };

        current.sort_by(|a, b| a.watch_channel.cmp(&b.watch_channel));

        if let Some(entry) = current.get((id_gallery - 1) as usize) {
            if let Err(e) = diesel::delete(galleries
                    .filter(watch_channel.eq(entry.watch_channel))
                    .filter(webhook_url.eq(&entry.webhook_url))
                )
                .execute(&conn) {
                
                warn_discord!("[DB:delete_gallery] Error while deleting gallery: {}", e);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// LASTFM
    pub fn set_lastfm_username(&self, user_id: u64, fm_username: &str) {
        use schema::users;
        use schema::users::dsl::*;

        let conn = self.connection();


        if let Err(e) = diesel::update(users::table)
            .filter(id.eq(user_id as i64))
            .set(lastfm.eq(fm_username))
            .execute(&conn) {
                warn_discord!("[DB:set_lastfm_username] Error setting lastfm username: {}", e);
        }
    }

    pub fn get_lastfm_username(&self, user_id: u64) -> Option<String> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(user_id as i64))
            .select(lastfm)
            .first::<Option<String>>(&conn)
            .unwrap_or(None)
    }

    /// TAGS
    pub fn add_tag(&self, user_id: u64, guild: u64, name: &str, cntent: &str) -> bool {
        use schema::tags;

        let conn = self.connection();

        let now = now_utc();

        let new_tag = NewTag {
            owner_id: user_id as i64,
            guild_id: guild as i64,
            tag_name: name,
            content: cntent,
            count: 0,
            created: &now,
        };

        if let Err(e) = diesel::insert_into(tags::table)
            .values(&new_tag)
            .execute(&conn) {
                warn_discord!("[DB:add_tag] Error while adding new tag: {}", e);
                false
        } else {
            true
        }
    }

    pub fn increment_tag(&self, guild: u64, name: &str) {
        use schema::tags;
        use schema::tags::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(tags::table)
            .filter(guild_id.eq(guild as i64))
            .filter(tag_name.eq(name))
            .set(count.eq(count + 1))
            .execute(&conn) {
                warn_discord!("[DB:edit_tag] Error while incrementing tag count: {}", e);
        }
    }

    pub fn get_tag(&self, guild: u64, name: &str) -> Option<Tag> {
        use schema::tags::dsl::*;

        let conn = self.connection();

        tags
            .filter(tag_name.eq(name))
            .filter(guild_id.eq(guild as i64))
            .first::<Tag>(&conn)
            .ok()
    }

    pub fn get_random_tag(&self, guild: u64) -> Option<Tag> {
        use schema::tags::dsl::*;

        no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");

        let conn = self.connection();

        tags
            .order(RANDOM)
            .filter(guild_id.eq(guild as i64))
            .first::<Tag>(&conn)
            .ok()
    }

    pub fn get_tags(&self, guild: u64) -> Option<Vec<Tag>> {
        use schema::tags::dsl::*;

        let conn = self.connection();

        tags
            .filter(guild_id.eq(guild as i64))
            .order(tag_name.desc())
            .load::<Tag>(&conn)
            .ok()
    }

    pub fn get_tags_top(&self, guild: u64) -> Option<Vec<Tag>> {
        use schema::tags::dsl::*;

        let conn = self.connection();

        tags
            .filter(guild_id.eq(guild as i64))
            .order(count.desc())
            .limit(10)
            .load::<Tag>(&conn)
            .ok()
    }

    pub fn search_tag(&self, guild: u64, name: &str) -> Option<Vec<Tag>> {
        use schema::tags::dsl::*;

        let conn = self.connection();

        tags
            .filter(guild_id.eq(guild as i64))
            .filter(tag_name.like(&format!("%{}%", name)))
            .order(count.desc())
            .limit(10)
            .load::<Tag>(&conn)
            .ok()
    }

    pub fn delete_tag(&self, guild: u64, name: &str) -> bool {
        use schema::tags::dsl::*;

        let conn = self.connection();

        match diesel::delete(
                tags
                    .filter(guild_id.eq(guild as i64))
                    .filter(tag_name.eq(name))
                )
                .execute(&conn) {

                
                Ok(rows) => {
                    // returns number of rows affected, this should be 1
                    // if it was successfully deleted
                    rows == 1
                },
                Err(e) => {
                    warn_discord!("[DB:delete_tag] Error while deleting tag: {}", e);
                    false
                }
        }
    }

    pub fn edit_tag(&self, guild: u64, name: &str, new_name: &str, new_content: &str) -> bool {
        use schema::tags;
        use schema::tags::dsl::*;

        let conn = self.connection();

        if let Err(e) = diesel::update(tags::table)
            .filter(guild_id.eq(guild as i64))
            .filter(tag_name.eq(name))
            .set((
                tag_name.eq(new_name),
                content.eq(new_content),
            ))
            .execute(&conn) {
                warn_discord!("[DB:edit_tag] Error while updating tag: {}", e);

                false
        } else {
            true
        }
    }

    pub fn log_member_event(&self, guild: u64, user: u64, name: &str) {
        use schema::member_events;

        let conn = self.connection();

        let now = now_utc();

        let new_member_event = NewMemberEvent {
            guild_id: guild as i64,
            user_id: user as i64,
            event_name: name,
            event_time: &now,
        };

        if let Err(e) = diesel::insert_into(member_events::table)
            .values(&new_member_event)
            .execute(&conn) {

            warn_discord!("[DB:log_member_event] Failed to insert member event: {}", e);
        }
    }

    // DATABASE CACHE FOR WEBSITE
    pub fn update_cache_guild(&self, guild: &Guild) -> Result<usize, Error> {
        use schema::cache_guilds::dsl::*;

        let conn = self.connection();

        let icon_url = guild.icon_url();

        let new_cache_guild = NewCachedGuild {
            id: guild.id.0 as i64,
            guild_name: &guild.name,
            icon: icon_url.as_ref().map(|x| &**x),
            member_count: guild.member_count as i64, // Option<String> to Option<&str>
            owner_id: guild.owner_id.0 as i64,
        };

        // inserts new cached guild data,
        // if it already exists, just update the existing
        diesel::insert_into(cache_guilds)
            .values(&new_cache_guild)
            .on_conflict(id)
            .do_update()
            .set(&new_cache_guild)
            .execute(&conn)
    }

    pub fn update_cache_members(&self, guild: u64, users: Vec<u64>) -> Result<(), Error> {
        use schema::cache_members::dsl::*;

        let conn = self.connection();

        for user in users {
            let a_member = NewCachedMember {
                user_id: user as i64,
                guild_id: guild as i64,
            };

            diesel::insert_into(cache_members)
                .values(&a_member)
                .on_conflict_do_nothing()
                .execute(&conn)?;
        }

        Ok(())
    }

    pub fn remove_cache_member(&self, guild: u64, user: u64) -> Result<usize, Error> {
        use schema::cache_members::dsl::*;

        let conn = self.connection();

        diesel::delete(cache_members)
            .filter(user_id.eq(user as i64))
            .filter(guild_id.eq(guild as i64))
            .execute(&conn)
    }

    // STATS
    pub fn update_stat(&self, new_cat: &str, new_stat: &str, added: i64) {
        use utils::datadog;
        use schema::stats::dsl::*;

        let conn = self.connection();

        let new_stat = NewStat {
            stat_name: new_stat,
            count: added,
            category: new_cat,
        };

        match diesel::insert_into(stats)
            .values(&new_stat)
            .on_conflict(stat_name)
            .do_update()
            .set(count.eq(count + added))
            .get_result::<Stat>(&conn) {


            Ok(val) => datadog::set(&format!("sushii.{}.{}", val.category, val.stat_name), val.count, &[]),
            Err(e) => warn_discord!("[DB:update_stat] Error while updating statistic: {}", e),
        };
    }
}


/// checks if a new interval has passed and reset message counts accordingly
pub fn level_interval(user_level: &UserLevel) -> UserLevel {
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

    UserLevel {
        user_id: user_level.user_id,
        guild_id: user_level.guild_id,
        msg_all_time: user_level.msg_all_time,
        msg_month,
        msg_week,
        msg_day,
        last_msg: user_level.last_msg,
    }
}

/// checks if a new interval has passed and reset message counts accordingly
pub fn level_interval_ranked(user_level: &UserLevelRanked) -> UserLevelRanked {
    let utc: DateTime<Utc> = Utc::now();
    let now = utc.naive_utc();

    let last_msg = user_level.last_msg;
    let mut msg_day = user_level.msg_day;
    let mut msg_week = user_level.msg_week;
    let mut msg_month = user_level.msg_month;

    let mut msg_day_rank = user_level.msg_day_rank;
    let mut msg_week_rank = user_level.msg_week_rank;
    let mut msg_month_rank = user_level.msg_month_rank;

    // check if new day (could possible be same day 1 year apart but unlikey)
    if now.ordinal() != last_msg.ordinal() {
        msg_day = 0;
        msg_day_rank = 0;
    }

    // check if new week
    if now.iso_week() != last_msg.iso_week() {
        msg_week = 0;
        msg_week_rank = 0;
    }

    // check if new month
    if now.month() != last_msg.month() {
        msg_month = 0;
        msg_month_rank = 0;
    }

    UserLevelRanked {
        user_id: user_level.user_id,
        guild_id: user_level.guild_id,
        msg_all_time: user_level.msg_all_time,
        msg_month,
        msg_week,
        msg_day,
        last_msg: user_level.last_msg,
        msg_day_rank,
        msg_day_total: user_level.msg_day_total,

        msg_week_rank,
        msg_week_total: user_level.msg_week_total,

        msg_month_rank,
        msg_month_total: user_level.msg_month_total,

        msg_all_time_rank: user_level.msg_all_time_rank,
        msg_all_time_total: user_level.msg_all_time_total,
    }
}
