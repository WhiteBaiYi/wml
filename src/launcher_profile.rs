use serde::{Deserialize,Serialize};
use std::{error::Error, path::Path};
use std::fs;
use std::collections::HashMap;
use sha1::{Sha1, Digest};

#[derive(Deserialize,Serialize)]
pub struct LauncherProfile {
    pub profiles : HashMap<String,GameProfiles>
}

#[derive(Deserialize,Serialize)]
pub struct GameProfiles {
    #[serde(skip_serializing_if = "Option::is_none")]
     pub gameDir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
     javaDir: Option<String>,
     pub lastVersionId: String,
     pub name: String,
    #[serde(rename = "type")]
     profile_type: String,
}    

pub fn generate_launch_profile (version:String,game_name:String,path:&String) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut launcher_profiles: LauncherProfile = if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or(LauncherProfile {
            profiles: HashMap::new(),
        })
    } else {
        LauncherProfile {
            profiles: HashMap::new(),
        }
    };

    let mut hasher = Sha1::new();
    hasher.update(game_name.as_bytes());
    let result = format!("{:x}", hasher.finalize());    
    let home = match dirs::home_dir() {
        Some(s) => s.to_string_lossy().to_string(),
        None => "".to_string()
    };
    let game_dir = format!("{}/.minecraft/versions/{}/",home ,game_name);
    launcher_profiles.profiles.insert(
        result,
        GameProfiles {
            gameDir: Some(game_dir),
            javaDir: Some("/usr/bin/java".to_string()),
            lastVersionId: version,
            name: game_name,
            profile_type: "custom".to_string(),
        },
    );
    
    let json_str = serde_json::to_string_pretty(&launcher_profiles)?;
    fs::write(path, json_str)?;
    
    Ok(())
}
