use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use tokio::sync::Mutex;
use crate::parser::GemfileData;

pub mod parser;
pub mod download;
pub mod unpack_gem;
pub mod unpack_tar_gz;
pub mod gem_version;

///
/// インストール結果の情報
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallInfo {
    // インストールしたGemの一覧
    pub install_gems: Vec<String>,
    // Gemfileが含まれていた場合、すべてのGem名とGemfileのパス
    pub find_gemfiles: Vec<FindGemFileInfo>,
}

///
/// インストール時に見つかったGemfileの情報
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindGemFileInfo {
    // Gemの名前
    pub gem_name: String,
    // Gemfileのパス
    pub gemfile_path: PathBuf,
}

///
/// Gemfileを読み込み、Gemのインストールを行う
///
/// * gemfile - Gemfileのパス
/// * install_dictionary - Gemのインストール先のディレクトリ
/// * cache_directory - Gemのダウンロード先のキャッシュディレクトリ
///
/// return -  インストール処理の結果
///
pub async fn install_from_gemfile_file(gemfile: &Path, install_dictionary: &Path, cache_directory: &Path) -> Result<InstallInfo, Box<dyn Error>> {
    // Gemfileの内容を取得
    let gemfile_context = read_to_string(gemfile).await?;

    // Gemのダウンロード
    install_from_gemfile_literal(&gemfile_context, install_dictionary, cache_directory).await
}

///
/// Gemfileの文字列のデータから、Gemのインストールを行う
///
/// * gemfile_context - Gemfileの内容
/// * install_dictionary - Gemのインストール先のディレクトリ
/// * cache_directory - Gemのダウンロード先のキャッシュディレクトリ
///
/// return - インストール処理の結果
///
pub async fn install_from_gemfile_literal(gemfile_context: &str, install_dictionary: &Path, cache_directory: &Path) -> Result<InstallInfo, Box<dyn Error>> {
    // パース
    let gemfile_data = parser::GemfileData::parse(gemfile_context).await?;

    install_gems(gemfile_data, install_dictionary, cache_directory).await
}

///
/// Gemのインストールを行う
///
/// * gemfile_data - Gemfileの読み込み済みデータ
/// * install_dictionary - Gemのインストール先のディレクトリ
/// * cache_directory - Gemのダウンロード先のキャッシュディレクトリ
///
/// return - インストール処理の結果
///
pub async fn install_gems(gemfile_data: GemfileData, install_dictionary: &Path, cache_directory: &Path) -> Result<InstallInfo, Box<dyn Error>>{

    // インストールしたGemの一覧
    let installed_gems: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    // インストールしたGemに含まれていたGemfileのパス
    let gemfiles: Arc<Mutex<Vec<FindGemFileInfo>>> = Arc::new(Mutex::new(Vec::new()));

    // gemをすべてダウンロード
    let tasks: Vec<_> = gemfile_data.gems.into_iter().map(|gem| {
        let installed_gems = Arc::clone(&installed_gems);
        let gemfiles = Arc::clone(&gemfiles);
        let source = gemfile_data.source.clone();

        async move {
            // ダウンロード
            let download_result = download::download_gem(cache_directory, &source, &gem).await;
            let Ok(download_result) = download_result else {
                return;
            };
            let gem_name = download_result.file_stem();
            let Some(gem_name) = gem_name else {
                return;
            };

            // キャッシュディレクトリ
            let cache_directory =  &cache_directory.join(gem_name);
            // gemの本体を置くディレクトリ
            let gems_directory = &install_dictionary.join(gem_name);

            // .gemを解凍
            let gz_result = unpack_gem::unpack_gem(&download_result, cache_directory);
            let Ok(gz_result) = gz_result else {
                return;
            };

            // .tar.gzを解凍
            let tar_gz_result = unpack_tar_gz::unpack_tar_gz(&gz_result, cache_directory, gems_directory);
            let Ok(tar_gz_result) = tar_gz_result else {
                return;
            };

            let gem_name = gem_name.to_string_lossy().to_string();
            // インストール一覧に追加
            installed_gems.lock().await.push(gem_name.clone());

            // gemfileのパスを追加
            if let Some(gemfile) = tar_gz_result {
                gemfiles.lock().await.push(FindGemFileInfo{
                    gem_name,
                    gemfile_path: gemfile,
                });
            }
        }
    }).collect();
    join_all(tasks).await;

    // Arcを外す
    let Ok(installed_gems) = Arc::try_unwrap(installed_gems) else {
        return Err("installed_gems unwrap error".into());
    };
    let Ok(gemfiles) = Arc::try_unwrap(gemfiles) else {
        return Err("gemfiles unwrap error".into());
    };

    Ok(InstallInfo{
        install_gems: installed_gems.into_inner(),
        find_gemfiles: gemfiles.into_inner(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::install_from_gemfile_literal;

    ///
    /// Gemsのダウンロードのテスト
    ///
    #[tokio::test]
    pub async fn gems_download_test() {
        // gemのダウンロードを置くキャッシュ
        let gems_cache_directory = Path::new("./target/gems_cache");
        // gemを最終的に解凍するディレクトリ
        let gems_directory = Path::new("./target/gems");

        // ファイルの内容
        let gemfile = "
source \"https://rubygems.org\"

gemspec

group :development, :test do
 gem \"docile\", \"~> 1.4.0\"
 gem \"simplecov-html\", \"~> 0.12.3\"
 gem \"i18n\", \"~> 1.8.5\"
 gem \"concurrent-ruby\", \"~> 1.3.4\"
end";
        let result = install_from_gemfile_literal(gemfile, gems_directory, gems_cache_directory).await;
        assert!(result.is_ok());

        result.unwrap().find_gemfiles.iter().for_each(|find_gemfile| {
            println!("gemfile: {:?}", find_gemfile.gemfile_path);
        });
    }
}