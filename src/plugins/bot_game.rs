use serenity::prelude::Context;
use serenity::model::gateway::Ready;
use serenity::model::gateway::Game;
use std::env;

pub fn on_ready(ctx: &Context, _: &Ready) {
    match env::var("BOT_GAME") {
        Ok(val) => ctx.set_game(Game::playing(&val)),
        Err(_) => return,
    };
}