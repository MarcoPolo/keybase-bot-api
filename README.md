# Keybase Bot API for Rust

Script Keybase Chat in Rust!

This module is a side-project/work in progress and may change or have crashers, but feel free to play with it. As long as you're logged in as a Keybase user, you can use this module to script basic chat commands.

## Prerequisites

Make sure to [install Keybase](https://keybase.io/download).

### Hello world

```rust
use keybase_bot_api::chat;

fn main() {
  let my_username = "marcopolo";
  let channel = chat::ChannelParams {
    name: format!("{},{}", my_username, "kb_monbot"),
    ..Default::default()
  };
  let msg = "Hello World";

  if let Err(e) = chat::send_msg(&channel, &msg) {
    println!("Failed to send message: {:?}", e);
  }
}
```

### More examples

Look at the examples folder for a full list of examples. Run them with cargo like so: `cargo run --example read`.
