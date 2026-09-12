use std::default;

use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};

use crate::domain::{category, product};
use crate::domain::product::dto::{ProductResponseDTO, ProductTemplate, ProductWithCategoryDTO, SubCategoriesList, FormDTO, FormErrors, FlashParams, CreateFormPage};
use crate::state::{self, AppState};

use crate::web::category::get_all_categories;


pub fn router() -> Router<AppState> {
    
    Router::new()
    .route("/", get(render_products_page))
    .route("/", post(create_product))
    .route("/new", get(render_new_product_page))
}






pub async fn get_products_with_full_category(
    state: &AppState,
) -> Result<Vec<ProductWithCategoryDTO>, sqlx::Error> {
    let products = sqlx::query_as!(
        ProductWithCategoryDTO,
        r#"
        SELECT 
            p.id,
            p.category_id,
            p.name_ar,
            p.name_en,
            p.notes,
            p.created_at,
            p.updated_at,
            c.name_ar AS category_name_ar,
            parent.name_ar AS parent_category_name_ar
        FROM products p
        INNER JOIN categories c ON p.category_id = c.id
        LEFT JOIN categories parent ON c.parent_id = parent.id
        ORDER BY p.id DESC
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(products)
}


async fn get_sub_categories_list(
    state: &AppState
) -> Result<Vec<SubCategoriesList>, sqlx::Error> {
    let sub_categories_list = sqlx::query_as!(
        SubCategoriesList,
        r#"
        SELECT
    c.id,
    c.parent_id,
    c.name_en,
    c.name_ar,
    c.notes,
    c.created_at,
    c.updated_at,
    p.name_en AS parent_name_en,
    p.name_ar AS parent_name_ar
FROM public.categories c
INNER JOIN public.categories p ON c.parent_id = p.id
WHERE c.parent_id IS NOT NULL;
        "#
    ).fetch_all(&state.pool)
    .await?;
Ok(sub_categories_list)
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

    let products = match get_products_with_full_category(&state).await {
        Ok(products) => products,
        Err(err) => {
            tracing::error!("فشل جلب الفئات: {:#?}", err);
            return Redirect::to("/web/products?error=server_error").into_response();
        }
    };



   return  ProductTemplate {
    products,
    error_message: None,
    success_message: None,
    current_page: "products".to_string(),
    }.into_response()

}

async fn render_new_product_page(
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

    match get_sub_categories_list(&state).await {
        Ok(categories_list) => CreateFormPage {
            form: FormDTO::default(),
            categories_list,
            errors: None,
            current_page: "categories".to_string(),
            success_message: None,
            error_message: None,
        }.into_response(),

       Err(err) => CreateFormPage {
    form: FormDTO::default(),
    categories_list: vec![],
    errors: None,
    current_page: "categories".to_string(),
    success_message: None,
    error_message: Some(err.to_string()),
}.into_response(),
    }
   }

async fn create_product(
    State(state): State<AppState>,
    Query(params): Query<FlashParams>,
    Form(mut form): Form<FormDTO>
) -> Response {
    Json("بسم الله الرحمن الرحيم").into_response()
}