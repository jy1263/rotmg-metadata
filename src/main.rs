use std::io::Write;
use std::path::PathBuf;
use std::{fs, io, path::Path};

mod rotmg_driver;
mod asset_ripper;

use std::process::Command;
use asset_ripper::download_asset_ripper_to;
use rotmg_driver::download_essential;

static ASSET_RIPPER_PLATFORM: &str = "linux_x64";
static PLATFORM: &str = "rotmg-exalt-win-64";
static OUT: &str = "./out";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !generate(exalta_core::Build::Testing).await.is_ok(){
        println!("Testing build generation failed");
    }
    generate(exalta_core::Build::Production).await?;

    Ok(())
}

async fn generate(build: exalta_core::Build) -> Result<(), Box<dyn std::error::Error>> {
    let build_str = match build {
        exalta_core::Build::Production => "production",
        exalta_core::Build::Testing => "testing",
    };

    let output_path = Path::new(OUT);
    let output_rotmg_path = output_path.join(format!("rotmg/{}", build_str));
    let output_rotmg_version_path = output_rotmg_path.join("version.txt");
    let output_rotmg_data_path = output_rotmg_path.join("RotMG Exalt_Data");

    let asset_ripper_path = output_path.join("AssetRipperConsole");
    let ripped_path = output_path.join(format!("ripped/{}", build_str));
    let exported_project_assets_path = ripped_path.join("ExportedProject/Assets");
    
    let final_output_path = output_path.join("output_final");
    let final_output_path_assets = final_output_path.join("assets");
    let final_output_path_assets_production = final_output_path_assets.join(build_str);
    let final_output_atlases_path_production = final_output_path_assets_production.join("atlases");
    let final_output_xml_path_production = final_output_path_assets_production.join("xml");

    println!("Generating {}", build_str);
    exalta_core::set_build(build).await;
    let build_hash = exalta_core::misc::init(None, None).await?.build_hash;
    let need_to_update = fs::read_to_string(&output_rotmg_version_path).unwrap_or_default() != build_hash;
    if need_to_update {
        println!("Downloading rotmg data");
        download_essential(&build_hash, output_rotmg_path.to_path_buf()).await?;
        create_or_overwrite(&output_rotmg_version_path, build_hash.as_bytes())?;
    }

    
    if !asset_ripper_path.exists() {
        download_asset_ripper_to(output_path.to_path_buf()).await?;
    }

    if need_to_update {
        if ripped_path.exists() {
            fs::remove_dir_all(&ripped_path)?;
        }

        #[cfg(unix)]
        {
            println!(
                "{}",
                Command::new("chmod")
                    .args([
                        "+x",
                        &output_path.join("AssetRipperConsole").to_str().unwrap()
                    ])
                    .output()?
                    .status
            );
            println!(
                "{}",
                String::from_utf8_lossy(&Command::new(output_path.join("AssetRipperConsole"))
                    .args([
                        &output_rotmg_data_path.join("resources.assets").to_string_lossy().to_string(),
                        "--output",
                        &ripped_path.to_string_lossy().to_string()
                    ])
                    .output()?
                    .stdout)
            );
        }
    }
    
    let mut xml = vec![];
    let mut atlases = vec![];

    for path in exported_project_assets_path.join("TextAsset").read_dir()? {
        let res = path?;
        let fname = res.file_name().to_string_lossy().to_string();
        if fname == "spritesheet.json" {
            atlases.push(res.path());
        }
        else if fname == "assets_manifest.txt" {
            atlases.push(res.path());
        }
        else if fname.ends_with(".txt") {
            xml.push(res.path());
        }
    } 

    for path in exported_project_assets_path.join("Texture2D").read_dir()? {
        let res = path?;
        let fname = res.file_name().to_string_lossy().to_string();
        match fname.as_str() {
            "characters_masks.png" | "characters.png" | "groundTiles.png" | "mapObjects.png" => {
                atlases.push(res.path());
            }
            _ => {
            }
        };
    }

    fs::create_dir_all(&final_output_atlases_path_production)?;
    for atlas in atlases {
        let atlas_name = atlas.file_name().unwrap();
        if atlas_name.to_string_lossy().ends_with(".txt") {
            fs::copy(&atlas, final_output_atlases_path_production.join(&atlas.with_extension("xml").file_name().unwrap()))?;
        }
        else {
            fs::copy(&atlas, final_output_atlases_path_production.join(&atlas_name))?;
        }
    }

    fs::create_dir_all(&final_output_xml_path_production)?;
    for xml_e in xml {
        let out = final_output_xml_path_production.join(&xml_e.with_extension("xml").file_name().unwrap());
        fs::copy(&xml_e, out)?;
    }
    
    let public_path = Path::new("public");
    if public_path.exists() {
        let mut opts = fs_extra::dir::CopyOptions::new();
        opts.overwrite = true;
        opts.content_only = true;
        fs_extra::dir::copy(public_path, &final_output_path, &opts)?;
    }

    create_or_overwrite(&final_output_path_assets_production.join("version.txt"), build_hash.as_bytes())?;

    Ok(())
}

fn create_or_overwrite(file_path: &PathBuf, buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?
        .write_all(buf)?;
    Ok(())
}