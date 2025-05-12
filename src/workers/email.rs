// Enviame - Full-stack Priority Messenger with a Rust backend that respects priority settings and delivers messages.
// Copyright (C) 2025 Brian Chen (differental)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use askama::Template;
use std::{collections::HashMap, time::Duration};
use tokio::{task, time::sleep};

use crate::constants::{
    CARGO_PKG_VERSION, EMAIL_DATETIME_FORMAT, FROM_IMMEDIATE, FROM_STANDARD, FROM_URGENT,
    NOTIFICATION_EMAIL,
};
use crate::state::AppState;
use crate::utils::{capitalize_first, escape_html, send_email};

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

pub async fn email_worker(state: AppState) {
    // SMTP_FROM(s) are emails where all the emails are sent from
    // This can be different from SMTP_USERNAME
    let from_standard = &*FROM_STANDARD;
    let from_urgent = &*FROM_URGENT;
    let from_immediate = &*FROM_IMMEDIATE;

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
            let message_content = escape_html(msg.message);

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
                    from,
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
                    from,
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
