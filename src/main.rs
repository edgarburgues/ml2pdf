mod api;
mod html_utils;
mod url_utils;
mod pdf;

use std::error::Error;
use std::env;
use reqwest::Client;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(Box::from("URL not provided"));
    }

    let url = &args[1];

    println!("Provided URL: {}", url);

    if let Some((locale, api_url)) = url_utils::convert_course_url(url) {
        let course_code = url_utils::extract_course_code(url)?;

        println!("Locale: {}", locale);
        println!("course_code: {}", course_code);

        let client = Client::new();
        
        match api::fetch_course_data(&api_url, &client).await {
            Ok(json) => {
                let base_url = format!("https://learn.microsoft.com/{}/", locale);
                let module_urls = api::extract_module_urls(&json, &base_url);
                let mut unit_urls: Vec<String> = Vec::new();

                for url in &module_urls {
                    if let Some(transformed_url) = url_utils::transform_module_url(url) {
                        match api::fetch_unit_data(&transformed_url, &client).await {
                            Ok(unit_json) => {
                                let units = api::extract_units(&unit_json, &locale);
                                for (_, unit_url) in units {
                                    unit_urls.push(unit_url);
                                }
                            },
                            Err(e) => {
                                println!("Failed to fetch unit data: {}", e);
                            }
                        }
                    }
                }

                pdf::generate_course_pdf(&client, &base_url, &unit_urls, &course_code).await?;
            },
            Err(e) => {
                println!("Failed to fetch course data: {}", e);
            }
        }
    } else {
        println!("Invalid URL.");
    }

    Ok(())
}
