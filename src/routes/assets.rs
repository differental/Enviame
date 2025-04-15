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
