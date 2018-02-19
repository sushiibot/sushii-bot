use dogstatsd::{Client, Options};
	
lazy_static! {
    pub static ref DOGO: Client = Client::new(Options::default()).unwrap();
}

fn vec() -> Vec<String> {
    vec!["sushii".to_string()]
}

#[allow(dead_code)]
pub fn event(title: &str, content: &str, tags: Vec<String>) {
    DOGO.event(title, content, [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn incr(name: &str, tags: Vec<String>) {
    DOGO.incr(name, [&vec()[..], &tags[..]].concat()).unwrap();
}
#[allow(dead_code)]
pub fn decr(name: &str, tags: Vec<String>) {
    DOGO.decr(name, [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn set(name: &str, val: i64, tags: Vec<String>) {
    DOGO.gauge(name, &val.to_string(), [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn time<F: FnOnce()>(name: &str, tags: Vec<String>, block: F) {
    DOGO.time(name, tags, block).unwrap();
}