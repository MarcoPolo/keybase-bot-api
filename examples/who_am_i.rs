use keybase_bot_api::{Bot, Status};
fn main() {
    let bot = Bot::new(
        "mmou",
        option_env!("KEYBASE_PAPERKEY").expect("Missing KEYBASE_PAPERKEY env"),
    )
    .unwrap();
    println!("Username is: {:?}", bot.status().unwrap().username);
}
