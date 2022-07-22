use std::path::PathBuf;

use exalta_core::{
    download::{download_files_from_checksums, request_checksums},
    misc::init,
};

use crate::PLATFORM;

pub async fn download_essential(output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let build_hash = init(None, None).await?.build_hash;
    println!("{}", build_hash);

    let mut checksums_files = request_checksums(&build_hash, PLATFORM).await?.files;
    checksums_files.retain(|e| {
        return e.file.contains("resources.");
    });

    println!("{:?}", checksums_files);

    download_files_from_checksums(&build_hash, PLATFORM, &output_path.to_path_buf(), &checksums_files, None).await?;
    Ok(())
}