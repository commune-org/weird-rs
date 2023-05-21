pub use self::zip::zip_dir;
use ::zip::ZipArchive;
use log::debug;
use serde_json::from_reader;
use std::{
    fs::{self, File, Permissions},
    io::{self, BufReader},
    path::{Path, PathBuf},
};
use tauri::AppHandle;

use crate::{
    prelude::*,
    state::{Content, Link, User},
};

pub mod zip;

pub fn load_user(handle: AppHandle) -> Result<User> {
    let user_file = handle
        .path_resolver()
        .app_config_dir()
        .unwrap()
        .join("user.json");
    let file = File::open(user_file)?;
    let reader = BufReader::new(file);
    let user = from_reader(reader)?;
    Ok(user)
}

pub fn load_links(handle: AppHandle) -> Result<Vec<Link>> {
    let content_file = handle
        .path_resolver()
        .app_local_data_dir()
        .unwrap()
        .join("template/content.json");
    let file = File::open(content_file)?;
    let reader = BufReader::new(file);
    let content: Content = from_reader(reader)?;
    Ok(content.links)
}

pub fn extract_template(filepath: PathBuf, dest: &Path) {
    let file = File::open(filepath).unwrap();

    let mut archive = ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            debug!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            debug!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, Permissions::from_mode(mode)).unwrap();
            }
        }
    }
}
