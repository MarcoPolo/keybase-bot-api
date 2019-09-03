use rusty_keybase_bot::keybase_cmd::which_keybase;
fn main() {
  println!("Keybase is at: {}", which_keybase());
}