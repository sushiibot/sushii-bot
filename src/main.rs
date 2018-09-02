#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![allow(unknown_lints)]
#![allow(unreadable_literal)]

#![recursion_limit="256"]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serenity;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate lazy_static;

extern crate dotenv;
extern crate env_logger;
extern crate reqwest;
extern crate typemap;
extern crate chrono;
extern crate chrono_humanize;
extern crate rand;
extern crate inflector;
extern crate regex;
extern crate darksky;
extern crate hourglass;
extern crate psutil;
extern crate sys_info;
extern crate parking_lot;
extern crate base64;
extern crate dogstatsd;
extern crate bigdecimal;
extern crate num_traits;
extern crate urbandictionary;
extern crate vlive;

pub use diesel::r2d2;

pub mod schema;
pub mod models;
#[macro_use]
pub mod utils;

#[macro_use]
mod plugins;
mod commands;
mod tasks;
mod handler;
mod database;
mod framework;
mod keys;

use serenity::prelude::*;
use chrono::Utc;
use std::{
    env,
    sync::Arc,
    collections::HashMap,
};
use dotenv::dotenv;
use database::ConnectionPool;
use framework::get_framework;
use keys::*;

fn main() {
    dotenv().ok();
    env_logger::init().expect("Failed to initialize env_logger");

    // let handler = handler::Modules::new()
    //     .add_module(handler::TestEcho);

    let (framework, commands_list) = get_framework();

    let mut client =
        Client::new(
            &env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment."),
            handler::Handler,
        ).expect("Failed to create a new client");

    {
        let mut data = client.data.lock();
        let pool = database::ConnectionPool::new();

        data.insert::<ConnectionPool>(pool);
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<Uptime>(Utc::now());
        data.insert::<CommandsList>(commands_list);
        data.insert::<GuildConfigCache>(HashMap::new());
    }

    client.with_framework(framework);

    client.threadpool.set_num_threads(10);

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
