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
