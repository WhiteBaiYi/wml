use serde::{Deserialize};
use std::fs;
use std::collections::HashSet;

#[derive(Deserialize)]
struct version_json {
    libraries: Vec<Libraries>
}

#[derive(Deserialize)]
struct Libraries {
    downloads: Artifact,
    name: String,
}

#[derive(Deserialize)]
struct Artifact {
    artifact: artifact,
}

#[derive(Deserialize)]
struct artifact {
    path: String,
    sha1: String,
    size: u64,
    url: String,
}

#[derive(Deserialize)]
struct ModJsonT1 {
    libraries: Vec<Libraries>
}

pub fn lib_get (version_json:&String,mod_json:Option<&String>) -> String {
    let content_v: String = fs::read_to_string(version_json).expect("version_json file not exist");
    let version_json :version_json = serde_json::from_str(&content_v).expect("Failed to read version.json");
    let mut seen:HashSet<String> = HashSet::new();
    let mut lib = Vec::new();
    let home = match dirs::home_dir() {
        Some(s) => s.to_string_lossy().to_string(),
        None => "".to_string()
    };
    
    for libraries in version_json.libraries.iter() {
        let file = format!("{}/.minecraft/libraries/{}",home,libraries.downloads.artifact.path);
        if seen.insert(file.clone()) {
                lib.push(file);
            }
    }
    
    let mod_json =  match mod_json {
        Some(mod_json) => {
            let mod_path = mod_json;
            let content_m: String = fs::read_to_string(mod_path).expect("Failed to read mod json");
            let mod_json :ModJsonT1 = serde_json::from_str(&content_m).expect("Failed to read <mod>.json");
            for libraries in mod_json.libraries.iter() {
                let file = format!("{}/.minecraft/libraries/{}",home,libraries.downloads.artifact.path);
                    if seen.insert(file.clone()) {
                        lib.push(file);
                }
            }
        }
        None => {

        }
    };   
    let mut bootstrap = None;
    lib.retain(|path| {
        if path.contains("bootstraplauncher") {
            bootstrap = Some(path.clone());
            false
        } else {
            true
        }
    });

    if let Some(b) = bootstrap {
        let mut final_libs = vec![b];
        final_libs.extend(lib);
        return final_libs.join(":");
    }
    
    let lib = lib.join(":");
    lib
}
