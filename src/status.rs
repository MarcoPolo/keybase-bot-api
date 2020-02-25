use super::ApiError;
use crate::bot::Bot;
use crate::keybase_cmd::{call_status, StatusRes};

pub trait Status {
    fn status(&self) -> Result<StatusRes, ApiError>;
}

impl Status for Bot {
    fn status(&self) -> Result<StatusRes, ApiError> {
        call_status(self.keybase_path.as_path(), self.home_dir.as_path())
    }
}
