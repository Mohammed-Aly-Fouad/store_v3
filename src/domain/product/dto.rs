
use std::{collections::HashMap, fmt::Display, str::FromStr};

use askama::Template;
use askama_web::WebTemplate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::prelude::FromRow;

use crate::domain::category::dto::{CategoryResponseDTO, CategoryTree};

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
    pub products: Vec<ProductResponseDTO>,
    pub category_tree: Vec<CategoryTree>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub current_page: String,
}


