use serenity::model::Message;
use serenity::prelude::Context;

pub fn on_message(ctx: Context, msg: Message) {
    println!("received message {}", msg.content);
}
