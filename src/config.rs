use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

pub const TOML_FILE: &str = "jaxon.toml";

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub project: Project,
    pub generation: Generation,
    pub files: Files,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Project {
    pub universe_id: i64,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Generation {
    pub typescript: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Files {
    pub output: String,
}

pub fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string(TOML_FILE)
        .context("Failed to load config: jaxon.toml not found. Did you run `jaxon init`?")?;

    let config: Config = toml::from_str(&content).context("Failed to parse jaxon.toml")?;

    if config.project.universe_id == 0 {
        anyhow::bail!("Config missing required field or invalid: universe_id");
    }

    Ok(config)
}

pub fn load_env() -> Result<String> {
    let api_key =
        env::var("JAXON_API_KEY").context("JAXON_API_KEY not set in .env or environment")?;

    Ok(api_key)
}

pub fn init_toml() -> Result<()> {
    if std::path::Path::new(TOML_FILE).exists() {
        println!("jaxon.toml already exists");
        return Ok(());
    }

    let content = r#"[project]
universe_id = 0

[generation]
typescript = false

[files]
output = "src/Shared/Products.luau"
"#;

    std::fs::write(TOML_FILE, content).context("Failed to write jaxon.toml")?;
    println!("Created jaxon.toml");
    Ok(())
}
