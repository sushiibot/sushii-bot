use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::id::EmojiId;
use serenity::CACHE;
use serenity::prelude::Context;
use database::ConnectionPool;

pub fn on_message(_ctx: &Context, _pool: &ConnectionPool, msg: &Message) {
    
    let mention = {
        format!("<@{}>", CACHE.read().user.id)
    };

    if msg.content == mention {
        let _ = msg.react(ReactionType::Custom {
            animated: false,
            id: EmojiId(416735154137202688),
            name: Some("sushiiPing".to_owned())
        });
    }
}
