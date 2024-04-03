use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

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
    let smtp_key: &str = "SB5A9hVYgzvNJywp";
    let from_email: &str = "ddincoshopinnovate@gmail.com";
    let host: &str = "smtp-relay.brevo.com";


    let email: Message = Message::builder()
        .from(from_email.parse().unwrap())
        .to(destination_email.parse().unwrap())
        .subject("Verification code")
        .body(code)
        .unwrap();

    let creds: Credentials = Credentials::new(from_email.to_string(), smtp_key.to_string());

    // Open a remote connection to gmail
    let mailer: SmtpTransport = SmtpTransport::relay(&host)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
