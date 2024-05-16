use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub async fn fetch_html(url: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}

pub async fn fetch_course_data(url: &str, client: &Client) -> Result<Value, Box<dyn Error>> {
    let response = client.get(url).send().await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;
    Ok(json)
}

pub async fn fetch_unit_data(url: &str, client: &Client) -> Result<Value, Box<dyn Error>> {
    let response = client.get(url).send().await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;
    Ok(json)
}

pub fn extract_module_urls(json: &Value, base_url: &str) -> Vec<String> {
    json["items"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|item| {
            item["data"]["url"].as_str().map(|url| {
                format!("{}{}", base_url.trim_end_matches('/'), url)
            })
        })
        .collect::<Vec<_>>()
}

pub fn extract_units(json: &Value, locale: &str) -> Vec<(String, String)> {
    json["units"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|unit| {
            let title = unit["title"].as_str().map(|s| s.to_string());
            let unit_url = unit["url"].as_str().map(|s| format!("https://learn.microsoft.com/{}/{}", locale, s.trim_start_matches('/')));
            match (title, unit_url) {
                (Some(t), Some(u)) => Some((t, u)),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
}
