use serenity::model::channel::Message;
use serenity::prelude::Context;

use database::ConnectionPool;

pub fn on_message(_ctx: &Context, _pool: &ConnectionPool, msg: &Message) {
    if msg.is_private() {
        let s = format!("DM from {} ({}):\nMessage: {}\nAttachments: {:?}", msg.author.tag(),
            msg.author.id, msg.content, msg.attachments.iter().map(|x| &x.url).collect::<Vec<&String>>());
        info_discord!(&s);
    }
}
