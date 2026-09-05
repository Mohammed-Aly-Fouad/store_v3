use std::{collections::HashMap, fmt::Display, str::FromStr};

use askama::Template;
use askama_web::WebTemplate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::prelude::FromRow;

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
        let (mains, subs): (Vec<_>, Vec<_>) = categories
            .into_iter()
            .partition(|c| c.parent_id.is_none());

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

#[derive(Template, WebTemplate)]
#[template(path = "categories/index.html")]
pub struct CategoryTemplate {
    pub category_tree: Vec<CategoryTree>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub current_page: String,

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
    pub error: Option<String>
}