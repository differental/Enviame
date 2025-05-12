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

use hmac::{Hmac, Mac};
use lettre::{Message, Transport, message::header::ContentType};
use rand::{Rng, distr::Alphanumeric};
use sha2::Sha256;

use crate::constants::MAILER;

pub fn generate_random_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32) // 32-character token
        .map(char::from)
        .collect()
}

pub fn capitalize_first(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

pub fn generate_hash(str: &str, hash_key: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(hash_key.as_bytes())
        .expect("HMAC can take a key of any size");
    mac.update(str.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes) // Convert to hex string
}

pub fn check_hash(str: &str, provided_hash: &str, hash_key: &str) -> bool {
    let expected_hash = generate_hash(str, hash_key);
    expected_hash == provided_hash
}

pub fn escape_html(str: String) -> String {
    str.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
        .replace('\n', "<br>")
}

pub async fn send_email(
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
