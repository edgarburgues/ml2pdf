use std::error::Error;
use std::fs::{self, File, remove_file};
use std::path::Path;
use std::process::{Command, Output};
use std::io::Write;
use reqwest::Client;
use regex::Regex; 
use crate::html_utils;
use crate::url_utils; 
use crate::api;

// Función para descargar y convertir imágenes SVG a PNG
async fn convert_svg_to_png(client: &Client, url: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    // Descargar la imagen SVG
    let response = client.get(url).send().await?;
    let svg_data = response.bytes().await?;
    
    // Limpiar el contenido del SVG
    let svg_content = String::from_utf8_lossy(&svg_data);
    let cleaned_svg_content = clean_svg_content(&svg_content);
    
    // Guardar el archivo SVG temporalmente
    let svg_temp_path = format!("{}.svg", output_path);
    let mut file = File::create(&svg_temp_path)?;
    file.write_all(cleaned_svg_content.as_bytes())?;

    // Convertir SVG a PNG usando resvg
    let output: Output = Command::new("resvg")
        .arg(&svg_temp_path)
        .arg(output_path)
        .output()?;

    if !output.status.success() {
        return Err(Box::from(format!(
            "Error converting SVG to PNG. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Eliminar el archivo SVG temporal
    remove_file(&svg_temp_path)?;

    Ok(())
}


pub async fn generate_course_pdf(client: &Client, base_url: &str, unit_urls: &[String], course_code: &str) -> Result<(), Box<dyn Error>> {
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
        let html_content = api::fetch_html(unit_url, &client).await?;
        let inner_section_content = html_utils::extract_unit_inner_section(&html_content);
        let head_content = html_utils::extract_head_section(&html_content);
        
        // Manejar la conversión de imágenes SVG a PNG
        let updated_content = replace_and_convert_svg_images(&client, &inner_section_content, base_url, temp_dir).await?;
        
        let full_html_content = html_utils::wrap_html(
            &html_utils::replace_image_urls(&html_utils::replace_css_urls(&updated_content, &base_url)), 
            &html_utils::replace_css_urls(&head_content, &base_url)
        );
        let mut file = File::create(&filename)?;
        file.write_all(full_html_content.as_bytes())?;
        
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

    Ok(())
}

// Función para reemplazar URLs de imágenes SVG y convertirlas a PNG
async fn replace_and_convert_svg_images(
    client: &Client, 
    html_content: &str, 
    base_url: &str, 
    temp_dir: &str
) -> Result<String, Box<dyn Error>> {
    let img_re = Regex::new(r#"src="([^"]+\.svg)""#).unwrap();
    let mut updated_content = html_content.to_string();

    for caps in img_re.captures_iter(html_content) {
        let svg_url = &caps[1];

        let absolute_url = if svg_url.starts_with("http") {
            svg_url.to_string()
        } else {
            let abs_url = url_utils::make_absolute_image_url(base_url, svg_url);
            abs_url
        };

        let png_filename = format!("{}/{}.png", temp_dir, uuid::Uuid::new_v4());

        match convert_svg_to_png(client, &absolute_url, &png_filename).await {
            Ok(_) => {
                updated_content = updated_content.replace(svg_url, &png_filename);

            }
            Err(e) => {
                println!("Error converting SVG to PNG for URL {}: {}", absolute_url, e);
                // Continuar sin detener la ejecución
            }
        }
    }

    Ok(updated_content)
}


fn clean_svg_content(svg_content: &str) -> String {
    let cleaned_svg = svg_content.replace("<head>", "").replace("</head>", "");
    cleaned_svg
}