use std::{fs, path};
use std::path::{Path,PathBuf};
use std::env;
use clap::{Parser, Subcommand};
use dirs::home_dir;
use std::io::Write;

use crate::installer;

pub fn create_dir_if_not_exists(path: &String) {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path).unwrap();
    }
}

pub fn main() {
    let version_path = crate::Options::parse();
    if let Some(home_path) = dirs::home_dir() {
    let home = home_path.to_string_lossy().to_string();
     match version_path.args {
        crate::Argument::Install { version, name } => {
            let path = format!("{}/.minecraft/versions/{}", home, name);
            let path_e = path::Path::new(&path);
            if !path_e.exists() { 
                println!("Will be installed to {}",path);
                create_dir_if_not_exists(&path);  
            }
        }
        crate::Argument::Modpack { file, name } => {
            let path = format!("{}/.wml/minecraft/versions/{}", home, name);
            let path_e = path::Path::new(&path);
            if !path_e.exists() {
                println!("Will be installed to {}",path);
                create_dir_if_not_exists(&path);
            }
        }
        crate::Argument::Generate { game_path, client, mod_json, output_path } => {}
        crate::Argument::List {  } => {}
     }
}
}

