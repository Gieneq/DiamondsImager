use axum::{
    Router, 
    routing::get
};

use crate::handlers::overall_status_handler;

pub fn get_router() -> Router {
    Router::new().route("/", get(overall_status_handler))
}