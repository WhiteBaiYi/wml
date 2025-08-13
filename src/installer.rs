use crate::generate_launch_script;
use crate::generate_launch_script::normalize_path;
use crate::init;
use crate::init::create_dir_if_not_exists;
use crate::lib_get_forgelike;
use crate::launcher_profile;

use clap::{Parser};
use reqwest::{Client};
use serde::{Deserialize};
use serde_json::{Value};
use std::collections::HashMap;
use regex::Regex;
use std::path;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use sha1::{Digest,Sha1};
use tokio::io::{AsyncWriteExt, Ready};
use tokio::time::{timeout, Duration};
use futures::stream::{self, StreamExt};
use walkdir::WalkDir;
use zip::ZipArchive;

//All the struct above is for clap

#[derive(Deserialize)]
struct Manifest {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Deserialize)]
struct FileToDownload {
    assetIndex: AssetIndex,
    downloads: McClient,
    libraries: Vec<LibraryDownloads>,
}

#[derive(Deserialize)]
struct AssetIndex {
    id: String,
    sha1: String,
    size: u64,
    totalSize: u64,
    url: String,
}

#[derive(Deserialize)]
struct McClient {
    client: client,
}

#[derive(Deserialize)]
struct client {
    sha1: String,
    size: u64,
    url: String,
}

#[derive(Deserialize)]
struct LibraryDownloads {
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
struct AssetIndexInasset {
    objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
    size: u64,
}

//All the struct above is for json deserialize for vanilla minecraft

#[derive(Deserialize)]
struct Modrinth {
    versionId:String,
    name:String,
    files:Vec<ModFile>,
    dependencies: Info
}

#[derive(Deserialize)]
struct ModFile {
    path:String,
    hashes:Hashes,
    downloads:Vec<String>,
    fileSize:u32
}

#[derive(Deserialize)]
struct Hashes {
    sha1:String,
    sha512:String
}

#[derive(Deserialize)]
struct Info {
    #[serde(rename = "fabric-loader")]
    #[serde(skip_serializing_if = "Option::is_none")]
    fabric_loader:Option<String>,
    #[serde(rename = "forge-loader")]
    #[serde(skip_serializing_if = "Option::is_none")]
    forge_loader:Option<String>,
    #[serde(rename = "neoforge-loader")]
    #[serde(skip_serializing_if = "Option::is_none")]
    neoforge_loader:Option<String>,
    minecraft:String
}
//above this is for modrinth modpack json file

#[tokio::main]
pub async fn downloade_game() -> Result<(), Box<dyn std::error::Error>> {
    let args = crate::Options::parse();

    match args.args {
        // download vanilla
        crate::Argument::List {} => {}
        crate::Argument::Generate { game_path, client, mod_json, output_path } => {}
        crate::Argument::Install { version, name } => {
            let version_list =
                reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest.json");
            let manifest: Manifest = version_list.await?.json().await?;
            let mc_client = "vanilla".to_string();
            if let Some(v) = manifest.versions.iter().find(|v| &v.id == &version) {
                let home_path =match dirs::home_dir() {
                    Some(s) => s,
                    None => path::PathBuf::new()
                };
                let home = home_path.to_string_lossy().to_string();
                let root_path = format!("{}/.minecraft", home);
                init::create_dir_if_not_exists(&root_path);
                let version_json_file = format!("{}/versions/{}/version.json",root_path,name);
                let client = reqwest::Client::builder()
                    .user_agent("wml/1.0")
                    .build()?;
                let version_json_file_path = Path::new(&version_json_file).parent().unwrap().to_str().unwrap().to_string();
                init::create_dir_if_not_exists(&version_json_file_path);

                downloader(client.clone(), &v.url, Path::new(&version_json_file),None).await?;

                pretty_print_json_file(&version_json_file, true).unwrap();

                let mut contents = String::new();
                fs::File::open(&version_json_file)
                    .unwrap()
                    .read_to_string(&mut contents)?;
                let version_json: FileToDownload = serde_json::from_str(&contents)?;
                
                //    download client
                let client_file_path = format!("{}/versions/{}/{}.jar",root_path,name,version);
                downloader(
                    client.clone(),
                    &version_json.downloads.client.url,
                    Path::new(&client_file_path),
                    None
                    )
                .await?;

                //    download atssetIndex
                let asset_index_path = format!("{}/assets/indexes", root_path);
                let asset_index_file_path = format!("{}/{}.json", asset_index_path, version);
                init::create_dir_if_not_exists(&asset_index_path);
                downloader(
                    client.clone(),
                    &version_json.assetIndex.url,
                    Path::new(&asset_index_file_path),
                    None
                    )
                .await?;
                pretty_print_json_file(asset_index_file_path, true)?;

                //    download libraries
                let task_libs = version_json.libraries
                    .iter()
                    .map(|file| {
                                let client = client.clone();             
                                let root_path = root_path.clone();       
                                async move {
                                    let save_path = format!("{}/libraries/{}", root_path, file.downloads.artifact.path);
                                    let path = Path::new(&save_path);
                                    let url = &file.downloads.artifact.url;
                                    if path.exists() {
                                        return Ok(());
                                    }

                                    init::create_dir_if_not_exists(&path.parent().unwrap().to_str().unwrap().to_string());

                                    downloader(client, &url, path, Some(&file.downloads.artifact.sha1)).await
                                }
                            });
                stream::iter(task_libs)
                    .buffer_unordered(16)
                    .collect::<Vec<_>>()
                    .await;
                
                //    extract natives
                println!("start to extract natives files");
                let libs = format!("{}/libraries/", root_path);
                let libs_path = Path::new(&libs);
                let output = format!("{}/versions/{}/natives/", root_path,name);
                let output_path = Path::new(&output);
                extract_all_natives(libs_path, output_path).unwrap();

                //    list libraries that should be used
                let mut lib = vec![];
                
                for libraries in version_json.libraries.iter() {
                    let file = format!("{}/.minecraft/libraries/{}",home,libraries.downloads.artifact.path);
                    lib.push(file);
                }
                let lib = lib.join(":");
                //    download assets
                let index_path = format!("{}/assets/indexes/{}.json", root_path,version);
                let data = fs::read_to_string(index_path)?;
                let index: AssetIndexInasset = serde_json::from_str(&data)?;
                
                let asset_entries: Vec<(String, AssetObject)> = index.objects.into_iter().collect();
                
                stream::iter(asset_entries.into_iter().map(|(name, obj)| {
                let client = client.clone();
                let root_path = root_path.clone();

                async move {
                    let prefix = &obj.hash[0..2];
                    let url = format!(
                        "https://resources.download.minecraft.net/{}/{}",
                        prefix, obj.hash
                    );

                    let save_path_full = format!("{}/assets/objects/{}/{}", root_path,prefix, obj.hash);
                    let path = Path::new(&save_path_full);

                    if path.exists() {
                        return Ok::<(), Box<dyn std::error::Error>>(());
                    }

                    let parent = path.parent().unwrap();
                    init::create_dir_if_not_exists(&parent.to_str().unwrap().to_string());

                    downloader(client, &url, path, Some(&obj.hash)).await
                }
            }))
            .buffer_unordered(16) 
            .collect::<Vec<_>>()
            .await;
             
            let version_json_path = format!("{}/versions/{}/version.json",root_path,name);
            let lib = lib_get_forgelike::lib_get(&version_json_path, None);
            let path = format!("{}/versions/{}/",root_path,name);
            generate_launch_script::generate_launch_sh_after_install(&path, &version,&lib, &mc_client).expect("failed to create launche script");
            
            let profile_path = format!("{}/launcher_profiles.json",&root_path);
            launcher_profile::generate_launch_profile(version,name, &profile_path)?;

            }
        }
        // install modpack
        crate::Argument::Modpack { file, name } => {
            let client = Client::new();
            let home = dirs::home_dir().expect("could not found your home path").to_string_lossy().to_string();
            let file_json_type = Regex::new(r"modrinth").expect("error code 0");
            let mut download_url:Vec<String> = Vec::new();
            let mut save_path:Vec<String> = Vec::new();
            let mut hash:Vec<String> = Vec::new();

            if file_json_type.is_match(&file) {
                let modpack_path = format!("{}/.minecraft/versions/{}",home,name);
                create_dir_if_not_exists(&modpack_path);
                let file_json:Modrinth = serde_json::from_str(&fs::read_to_string(&normalize_path(&file)).unwrap()).expect("failed to read your modrinth.index.json");
                match file_json.dependencies.fabric_loader {
                    Some(v) => {println!("minecraft:{}\nfabric:{}",file_json.dependencies.minecraft,v);}
                    None => {}
                }
                match file_json.dependencies.forge_loader {
                    Some(v) => {println!("minecraft:{}\nfabric:{}",file_json.dependencies.minecraft,v);}
                    None => {}
                }
                match file_json.dependencies.neoforge_loader {
                    Some(v) => {println!("minecraft:{}\nfabric:{}",file_json.dependencies.minecraft,v);}
                    None => {}
                }
                for mods in file_json.files.iter() {
                    let url = mods.downloads.first().expect("failed to get the download url,please check the modrinth.index.json");
                    download_url.push(url.clone());
                    let sha1 = &mods.hashes.sha1;
                    hash.push(sha1.clone());
                    let path = format!("{}/{}",modpack_path,mods.path);
                    save_path.push(path);
                }
                let tasks = download_url.into_iter()
                .zip(save_path.into_iter())
                .zip(hash.into_iter())
                .map(|((url, path), sha1)| {
                    let client = client.clone();
                    async move {
                        println!("{}",path);
                        let path = PathBuf::from(path);
                        let dir_to_save = path.parent().unwrap();
                        create_dir_if_not_exists(&dir_to_save.to_string_lossy().to_string());
                        downloader(client, &url, &path, Some(&sha1)).await
                    }
                });
                stream::iter(tasks)
                .buffer_unordered(16)
                .for_each(|res| async {
                    if let Err(e) = res {
                        eprintln!("failed to download: {}", e);
                    }
                })
                .await;   
            }
        }
    }
    Ok(())
}

pub async fn downloader(
    client: Client,
    url: &str,
    path: &Path,
    expect_sha1: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    
    let response = timeout(Duration::from_secs(3), client.get(url).send()).await;
    
    match response {
        Ok(Ok(response)) => {
            let bytes = response.bytes().await?;
            let mut file = tokio::fs::File::create(path).await?;
            file.write_all(&bytes).await?;
            file.flush().await?;

            println!("✅ Downloaded: {}", path.display());
            // verifing hasher
            let mut hasher = Sha1::new();
            hasher.update(&bytes);
            let result_sha1 = format!("{:x}", hasher.finalize());

             if let Some(expected) = expect_sha1 {
                let mut hasher = Sha1::new();
                hasher.update(&bytes);
                let result_sha1 = format!("{:x}", hasher.finalize());

                if result_sha1 != expected {
                    println!("❌ SHA1 mismatch for {}, expected {}, got {}", path.display(), expected, result_sha1);
                    tokio::fs::remove_file(path).await?;
                    return Err("SHA1 mismatch".into());
                }
            }

            println!("✅ Downloaded and verified: {}", path.display());
        }
        Ok(Err(_)) => {
            println!("❌ response {} : failed",url);
        }
        Err(_) => {
            println!("response timeout");
        }
    }

    Ok(())

}

fn pretty_print_json_file<P: AsRef<Path>>(
    path: P,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json: Value = serde_json::from_str(&contents)?;

    let formatted = serde_json::to_string_pretty(&json)?;

    if overwrite {
        let mut out_file = fs::File::create(&path)?;
        out_file.write_all(formatted.as_bytes())?;
    } else {
        let new_path = path.as_ref().with_file_name("formatted_version.json");
        let mut out_file = fs::File::create(&new_path)?;
        out_file.write_all(formatted.as_bytes())?;
    }

    Ok(())
}

pub fn extract_all_natives(
    libraries_path: &Path,
    natives_output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(natives_output_path)?;

    for entry in WalkDir::new(libraries_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .map(|ext| ext == "jar")
                    .unwrap_or(false)
                && e.path()
                    .file_name()
                    .map(|n| n.to_string_lossy().contains("natives"))
                    .unwrap_or(false)
        })
    {
        let jar_path = entry.path();
        println!("Processing native jar: {}", jar_path.display());

        let file = fs::File::open(jar_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file_in_zip = archive.by_index(i)?;
            let name = file_in_zip.name();

            if name.ends_with(".so") || name.ends_with(".dll") || name.ends_with(".dylib") {
                let filename = Path::new(name)
                    .file_name()
                    .ok_or("Invalid filename in jar")?;
                let out_file_path = natives_output_path.join(filename);

                let mut out_file = fs::File::create(&out_file_path)?;
                std::io::copy(&mut file_in_zip, &mut out_file)?;
                println!("  → Extracted: {}", out_file_path.display());
            }
        }
    }

    Ok(())
}
