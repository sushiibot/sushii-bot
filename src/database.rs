use diesel;
use diesel::QueryDsl;
use diesel::dsl::max;
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

use serenity;
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

        if let Ok(config) = config {
            Ok(config)
        } else {
            self.new_guild(guild_id)
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
        let result = guilds
            .filter(id.eq(guild_id as i64))
            .load::<GuildConfig>(&conn)
            .ok();


        if let Some(rows) = result {
            let guild = rows[0].clone();
            // check if guild has same prefix
            if let Some(existing_prefix) = guild.prefix {
                if new_prefix == existing_prefix {
                    return false;
                }
            }
        } else {
            // check if this is a new guild,
            // make a new config if it is
            if let Err(_) = self.new_guild(guild_id) {
                return false;
            }
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

        // fetch event
        let event = events
            .filter(name.eq(&event_name))
            .first::<EventCounter>(&conn);

        // check if a row was found
        if event.is_ok() {
            // increment the counter
            diesel::update(events)
                .filter(name.eq(&event_name))
                .set(count.eq(count + 1))
                .execute(&conn)?;
        } else {
            let new_event = NewEventCounter {
                name: &event_name,
                count: 1,
            };

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
                        DENSE_RANK() OVER(PARTITION BY EXTRACT(DOY FROM last_msg) ORDER BY msg_day DESC) AS msg_day_rank,
                        COUNT(*) OVER(PARTITION BY EXTRACT(DOY FROM last_msg)) AS msg_day_total,

                        DENSE_RANK() OVER(PARTITION BY EXTRACT(WEEK FROM last_msg) ORDER BY msg_day DESC) AS msg_week_rank,
                        COUNT(*) OVER(PARTITION BY EXTRACT(WEEK FROM last_msg)) AS msg_week_total,

                        DENSE_RANK() OVER(PARTITION BY EXTRACT(MONTH FROM last_msg) ORDER BY msg_month DESC) AS msg_month_rank,
                        COUNT(*) OVER(PARTITION BY EXTRACT(MONTH FROM last_msg)) AS msg_month_total,

                        DENSE_RANK() OVER(ORDER BY msg_all_time DESC) AS msg_all_time_rank,
                        COUNT(*) OVER() AS msg_all_time_total
                    FROM levels WHERE guild_id = $1 
                ) t
            WHERE t.user_id = $2 ORDER BY id ASC
        "#)
            .bind::<BigInt, i64>(id_guild as i64)
            .bind::<BigInt, i64>(id_user as i64)
            .load(&conn) {

            Ok(val) => val.get(0).map(|x| level_interval_ranked(&x)),
            Err(e) => {
                warn_discord!("[DB:get_level] Error while getting level: {}", e);
                None
            },
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

        if let Ok(user) = user {
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
                *elem = *elem + 1;
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
                
        } else {
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
            };

            if let Err(e) = diesel::insert_into(users::table)
                .values(&new_user)
                .execute(&conn) {

                warn_discord!("[DB:update_user_activity_message] Failed to insert new user row: {}", e);
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

    pub fn get_user_last_message(&self, id_user: u64) -> Option<NaiveDateTime> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .load::<User>(&conn)
            .ok().map(|x: Vec<User>| x[0].last_msg.clone())
    }

    pub fn get_last_rep(&self, id_user: u64) -> Option<NaiveDateTime> {
        use schema::users::dsl::*;

        let conn = self.connection();

        users
            .filter(id.eq(id_user as i64))
            .select(last_rep)
            .first::<Option<NaiveDateTime>>(&conn)
            .unwrap_or(None)
    }

    /// REP
    pub fn rep_user(&self, id_user: u64, id_target: u64, action: &str) {
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

        let result = if action == "+" {
            diesel::update(users)
                .filter(id.eq(id_target as i64))
                .set(rep.eq(rep + 1))
                .execute(&conn)
        } else {
            diesel::update(users)
                .filter(id.eq(id_target as i64))
                .set(rep.eq(rep - 1))
                .execute(&conn)
        };
        
        if let Err(e) = result {
            warn_discord!("[DB:rep_user] Error when updating rep: {}", e);
        }
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
                    Some(val) => val.clone(),
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

    pub fn update_mod_action(&self, entry: ModAction) {
        use schema::mod_log;
        use schema::mod_log::dsl::*;

        let conn = self.connection();

        // id/entry_id is the unique global serial id, not the case id
        if let Err(e) = diesel::update(mod_log::table)
            .filter(id.eq(entry.id))
            .set(&entry)
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

        match diesel::update(users::table)
            .filter(id.eq(id_user as i64))
            .set((
                latitude.eq(Some(lat)),
                longitude.eq(Some(lng)),
                address.eq(Some(loc))
            ))
            .execute(&conn) {
                Err(e) => warn_discord!("[DB:save_weather_location] Error while updating a user weather location: {}", e),
                _ => {},
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

        if let Ok(_) = mutes
            .filter(user_id.eq(id_user as i64))
            .filter(guild_id.eq(id_guild as i64))
            .first::<Mute>(&conn) {
            
            true
        } else {
            false
        }
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

    pub fn get_tags(&self, guild: u64) -> Option<Vec<Tag>> {
        use schema::tags::dsl::*;

        let conn = self.connection();

        tags
            .filter(guild_id.eq(guild as i64))
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
                    if rows == 1 {
                        true
                    } else {
                        false
                    }
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
        id: user_level.id,
        user_id: user_level.user_id,
        guild_id: user_level.guild_id,
        msg_all_time: user_level.msg_all_time,
        msg_month: msg_month,
        msg_week: msg_week,
        msg_day: msg_day,
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

    UserLevelRanked {
        id: user_level.id,
        user_id: user_level.user_id,
        guild_id: user_level.guild_id,
        msg_all_time: user_level.msg_all_time,
        msg_month: msg_month,
        msg_week: msg_week,
        msg_day: msg_day,
        last_msg: user_level.last_msg,
        msg_day_rank: user_level.msg_day_rank,
        msg_day_total: user_level.msg_day_total,

        msg_week_rank: user_level.msg_week_rank,
        msg_week_total: user_level.msg_week_total,

        msg_month_rank: user_level.msg_month_rank,
        msg_month_total: user_level.msg_month_total,

        msg_all_time_rank: user_level.msg_all_time_rank,
        msg_all_time_total: user_level.msg_all_time_total,
    }
}
