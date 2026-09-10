use std::default;

use axum::routing::get;
use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};

use crate::domain::product::dto::ProductResponseDTO;
use crate::state::{self, AppState};

use crate::web::category::get_all_categories;
use crate::domain::category::dto::{CategoryResponseDTO, CategoryTree, FlashParams};


pub fn router() -> Router<AppState> {
    
    Router::new()
    .route("/", get(render_products_page))
}



pub async fn get_all_products(
    state: &AppState,
) -> Result<Vec<ProductResponseDTO>, sqlx::Error> {
    let products = sqlx::query_as!(
        ProductResponseDTO,
        r#"
        SELECT id, category_id, name_ar, name_en, notes, created_at, updated_at 
        FROM products 
        ORDER BY id DESC
        "#
    )
    .fetch_all(&state.pool)
    .await?; // ترجع Err(sqlx::Error) فوراً عند حدوث أي خطأ

    Ok(products)
}



async fn render_products_page(
    State(state): State<AppState>,
    Query(params): Query<FlashParams>,
) -> Response {
    let success_message = match params.action.as_deref() {
        Some("created") => Some("تم إضافة المنتج الصنف بنجاح".to_string()),
        Some("updated") => Some("تم تعديل الصنف بنجاح".to_string()),
        Some("deleteed") => Some("تم حذف الصنف بنجاح".to_string()),
        _ => None,
    };

    let error_message = match params.error.as_deref() {
        Some("not_found")=> Some("غير موجود بقاعدة البيانات".to_string()),
        Some("db_err") => Some("خطأ عام بقاعدة البيانات".to_string()),
        _ => None,
    };

    

  


    Json("hello from product page").into_response()
}