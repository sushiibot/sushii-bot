use chrono::Utc;
use chrono::naive::NaiveDateTime;

pub fn get_now_utc() -> NaiveDateTime {
    // get current timestamp
    Utc::now().naive_utc()
}
