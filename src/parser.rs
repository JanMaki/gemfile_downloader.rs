//!
//! Gemfileのテキストをパースします
//!

use std::error::Error;
use regex::Regex;
use crate::gem_version::GemVersion;

// バージョンの正規表現
const GEM_VERSION_REGEX: &str = "[0-9]+\\.[0-9]+\\.[0-9]+";

///
/// 各Gemのデータ
///
#[derive(Debug)]
pub struct Gem {
    /// Gemの名前
    pub name: String,
    // Gemのバージョン
    pub version: String,
}

///
/// Gemfileのデータ
///
#[derive(Debug)]
pub struct GemfileData {
    // gemのダウンロードを行うソース
    pub source: String,
    // Gemのリスト
    pub gems: Vec<Gem>,
}

impl GemfileData {
    ///
    ///  Gemfileのテキストをパースします
    ///
    pub async fn parse(data: &str) -> Result<GemfileData, Box<dyn Error>>{
        // デフォルトの値を設定
        let mut source = "https://rubygems.org".to_string();
        let mut gems: Vec<Gem> = Vec::new();

        // 行ごとに処理
        for mut line in data.lines() {
            // 行の前後の空白を削除
            loop {
                if !line.starts_with(" ") {
                    break;
                }
                line = &line[1..];
            }

            // sourceの行の場合、sourceの値を取得
            if line.starts_with("source ") {
                source = line.replace("source ", "")
                    .replace("\"", "")
                    .replace("'", "");
            }
            // gemの行の場合
            if line.starts_with("gem "){
                // 余分な個所を削除
                let trimmed = line.replace("gem ", "")
                    .replace("\"", "")
                    .replace("~>", "")
                    .replace(" ", "")
                    .replace("\"", "")
                    .replace("\'", "");
                // カンマで分割
                let splitted = trimmed.split(",").collect::<Vec<&str>>();
                let version_regex = Regex::new(GEM_VERSION_REGEX)?;
                // バージョンが指定されているかを確認
                if splitted.len() >= 2 && version_regex.is_match(splitted[1]) {
                    // バージョンを指定している場合はそのままgemを作成
                    gems.push(Gem {
                        name: splitted[0].to_string(),
                        version: splitted[1].to_string(),
                    });
                } else if splitted.len() >= 1 {
                    // バージョン指定がされていない場合はAPIから取得
                    let version = GemVersion::get_version(&source, splitted[0]).await?;

                    // Gemのデータを追加
                    gems.push(Gem {
                        name: splitted[0].to_string(),
                        version: version.version
                    });
                }
            }
        }

        Ok(GemfileData { source, gems })
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::GemfileData;
    #[tokio::test]
    pub async fn parse_test() {
        // パースをテスト
        let gemfile_data = GemfileData::parse("
source \"https://rubygems.org\"

gemspec

group :development, :test do
 gem \"docile\", \"~> 1.4.0\"
 gem \"simplecov-html\", \"~> 0.12.3\"
 gem \"i18n\", \"~> 1.8.5\"
 gem \"concurrent-ruby\", \"~> 1.1.9\"\
end").await;

        // 簡単に検証
        assert!(gemfile_data.is_ok());
        let gemfile_data = gemfile_data.unwrap();
        assert_eq!(gemfile_data.source, "https://rubygems.org");
        assert_eq!(gemfile_data.gems.len(), 4);
    }


    #[tokio::test]
    pub  async fn parse_test2() {
        // パースをテスト
        let gemfile_data = GemfileData::parse("
source 'https://rubygems.org'

require File.join(File.dirname(__FILE__), 'lib/concurrent-ruby/concurrent/version')
require File.join(File.dirname(__FILE__ ), 'lib/concurrent-ruby-edge/concurrent/edge/version')
require File.join(File.dirname(__FILE__ ), 'lib/concurrent-ruby/concurrent/utility/engine')

no_path = ENV['NO_PATH']
options = no_path ? {} : { path: '.' }

gem 'concurrent-ruby', Concurrent::VERSION, options
gem 'concurrent-ruby-edge', Concurrent::EDGE_VERSION, options
gem 'concurrent-ruby-ext', Concurrent::VERSION, options.merge(platform: :mri)

group :development do
  gem 'rake', (Concurrent.ruby_version :<, 2, 2, 0) ? '~> 12.0' : '~> 13.0'
  gem 'rake-compiler', '~> 1.0', '>= 1.0.7'
  gem 'rake-compiler-dock', '~> 1.0'
  gem 'pry', '~> 0.11', platforms: :mri
end

group :documentation, optional: true do
  gem 'yard', '~> 0.9.0', require: false
  gem 'redcarpet', '~> 3.0', platforms: :mri # understands github markdown
  gem 'md-ruby-eval', '~> 0.6'
end

group :testing do
  gem 'rspec', '~> 3.7'
  gem 'timecop', '~> 0.7.4'
  gem 'sigdump', require: false
end

# made opt-in since it will not install on jruby 1.7
group :coverage, optional: !ENV['COVERAGE'] do
  gem 'simplecov', '~> 0.16.0', require: false
  gem 'coveralls', '~> 0.8.2', require: false
end

group :benchmarks, optional: true do
  gem 'benchmark-ips', '~> 2.7'
  gem 'bench9000'
end").await;

        // 簡単に検証
        assert!(gemfile_data.is_ok());
        let gemfile_data = gemfile_data.unwrap();
        assert_eq!(gemfile_data.source, "https://rubygems.org");
        assert_eq!(gemfile_data.gems.len(), 17);
    }
}