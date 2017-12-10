use serenity::model::event::ResumedEvent;
use serenity::model::Ready;
use serenity::prelude::*;

pub struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.tag());
    }

    fn on_resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}
