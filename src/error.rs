use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("العنصر موجود بالفعل: {0}")]
    Duplicate(String), // 409 Conflict (23505)

    #[error("بيانات غير صالحة: {0}")]
    BadRequest(String), // 400 Bad Request (23503, 23514, 23502)

    #[error("العنصر غير موجود")]
    NotFound, // 404 Not Found

    #[error("خطأ في قاعدة البيانات: {0}")]
    Database(sqlx::Error), // 500 Internal Server Error
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db_err) = err {
            let constraint = db_err.constraint().unwrap_or("");
            let column = db_err.column().unwrap_or("");

            match db_err.code().as_deref() {
                // 1. unique_violation (23505)
                Some("23505") => {
                    let message = match constraint {
                        "idx_products_unique_name_ar_lower" => "اسم المنتج بالعربية مستخدم بالفعل",
                        "idx_products_unique_name_en_lower" => "اسم المنتج بالإنجليزية مستخدم بالفعل",
                        _ => "هذا العنصر مسجل مسبقاً في النظام",
                    };
                    return AppError::Duplicate(message.to_string());
                }

                // 2. foreign_key_violation (23503)
                Some("23503") => {
                    let message = match constraint {
                        "fk_products_category" => "الفئة المختارة غير موجودة في النظام",
                        _ => "تم الإشارة إلى عنصر غير موجود في قاعدة البيانات",
                    };
                    return AppError::BadRequest(message.to_string());
                }

                // 3. check_violation (23514) - يشمل أخطاء التريجر الخاص بالفئات
                Some("23514") => {
                    let db_msg = db_err.message();
                    let message = if db_msg.contains("top-level category") {
                        "لا يمكن ربط المنتج بفئة رئيسية، يجب اختيار فئة فرعية"
                    } else {
                        "البيانات المدخلة تخالف شروط صحة المدخلات في النظام"
                    };
                    return AppError::BadRequest(message.to_string());
                }

                // 4. not_null_violation (23502)
                Some("23502") => {
                    let message = if !column.is_empty() {
                        format!("الحقل ({}) مطلوب ولا يمكن أن يكون فارغاً", column)
                    } else {
                        "هناك حقل مطلوب لم يتم إدخاله".to_string()
                    };
                    return AppError::BadRequest(message);
                }

                _ => {}
            }
        }

        AppError::Database(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Duplicate(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound => (StatusCode::NOT_FOUND, "الطلب غير موجود".to_string()),
            AppError::Database(ref err) => {
                // تسجيل التفاصيل التقنية داخل أوراق السيرفر فقط
                tracing::error!("Database error details: {:#?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "حدث خطأ داخلي في الخادم".to_string())
            }
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}