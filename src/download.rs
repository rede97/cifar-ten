use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use curl::easy::Easy;
use std::fs::File;

use pbr::ProgressBar;
use std::convert::TryInto;
use std::thread;

use std::error::Error;
use tar::Archive;

const ARCHIVE: &str = "cifar-10-binary.tar.gz";
const ARCHIVE_DOWNLOAD_SIZE: usize = 170052171;

pub(super) fn download_and_extract(
    download_url: String,
    base_path: impl Into<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let download_dir = base_path.into();
    if !download_dir.exists() {
        println!(
            "Download directory {} does not exists. Creating....",
            download_dir.display()
        );
        fs::create_dir_all(&download_dir)?;
    }
    println!("Attempting to download and extract {}...", ARCHIVE);
    download(download_url, &download_dir)?;
    extract(&ARCHIVE, &download_dir)?;

    Ok(())
}

fn download(url: String, download_dir: impl Into<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut easy = Easy::new();

    let file_name = download_dir.into().join(ARCHIVE); //.clone();
    if Path::new(&file_name).exists() {
        println!(
            "  File {:?} already exists, skipping downloading.",
            file_name
        );
    } else {
        println!(
            "- Downloading from file from {} and saving to file as: {}",
            url,
            file_name.display()
        );

        let mut file = File::create(file_name.clone()).unwrap();

        let full_size = ARCHIVE_DOWNLOAD_SIZE;

        let mut pb = ProgressBar::new(full_size.try_into().unwrap());

        easy.progress_function(move |total_download_bytes, cur_download_bytes, _, _| {
            if cur_download_bytes < total_download_bytes {
                pb.set(cur_download_bytes.round() as u64);
            } else {
                pb.finish_println("Download done!");
            }
            true
        })
        .unwrap();

        easy.url(&url).unwrap();
        easy.write_function(move |data| {
            file.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
        easy.perform().unwrap();
    }

    Ok(())
}

fn extract(archive_name: &str, download_dir: &Path) -> Result<(), Box<dyn Error>> {
    // And extract the contents
    let archive = download_dir.to_owned().join(archive_name);

    let extract_to = download_dir.to_owned().join("cifar-10-batches-bin");
    if Path::new(&extract_to).exists() {
        println!(
            "  Extracted file {:?} already exists, skipping extraction.",
            extract_to
        );
    } else {
        println!("Beginning extraction of {:?} to {:?}", archive, extract_to);
        use flate2::read::GzDecoder;
        let tar_gz = File::open(archive)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(download_dir)?;
    }
    Ok(())
}
