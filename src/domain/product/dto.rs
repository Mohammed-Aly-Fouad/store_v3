
use std::{collections::HashMap, fmt::Display, str::FromStr};

use askama::Template;
use askama_web::WebTemplate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::prelude::FromRow;

use crate::domain::category::{self, dto::{CategoryResponseDTO, CategoryTree}};

pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.trim().is_empty()))
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct ProductResponseDTO {
    pub id: i64,
    pub category_id: i64,
    pub name_ar: String,
    pub name_en: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}





#[derive(Template, WebTemplate)]
#[template(path = "products/index.html")]
pub struct ProductTemplate {
    pub products: Vec<ProductWithCategoryDTO>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub current_page: String,
}


#[derive(Debug)]
pub struct ProductWithCategoryDTO {
    pub id: i64,
    pub category_id: i64,
    pub name_ar: String,
    pub name_en: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category_name_ar: String,
    pub parent_category_name_ar: Option<String>, // SQLx maps LEFT JOIN results to Option
}
#[derive(Debug)]
pub struct SubCategoriesList {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name_ar: String,
    pub name_en: String,
    pub notes: Option<String>,
    pub parent_name_ar: String,
    pub parent_name_en: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

}



#[derive(Template, WebTemplate)]
#[template(path = "products/product_create_form.html")]
pub struct CreateFormPage {
    pub categories_list: Vec<SubCategoriesList>,
    pub form: FormDTO,
    pub errors: Option<FormErrors>,
    pub current_page: String,
    pub error_message: Option<String>,
    pub success_message: Option<String>
}


#[derive(Default, Debug, Serialize)]
pub struct FormErrors {
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
}

impl FormErrors {
    pub fn has_errors(&self) -> bool {
        self.name_ar.is_some() || self.name_en.is_some() || self.notes.is_some()
    }
}

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct FormDTO {
    pub name_ar: String,
    pub name_en: String,
    pub category: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub notes: Option<String>,
}

impl FormDTO {
    pub fn sanitize(&mut self) {
        self.name_ar = self.name_ar.trim().to_string();
        self.name_en = self.name_en.trim().to_string();
        self.category = self.category.trim().to_string();
        self.notes = self.notes.take().map(|s| s.trim().to_string());
    }

    pub fn validate(&self, category_tree: &[CategoryTree]) -> Result<(), FormErrors> {
        let mut errors = FormErrors::default();

        let name_ar = self.name_ar.trim();
        if name_ar.is_empty() {
            errors.name_ar = Some("لا يمكن ترك الاسم خاليا".to_string());
            } else if name_ar.chars().count() > 100 {
            errors.name_ar = Some("لا يمكن أن تتعدى خانة الاسم 100 حرف".to_string());
        }

        let name_en = self.name_en.trim();
        if name_en.is_empty() {
            errors.name_en = Some("Thid fild is not allowed to be empty".to_string());
            } else if name_en.chars().count() > 100 {
            errors.name_en =
                Some("This filed is not allowed to be more than 100 character".to_string());
        }

        let category = self.category.trim();
        

        if let Some(notes) = self.notes.as_deref() {
            if notes.trim().chars().count() > 500 {
                errors.notes = Some("لا يمكن أن يتعدى هذا الحقل 500 حرف".to_string());
            }
        }

        if errors.has_errors() {
            Err(errors)
            } else {
            Ok(())
        }
    }
}





// ---------------------------------------------------------------------------
// 4.1 Custom Askama Filters
// ---------------------------------------------------------------------------

// We name it filter so no need to "use filters"
pub mod filters {
    use askama::Values;

    /// Extract the capitalized initial character of a brand name for UI avatar circles.
    #[askama::filter_fn]
    pub fn first_letter(name: &str, _values: &dyn Values) -> askama::Result<String> {
        Ok(name
            .trim()
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "؟".to_string()))
    }

    /// Generates a deterministic hex color code based on the brand string hash.
    /// Ensures the same brand always gets the exact same avatar background color across renders.
    #[askama::filter_fn]
    pub fn initial_color(name: &str, _values: &dyn Values) -> askama::Result<String> {
        const PALETTE: [&str; 6] = [
            "#0E7C66", "#2563EB", "#D97706", "#7C3AED", "#DB2777", "#0891B2",
        ];
        let sum: u32 = name.bytes().map(|b| b as u32).sum();
        Ok(PALETTE[sum as usize % PALETTE.len()].to_string())
    }
}

// ============================================================================
// FLASH MESSAGES & QUERY PARAMS
// ============================================================================

/// Query parameters used to carry one-time "Flash Messages" across HTTP Redirects.

#[derive(Debug, Deserialize)]
pub struct FlashParams {
    pub action: Option<String>,
    pub error: Option<String>,
}


/// Partial HTML snippet template for HTMX/Dynamic live category search.
#[derive(Template, WebTemplate)]
#[template(path = "categories/category_search_results.html")]
pub struct CategorySearchResultsTemplate {
    pub categories: Vec<CategoryResponseDTO>,
    pub query: String,
}

/// URL Query parameter extractor for category live-search requests.
#[derive(Debug, Deserialize)]
pub struct CategorySearchQuery {
    #[serde(default)]
    pub q: String,
}
