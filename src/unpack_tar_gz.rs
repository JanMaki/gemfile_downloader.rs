//!
//! .tar.gzファイルを解凍します
//!
use std::error::Error;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use flate2::read::MultiGzDecoder;
use tar::Archive;

///
/// .tar.gzファイルを解凍する
///
/// * tar_gz_path - .tar.gzファイルのパス
/// * cache_directory - 一時的に回答した.tarを置くキャッシュディレクトリ
/// * directory - 解凍先のディレクトリ
///
/// return - 解凍処理の結果で、Gemfileが含まれている場合パスを返す
///
pub fn unpack_tar_gz(tar_gz_path: &Path, cache_directory: &Path, directory: &Path) -> Result<Option<PathBuf>, Box<dyn Error>> {
    // .gzファイルを解凍
    let tar_file_path = unpack_gz(tar_gz_path, cache_directory)?;
    // .tarファイルを解凍
    unpack_tar(&tar_file_path, directory)
}


///
/// .gzファイルを解凍する
///
/// * gz_path - .gzファイルのパス
/// * directory - 解凍先のディレクトリ
///
/// return - 解凍後のファイルのパス
///
fn unpack_gz(gz_path: &Path, directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if !directory.exists() {
        create_dir_all(directory)?;
    }

    // gzipファイルを読み込み
    let gzip_file = File::open(gz_path)?;
    let mut decoder = MultiGzDecoder::new(&gzip_file);

    // 出力ファイルを作成
    let gz_file_stem = gz_path.file_stem();
    let Some(gz_file_stem) = gz_file_stem else {
        return Err(".gz file has not stem".into());
    };
    let output_file_path = directory.join(gz_file_stem);
    let mut output_file = File::create(&output_file_path)?;

    // 書き込み
    copy(&mut decoder, &mut output_file)?;

    Ok(output_file_path)
}

///
/// .tarファイルを解凍する
///
/// * tar_path - .tarファイルのパス
/// * directory - 解凍先のディレクトリ
///
/// return - Gemfileが含まれている場合パスを返す
///
fn unpack_tar(tar_path: &Path, directory: &Path) -> Result<Option<PathBuf>, Box<dyn Error>> {
    if directory.exists() {
        remove_dir_all(directory)?;
    }
    create_dir_all(directory)?;

    // tar内にあるGemfileのパス
    let mut entry_gemfile: Option<PathBuf> = None;

    // tarファイルを読み込み、解答
    let tar_file = File::open(tar_path)?;
    let mut archive = Archive::new(tar_file);
    let entries = archive.entries()?;

    for file in entries {
        let mut file = file?;

        let file_path = directory.join(file.path()?);
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                create_dir_all(parent)?;
            }
        }

        file.unpack(&file_path)?;

        // Gemfileの場合パスを保管
        if let Some(file_name) = file_path.file_name() {
            if file_name == "Gemfile" {
                entry_gemfile = Some(file_path);
            }
        }
    }

    Ok(entry_gemfile)
}