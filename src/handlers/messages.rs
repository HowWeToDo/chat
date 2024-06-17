use axum::response::IntoResponse;

pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "Send Message"
}

pub(crate) async fn list_message_handler() -> impl IntoResponse {
    "List Message"
}
