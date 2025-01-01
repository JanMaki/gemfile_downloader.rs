//!
//! GemのバージョンをAPIから取得する
//!
use std::error::Error;
use serde::{Deserialize, Serialize};

///
/// GemのSerialize/Deserialize用の構造体
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  GemVersion {
    pub version: String,
}

impl GemVersion {
    ///
    /// APIからGemのバージョンを取得する
    ///
    /// * source - APIのURL
    /// * gem_name - Gemの名前
    ///
    /// return - 成功するとGemのバージョンを返す
    ///
    pub async fn get_version(source: &str, gem_name: &str) -> Result<GemVersion, Box<dyn Error>> {
        // urlを作成
        let url = format!("{}/api/v1/gems/{}.json", source, gem_name);
        let response = reqwest::get(&url).await?;
        // status codeを確認
        if response.status() != 200 {
            return Err(format!("Failed to get gem version {}", gem_name).into());
        }

        // デシリアライズして返す
        let gem_version: GemVersion = response.json().await?;
        Ok(gem_version)
    }
}