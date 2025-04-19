use lettre::{
    SmtpTransport,
    transport::smtp::authentication::{Credentials, Mechanism},
};
use once_cell::sync::Lazy;
use std::env;

// --- Formats ---
// Datetime format, used when sending emails
pub static EMAIL_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

// Datetime format, used in calendar API. Frontend js must be updated if this is changed
pub static CALENDAR_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M";

// --- General Configuration ---
// Deploy environment, relevant in displaying beta warning and modifying db below
pub static DEPLOY_ENV: Lazy<String> =
    Lazy::new(|| env::var("DEPLOY_ENV").expect("DEPLOY_ENV must be set"));

// Whether to allow registration and form submission requests
pub static ALLOW_MODIFY_DB: Lazy<bool> =
    Lazy::new(|| *DEPLOY_ENV == "prod" || *DEPLOY_ENV == "beta");

// Cargo package version, as specified in Cargo.toml
pub static CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// --- Keys and Secrets ---
// Message ID Hash Key, used in message query API
pub static MID_HASH_KEY: Lazy<String> =
    Lazy::new(|| env::var("HASH_KEY").expect("HASH_KEY must be set"));

#[cfg(any())]
// In preparation for v1.1.0 features
// Message ID Hash Key, used in message query API
// This should not be the same as UID_HASH_KEY
pub static MID_HASH_KEY: Lazy<String> =
    Lazy::new(|| env::var("MID_HASH_KEY").expect("MID_HASH_KEY must be set"));

#[cfg(any())]
// In preparation for v1.1.0 features
// User ID Hash Key, used to generate user login tokens
// This should not be the same as MID_HASH_KEY
pub static UID_HASH_KEY: Lazy<String> =
    Lazy::new(|| env::var("UID_HASH_KEY").expect("UID_HASH_KEY must be set"));

// Recaptcha site key, embedded in HTML
pub static RECAPTCHA_SITE_KEY: Lazy<String> =
    Lazy::new(|| env::var("RECAPTCHA_SITE_KEY").expect("RECAPTCHA_SITE_KEY must be set"));

// Recaptcha secret key, used when verifying requests
pub static RECAPTCHA_SECRET_KEY: Lazy<String> =
    Lazy::new(|| env::var("RECAPTCHA_SECRET_KEY").expect("RECAPTCHA_SECRET_KEY must be set"));

// --- SMTP Email Configurations ---
// Recipient address of all notification emails, and reply_to address of all user emails
pub static NOTIFICATION_EMAIL: Lazy<String> =
    Lazy::new(|| env::var("NOTIFICATION_EMAIL").expect("NOTIFICATION_EMAIL must be set"));

// SMTP Server
pub static SMTP_SERVER: Lazy<String> =
    Lazy::new(|| env::var("SMTP_SERVER").expect("SMTP_SERVER must be set"));

// SMTP Credentials
pub static SMTP_CREDS: Lazy<Credentials> = Lazy::new(|| {
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    Credentials::new(smtp_username, smtp_password)
});

// SMTP Port
pub static SMTP_PORT: Lazy<u16> = Lazy::new(|| {
    env::var("SMTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(587)
});

// SMTP Mailer using lettre
pub static MAILER: Lazy<SmtpTransport> = Lazy::new(|| {
    SmtpTransport::starttls_relay(SMTP_SERVER.as_str())
        .expect("Invalid SMTP server")
        .port(*SMTP_PORT)
        .credentials(SMTP_CREDS.clone())
        .authentication(vec![Mechanism::Plain])
        .build()
});
