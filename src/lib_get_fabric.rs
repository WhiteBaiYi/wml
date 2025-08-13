use serde::{Deserialize};
use std::collections::HashSet;
use std::fs;

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
struct Fabric {
    libraries: Vec<FabricLibraries>
}

#[derive(Deserialize)]
struct FabricLibraries {
    name:String,
    url:String,
}

pub fn lib_get (version_json:&String,mod_json:Option<&String>) -> String {
    let content_v: String = fs::read_to_string(version_json).expect("File not exist");
    let version_json :version_json = serde_json::from_str(&content_v).expect("Failed to read version.json");
    let mut lib = Vec::new();
    let mut seen:HashSet<String> = HashSet::new();
    let home = match dirs::home_dir() {
        Some(s) => s.to_string_lossy().to_string(),
        None => "".to_string()
    };
    
    for libraries in version_json.libraries.iter() {
        let file = format!("{}/.minecraft/libraries/{}",home,libraries.downloads.artifact.path);
        if seen.insert(file.clone()) {
                lib.push(file);
            }    }
    
    match mod_json {
        Some(mod_json) => {
            let mod_path = mod_json;
            let content_m: String = fs::read_to_string(mod_path).expect("Failed to read mod json");
            let mod_json :Fabric = serde_json::from_str(&content_m).expect("Failed to read <mod>.json");
            for libraries in mod_json.libraries.iter() {
                let file= maven_name_to_path(&libraries.name);
                if seen.insert(file.clone()) {
                lib.push(file);
            }
            }
        }
        None => {
            
        }
    };   

    let lib = lib.join(":");
    lib
}

fn maven_name_to_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() != 3 {
        panic!("Invalid maven name format");
    }
    let home = match dirs::home_dir() {
        Some(s) => s.to_string_lossy().to_string(),
        None => "".to_string()
    };
    let group_path = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let lib_dir = format!("{}/.minecraft/libraries",home);
    format!("{}/{}/{}/{}/{}-{}.jar",
    lib_dir,
    group_path,
    artifact,
    version,
    artifact,
    version
)
}
