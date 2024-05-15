use reqwest;
use regex::Regex;
use serde_json::Value;
use std::error::Error;
use std::process::Command;
use std::fs::{self, File, remove_file};
use std::io::Write;
use std::path::Path;
use std::env;

// Funciones relacionadas con el reemplazo de URLs locales en el contenido HTML
fn replace_local_urls(html_content: &str) -> String {
    let css_re = Regex::new(r#"href="(/_themes/[^"]+)""#).unwrap();
    let js_re = Regex::new(r#"src="(/_themes/[^"]+)""#).unwrap();
    let replaced_css = css_re.replace_all(html_content, r#"href="https://learn.microsoft.com$1""#);
    let replaced_js = js_re.replace_all(&replaced_css, r#"src="https://learn.microsoft.com$1""#);
    replaced_js.to_string()
}

fn wrap_html(content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
</head>
<body>
    {}
</body>
</html>"#,
        content
    )
}

// Funciones relacionadas con la conversión y extracción de URLs
fn convert_course_url(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/courses/([^/]+)").unwrap();
    re.captures(url).map(|caps| {
        let locale = &caps[1];
        let course_code = &caps[2];
        (locale.to_string(), format!("https://learn.microsoft.com/api/lists/studyguide/course/course.{}?locale={}", course_code, locale))
    })
}

fn extract_module_urls(json: &Value, base_url: &str) -> Vec<String> {
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

fn transform_module_url(url: &str) -> Option<String> {
    let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/modules/([^/]+)/?").unwrap();
    re.captures(url).map(|caps| {
        let locale = &caps[1];
        let module_id = &caps[2];
        format!("https://learn.microsoft.com/api/hierarchy/modules/learn.wwl.{}?locale={}", module_id, locale)
    })
}

// Funciones relacionadas con la obtención de datos de los cursos y unidades
async fn fetch_course_data(url: &str) -> Result<Value, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;
    Ok(json)
}

async fn fetch_unit_data(url: &str) -> Result<Value, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let json: Value = serde_json::from_str(&response)?;
    Ok(json)
}

fn extract_units(json: &Value, locale: &str) -> Vec<(String, String)> {
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

// Función para obtener contenido HTML
async fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    Ok(replace_local_urls(&response))
}

// Función principal que coordina todo el proceso
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(Box::from("URL not provided"));
    }

    let url = &args[1];

    if let Some((locale, api_url)) = convert_course_url(url) {
        let course_code_re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/courses/([^/]+)").unwrap();
        let course_code = match course_code_re.captures(url) {
            Some(caps) => caps[2].to_string(),
            None => return Err(Box::from("Invalid URL")),
        };

        println!("Locale: {}", locale);
        println!("course_code: {}", course_code);

        match fetch_course_data(&api_url).await {
            Ok(json) => {
                let base_url = format!("https://learn.microsoft.com/{}/", locale);
                let module_urls = extract_module_urls(&json, &base_url);
                let mut unit_urls: Vec<String> = Vec::new();

                for url in &module_urls {
                    if let Some(transformed_url) = transform_module_url(url) {
                        match fetch_unit_data(&transformed_url).await {
                            Ok(unit_json) => {
                                let units = extract_units(&unit_json, &locale);
                                for (_, unit_url) in units {
                                    unit_urls.push(unit_url);
                                }
                            },
                            Err(_) => {}
                        }
                    }
                }

                match Command::new("wkhtmltopdf").arg("--version").output() {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(Box::from("wkhtmltopdf not found"));
                    }
                }

                let temp_dir = "temp_html_files";
                if !Path::new(temp_dir).exists() {
                    fs::create_dir(temp_dir)?;
                }

                let mut pdf_files = vec![];
                let total_units = unit_urls.len();

                for (i, unit_url) in unit_urls.iter().enumerate() {
                    let filename = format!("{}/unit_{}.html", temp_dir, i);
                    let html_content = fetch_html(unit_url).await?;
                    let wrapped_content = wrap_html(&html_content);
                    let mut file = File::create(&filename)?;
                    file.write_all(wrapped_content.as_bytes())?;
                    
                    let pdf_filename = format!("{}/unit_{}.pdf", temp_dir, i);
                    pdf_files.push(pdf_filename.clone());
                    
                    Command::new("wkhtmltopdf")
                        .arg("--enable-local-file-access")
                        .arg("--no-stop-slow-scripts")
                        .arg("--disable-smart-shrinking")
                        .arg(&filename)
                        .arg(&pdf_filename)
                        .output()?;

                    let progress = ((i + 1) as f32 / total_units as f32) * 100.0;
                    println!("Progress: {:.2}%", progress);
                }

                let mut pdf_cmd = Command::new("pdfunite");
                for pdf_file in &pdf_files {
                    pdf_cmd.arg(pdf_file);
                }
                pdf_cmd.arg(format!("{}.pdf", course_code));

                let output = pdf_cmd.output()?;

                if output.status.success() {
                    for pdf_file in pdf_files {
                        if Path::new(&pdf_file).exists() {
                            remove_file(&pdf_file)?;
                        }
                    }
                    for i in 0..unit_urls.len() {
                        let filename = format!("{}/unit_{}.html", temp_dir, i);
                        if Path::new(&filename).exists() {
                            remove_file(&filename)?;
                        }
                    }
                    if Path::new(temp_dir).exists() {
                        fs::remove_dir(temp_dir)?;
                    }
                }
                println!("El proceso ha terminado.");
            },
            Err(_) => {}
        }
    } else {
        println!("Invalid URL.");
    }

    Ok(())
}
