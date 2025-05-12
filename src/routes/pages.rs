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
use axum::response::{Html, IntoResponse};
use axum_csrf::CsrfToken;

use crate::constants::RECAPTCHA_SITE_KEY;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPageTemplate {
    csrf_token: String,
}

pub async fn serve_index(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();

    let template = IndexPageTemplate { csrf_token };
    let rendered = template.render().unwrap();

    (token, Html(rendered)).into_response()
}

#[derive(Template)]
#[template(path = "apply.html")]
struct ApplyPageTemplate<'a> {
    csrf_token: String,
    recaptcha_site_token: &'a str,
}

pub async fn serve_apply_form(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();

    let template = ApplyPageTemplate {
        csrf_token,
        recaptcha_site_token: RECAPTCHA_SITE_KEY.as_str(),
    };
    let rendered = template.render().unwrap();

    (token, Html(rendered)).into_response()
}

#[derive(Template)]
#[template(path = "link.html")]
struct ResendLinkPageTemplate<'a> {
    csrf_token: String,
    recaptcha_site_token: &'a str,
}

pub async fn serve_resend_link_form(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();

    let template = ResendLinkPageTemplate {
        csrf_token,
        recaptcha_site_token: RECAPTCHA_SITE_KEY.as_str(),
    };
    let rendered = template.render().unwrap();

    (token, Html(rendered)).into_response()
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutPageTemplate;

pub async fn serve_about_page() -> impl IntoResponse {
    let template = AboutPageTemplate;
    let rendered = template.render().unwrap();

    Html(rendered).into_response()
}
