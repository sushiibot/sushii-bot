use dogstatsd::{Client, Options};
use std::env;
	
lazy_static! {
    pub static ref DOGO: Client = Client::new(Options::new(
        &env::var("DD_FROM_ADDR").unwrap_or_else(|_| "127.0.0.1:8126".to_owned()),
        &env::var("DD_TO_ADDR").unwrap_or_else(|_| "127.0.0.1:8125".to_owned()),
        "")
    ).unwrap();
}

fn vec() -> Vec<String> {
    vec![env::var("DD_TAG").unwrap_or_else(|_| "sushii".to_owned())]
}

#[allow(dead_code)]
pub fn event(title: &str, content: &str, tags: &[String]) {
    DOGO.event(title, content, [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn incr(name: &str, tags: &[String]) {
    DOGO.incr(name, [&vec()[..], &tags[..]].concat()).unwrap();
}
#[allow(dead_code)]
pub fn decr(name: &str, tags: &[String]) {
    DOGO.decr(name, [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn set(name: &str, val: i64, tags: &[String]) {
    DOGO.gauge(name, &val.to_string(), [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn time<F: FnOnce()>(name: &str, tags: Vec<String>, block: F) {
    DOGO.time(name, tags, block).unwrap();
}