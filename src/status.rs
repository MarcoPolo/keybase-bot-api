use super::ApiError;
use crate::keybase_cmd::{StatusRes, call_status};

pub fn status() -> Result<StatusRes, ApiError> {
  call_status()
}
