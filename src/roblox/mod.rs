pub mod api;

use serde::{Deserialize, Serialize};

pub use api::{create_gamepass, create_product, update_gamepass, update_product};

const ROBLOX_API_URL: &str = "https://apis.roblox.com";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    pub name: String,
    pub description: String,
    #[serde(rename = "imageFile")]
    pub image_file: String,
    pub price: i64,
    #[serde(rename = "isRegionalPricingEnabled")]
    pub regional_pricing: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub product_id: i64,
    pub name: String,
    pub icon_image_asset_id: i64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GamepassResponse {
    pub game_pass_id: i64,
    pub name: String,
    pub icon_asset_id: i64,
}

pub fn gamepass_url(universe_id: i64) -> String {
    format!(
        "{}/game-passes/v1/universes/{}/game-passes",
        ROBLOX_API_URL, universe_id
    )
}

pub fn product_url(universe_id: i64) -> String {
    format!(
        "{}/developer-products/v2/universes/{}/developer-products",
        ROBLOX_API_URL, universe_id
    )
}

pub fn gamepass_update_url(universe_id: i64, id: i64) -> String {
    format!(
        "{}/game-passes/v1/universes/{}/game-passes/{}",
        ROBLOX_API_URL, universe_id, id
    )
}

pub fn product_update_url(universe_id: i64, id: i64) -> String {
    format!(
        "{}/developer-products/v2/universes/{}/developer-products/{}",
        ROBLOX_API_URL, universe_id, id
    )
}

pub fn gamepass_info_url(universe_id: i64, id: i64) -> String {
    format!(
        "{}/game-passes/v1/universes/{}/game-passes/{}/creator",
        ROBLOX_API_URL, universe_id, id
    )
}

pub fn product_info_url(universe_id: i64, id: i64) -> String {
    format!(
        "{}/developer-products/v2/universes/{}/developer-products/{}/creator",
        ROBLOX_API_URL, universe_id, id
    )
}
