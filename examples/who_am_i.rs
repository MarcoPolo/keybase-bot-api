use keybase_bot_api::status;

fn main() {
  println!("Username is: {:?}", status::status().unwrap().username);
}