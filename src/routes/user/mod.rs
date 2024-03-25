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
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    // Open a local connection on port 25
    let creds = Credentials::new("ddincoshopinnovation".to_string(), "tnzt sywi ywwj hysm".to_string());

// Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}