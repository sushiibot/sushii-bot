use std::{thread, time};

use serenity::prelude::Context;
use serenity::model::Ready;
use serenity::http;

use chrono_humanize::HumanTime;
use std::sync::{Once, ONCE_INIT};

use database;

static INIT: Once = ONCE_INIT;

pub fn on_ready(ctx: &Context, _: &Ready) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap().clone();
    INIT.call_once(|| {
        thread::spawn(move || loop {
            let ten_sec = time::Duration::from_secs(10);

            if let Some(reminders) = pool.get_overdue_reminders() {
                // loop through reminders
                for reminder in reminders {
                    // get user by id
                    if let Ok(user) = http::get_user(reminder.user_id as u64) {
                        let since = reminder.time_set.signed_duration_since(
                            reminder.time_to_remind,
                        );

                        let ht = HumanTime::from(since);

                        let s =
                            format!(
                            "Ding dong! The reminder you set {} has expired \n```{}```",
                            ht,
                            reminder.description,
                        );
                        if let Err(why) = user.direct_message(|m| m.content(&s)) {
                            error!(
                                "Failed to send message to {} for reminder: {}\n{}",
                                user.tag(),
                                reminder.description,
                                why
                            );
                        }
                    }

                    pool.remove_reminder(reminder.id);
                }
            }

            thread::sleep(ten_sec);
        });
    });
}
