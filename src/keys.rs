use serenity::{
    client::bridge::gateway::ShardManager,
    framework::standard::CommandOptions,
};

use chrono::{DateTime, Utc};
use typemap::Key;
use database::ConnectionPool;
use parking_lot::Mutex;
use std::{
    sync::Arc,
    collections::HashMap,
};
use models;
use reqwest;


impl Key for ConnectionPool {
    type Value = ConnectionPool;
}

pub struct SerenityShardManager;
impl Key for SerenityShardManager {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct Uptime;
impl Key for Uptime {
    type Value = DateTime<Utc>;
}

pub struct CommandsList;
impl Key for CommandsList {
    type Value = HashMap<String, Arc<CommandOptions>>;
}

pub struct GuildConfigCache;
impl Key for GuildConfigCache {
    type Value = HashMap<u64, models::GuildConfig>;
}

pub struct Reqwest;
impl Key for Reqwest {
    type Value = Arc<reqwest::Client>; 
}
