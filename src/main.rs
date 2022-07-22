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
    let output_path = Path::new(OUT);
    let output_rotmg_path = output_path.join("RotMG Exalt_Data");
    let asset_ripper_path = output_path.join("AssetRipperConsole");
    let exported_project_assets_path = output_path.join("Ripped/ExportedProject/Assets");
    
    let final_output_path = output_path.join("output_final");
    let final_output_path_assets = final_output_path.join("assets");
    let final_output_path_assets_production = final_output_path_assets.join("production");
    let final_output_atlases_path_production = final_output_path_assets_production.join("atlases");
    let final_output_xml_path_production = final_output_path_assets_production.join("xml");

    if !output_rotmg_path.exists() {
        download_essential(output_path.to_path_buf()).await?;
    }

    
    if !asset_ripper_path.exists() {
        download_asset_ripper_to(output_path.to_path_buf()).await?;
    }

    if !exported_project_assets_path.exists() {
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
                        output_path.join("RotMG Exalt_Data/resources.assets")
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
        fs_extra::dir::copy(public_path, final_output_path, &opts)?;
    }

    Ok(())
}
