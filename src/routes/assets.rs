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

use axum::{
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub async fn serve_embedded_assets(Path(file): Path<String>) -> Response {
    match Assets::get(&file) {
        Some(content) => {
            let body = content.data.into_owned();
            let mime = from_path(&file).first_or_octet_stream();

            let mut headers = HeaderMap::new();
            headers.insert(
                "Content-Type",
                HeaderValue::from_str(mime.as_ref()).unwrap(),
            );

            (StatusCode::OK, headers, body).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
