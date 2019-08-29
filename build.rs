use pest::Parser;
use avdl_parser::{Rule, AVDLParser, to_rust::build_rust_code_from_avdl};
use std::fs::{self, File, DirEntry};
use std::error::Error;
use std::io;
use std::path::Path;

fn create_rust_version(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
  let input = fs::read_to_string(input_path)?;
  let parsed = AVDLParser::parse(Rule::avdl_protocol, &input)?;
  let mut output = File::create(output_path)?;
  build_rust_code_from_avdl(parsed, &mut output)?;
  Ok(())
}

fn visit_dirs<F>(dir: &Path, cb: &F) -> io::Result<()> where F: Fn(&DirEntry) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn main() {
  visit_dirs(Path::new("keybase-protocol/chat1/"), &|entry: &DirEntry| {
    let mut output_path = Path::new("rusty-keybase-protocol/");
    let output_filename = Path::new(entry.path().file_name().unwrap()).with_extension("rs");
    let mut output_buf = output_path.to_path_buf();
    output_buf.push(output_filename);
    if let Err(e) = create_rust_version(entry.path().to_str().unwrap(), output_buf.to_str().unwrap()) {
      println!("{:#?}", e);
    } else {
      println!("Success");
    }
  });
}