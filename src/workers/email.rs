use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use time::{UtcDateTime, format_description};
use tokio::task;
use tokio::time::sleep;

use crate::{state::AppState, utils::capitalize_first};

static NOTIFICATION_EMAIL: Lazy<String> =
    Lazy::new(|| env::var("NOTIFICATION_EMAIL").expect("NOTIFICATION_EMAIL must be set"));

static TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

static USER_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
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

static NOTIFICATION_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
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

async fn send_email(from: &str, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
    let smtp_server = env::var("SMTP_SERVER").expect("SMTP_SERVER must be set");
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_port = env::var("SMTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(587);

    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body.to_string())?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::starttls_relay(&smtp_server)?
        .port(smtp_port)
        .credentials(creds)
        .authentication(vec![Mechanism::Plain])
        .build();

    mailer.send(&email)?;

    Ok(())
}

pub async fn email_worker(state: AppState) {
    let from_standard = env::var("SMTP_FROM").expect("Must include a SMTP_FROM email");
    let from_urgent = env::var("SMTP_FROM_URGENT").unwrap_or_else(|_| from_standard.clone());
    let from_immediate = env::var("SMTP_FROM_IMMEDIATE").unwrap_or_else(|_| from_standard.clone());

    let mut from_map = HashMap::new();
    from_map.insert("standard".to_string(), from_standard);
    from_map.insert("urgent".to_string(), from_urgent);
    from_map.insert("immediate".to_string(), from_immediate);

    let time_format = format_description::parse(TIME_FORMAT).unwrap();

    let cargo_version = env!("CARGO_PKG_VERSION").to_string();

    loop {
        let messages = sqlx::query!("SELECT id, name, email, message, priority, sender, submitted_time FROM messages WHERE status = 'pending'")
            .fetch_all(&state.db)
            .await
            .unwrap();

        if messages.is_empty() {
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        for msg in messages {
            // Clone state for new thread
            let state = state.clone();

            // Properties
            let from = from_map
                .get(msg.priority.as_str())
                .cloned()
                .expect("Priority must be one of the three options");
            let priority_capitalised = capitalize_first(msg.priority);
            let sender_type_capitalised = capitalize_first(msg.sender);
            let utc_now = UtcDateTime::now().format(&time_format).unwrap();
            let submitted_time = msg.submitted_time.to_utc().format(&time_format).unwrap();

            // Email contents
            let notification_subject = format!(
                "[Enviame] {} Message from {}({})",
                priority_capitalised, msg.name, sender_type_capitalised
            );
            let notification_body = NOTIFICATION_EMAIL_TEMPLATE
                .replace("{{message}}", &msg.message.replace("\n", "<br>"))
                .replace("{{priority}}", &priority_capitalised)
                .replace("{{name}}", &msg.name)
                .replace("{{email}}", &msg.email)
                .replace("{{status}}", &sender_type_capitalised)
                .replace("{{submitted_time}}", &submitted_time)
                .replace("{{delivered_time}}", &utc_now)
                .replace("{{version}}", &cargo_version);

            let user_subject = format!("[Enviame] {} Message Delivered", priority_capitalised);
            let user_body = USER_EMAIL_TEMPLATE
                .replace("{{name}}", &msg.name)
                .replace("{{email}}", &msg.email)
                .replace("{{message}}", &msg.message.replace("\n", "<br>"))
                .replace("{{version}}", &cargo_version);

            // Send email in new thread
            task::spawn(async move {
                sqlx::query!(
                    "UPDATE messages SET status = 'sending' WHERE id = $1",
                    msg.id
                )
                .execute(&state.db)
                .await
                .unwrap();

                let mut is_ok = true;

                let notification_result = send_email(
                    &from,
                    &NOTIFICATION_EMAIL,
                    &notification_subject,
                    &notification_body,
                )
                .await;

                if let Err(ref err) = notification_result {
                    eprintln!("{:?}", err);
                    is_ok = false;
                }

                let user_result = send_email(&from, &msg.email, &user_subject, &user_body).await;

                if let Err(ref err) = user_result {
                    eprintln!("{:?}", err);
                    is_ok = false;
                }

                let new_status = if is_ok { "sent" } else { "failed" };

                sqlx::query!(
                    "UPDATE messages SET status = $1 WHERE id = $2",
                    new_status,
                    msg.id
                )
                .execute(&state.db)
                .await
                .unwrap();
            });

            sleep(Duration::from_secs(10)).await;
        }
    }
}
