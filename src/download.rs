//!
//! Gemのダウンロード処理
//!

use std::error::Error;
use std::fs::{create_dir, exists, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use crate::parser::Gem;

///
/// ダウンロードを行う
///
/// * directory - ダウンロード先のディレクトリ
/// * source - ダウンロード元のURL
/// * gem - ダウンロードするGemのデータ
///
/// return - ダウンロード処理の結果
///
pub async fn download_gem(directory: &Path, source: &str, gem: &Gem) -> Result<(PathBuf), Box<dyn Error>> {
    // urlの作成
    let url = format!("{}/downloads/{}-{}.gem", source, gem.name, gem.version);
    // ファイル名の作成
    let filename = format!("{}-{}.gem", gem.name, gem.version);

    // ダウンロード
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;

    // ファイルに書き込み
    if exists(directory)? == false {
        create_dir(directory)?;
    }
    let path = directory.join(filename);
    let mut out = File::create(&path)?;
    copy(&mut bytes.as_ref(), &mut out)?;

    // Ok
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::download::download_gem;
    use crate::parser::Gem;

    ///
    /// ダウンロードのテスト
    ///
    #[tokio::test]
    pub async fn download_test() {
        // テストケース
        let directory = Path::new("./target/gems");
        let source = "https://rubygems.org";
        let gem = Gem {
            name: "rake".to_string(),
            version: "13.0.1".to_string(),
        };

        // ダウンロード
        let result = download_gem(directory, source, &gem).await;

        //　ダウンロード処理が正常に終了しているか
        assert!(result.is_ok());
    }
}