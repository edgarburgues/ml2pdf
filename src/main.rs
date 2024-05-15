use reqwest;
use regex::Regex;
use serde_json::Value;
use std::error::Error;

fn step_1_convert_url(url: &str) -> Option<String> {
    let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/courses/([^/]+)").unwrap();
    re.captures(url).map(|caps| {
        let locale = &caps[1];
        let course_code = &caps[2];
        format!("https://learn.microsoft.com/api/lists/studyguide/course/course.{}?locale={}", course_code, locale)
    })
}

async fn step_2_fetch_course_data(url: &str) -> Result<Value, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;
    Ok(json)
}

fn step_3_extract_module_urls(json: &Value, base_url: &str) -> Vec<String> {
    json["items"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|item| {
            item["data"]["url"].as_str().map(|url| {
                format!("{}{}", base_url.trim_end_matches('/'), url)
            })
        })
        .collect()
}

fn step_4_transform_module_url(url: &str) -> Option<String> {
    let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/modules/([^/]+)/?").unwrap();
    re.captures(url).map(|caps| {
        let locale = &caps[1];
        let module_id = &caps[2];
        format!("https://learn.microsoft.com/api/hierarchy/modules/learn.wwl.{}?locale={}", module_id, locale)
    })
}

async fn step_5_fetch_and_extract_unit_data(url: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;
    let units = json["units"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|unit| {
            let title = unit["title"].as_str().map(|s| s.to_string());
            let unit_url = unit["url"].as_str().map(|s| format!("https://learn.microsoft.com{}", s));
            match (title, unit_url) {
                (Some(t), Some(u)) => Some((t, u)),
                _ => None,
            }
        })
        .collect();
    Ok(units)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://learn.microsoft.com/es-es/training/courses/az-900t00";
    if let Some(api_url) = step_1_convert_url(url) {
        println!("Nueva URL: {}", api_url);

        match step_2_fetch_course_data(&api_url).await {
            Ok(json) => {
                // Extraer el idioma de la URL original para construir la base_url
                let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/courses/").unwrap();
                let base_url = match re.captures(url) {
                    Some(caps) => format!("https://learn.microsoft.com/{}/", &caps[1]),
                    None => {
                        println!("No se pudo extraer la base_url.");
                        return Ok(());
                    }
                };

                let module_urls = step_3_extract_module_urls(&json, &base_url);
                let mut urls_vec: Vec<String> = Vec::new();
                for url in module_urls {
                    urls_vec.push(url);
                }

                for url in &urls_vec {
                    if let Some(transformed_url) = step_4_transform_module_url(url) {
                        println!("URL transformada del módulo: {}", transformed_url);

                        match step_5_fetch_and_extract_unit_data(&transformed_url).await {
                            Ok(units) => {
                                for (title, unit_url) in units {
                                    println!("Título de la unidad: {}", title);
                                    println!("URL de la unidad: {}", unit_url);
                                }
                            },
                            Err(e) => println!("Error al obtener los datos de la unidad: {}", e),
                        }
                    }
                }
            },
            Err(e) => println!("Error al obtener los datos del curso: {}", e),
        }
    } else {
        println!("URL no válida.");
    }

    Ok(())
}
