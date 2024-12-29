pub mod parser;
pub mod download;
pub mod unpack_gem;

#[cfg(test)]
mod tests {
    use std::path::Path;
    use futures::future::join_all;
    use crate::download::download_gem;
    use crate::parser::GemfileData;
    use crate::unpack_gem::unpack_gem;

    ///
    /// Gemsのダウンロードのテスト
    ///
    #[tokio::test]
    pub async fn gems_download_test() {
        // gemをダウンロードを置くキャッシュ
        let gem_directory = Path::new("./target/gems_cache");

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
        // パース
        let gemfile_data = GemfileData::parse(gemfile);

        // gemをすべてダウンロード
        let tasks: Vec<_> = gemfile_data.gems.into_iter().map(|gem| {
            let source = gemfile_data.source.clone();
            async move {
                // ダウンロード
                let result = download_gem(gem_directory, &source, &gem).await;
                assert!(result.is_ok());
                let Ok(result) = result else {
                    return;
                };

                // ,gemを解凍
                let result = unpack_gem(&result, &gem_directory.join(result.file_stem().unwrap()));
                assert!(result.is_ok());
                let Ok(result) = result else {
                    return;
                };


            }
        }).collect();
        join_all(tasks).await;
    }
}