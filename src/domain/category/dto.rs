use std::{collections::HashMap, fmt::Display, str::FromStr};

use askama::Template;
use askama_web::WebTemplate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::prelude::FromRow;

use crate::domain::category;

// pub fn empty_number_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
// where
//     D: Deserializer<'de>,
//     T: FromStr,
//     T::Err: Display,
// {
//     let opt = Option::<String>::deserialize(deserializer)?;

//     match opt.as_deref().map(str::trim) {
//         Some("") | None => Ok(None),
//         Some(s) => s.parse::<T>().map(Some).map_err(serde::de::Error::custom),
//     }
// }

pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.trim().is_empty()))
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct CategoryResponseDTO {
    pub id: i64,
    pub name_en: String,
    pub name_ar: String,
    pub parent_id: Option<i64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryTree {
    pub main: CategoryResponseDTO,
    pub subs: Vec<CategoryResponseDTO>,
}

impl CategoryTree {
    pub fn build_tree(categories: Vec<CategoryResponseDTO>) -> Vec<Self> {
        let (mains, subs): (Vec<_>, Vec<_>) =
            categories.into_iter().partition(|c| c.parent_id.is_none());

        mains
            .into_iter()
            .map(|main_cat| {
                let child_subs = subs
                    .iter()
                    .filter(|s| s.parent_id == Some(main_cat.id))
                    .cloned()
                    .collect();

                CategoryTree {
                    main: main_cat,
                    subs: child_subs,
                }
            })
            .collect()
    }
}
//##########################################
//########### Category Create DTOs       ############>>>>>>
//##########################################

#[derive(Default, Debug, Serialize)]
pub struct CategoryFormErrors {
    pub name_ar: Option<String>,
    pub name_en: Option<String>,
    pub parent_name: Option<String>,
    pub notes: Option<String>,
}

impl CategoryFormErrors {
    pub fn has_errors(&self) -> bool {
        self.name_ar.is_some() || self.name_en.is_some() || self.notes.is_some()
    }
}

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct CategoryFormDTO {
    pub name_ar: String,
    pub name_en: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub parent_name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub notes: Option<String>,
}

impl CategoryFormDTO {
    pub fn sanitize(&mut self) {
        self.name_ar = self.name_ar.trim().to_string();
        self.name_en = self.name_en.trim().to_string();
        self.parent_name = self.parent_name.take().map(|s| s.trim().to_string());
        self.notes = self.notes.take().map(|s| s.trim().to_string());
    }

    pub fn validate(&self, category_tree: &[CategoryTree]) -> Result<(), CategoryFormErrors> {
        let mut errors = CategoryFormErrors::default();

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

        if let Some(parent_name) = self.parent_name.as_deref() {
            let trimmed = parent_name.trim();
            let matches_any = category_tree
                .iter()
                .any(|category| category.main.name_ar == trimmed);

            if !matches_any {
                errors.parent_name = Some("هذا الاسم لا يصلح كفئة رئيسية".to_string());
            }
        }

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

//######################################################
//##########  Templates                             ###########
//######################################################
#[derive(Template, WebTemplate)]
#[template(path = "categories/index.html")]
pub struct CategoryTemplate {
    pub category_tree: Vec<CategoryTree>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub current_page: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "categories/category_detail.html")]
pub struct CategoryDetailTemplate {
    pub category_tree: Vec<CategoryTree>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub current_page: String,
}


#[derive(Template, WebTemplate)]
#[template(path = "categories/category_create_form.html")]
pub struct FormPage {
    pub category_tree: Vec<CategoryTree>,
    pub form: CategoryFormDTO,
    pub errors: Option<CategoryFormErrors>,
    pub current_page: String,
    pub error_message: Option<String>,
    pub success_message: Option<String>
}

#[derive(Template, WebTemplate)]
#[template(path = "categories/category_edit_form.html")]
pub struct EditFormPage {
    pub category_tree: Vec<CategoryTree>,
    pub form: CategoryFormDTO,
    pub errors: Option<CategoryFormErrors>,
    pub current_page: String,
    pub error_message: Option<String>,
    pub success_message: Option<String>
}

#[derive(Template, WebTemplate)]
#[template(path = "categories/children.html")]
pub struct ChildrenTemplate {
    pub children: Vec<CategoryResponseDTO>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub current_page: String,
}



//
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
