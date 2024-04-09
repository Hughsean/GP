// pub mod common;

use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub enum R {
    Login(String),
    Call(String),
}
