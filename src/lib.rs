pub mod parser;
pub mod download;

#[cfg(test)]
mod tests {
    use futures::future::join_all;
    use crate::download::download_gem;
    use crate::parser::GemfileData;

    ///
    /// Gemsのダウンロードのテスト
    ///
    #[tokio::test]
    pub async fn gems_download_test() {
        // ファイルの内容
        let gemfile = "
source \"https://rubygems.org\"

gemspec

group :development, :test do
gem \"docile\", \"~> 1.4.0\"
gem \"simplecov-html\", \"~> 0.12.3\"
gem \"i18n\", \"~> 1.8.5\"
gem \"concurrent-ruby\", \"~> 1.1.9\"\
end";
        // パース
        let gemfile_data = GemfileData::parse(gemfile);

        // gemをすべてダウンロード
        let tasks: Vec<_> = gemfile_data.gems.into_iter().map(|gem| {
            let source = gemfile_data.source.clone();
            async move {
                let result = download_gem(std::path::Path::new("./target/gems"), &source, &gem).await;
                assert!(result.is_ok())
            }
        }).collect();
        join_all(tasks).await;
    }
}