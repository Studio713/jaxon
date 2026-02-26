use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    code::{ProductCodeMap, generate_code},
    config, lock, products,
    roblox::{self, Product},
};

const MAX_WORKERS: usize = 1;

pub fn run() -> Result<()> {
    let config = config::load_config()?;
    let api_key = config::load_env()?;

    let products_list = products::read_products()?;
    let hashes = lock::get_hashes()?;

    let hashes = Arc::new(Mutex::new(hashes));
    let products_map: Arc<Mutex<HashMap<i64, ProductCodeMap>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let passes_map: Arc<Mutex<HashMap<i64, ProductCodeMap>>> = Arc::new(Mutex::new(HashMap::new()));
    let products_list = Arc::new(Mutex::new(products_list));

    let universe_id = config.project.universe_id;
    let api_key = Arc::new(api_key);

    // Channel for errors from worker threads
    let (err_tx, err_rx) = std::sync::mpsc::channel::<anyhow::Error>();
    let (tx, rx) = std::sync::mpsc::channel::<usize>();
    let rx = Arc::new(Mutex::new(rx));

    println!("Syncing products...");

    // Enqueue jobs first
    let job_count = products_list.lock().unwrap().len();
    for i in 0..job_count {
        tx.send(i).ok();
    }
    drop(tx);
    let mut handles = vec![];
    for _ in 0..MAX_WORKERS {
        let rx = Arc::clone(&rx);
        let hashes = Arc::clone(&hashes);
        let products_map = Arc::clone(&products_map);
        let passes_map = Arc::clone(&passes_map);
        let products_list = Arc::clone(&products_list);
        let api_key = Arc::clone(&api_key);
        let err_tx = err_tx.clone();

        let handle = thread::spawn(move || {
            loop {
                let idx = {
                    let rx = rx.lock().unwrap();
                    match rx.recv() {
                        Ok(idx) => idx,
                        Err(_) => break,
                    }
                };

                let product = {
                    let list = products_list.lock().unwrap();
                    list[idx].clone()
                };

                let product_hash = match lock::get_product_hash(&product) {
                    Ok(h) => h,
                    Err(e) => {
                        err_tx.send(e).ok();
                        break;
                    }
                };

                let product_struct = Product {
                    name: product.name.clone(),
                    description: product.description.clone(),
                    image_file: product.image.clone(),
                    price: product.price,
                    regional_pricing: product.regional_pricing,
                };

                if product.id > 0 {
                    // Check if hash changed — skip if unchanged
                    let existing_hash = {
                        let h = hashes.lock().unwrap();
                        h.get(&product.id).cloned()
                    };
                    if let Some(existing) = existing_hash {
                        if existing == product_hash {
                            continue;
                        }
                    }

                    match product.product_type.as_str() {
                        "Product" => {
                            match roblox::update_product(
                                universe_id,
                                product.id,
                                &api_key,
                                &product_struct,
                            ) {
                                Ok(info) => {
                                    let mut h = hashes.lock().unwrap();
                                    h.insert(product.id, product_hash);
                                    drop(h);
                                    products_list.lock().unwrap()[idx].id = info.product_id;
                                    products_map.lock().unwrap().insert(
                                        info.product_id,
                                        ProductCodeMap {
                                            name: info.name.clone(),
                                            id: info.product_id,
                                            image: format!(
                                                "rbxassetid://{}",
                                                info.icon_image_asset_id.unwrap_or(0)
                                            ),
                                        },
                                    );
                                }
                                Err(e) => {
                                    err_tx.send(e).ok();
                                    break;
                                }
                            }
                        }
                        "Gamepass" => {
                            match roblox::update_gamepass(
                                universe_id,
                                product.id,
                                &api_key,
                                &product_struct,
                            ) {
                                Ok(info) => {
                                    let mut h = hashes.lock().unwrap();
                                    h.insert(product.id, product_hash);
                                    drop(h);
                                    products_list.lock().unwrap()[idx].id = info.game_pass_id;
                                    passes_map.lock().unwrap().insert(
                                        info.game_pass_id,
                                        ProductCodeMap {
                                            name: info.name.clone(),
                                            id: info.game_pass_id,
                                            image: format!(
                                                "rbxassetid://{}",
                                                info.icon_asset_id.unwrap_or(0)
                                            ),
                                        },
                                    );
                                }
                                Err(e) => {
                                    err_tx.send(e).ok();
                                    break;
                                }
                            }
                        }
                        other => println!("Unknown product type: {}", other),
                    }
                } else {
                    match product.product_type.as_str() {
                        "Product" => {
                            match roblox::create_product(universe_id, &api_key, &product_struct) {
                                Ok(info) => {
                                    let mut h = hashes.lock().unwrap();
                                    h.insert(product.id, product_hash);
                                    drop(h);
                                    products_list.lock().unwrap()[idx].id = info.product_id;
                                    products_map.lock().unwrap().insert(
                                        info.product_id,
                                        ProductCodeMap {
                                            name: info.name.clone(),
                                            id: info.product_id,
                                            image: format!(
                                                "rbxassetid://{}",
                                                info.icon_image_asset_id.unwrap_or(0)
                                            ),
                                        },
                                    );
                                }
                                Err(e) => {
                                    err_tx.send(e).ok();
                                    break;
                                }
                            }
                        }
                        "Gamepass" => {
                            match roblox::create_gamepass(universe_id, &api_key, &product_struct) {
                                Ok(info) => {
                                    let mut h = hashes.lock().unwrap();
                                    h.insert(product.id, product_hash);
                                    drop(h);
                                    products_list.lock().unwrap()[idx].id = info.game_pass_id;
                                    passes_map.lock().unwrap().insert(
                                        info.game_pass_id,
                                        ProductCodeMap {
                                            name: info.name.clone(),
                                            id: info.game_pass_id,
                                            image: format!(
                                                "rbxassetid://{}",
                                                info.icon_asset_id.unwrap_or(0)
                                            ),
                                        },
                                    );
                                }
                                Err(e) => {
                                    err_tx.send(e).ok();
                                    break;
                                }
                            }
                        }
                        other => println!("Unknown product type: {}", other),
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Drop the extra err_tx so the channel closes when all threads finish
    drop(err_tx);

    // Wait for all workers
    for handle in handles {
        handle.join().ok();
    }

    // Check if any thread sent an error
    if let Ok(e) = err_rx.try_recv() {
        return Err(e);
    }

    let final_products = products_list.lock().unwrap().clone();
    products::write_products(&final_products)?;

    let final_hashes = hashes.lock().unwrap().clone();
    lock::write_hashes_to_lockfile(&final_hashes)?;

    let products_map = products_map.lock().unwrap().clone();
    let passes_map = passes_map.lock().unwrap().clone();
    generate_code(
        &products_map,
        &passes_map,
        &config.generation,
        &config.files,
    )?;

    println!("Synced products");
    Ok(())
}
