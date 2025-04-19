use lettre::{
    SmtpTransport,
    transport::smtp::authentication::{Credentials, Mechanism},
};
use once_cell::sync::Lazy;
use std::env;

// --- Formats ---
// Datetime format, used when sending emails
pub static EMAIL_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M";

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

// --- Email Configurations ---
// Notification email, used when sending emails
pub static NOTIFICATION_EMAIL: Lazy<String> =
    Lazy::new(|| env::var("NOTIFICATION_EMAIL").expect("NOTIFICATION_EMAIL must be set"));

// Email templates
pub static USER_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Message Delivered Successfully</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            background-color: #f4f4f4;
            padding: 20px;
        }}
        .container {{
            max-width: 500px;
            background: #ffffff;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0px 0px 10px rgba(0, 0, 0, 0.1);
            margin: auto;
        }}
        .header {{
            font-size: 18px;
            font-weight: bold;
            color: #333;
        }}
        .message {{
            font-size: 16px;
            color: #555;
            padding: 10px;
            background: #f9f9f9;
            border-left: 4px solid #007BFF;
            margin-top: 10px;
        }}
        .footer {{
            font-size: 12px;
            color: #777;
            margin-top: 20px;
            text-align: center;
        }}
    </style>
</head>
<body>

<div class="container">
    <div class="header">Your message has been delivered successfully. A copy has been attached below.</div>
    
    <div class="message">
        <p><strong>From:</strong> {{name}} ({{email}})</p>
        <p><strong>Message:</strong></p>
        <p>{{message}}</p>
    </div>

    <div class="footer">
        Delivered by Enviame {{version}}
    </div>
</div>

</body>
</html>"#;

pub static NOTIFICATION_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Message Notification</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            background-color: #f4f4f4;
            padding: 20px;
        }}
        .container {{
            max-width: 500px;
            background: #ffffff;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0px 0px 10px rgba(0, 0, 0, 0.1);
            margin: auto;
        }}
        .message {{
            font-size: 16px;
            color: #555;
            padding: 10px;
            background: #f9f9f9;
            border-left: 4px solid #007BFF;
            margin-top: 10px;
        }}
        .details {{
            font-size: 14px;
            color: #444;
            margin-top: 15px;
            padding: 10px;
            background: #f4f4f4;
            border-radius: 5px;
        }}
        .footer {{
            font-size: 12px;
            color: #777;
            margin-top: 20px;
            text-align: center;
        }}
    </style>
</head>
<body>

<div class="container">
    <div class="message">
        <p><strong>Message:</strong></p>
        <p>{{message}}</p>
    </div>

    <div class="details">
        <p><strong>Priority:</strong> {{priority}}</p>
        <p><strong>Name:</strong> {{name}}</p>
        <p><strong>Email:</strong> {{email}}</p>
        <p><strong>Status:</strong> {{status}}</p>
        <p><strong>Submitted at:</strong> {{submitted_time}}</p>
        <p><strong>Delivered at:</strong> {{delivered_time}}</p>
    </div>

    <div class="footer">
        Message delivered by Enviame {{version}}
    </div>
</div>

</body>
</html>"#;
