use std::fs::File;

use log::{debug, warn};
use serde_json::to_writer;
use tauri::{
    api::{dialog::FileDialogBuilder, path::download_dir},
    command, AppHandle, Manager, State,
};

use crate::{
    build::build,
    prelude::*,
    state::{AppState, Content, Link, Links, User},
    utils::{self, write_config, zip_dir},
};

#[command]
pub fn toggle_preview_window(handle: AppHandle) -> Result<()> {
    if let Some(w) = handle.get_window("preview") {
        if let Ok(visible) = w.is_visible() {
            if visible {
                w.hide()?;
            } else {
                w.show()?;
            }
        }
    }
    Ok(())
}

#[command]
pub fn generate_site(state: State<'_, AppState>, handle: AppHandle) -> Result<()> {
    let output_dir = handle
        .path_resolver()
        .app_cache_dir()
        .unwrap()
        .join("dist/");
    let zip_file = handle
        .path_resolver()
        .app_cache_dir()
        .unwrap()
        .join("website.zip");
    let config_path = handle
        .path_resolver()
        .app_local_data_dir()
        .unwrap()
        .join("template/config.toml");

    let user: &User = &state.user.lock().unwrap();
    let links: Links = state.links.lock().unwrap().to_vec();
    let links_file = File::create(
        handle
            .path_resolver()
            .app_config_dir()
            .unwrap()
            .join("links.json"),
    )?;

    to_writer(links_file, &links)?;

    write_config(
        Content {
            user: user.clone(),
            links,
        },
        &config_path,
    );

    build(
        &handle
            .path_resolver()
            .app_local_data_dir()
            .unwrap()
            .join("template/"),
        &config_path,
        &output_dir,
        Some("/"),
    )?;

    // zip the website bundle.
    zip_dir(
        output_dir.to_str().unwrap(),
        zip_file.to_str().unwrap(),
        zip::CompressionMethod::Deflated,
    )?;

    Ok(())
}

/// Get the export zip file contents encoded as base64
#[command]
pub fn get_export_zip_base64(handle: AppHandle) -> Result<String> {
    use base64::Engine;
    let zip_file = handle
        .path_resolver()
        .app_cache_dir()
        .unwrap()
        .join("website.zip");
    let contents = std::fs::read(zip_file)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(contents))
}

#[command]
pub fn export_zip(handle: AppHandle) -> Result<()> {
    let zip_file = handle
        .path_resolver()
        .app_cache_dir()
        .unwrap()
        .join("website.zip");
    FileDialogBuilder::new()
        .set_file_name("website.zip")
        .set_directory(download_dir().unwrap())
        .save_file(move |f| {
            if let Some(file) = f {
                debug!("Saving website bundle to {}", file.to_str().unwrap());
                std::fs::copy(zip_file, file).unwrap();
            }
        });
    Ok(())
}

#[command]
pub fn remove_link(id: usize, state: State<'_, AppState>) -> Result<()> {
    state.links.lock().unwrap().retain(|l| l.id != id);
    Ok(())
}

#[command]
pub fn add_link(link: Link, state: State<'_, AppState>) -> Result<()> {
    state.links.lock().unwrap().push(link);
    Ok(())
}

#[command]
pub fn update_user(user: User, state: State<'_, AppState>, handle: AppHandle) -> Result<()> {
    let user_file = File::create(
        handle
            .path_resolver()
            .app_config_dir()
            .unwrap()
            .join("user.json"),
    )?;
    to_writer(user_file, &user)?;
    *state.user.lock().unwrap() = user;
    Ok(())
}

#[command]
pub fn get_user(state: State<'_, AppState>, handle: AppHandle) -> Result<User> {
    let user: User = match utils::load_user(handle) {
        Ok(u) => {
            *state.user.lock().unwrap() = u.clone();
            u
        }
        Err(e) => {
            warn!("Could not load user data: {e}");
            let user: &User = &state.user.lock().unwrap();

            user.clone()
        }
    };
    Ok(user)
}

#[command]
pub fn get_links(state: State<'_, AppState>, handle: AppHandle) -> Result<Links> {
    let links: Links = match utils::load_links(handle) {
        Ok(u) => {
            *state.links.lock().unwrap() = u.to_vec();
            u
        }
        Err(e) => {
            warn!("Could not load links: {e}");
            state.links.lock().unwrap().to_vec()
        }
    };
    Ok(links)
}
