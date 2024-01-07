use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub mod router;
pub mod create;
pub mod verify;
pub mod login;

pub(crate) const SESSION_KEY: &str = "session-key";
const FIRST_NAME: &str = "first_name";
const LAST_NAME: &str = "last_name";
const PHONE_NUMBER: &str = "phone_number";
const CODE: &str = "code";
const TYPE: &str = "type";

#[derive(Serialize, Deserialize, EnumString, Display, Debug)]
pub enum AuthType{
    Register,
    Login
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct VerificationData {
    verification_id: String,
}