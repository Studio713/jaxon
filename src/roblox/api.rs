use anyhow::{Context, Result};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use reqwest::blocking::multipart;
use std::{num::NonZeroU32, path::Path, sync::OnceLock};

use super::{
    GamepassResponse, Product, ProductResponse, gamepass_info_url, gamepass_update_url,
    gamepass_url, product_info_url, product_update_url, product_url,
};
static RATE_LIMITER: OnceLock<DefaultDirectRateLimiter> = OnceLock::new();

fn get_limiter() -> &'static DefaultDirectRateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::direct(Quota::per_second(NonZeroU32::new(3).unwrap())))
}

fn wait_for_token() {
    let limiter = get_limiter();

    while limiter.check().is_err() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn build_form(data: &Product) -> Result<multipart::Form> {
    let mut form = multipart::Form::new()
        .text("name", data.name.clone())
        .text("description", data.description.clone())
        .text("price", data.price.to_string())
        .text("isForSale", "true")
        .text(
            "isRegionalPricingEnabled",
            data.regional_pricing.to_string(),
        );

    if !data.image_file.is_empty() && Path::new(&data.image_file).exists() {
        form = form
            .file("imageFile", &data.image_file)
            .with_context(|| format!("Failed to attach image file {}", data.image_file))?;
    }

    Ok(form)
}

fn post<T: serde::de::DeserializeOwned>(
    url: &str,
    api_key: &str,
    form: multipart::Form,
) -> Result<T> {
    wait_for_token();

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(url)
        .header("x-api-key", api_key)
        .multipart(form)
        .send()
        .context("POST request failed")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("Request failed ({}): {}", status, body);
    }

    resp.json::<T>().context("Failed to decode response")
}

fn patch(url: &str, api_key: &str, form: multipart::Form) -> Result<()> {
    wait_for_token();

    let client = reqwest::blocking::Client::new();
    let resp = client
        .patch(url)
        .header("x-api-key", api_key)
        .multipart(form)
        .send()
        .context("PATCH request failed")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("Request failed ({}): {}", status, body);
    }

    Ok(())
}

fn get<T: serde::de::DeserializeOwned>(url: &str, api_key: &str) -> Result<T> {
    wait_for_token();

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header("x-api-key", api_key)
        .send()
        .context("GET request failed")?;

    resp.json::<T>().context("Failed to decode response")
}
pub fn create_product(universe_id: i64, api_key: &str, data: &Product) -> Result<ProductResponse> {
    let url = product_url(universe_id);
    let form = build_form(data)?;
    post::<ProductResponse>(&url, api_key, form)
}

pub fn create_gamepass(
    universe_id: i64,
    api_key: &str,
    data: &Product,
) -> Result<GamepassResponse> {
    let url = gamepass_url(universe_id);
    let form = build_form(data)?;
    post::<GamepassResponse>(&url, api_key, form)
}

pub fn update_product(
    universe_id: i64,
    id: i64,
    api_key: &str,
    data: &Product,
) -> Result<ProductResponse> {
    let url = product_update_url(universe_id, id);
    let form = build_form(data)?;
    patch(&url, api_key, form)?;

    let info_url = product_info_url(universe_id, id);
    get::<ProductResponse>(&info_url, api_key)
}

pub fn update_gamepass(
    universe_id: i64,
    id: i64,
    api_key: &str,
    data: &Product,
) -> Result<GamepassResponse> {
    let url = gamepass_update_url(universe_id, id);
    let form = build_form(data)?;
    patch(&url, api_key, form)?;

    let info_url = gamepass_info_url(universe_id, id);
    get::<GamepassResponse>(&info_url, api_key)
}
