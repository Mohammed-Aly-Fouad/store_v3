use std::collections::HashMap;
use std::ptr::null;
use std::result;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Form, Json, Router};
use sqlx::encode::IsNull::No;

use crate::domain::category;
use crate::domain::category::dto::{
    CategoryDetailTemplate, CategoryFormDTO, CategoryResponseDTO, CategorySearchQuery, CategorySearchResultsTemplate, CategoryTemplate, CategoryTree, ChildrenTemplate, CreateFormPage, EditFormPage, FlashParams,
};
use crate::main;
use crate::state::{self, AppState};

//#####################################
//######        Router            ############
//#####################################

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(render_categories_page))
        .route("/", post(create_category))
        .route("/new", get(render_new_category_page))
        .route("/{id}", get(render_main_category_details_page))
        .route("/{id}/edit", get(render_edit_category_form))
        .route("/{id}/edit", post(edit_category))
        .route("/search", get(search_categories))
}
//#########################################
//#########  get all categories handler  ################################
//#########################################

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

//#########################################
//#########  get Node (main with subs)  ################################
//#########################################
async fn get_category_branch(
    state: &AppState,
    id: i64,
) -> Result<Vec<CategoryResponseDTO>, sqlx::Error> {
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
WHERE c.id = $1 OR c.parent_id = $1
ORDER BY c.parent_id IS NOT NULL, c.id DESC;
        "#,
        id
    )
    .fetch_all(&state.pool)
    .await
}


//#########################################
//#########  fetch category by id  ################################
//#########################################

async fn fetch_category_by_id(
    state: &AppState,
    id: i64,
) -> Result<CategoryFormDTO, sqlx::Error> {
    sqlx::query_as!(
        CategoryFormDTO,
        r#"SELECT
        c.name_en,
        c.name_ar,
        c.notes,
        p.name_ar AS "parent_name?"
    FROM categories c
    LEFT JOIN categories p ON c.parent_id = p.id
    WHERE c.id = $1"#,
    id,
    ).fetch_one(&state.pool)
    .await
}
//#########################################
//########## render categories page     ###############################
//#########################################

#[axum::debug_handler]
pub async fn render_categories_page(
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
            error_message,
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

//#########################################
//########## render create page     ###############################
//#########################################
#[axum::debug_handler]
async fn render_new_category_page(
    State(state): State<AppState>,
    Query(params): Query<FlashParams>,
) -> impl IntoResponse {
    match get_all_categories(&state).await {
        Ok(all_categories) => CreateFormPage {
            category_tree: CategoryTree::build_tree(all_categories),
            form: CategoryFormDTO::default(),
            errors: None,
            current_page: "categories".to_string(),
            success_message: None,
            error_message: None,
        },

        Err(err) => CreateFormPage {
            category_tree: Vec::new(),
            form: CategoryFormDTO::default(),
            errors: None,
            current_page: "categories".to_string(),
            success_message: None,
            error_message: Some(err.to_string()),
        },
    }
}

//#########################################
//########## Create Category handler     ###############################
//#########################################
async fn create_category(
    State(state): State<AppState>,
    Query(params): Query<FlashParams>,
    Form(mut form): Form<CategoryFormDTO>,
) -> Response {
    let all_categories = match get_all_categories(&state).await {
        Ok(categories) => categories,
        Err(err) => {
            tracing::error!("فشل جلب الفئات: {:#?}", err);
            return Redirect::to("/web/categories?error=server_error").into_response();
        }
    };

    let category_tree = CategoryTree::build_tree(all_categories);
    form.sanitize();
    match form.validate(&category_tree) {
        Err(err) => {
            tracing::error!("فشل التحقق من صحة النموذج: {:#?}", err);
            return CreateFormPage {
                form,
                category_tree,
                error_message: Some("من فضلك عدل الأخطاء لاستكمال التسجيل".to_string()),
                success_message: None,
                errors: Some(err),
                current_page: "categories".to_string(),
            }
            .into_response();
        }
        Ok(_) => {
            let insert_result = sqlx::query!(
                r#"
                INSERT INTO categories (name_en, name_ar, parent_id, notes)
                VALUES (
                $1,
                $2,
                (SELECT id FROM categories WHERE name_ar = $3 LIMIT 1),
                 $4
                 )
                 "#,
                &form.name_en,
                &form.name_ar,
                form.parent_name.as_deref(),
                form.notes.as_deref(),
            )
            .execute(&state.pool)
            .await;

            match insert_result {
                Ok(_) => Redirect::to("/web/categories?action=created").into_response(),
                Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                    let err_msg = format!("الفئة \"{}\" مسجلة بالفعل", form.name_ar);
                    return CreateFormPage {
                        form,
                        category_tree,
                        error_message: Some(err_msg),
                        success_message: None,
                        errors: None,
                        current_page: "categories".to_string(),
                    }
                    .into_response();
                }
                Err(err) => {
                    tracing::error!("خطأ عام: {:#?}", err);
                    return CreateFormPage {
                        form,
                        category_tree,
                        error_message: None,
                        success_message: None,
                        errors: None,
                        current_page: "categories".to_string(),
                    }
                    .into_response();
                }
            }
        }
    }
}
//#########################################
//########## render main category details page     ###############################
//#########################################
async fn render_main_category_details_page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<FlashParams>,
) -> impl IntoResponse {
    let node = match get_category_branch(&state, id).await {
        Ok(categories) => categories,
        Err(err) => {
            tracing::error!("فشل جلب الفئات: {:#?}", err);
            return Redirect::to("/web/categories?error=server_error").into_response();
        }
    }; 
    let category_tree = CategoryTree::build_tree(node);
    // Json(category_tree).into_response()
    CategoryDetailTemplate {
        category_tree,
        success_message: None,
        error_message: None,
        current_page: "categories".to_string(),
    }.into_response()
}

//#########################################
//########## render edit form     ###############################
//#########################################

async fn render_edit_category_form(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<FlashParams>,
) -> Response {
let all_categories = match get_all_categories(&state).await {
        Ok(categories) => categories,
        Err(err) => {
            tracing::error!("فشل جلب الفئات: {:#?}", err);
            return Redirect::to("/web/categories?error=server_error").into_response();
        }
    };

    let category_tree = CategoryTree::build_tree(all_categories);

    let category = sqlx::query_as!(
        CategoryFormDTO,
        r#"SELECT
        c.name_en,
        c.name_ar,
        c.notes,
        p.name_ar AS "parent_name?"
    FROM categories c
    LEFT JOIN categories p ON c.parent_id = p.id
    WHERE c.id = $1"#,
    id,
    ).fetch_optional(&state.pool)
    .await;
    
    match category {
        Ok(Some(cat)) => {
            let form = CategoryFormDTO {
                name_ar: cat.name_ar,
                name_en: cat.name_en,
                parent_name: cat.parent_name,
                notes: cat.notes,
               
            };

  EditFormPage {
                            form,
                            category_tree,
                            error_message: None,
                            success_message: None,
                            errors: None,
                            current_page: "categories".to_string(),
                            id,
                        }
                        .into_response()

                    }
            Ok(None) => {
                Redirect::to("/web/categories?error=not_found").into_response()
            }

            Err(e) => {
                tracing::error!("Failed to fetch category for edit: {:?}", e);
                Redirect::to("/web/categories?error=db_error").into_response()
        }
    }
    
}


//#########################################
//########## Edit Category handler     ###############################
//#########################################

async fn edit_category(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(mut form): Form<CategoryFormDTO>
) -> Response {

    let mut submitted_form = form;

    let existing_cat = match fetch_category_by_id(&state, id).await {
        Ok(cat) => cat,
        Err(e) => {
            tracing::error!(?e, "Failedto fetch category");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server errore",
            ).into_response()
        }
    };
    
    let all_categories = match get_all_categories(&state).await {
        Ok(categories) => categories,
        Err(err) => {
            tracing::error!("فشل جلب الفئات: {:#?}", err);
            return Redirect::to("/web/categories?error=server_error").into_response();
        }
    };

    let category_tree = CategoryTree::build_tree(all_categories); 

    submitted_form.sanitize();

    if let Err(form_err) = submitted_form.validate(&category_tree) {
        return EditFormPage {
            category_tree,
            id,
            form: submitted_form,
            errors: Some(form_err),
            current_page: "categories".to_string(),
            error_message: Some("يرجى تصحيح الأخطاء لإستكمال التعديل".to_string()),
            success_message: None,
        }
        .into_response();
    }

        let is_unchanged =
        existing_cat.name_ar == submitted_form.name_ar
        && existing_cat.name_en == submitted_form.name_en
        &&  existing_cat.parent_name == submitted_form.parent_name
        && existing_cat.notes == submitted_form.notes;


        if is_unchanged {
            return EditFormPage {
            category_tree,
            id,
            form: submitted_form,
            errors: None,
            current_page: "categories".to_string(),
           error_message: Some("لم يتم إجراء أي تغييرات على البيانات".to_string()),
            success_message: None,
        }.into_response()
        }
        
        let update_result = sqlx::query!(
    r#"
    UPDATE categories
    SET
        name_en = $1,
        name_ar = $2,
        parent_id = (SELECT id FROM categories WHERE name_ar = $3 LIMIT 1),
        notes = $4
    WHERE id = $5
    "#,
    &submitted_form.name_en,
    &submitted_form.name_ar,
    submitted_form.parent_name.as_deref(),
    submitted_form.notes.as_deref(),
    id,
)
.execute(&state.pool)
.await;

           Json("Ed").into_response()
    }
            
// ###########################################
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
        Ok(children) => children,
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
// ============================================================================
// HANDLERS: LIVE SEARCH
// ============================================================================

/// Dynamic search handler returning a rendered Askama partial snippet.
/// Designed for live search / auto-complete integrations.
pub async fn search_categories(
    State(state): State<AppState>,
    Query(query): Query<CategorySearchQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let q = query.q.trim();

    // إرجاع استجابة فارغة فوراً إن كان الاستعلام خالياً
    if q.is_empty() {
        return Ok(CategorySearchResultsTemplate {
            categories: vec![],
            query: String::new(),
        });
    }

    // إعداد نمط البحث غير حساس للحالة (Case-Insensitive) للغتين العربية والإنجليزية
    let search_pattern = format!("%{}%", q);

let categories = sqlx::query_as!(
    CategoryResponseDTO,
    r#"
    SELECT id, name_en, name_ar, parent_id, notes, created_at, updated_at
    FROM categories
    WHERE name_en ILIKE $1 
       OR regexp_replace(TRANSLATE(name_ar, 'أإآىة', 'ااايه'), '[\u064B-\u0652]', '', 'g') 
          ILIKE regexp_replace(TRANSLATE($1, 'أإآىة', 'ااايه'), '[\u064B-\u0652]', '', 'g')
    ORDER BY name_ar ASC
    LIMIT 10
    "#,
    search_pattern
)
.fetch_all(&state.pool)
.await
.map_err(|err| {
    tracing::error!("Failed to execute category search query: {:?}", err);
    StatusCode::INTERNAL_SERVER_ERROR
})?;

    Ok(CategorySearchResultsTemplate {
        categories,
        query: q.to_string(),
    })
}