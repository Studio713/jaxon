<h1 align="center">jaxon</h1>

# Get Started

## Installation

The recommended way to install it is using [rokit](https://github.com/rojo-rbx/rokit) by running:

```bash
rokit add studio713/jaxon
```

Alternatively, you can build it yourself or download the pre-built binaries.

## Usage

You will need to create an [API key](https://create.roblox.com/dashboard/credentials) and grant it the following permissions

- `developer-product:write`
- `developer-product:read`
- `game-pass:write`
- `game-pass:read`

then create a `.env` file in the root directory and write:
```
JAXON_API_KEY=APIKEY
```

## Commands

### `sync`

Syncs your local products.json to Roblox.

### `init` [option]

Initializes basic Jaxon files (jaxon.toml and products.json). Options:

-m | --minimal: Only generates the toml file.

## jaxon.toml

Example structure of jaxon.toml:

```toml
[project]
universe_id = 0 # Replace with your universe ID

[generation]
typescript = false # Set to true to generate a TypeScript definition file

[files]
product_dir = ""    # Directory of the products.json
output_dir = ""     # Directory for the output files
file_name = "products.luau" # Name of the output Luau file
```

## products.json

The products.json file must be an array of objects. Example structure:

```json
[
    {
        "name": "Example",  // Name of the product
        "description": "Example description", // Description
        "type": "Product",  // Either "Product" or "Gamepass"
        "image": "assets/products/test.png",  // Image path
        "price": 499,   // Price in Robux
        "regionalPricing": false,   // Enable regional pricing
        "productId": 0  // Optional: Assigned by Jaxon. Include if modifying an existing product.
    }
]
```
