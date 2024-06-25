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
        // Set the sender's name and email address
        .from("ddincoshopinnovate@gmail.com".parse().unwrap())
        // Set the recipient's name and email address
        .to(destination_email.parse().unwrap())
        // Set the subject of the email
        .subject("Verification Email")
        // Set the body content of the email
        .body(String::from(code))
        .unwrap();

    // Create SMTP client credentials using username and password
    let creds = Credentials::new("ddincoshopinnovate".to_string(), "uarjsgyzccntwole".to_string());

    // Open a secure connection to the SMTP server using STARTTLS
    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com")
        .unwrap()  // Unwrap the Result, panics in case of error
        .credentials(creds)  // Provide the credentials to the transport
        .build();  // Construct the transport

    // Attempt to send the email via the SMTP transport
    match mailer.send(&email) {
        // If email was sent successfully, print confirmation message
        Ok(_) => println!("Email sent successfully!"),
        // If there was an error sending the email, print the error
        Err(e) => eprintln!("Could not send email: {:?}", e),
    }
}
