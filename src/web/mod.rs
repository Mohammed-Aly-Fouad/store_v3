pub mod category;
pub mod product;

use crate::{state::AppState};
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/categories", category::router())
        .nest("/products", product::router())
      
}