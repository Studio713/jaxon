use crate::{config, products};
use anyhow::Result;

pub fn run(minimal: bool) -> Result<()> {
    config::init_toml()?;
    if !minimal {
        products::init_product_json()?;
    }

    Ok(())
}
