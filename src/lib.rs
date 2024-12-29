use std::error::Error;
use std::path::Path;
use futures::future::join_all;
use tokio::fs::read_to_string;

pub mod parser;
pub mod download;
pub mod unpack_gem;
pub mod unpack_tar_gz;

///
/// Gemfileを読み込み、Gemのインストールを行う
///
/// * gemfile - Gemfileのパス
/// * install_dictionary - Gemのインストール先のディレクトリ
/// * cache_directory - Gemのダウンロード先のキャッシュディレクトリ
///
/// return - インストール処理の結果
///
pub async fn install_from_gemfile(gemfile: &Path, install_dictionary: &Path, cache_directory: &Path) -> Result<(), Box<dyn Error>> {
    // Gemfileの内容を取得
    let gemfile_context = read_to_string(gemfile).await?;

    // Gemのダウンロード
    install_gems(&gemfile_context, install_dictionary, cache_directory).await?;

    Ok(())
}

///
/// Gemのインストールを行う
///
/// * gemfile_context - Gemfileの内容
/// * install_dictionary - Gemのインストール先のディレクトリ
/// * cache_directory - Gemのダウンロード先のキャッシュディレクトリ
///
/// return - インストール処理の結果
///
pub async fn install_gems(gemfile_context: &str, install_dictionary: &Path, cache_directory: &Path) -> Result<(), Box<dyn Error>>{
    // パース
    let gemfile_data = parser::GemfileData::parse(gemfile_context);

    // gemをすべてダウンロード
    let tasks: Vec<_> = gemfile_data.gems.into_iter().map(|gem| {
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
            let gz_result = unpack_gem::unpack_gem(&download_result, &cache_directory);
            let Ok(gz_result) = gz_result else {
                return;
            };

            // .tar.gzを解凍
            let tar_gz_result = unpack_tar_gz::unpack_tar_gz(&gz_result, &cache_directory, &gems_directory);
            let Ok(tar_gz_result) = tar_gz_result else {
                return;
            };
        }
    }).collect();
    join_all(tasks).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::install_gems;

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
        let result = install_gems(gemfile, gems_directory, gems_cache_directory).await;
        assert!(result.is_ok());
    }
}