use crate::lib_get_fabric;
use crate::lib_get_forgelike;

use std::fs;
use std::path;
use std::env;
use regex::Regex;
use std::fs::{File, set_permissions};
use std::io::{Write, Result};
use serde::Deserialize;
use serde_json;
use std::os::unix::fs::PermissionsExt;
use clap::{Parser};

#[derive(Deserialize)]
struct MainClass {
    inheritsFrom:String,
    mainClass:String,
    arguments:Game
}

#[derive(Deserialize)]
struct Game {
    game:Vec<String>,
    jvm:Vec<String>
}

pub fn generate_launch_sh_after_install(output_path: &String, version: &String,lib: &String, mod_client: &String)-> Result<()> {
    
if mod_client == "vanilla" {
        let home = match dirs::home_dir() {
            Some(s) => s.to_string_lossy().to_string(),
            None => "".to_string()
        };
        let client  = "net.minecraft.client.main.Main";
        let mut lines = Vec::new();

        lines.push("#!/bin/sh".to_string());
        lines.push("/usr/bin/java \\".to_string());
        lines.push("-Xmx8024m \\".to_string());
        lines.push("-Xmn128m \\".to_string());
        lines.push("-XX:+UseG1GC \\".to_string());
        lines.push("-XX:-UseAdaptiveSizePolicy \\".to_string());
        lines.push(format!("-Djava.library.path={}/natives \\",output_path));
        lines.push(format!("-cp \"{}:{}.jar\" \\", lib, version));
        lines.push(format!("{} \\", client));
        lines.push(format!("--version {} \\", version));
        lines.push(format!("--gameDir {} \\", output_path));
        lines.push(format!("--assetsDir {}/.minecraft/assets \\", home));
        lines.push(format!("--assetIndex {} \\", version));
        lines.push("--username whiterely \\".to_string());
        lines.push("--accessToken 0 \\".to_string());
        lines.push("--userType legacy".to_string());

        let script_content = lines.join("\n");
            
        let script_path = format!("{}/start.sh",output_path);

        let mut file = File::create(script_path)?;
        file.write_all(script_content.as_bytes())?;

        let perms = std::fs::Permissions::from_mode(0o755);
        let output_file = format!("{}/start.sh",output_path);
        set_permissions(output_file, perms)?;

        println!("✅ The lunche script has benn create in : {}", output_path);
       
    }

    Ok(())
}

pub fn normalize_path(user_path: &String) -> path::PathBuf {
    let mut path = path::PathBuf::from(user_path);

    if user_path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            path = home.join(user_path.trim_start_matches("~/"));
        }
    }

    if path.is_relative() {
        if let Ok(current_dir) = env::current_dir() {
            path = current_dir.join(path);
        }
    }

    path
}
pub fn generate_launch_sh() {
    let inargs = crate::Options::parse();
    let home_path =match dirs::home_dir() {
            Some(s) => s,
            None => path::PathBuf::new()
        };
    let home = home_path.to_string_lossy().to_string();
    match inargs.args {
        crate::Argument::Generate { game_path, client, mod_json, output_path } => {
            let game_path = normalize_path(&game_path);
            let output_path = normalize_path(&output_path).to_string_lossy().to_string();
            let version_path = game_path.to_string_lossy().to_string();
            let profile_path = format!("{}/.minecraft/launcher_profiles.json",&home);
            let content = fs::read_to_string(profile_path).unwrap();
            let version_json = format!("{}version.json",version_path);
            println!("{}",version_json);
            let mut lib = String::new();
            let mod_json = normalize_path(&mod_json.unwrap()).to_string_lossy().to_string();
            let launcher_profiles :crate::launcher_profile::LauncherProfile = serde_json::from_str(&content).expect("Unable to read launcher_profiles.json");
            let mut client_type = String::new();
            let mod_loader_json:MainClass = serde_json::from_str(&fs::read_to_string(&mod_json).unwrap()).unwrap();
            let main_class = mod_loader_json.mainClass;
            let maingame_version = format!("{}{}",version_path,mod_loader_json.inheritsFrom);
            let mut version = String::new();
            let mut forgelike=String::new();
            let mut forgelikeamc = String::new(); // This is the agrs that should be put behind the mainclass
            for (key, profile) in &launcher_profiles.profiles {
               if profile.name == client {
                        let last_version = &profile.lastVersionId;
                        println!("Name: {}, lastVersionId: {}", profile.name, last_version); 
                        client_type.push_str(last_version);
                        let regexfabric = Regex::new(r"fabric").expect("Please check the value of lastVersionId,mod loader type shoud be include in it");
                        if regexfabric.is_match(last_version){
                            version.push_str(&profile.lastVersionId);
                            lib.push_str(&lib_get_fabric::lib_get(&version_json, Some(&mod_json)));
                        } else {
                            version.push_str(&profile.lastVersionId);
                            lib.push_str(&lib_get_forgelike::lib_get(&version_json,Some(&mod_json)));
                            for args  in mod_loader_json.arguments.game.iter() {
                                forgelikeamc.push_str(args);
                                forgelikeamc.push(' ');
                            for jvm in mod_loader_json.arguments.jvm.iter() {
                                forgelike.push_str(jvm);
                                forgelike.push(' ');
                            }
                        }
                    }
               }
            let mut lines = Vec::new();
            
            lines.push("#!/bin/sh".to_string());
            lines.push(format!("export library_directory=\"{}/.minecraft/libraries\"",home));
            lines.push("export classpath_separator=\":\"".to_string());
            lines.push(format!("export version_name=\"{}\"",mod_loader_json.inheritsFrom));
            lines.push("/usr/bin/java \\".to_string());
            lines.push("--add-opens java.base/java.lang.invoke=ALL-UNNAMED \\".to_string());
            lines.push("--add-opens java.base/java.nio=ALL-UNNAMED \\".to_string());
            lines.push("--add-opens java.base/sun.nio.fs=ALL-UNNAMED \\".to_string());
            lines.push("-Xmx8024m \\".to_string());
            lines.push("-Xmn128m \\".to_string());
            lines.push("-XX:+UseG1GC \\".to_string());
            lines.push("-XX:-UseAdaptiveSizePolicy \\".to_string());
            lines.push(format!("-Djava.library.path={}natives \\",game_path.display()));
            lines.push(format!("{} \\",forgelike));
            lines.push(format!("-cp \"{}:{}.jar\" \\", lib, maingame_version));
            lines.push(format!("{} \\", main_class));
            lines.push(format!("{} \\",forgelikeamc));
            lines.push(format!("--version {} \\",mod_loader_json.inheritsFrom));
            lines.push(format!("--gameDir {} \\", output_path));
            lines.push(format!("--assetsDir {}/.minecraft/assets \\", home));
            lines.push(format!("--assetIndex {} \\", mod_loader_json.inheritsFrom));
            lines.push("--username whiterely \\".to_string());
            lines.push("--accessToken 0 \\".to_string());
            lines.push("--userType legacy".to_string());

            let script_content = lines.join("\n");                    
                    let script_path = format!("{}/start.sh",output_path);
                    let mut file = File::create(script_path).unwrap();
                    file.write_all(script_content.as_bytes()).unwrap();
            }
            let perms = std::fs::Permissions::from_mode(0o755);
            let output_file = format!("{}/start.sh",output_path);
            set_permissions(output_file, perms).unwrap();
            println!("success!");
        }
        crate::Argument::List {
        } => {
            let profile_path = format!("{}/.minecraft/launcher_profiles.json",&home);
            println!("{}",profile_path);
            let content = fs::read_to_string(profile_path).unwrap();
            let launcher_profiles:crate::launcher_profile::LauncherProfile = serde_json::from_str(&content).expect("Not found your launcher_profiles or failed to read it");
            
            for (key, profile) in &launcher_profiles.profiles {
                println!("Profile key: {}", key);
                println!("Name: {}", profile.name);
                println!("Last Version: {}", profile.lastVersionId);
                println!("");
            }
        }
        crate::Argument::Install { version, name } => {}
        crate::Argument::Modpack { file, name } => {}
    }
}
