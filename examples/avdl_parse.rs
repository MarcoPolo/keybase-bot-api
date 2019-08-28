use pest::Parser;
use rusty_keybase_bot::avdl::{self, AVDLParser};
use std::fs;

// @namespace("keybase.1")
fn main() {
  let input = fs::read_to_string("keybase-protocol/keybase1/common.avdl")
    .expect("Something went wrong reading the file");
  let parsed = AVDLParser::parse(avdl::Rule::avdl_protocol, &input);
  if let Err(e) = parsed {
    println!("{}", e);
  } else {
    println!("{:#?}", parsed.unwrap());
  }
}
