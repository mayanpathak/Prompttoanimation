use lettre::{
    transport::smtp::authentication::Credentials,
    SmtpTransport,
};
use std::env;

pub struct MailConfig {
    pub mailer: SmtpTransport,
    pub sender_email: String,
    pub sender_name: String,
}

pub fn create_mailer() -> MailConfig {
    let host = env::var("EMAIL_HOST").unwrap();
    let user = env::var("EMAIL_USER").unwrap();
    let pass = env::var("EMAIL_PASS").unwrap();
    let sender_name = env::var("SENDER_NAME").unwrap();

    let creds = Credentials::new(user.clone(), pass);

    let mailer = SmtpTransport::relay(&host)
        .unwrap()
        .credentials(creds)
        .build();

    MailConfig {
        mailer,
        sender_email: user,
        sender_name,
    }
}