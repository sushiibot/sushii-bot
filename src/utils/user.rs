pub fn get_id(value: &str) -> Option<u64> {
    // check if it's already an ID
    if let Ok(id) = value.parse::<u64>() {
        return Some(id);
    }

    // Derived from https://docs.rs/serenity/0.4.5/src/serenity/utils/mod.rs.html#158-172
    if value.starts_with("<@!") {
        let len = value.len() - 1;
        value[3..len].parse::<u64>().ok()
    } else if value.starts_with("<@") {
        let len = value.len() - 1;
        value[2..len].parse::<u64>().ok()
    } else {
        None
    }
}
