use anyhow::{Context, Result};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use reqwest::blocking::multipart;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

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
        std::thread::sleep(Duration::from_millis(50));
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
            .with_context(|| format!("Failed to attach image file: {}", data.image_file))?;
    }

    Ok(form)
}

// Retries on 429 with exponential backoff
fn handle_response<T: serde::de::DeserializeOwned>(
    make_request: impl Fn() -> Result<reqwest::blocking::Response>,
) -> Result<T> {
    let mut attempts = 0;

    loop {
        wait_for_token();

        let resp = make_request()?;
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            attempts += 1;
            if attempts >= 5 {
                anyhow::bail!(
                    "Rate limit hit too many times, giving up after {} attempts",
                    attempts
                );
            }
            let wait = Duration::from_millis(1000 * attempts);
            println!("Rate limited, retrying in {}ms...", wait.as_millis());
            std::thread::sleep(wait);
            continue;
        }

        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("Request failed ({}): {}", status, body);
        }

        return resp.json::<T>().context("Failed to decode response");
    }
}

fn handle_response_no_body(
    make_request: impl Fn() -> Result<reqwest::blocking::Response>,
) -> Result<()> {
    let mut attempts = 0;

    loop {
        wait_for_token();

        let resp = make_request()?;
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            attempts += 1;
            if attempts >= 5 {
                anyhow::bail!("Rate limited too many times after {} attempts", attempts);
            }
            let wait = Duration::from_millis(1000 * attempts);
            println!("Rate limited, retrying in {}ms...", wait.as_millis());
            std::thread::sleep(wait);
            continue;
        }

        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("Request failed ({}): {}", status, body);
        }

        return Ok(());
    }
}

fn post<T: serde::de::DeserializeOwned>(url: &str, api_key: &str, data: &Product) -> Result<T> {
    let client = reqwest::blocking::Client::new();
    handle_response(|| {
        let form = build_form(data)?;
        client
            .post(url)
            .header("x-api-key", api_key)
            .multipart(form)
            .send()
            .context("POST request failed")
    })
}

fn patch(url: &str, api_key: &str, data: &Product) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    handle_response_no_body(|| {
        let form = build_form(data)?;
        client
            .patch(url)
            .header("x-api-key", api_key)
            .multipart(form)
            .send()
            .context("PATCH request failed")
    })
}

fn get<T: serde::de::DeserializeOwned>(url: &str, api_key: &str) -> Result<T> {
    let client = reqwest::blocking::Client::new();
    handle_response(|| {
        client
            .get(url)
            .header("x-api-key", api_key)
            .send()
            .context("GET request failed")
    })
}

pub fn create_product(universe_id: i64, api_key: &str, data: &Product) -> Result<ProductResponse> {
    post(&product_url(universe_id), api_key, data)
}

pub fn create_gamepass(
    universe_id: i64,
    api_key: &str,
    data: &Product,
) -> Result<GamepassResponse> {
    post(&gamepass_url(universe_id), api_key, data)
}

pub fn update_product(
    universe_id: i64,
    id: i64,
    api_key: &str,
    data: &Product,
) -> Result<ProductResponse> {
    patch(&product_update_url(universe_id, id), api_key, data)?;
    get(&product_info_url(universe_id, id), api_key)
}

pub fn update_gamepass(
    universe_id: i64,
    id: i64,
    api_key: &str,
    data: &Product,
) -> Result<GamepassResponse> {
    patch(&gamepass_update_url(universe_id, id), api_key, data)?;
    get(&gamepass_info_url(universe_id, id), api_key)
}
