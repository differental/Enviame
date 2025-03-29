use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use time::format_description;
use tokio::time::sleep;

use crate::{state::AppState, utils::capitalize_first};

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
    let smtp_server = env::var("SMTP_SERVER")?;
    let smtp_username = env::var("SMTP_USERNAME")?;
    let smtp_password = env::var("SMTP_PASSWORD")?;
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
    let notification_email = env!("NOTIFICATION_EMAIL");

    let from_standard = std::env::var("SMTP_FROM").expect("Must include a SMTP_FROM email");
    let from_urgent_var = std::env::var("SMTP_FROM_URGENT");
    let from_immediate_var = std::env::var("SMTP_FROM_IMMEDIATE");

    let from_urgent = from_urgent_var.as_deref().unwrap_or(from_standard.as_ref());
    let from_immediate = from_immediate_var
        .as_deref()
        .unwrap_or(from_standard.as_ref());

    let mut from_map = HashMap::new();
    from_map.insert("standard", from_standard.as_ref());
    from_map.insert("urgent", from_urgent);
    from_map.insert("immediate", from_immediate);

    let cargo_version = env!("CARGO_PKG_VERSION");

    loop {
        let messages = sqlx::query!("SELECT id, name, email, message, priority, sender, submitted_time FROM messages WHERE status = 'pending'")
            .fetch_all(&state.db)
            .await
            .unwrap();

        for msg in messages {
            let priority = msg.priority.as_str();
            let from = from_map[priority];
            let priority_capitalised = capitalize_first(msg.priority);
            let sender_type_capitalised = capitalize_first(msg.sender);
            let utc_now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let format =
                format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
            let submitted_time = msg.submitted_time.to_utc().format(&format).unwrap();

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

            let notification_result = send_email(
                &from,
                notification_email,
                &notification_subject,
                &notification_body,
            )
            .await;

            if let Err(ref err) = notification_result {
                eprintln!("{:?}", err);
            }

            let user_subject = format!("[Enviame] {} Message Delivered", priority_capitalised);
            let user_body = USER_EMAIL_TEMPLATE
                .replace("{{name}}", &msg.name)
                .replace("{{email}}", &msg.email)
                .replace("{{message}}", &msg.message.replace("\n", "<br>"))
                .replace("{{version}}", &cargo_version);

            let user_result = send_email(&from, &msg.email, &user_subject, &user_body).await;

            if let Err(ref err) = user_result {
                eprintln!("{:?}", err);
            }

            let new_status = if notification_result.is_ok() && user_result.is_ok() {
                "sent"
            } else {
                "failed"
            };

            sqlx::query!(
                "UPDATE messages SET status = $1 WHERE id = $2",
                new_status,
                msg.id
            )
            .execute(&state.db)
            .await
            .unwrap();

            sleep(Duration::from_secs(10)).await;
        }
        sleep(Duration::from_secs(10)).await;
    }
}
