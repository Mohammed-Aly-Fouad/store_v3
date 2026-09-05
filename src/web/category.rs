use std::collections::HashMap;
use std::ptr::null;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Form, Json, Router};

use crate::domain::category::dto::{
     CategoryResponseDTO, CategoryTemplate, CategoryTree, ChildrenTemplate, FlashParams
};
use crate::main;
use crate::state::{self, AppState};

pub fn router() -> Router<AppState> {
    // Router::new().route("/", get(render_categories_page)),
    Router::new()
        .route("/", get(show_categories))
        // .route("/{id}", get(show_category))
        // .route("/create", get(render_create_page))
        // //     .route("/{id}/edit", get(render_edit_page))
        // .route("/", post(create_category))
}


async fn show_category(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<FlashParams>,
) -> impl IntoResponse {
    let success_message = match params.action.as_deref() {
        Some("created") => Some("تم إضافة الفئة بنجاح".to_string()),
        Some("updated") => Some("تم تعديل الفئة بنجاح".to_string()),
        Some("deleted") => Some("تم حذف الفئة بنجاح".to_string()),
        _ => None,
    };

    let error_message = match params.error.as_deref() {
        Some("not_found") => Some("غير موجود بقاعدة البيانات".to_string()),
        Some("db_error") => Some("خطأ عام بقاعدة البيانات".to_string()),
        _ => None,
    };
    let result = sqlx::query_as!(
        CategoryResponseDTO,
        r#"SELECT id, name_ar, name_en, parent_id, notes, created_at, updated_at FROM categories WHERE parent_id=$1"#,
        id,
    ).fetch_all(&state.pool)
    .await;
let children = match result {
    Ok(children) => {children},
    Err(err) => {
        tracing::error!("Failed: {:?}", err);
        vec![]
    }
   
    };
ChildrenTemplate {
        children,
        error_message: None,
        success_message: None,
        current_page: "categories".to_string(),
   
    
}

}




async fn get_all_categories(state: &AppState) -> Result<Vec<CategoryResponseDTO>, sqlx::Error> {
    sqlx::query_as!(
        CategoryResponseDTO,
        r#"
        SELECT
            c.id,
            c.name_en,
            c.name_ar,
            c.parent_id,
        
            c.notes,
            c.created_at,
            c.updated_at
        FROM categories c
        ORDER BY c.id DESC;
        "#
    )
    .fetch_all(&state.pool)
    .await
}

#[axum::debug_handler]
pub async fn show_categories(
    State(state): State<AppState>,
    Query(params): Query<FlashParams>,
) -> CategoryTemplate {
     let success_message = match params.action.as_deref() {
        Some("created") => Some("تم إضافة الفئة بنجاح".to_string()),
        Some("updated") => Some("تم تعديل الفئة بنجاح".to_string()),
        Some("deleted") => Some("تم حذف الفئة بنجاح".to_string()),
        _ => None,
    };

    let error_message = match params.error.as_deref() {
        Some("not_found") => Some("غير موجود بقاعدة البيانات".to_string()),
        Some("db_error") => Some("خطأ عام بقاعدة البيانات".to_string()),
        _ => None,
    };
    match get_all_categories(&state).await {
        Ok(all_categories) => CategoryTemplate {
            category_tree: CategoryTree::build_tree(all_categories),
            error_message: None,
            success_message: success_message,
            current_page: "categories".to_string(),
        },
        Err(err) => {
            tracing::error!("Failed to fetch categories: {:?}", err);
            CategoryTemplate {
                category_tree: Vec::new(),
                error_message: Some("Failed to load categories.".to_string()),
                success_message: None,
                current_page: "categories".to_string(),
            }
        }
    }
}