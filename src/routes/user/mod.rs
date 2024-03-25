use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::{SecurityLevel, SmtpTransportBuilder};
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

fn send_verification_code(code: &str, destination_email: &str) {
    let smtp_server = "smtp.googlemail.com";
    let smtp_username = "ddincoshopinnovate@gmail";
    let smtp_password = "tnzt sywi ywwj hysm";
    let smtp_port = 587u16;

    let email = EmailBuilder::new()
        .to(destination_email)
        .from(smtp_username)
        .subject("I am contacting you in respect of a family treasure of Gold deposited in my name")
        .body("i am Becki Ofori a Ghanian from Ashanti region Kumasi, Ghana.")
        .build().unwrap();

    let mut mailer = SmtpTransportBuilder::new((smtp_server, smtp_port)).unwrap()
        .hello_name("localhost")
        .credentials(smtp_username, smtp_password)
        .security_level(SecurityLevel::AlwaysEncrypt)
        .smtp_utf8(true)
        .build();

    let result = mailer.send(email.clone());
    match result {
        Ok(_) => println!("email sent"),
        Err(err) => println!("failed to send email alert: {}", err)
    }
}