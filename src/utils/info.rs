use std::env;

use reqwest;

use std::collections::HashMap;

pub fn bot_update_info(status: &str) {
    let webhook_url = env::var("INFO_WEBHOOK").expect("Expected INFO_WEBHOOK in the environment.");

    let mut data = HashMap::new();
    data.insert("content", status);

    // use lazy static as to not create a new client
    // every time this function is called
    // (causes thread leak, new thread for every new client)
    lazy_static! {
        static ref client: reqwest::Client = reqwest::Client::new();
    }
    let res = client
        .post(&webhook_url)
        .json(&data)
        .send();

    match res {
        Err(e) => error!("Failed to send info webhook: {}\nData: {}", e, &status),
        Ok(response) => {
            if let Err(server_err) = response.error_for_status() {
                error!(
                    "Failed to send info webhook: {}\nData: {}",
                    &server_err, &status
                );
            }
        }
    }
}
