use askama::Template;
use lettre::{Message, Transport, message::header::ContentType};
use std::{collections::HashMap, env, time::Duration};
use tokio::{task, time::sleep};

use crate::constants::{CARGO_PKG_VERSION, EMAIL_DATETIME_FORMAT, MAILER, NOTIFICATION_EMAIL};
use crate::state::AppState;
use crate::utils::capitalize_first;

#[derive(Template)]
#[template(path = "email_notification.html")]
struct NotificationEmailTemplate<'a> {
    message: &'a str,
    priority: &'a str,
    name: &'a str,
    email: &'a str,
    status: &'a str,
    submitted_time: &'a str,
    delivered_time: &'a str,
    version: &'a str,
}

#[derive(Template)]
#[template(path = "email_user.html")]
struct UserEmailTemplate<'a> {
    message: &'a str,
    name: &'a str,
    email: &'a str,
    version: &'a str,
}

async fn send_email(
    from: &str,
    to: &str,
    reply_to: &str,
    subject: &str,
    body: &str,
) -> anyhow::Result<()> {
    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .reply_to(reply_to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body.to_string())?;

    MAILER.send(&email)?;
    Ok(())
}

pub async fn email_worker(state: AppState) {
    // SMTP_FROM(s) are emails where all the emails are sent from
    // This can be different from SMTP_USERNAME
    let from_standard = env::var("SMTP_FROM").expect("Must include a SMTP_FROM email");
    let from_urgent = env::var("SMTP_FROM_URGENT").unwrap_or_else(|_| from_standard.clone());
    let from_immediate = env::var("SMTP_FROM_IMMEDIATE").unwrap_or_else(|_| from_standard.clone());

    let mut from_map = HashMap::new();
    from_map.insert("standard".to_string(), from_standard);
    from_map.insert("urgent".to_string(), from_urgent);
    from_map.insert("immediate".to_string(), from_immediate);

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
            let utc_now = chrono::Utc::now().format(EMAIL_DATETIME_FORMAT).to_string();
            let submitted_time = msg.submitted_time.format(EMAIL_DATETIME_FORMAT).to_string();

            // Email contents
            let message_content = msg.message.replace("\n", "<br>");

            let notification_subject = format!(
                "[Enviame] {} Message from {}({})",
                priority_capitalised, msg.name, sender_type_capitalised
            );
            let notification_template = NotificationEmailTemplate {
                message: &message_content,
                priority: &priority_capitalised,
                name: &msg.name,
                email: &msg.email,
                status: &sender_type_capitalised,
                submitted_time: &submitted_time,
                delivered_time: &utc_now,
                version: CARGO_PKG_VERSION,
            };
            let notification_body = notification_template
                .render()
                .expect("Notification email failed to render");

            let user_subject = format!("[Enviame] {} Message Delivered", priority_capitalised);
            let user_template = UserEmailTemplate {
                name: &msg.name,
                email: &msg.email,
                message: &message_content,
                version: CARGO_PKG_VERSION,
            };
            let user_body = user_template.render().expect("User email failed to render");

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
                    &msg.email,
                    &notification_subject,
                    &notification_body,
                )
                .await;

                if let Err(ref err) = notification_result {
                    eprintln!("Email worker failed to send notification: {:?}", err);
                    is_ok = false;
                }

                let user_result = send_email(
                    &from,
                    &msg.email,
                    &NOTIFICATION_EMAIL,
                    &user_subject,
                    &user_body,
                )
                .await;

                if let Err(ref err) = user_result {
                    eprintln!("Email worker failed to send message receipt: {:?}", err);
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
