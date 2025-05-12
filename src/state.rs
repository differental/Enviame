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

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, PartialEq)]
pub struct CalendarCache {
    pub is_busy: bool,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub status: Arc<RwLock<CalendarCache>>,
}
