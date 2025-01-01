//!
//! Gemfileのテキストをパースします
//!

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
    pub fn parse(data: &str) -> GemfileData{
        // デフォルトの値を設定
        let mut source = "https://rubygems.org".to_string();
        let mut gems: Vec<Gem> = Vec::new();

        // 行ごとに処理
        data.lines().for_each(|mut line| {
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
            }
            // gemの行の場合
            if line.starts_with("gem ") || line.starts_with(" gem"){
                // 余分な個所を削除
                let trimmed = line.replace("gem ", "")
                    .replace("\"", "")
                    .replace("~>", "")
                    .replace(" ", "")
                    .replace("\'", "");
                // カンマで分割
                let splitted = trimmed.split(",").collect::<Vec<&str>>();
                // Gemのデータを追加
                gems.push(Gem {
                    name: splitted[0].to_string(),
                    version: splitted[1].to_string(),
                });
            }
        });


        GemfileData { source, gems }
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::GemfileData;
    #[test]
    pub fn parse_test() {
        // パースをテスト
        let gemfile_data = GemfileData::parse("
source \"https://rubygems.org\"

gemspec

group :development, :test do
 gem \"docile\", \"~> 1.4.0\"
 gem \"simplecov-html\", \"~> 0.12.3\"
 gem \"i18n\", \"~> 1.8.5\"
 gem \"concurrent-ruby\", \"~> 1.1.9\"\
end");

        // 簡単に検証
        assert_eq!(gemfile_data.source, "https://rubygems.org");
        assert_eq!(gemfile_data.gems.len(), 4);
    }
}