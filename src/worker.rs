use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tokio::time::sleep;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

use crate::{state::AppState, utils::capitalize_first};

async fn send_email(
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> anyhow::Result<()> {
    let smtp_server = env::var("SMTP_SERVER")?;
    let smtp_username = env::var("SMTP_USERNAME")?;
    let smtp_password = env::var("SMTP_PASSWORD")?;

    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)?
        .credentials(creds)
        .build();

    mailer.send(email).await?;

    Ok(())
}

pub async fn email_worker(state: AppState) {
    let notification_email = env!("NOTIFICATION_EMAIL");
    
    let from_standard = std::env::var("SMTP_FROM").expect("Must include a SMTP_FROM email");
    let from_urgent_var = std::env::var("SMTP_FROM_URGENT");
    let from_immediate_var = std::env::var("SMTP_FROM_IMMEDIATE");

    let from_urgent = from_urgent_var.as_deref().unwrap_or(from_standard.as_ref());
    let from_immediate = from_immediate_var.as_deref().unwrap_or(from_standard.as_ref());

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
        
        println!("Fetched. {} messages found.", messages.len());

        for msg in messages {
            println!("Received: {}", msg.id);

            let priority = msg.priority.as_str();
            let from = from_map[priority];
            let priority_capitalised = capitalize_first(msg.priority);
            let utc_now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");

            let notification_subject = format!(
                "[Enviame] {} Message from {}({})",
                priority_capitalised, msg.name, msg.sender
            );
            let notification_body = format!(
                "Message details:\n\n---Start of Message---\n{}\n---End of Message---\n\nPriority: {}\nName: {}\nEmail: {}\nStatus: {}\nSubmitted at: {}\nDelivered at: {}\nMessage delivered by Enviame {}",
                msg.message,
                priority_capitalised,
                msg.name,
                msg.email,
                msg.sender,
                msg.submitted_time,
                utc_now,
                cargo_version,
            );
            println!("Sending Mail 1");
            let notification_result = send_email(&from, notification_email, &notification_subject, &notification_body).await;
            println!("Mail 1 sent {}", notification_result.is_ok());
            if let Err(ref err) = notification_result {
                println!("{:?}", err);
            }

            let user_subject = format!("[Enviame] {} Message Delivered", priority_capitalised);
            let user_body = format!(
                "Your following message has been delivered:\n\n---Start of Message---\n{}\n---End of Message---\n\nMessage written by {} ({}) and delivered by Enviame {}, {}",
                msg.message, msg.name, msg.email, cargo_version, utc_now
            );
            println!("Sending Mail 2");
            let user_result = send_email(&from, &msg.email, &user_subject, &user_body).await;
            println!("Mail 2 sent {}", user_result.is_ok());
            if let Err(ref err) = user_result {
                println!("{:?}", err);
            }

            let new_status = if notification_result.is_ok() && user_result.is_ok() { "sent" } else { "failed" };

            sqlx::query!("UPDATE messages SET status = $1 WHERE id = $2", new_status, msg.id)
                .execute(&state.db)
                .await
                .unwrap();

            println!("Db updated");

            sleep(Duration::from_secs(10)).await;
        }
        sleep(Duration::from_secs(10)).await;
    }
}