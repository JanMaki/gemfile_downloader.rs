//!
//!  .gemのファイルを解凍します
//!
use std::error::Error;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::{Path, PathBuf};
use tar::Archive;

/// .gemファイル内にある本体のデータ
const GEM_DATA_FILE: &str = "data.tar.gz";

///
/// .gemファイルを解凍する
///
/// * path - .gemファイルのパス
/// * directory - 解凍先のディレクトリ
///
/// return - 解凍処理の結果
///
pub fn unpack_gem(path: &Path, directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    // 解凍先ディレクトリの作成
    if directory.exists() {
        remove_dir_all(directory)?;
    }
    create_dir_all(directory)?;

    // .gemファイルの解凍
    let gem_file = File::open(path)?;
    let mut archive = Archive::new(gem_file);
    archive.unpack(directory)?;

    // data.tar.gzのパスを返す
    let data_path = directory.join(GEM_DATA_FILE);
    if !data_path.exists() {
        return Err("data.tar.gz not found".into());
    }
    Ok(data_path)
}
