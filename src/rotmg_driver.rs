use std::path::PathBuf;

use exalta_core::{
    download::{download_files_from_checksums, request_checksums},
    misc::init,
};

use crate::PLATFORM;

pub async fn download_essential(build_hash: &str, output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", build_hash);

    let mut checksums_files = request_checksums(&build_hash, PLATFORM).await?.files;
    checksums_files.retain(|e| {
        return e.file.ends_with("resources.assets");
    });

    download_files_from_checksums(build_hash, PLATFORM, &output_path.to_path_buf(), &checksums_files, None).await?;
    Ok(())
}