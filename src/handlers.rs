use axum::response::Html;

pub async fn overall_status_handler() -> Html<&'static str> {
    Html("<h1>Diamonds imager is running!</h1>")
}