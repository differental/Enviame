use askama::Template;
use axum::response::{Html, IntoResponse};
use axum_csrf::CsrfToken;

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
struct ApplyPageTemplate {
    csrf_token: String,
    recaptcha_site_token: String,
}

pub async fn serve_apply_form(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();
    let recaptcha_site_token = env!("RECAPTCHA_SITE_KEY").to_string();

    let template = ApplyPageTemplate {
        csrf_token,
        recaptcha_site_token,
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
