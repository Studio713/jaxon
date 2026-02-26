use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const PRODUCT_FILE: &str = "products.json";

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde[rename_all = "camelCase"]]
pub struct ProductJson {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub product_type: String,
    pub image: String,
    pub price: i64,
    #[serde(rename = "regionalPricing")]
    pub regional_pricing: bool,
    #[serde(rename = "productId")]
    pub id: i64,
}

pub fn read_products() -> Result<Vec<ProductJson>> {
    let content = std::fs::read_to_string("products.json")
        .context("Failed to open products.json. Did you run `jaxon init`?")?;

    let products: Vec<ProductJson> =
        serde_json::from_str(&content).context("Failed to parse products.json")?;

    Ok(products)
}

pub fn write_products(products: &[ProductJson]) -> Result<()> {
    let data = serde_json::to_string_pretty(products).context("Failed to serialize data")?;
    std::fs::write(PRODUCT_FILE, data).context("Failed to write products.json")?;
    Ok(())
}

pub fn init_product_json() -> Result<()> {
    if std::path::Path::new(PRODUCT_FILE).exists() {
        println!("products.json already exists");
        return Ok(());
    }

    let defaults = vec![
        ProductJson {
            name: "Example Product".into(),
            description: "Example product's description".into(),
            product_type: "Product".into(),
            image: "assets/products/example.png".into(),
            price: 499,
            regional_pricing: false,
            id: 0,
        },
        ProductJson {
            name: "Example Gamepass".into(),
            description: "Example gamepass's description".into(),
            product_type: "Gamepass".into(),
            image: "assets/gamepasses/example.png".into(),
            price: 499,
            regional_pricing: false,
            id: 0,
        },
    ];

    let data =
        serde_json::to_string_pretty(&defaults).context("Failed to serialize default products")?;
    std::fs::write(PRODUCT_FILE, data).context("Failed to write products.json")?;

    println!("Created products.json");
    Ok(())
}
