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

use axum::{Json, response::IntoResponse};
use serde::Serialize;

use crate::constants::{CARGO_PKG_VERSION, DEPLOY_ENV};

#[derive(Serialize)]
struct VersionResponse<'a> {
    version: &'a str,
    deployment: &'a str,
}

pub async fn handle_version() -> impl IntoResponse {
    Json(VersionResponse {
        version: CARGO_PKG_VERSION,
        deployment: DEPLOY_ENV.as_str(),
    })
    .into_response()
}
