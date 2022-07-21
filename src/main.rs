use std::{path::{Path, PathBuf}, iter::Zip, io::{Cursor, self}, fs};

use exalta_core::{download::{request_checksums, download_files_from_checksums}, misc::init};
use zip::ZipArchive;

static ASSET_RIPPER_PLATFORM: &str = "linux_x64";
static PLATFORM: &str = "rotmg-exalt-win-64";
static OUT: &str = "./out";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_hash = init(None, None).await?.build_hash;
    let output_path = Path::new(OUT);

    println!("{}", build_hash);

    let mut checksums_files = request_checksums(&build_hash, PLATFORM).await?.files;
    checksums_files.retain(|e| {
        return e.file.contains("resources.");
    });

    println!("{:?}", checksums_files);

    // download_files_from_checksums(&build_hash, PLATFORM, &output_path.to_path_buf(), &checksums_files, None).await?;


    let octocrab = octocrab::instance();
    let releases = octocrab.repos("AssetRipper", "AssetRipper").releases().list().send().await?.take_items();

    let unfound_err = "No Releases Found for AssetRipper";
    let release = releases.first().ok_or_else(|| anyhow::anyhow!(unfound_err))?;
    let release_asset = release.assets.iter().find(|e| e.name.contains(ASSET_RIPPER_PLATFORM)).ok_or_else(|| anyhow::anyhow!(unfound_err))?;
    let buf = reqwest::get(release_asset.browser_download_url.clone()).await?.bytes().await?;

    let mut zip = ZipArchive::new(std::io::Cursor::new(buf))?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}