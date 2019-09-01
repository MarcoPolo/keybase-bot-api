pub mod keybase_protocol;
pub mod which_keybase {
    use std::process::Command;
    pub fn which_keybase() -> String {
        String::from_utf8(
            Command::new("which")
                .arg("keybase")
                .output()
                .expect("Which is not installed")
                .stdout,
        )
        .expect("Output not in UTF-8")
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
