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

const MAX_WORKERS: usize = 5;

pub fn run() -> Result<()> {
    let config = config::load_config()?;
    let api_key = config::load_env()?;

    let products_list = products::read_products()?;
    let hashes = lock::get_hashes()?;

    // Wrap shared state in Arc<Mutex<>> so threads can safely access it
    let hashes = Arc::new(Mutex::new(hashes));
    let products_map: Arc<Mutex<HashMap<i64, ProductCodeMap>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let passes_map: Arc<Mutex<HashMap<i64, ProductCodeMap>>> = Arc::new(Mutex::new(HashMap::new()));
    let products_list = Arc::new(Mutex::new(products_list));

    let universe_id = config.project.universe_id;
    let api_key = Arc::new(api_key);

    // Channel for sending jobs to workers
    // crossbeam or std::sync::mpsc both work; using std here to avoid extra deps
    let (tx, rx) = std::sync::mpsc::channel::<usize>(); // send index into products_list
    let rx = Arc::new(Mutex::new(rx));

    log::info!("Syncing products");
    println!("Syncing products...");

    // Spawn worker threads
    let mut handles = vec![];
    for _ in 0..MAX_WORKERS {
        let rx = Arc::clone(&rx);
        let hashes = Arc::clone(&hashes);
        let products_map = Arc::clone(&products_map);
        let passes_map = Arc::clone(&passes_map);
        let products_list = Arc::clone(&products_list);
        let api_key = Arc::clone(&api_key);

        let handle = thread::spawn(move || -> Result<()> {
            loop {
                // Grab the next job index from the channel
                let idx = {
                    let rx = rx.lock().unwrap();
                    match rx.recv() {
                        Ok(idx) => idx,
                        Err(_) => break, // channel closed, no more jobs
                    }
                };

                // Get a clone of the product so we don't hold the lock while doing I/O
                let product = {
                    let list = products_list.lock().unwrap();
                    list[idx].clone()
                };

                let product_hash = lock::get_product_hash(&product)?;

                let product_struct = Product {
                    name: product.name.clone(),
                    description: product.description.clone(),
                    image_file: product.image.clone(),
                    price: product.price,
                    regional_pricing: product.regional_pricing,
                };

                if product.id > 0 {
                    // Product already exists — check if it changed
                    let existing_hash = {
                        let h = hashes.lock().unwrap();
                        h.get(&product.id).cloned()
                    };

                    // Skip if hash is the same (no changes)
                    if let Some(existing) = existing_hash {
                        if existing == product_hash {
                            continue;
                        }
                    }

                    match product.product_type.as_str() {
                        "Product" => {
                            let info = roblox::update_product(
                                universe_id,
                                product.id,
                                &api_key,
                                &product_struct,
                            )?;
                            let mut h = hashes.lock().unwrap();
                            h.insert(product.id, product_hash);
                            drop(h);
                            products_list.lock().unwrap()[idx].id = info.product_id;
                            products_map.lock().unwrap().insert(
                                info.product_id,
                                ProductCodeMap {
                                    name: info.name.clone(),
                                    id: info.product_id,
                                    image: format!("rbxassetid://{}", info.icon_image_asset_id),
                                },
                            );
                        }
                        "Gamepass" => {
                            let info = roblox::update_gamepass(
                                universe_id,
                                product.id,
                                &api_key,
                                &product_struct,
                            )?;
                            let mut h = hashes.lock().unwrap();
                            h.insert(product.id, product_hash);
                            drop(h);
                            products_list.lock().unwrap()[idx].id = info.game_pass_id;
                            passes_map.lock().unwrap().insert(
                                info.game_pass_id,
                                ProductCodeMap {
                                    name: info.name.clone(),
                                    id: info.game_pass_id,
                                    image: format!("rbxassetid://{}", info.icon_asset_id),
                                },
                            );
                        }
                        other => println!("Unknown product type: {}", other),
                    }
                } else {
                    // New product — create it
                    match product.product_type.as_str() {
                        "Product" => {
                            let info =
                                roblox::create_product(universe_id, &api_key, &product_struct)?;
                            let mut h = hashes.lock().unwrap();
                            h.insert(product.id, product_hash);
                            drop(h);
                            products_list.lock().unwrap()[idx].id = info.product_id;
                            products_map.lock().unwrap().insert(
                                info.product_id,
                                ProductCodeMap {
                                    name: info.name.clone(),
                                    id: info.product_id,
                                    image: format!("rbxassetid://{}", info.icon_image_asset_id),
                                },
                            );
                        }
                        "Gamepass" => {
                            let info =
                                roblox::create_gamepass(universe_id, &api_key, &product_struct)?;
                            let mut h = hashes.lock().unwrap();
                            h.insert(product.id, product_hash);
                            drop(h);
                            products_list.lock().unwrap()[idx].id = info.game_pass_id;
                            passes_map.lock().unwrap().insert(
                                info.game_pass_id,
                                ProductCodeMap {
                                    name: product_struct.name.clone(),
                                    id: info.game_pass_id,
                                    image: format!("rbxassetid://{}", info.icon_asset_id),
                                },
                            );
                        }
                        other => println!("Unknown product type: {}", other),
                    }
                }
            }
            Ok(())
        });

        handles.push(handle);
    }

    // Enqueue jobs (just the index of each product)
    let job_count = {
        let list = products_list.lock().unwrap();
        list.len()
    };
    for i in 0..job_count {
        tx.send(i).ok();
    }
    drop(tx); // Close the channel so workers know there are no more jobs

    // Wait for all workers to finish
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Write results back to disk
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
