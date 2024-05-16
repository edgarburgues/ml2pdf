use scraper::{Html, Selector};
use regex::Regex;

pub fn extract_unit_inner_section(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div#unit-inner-section").unwrap();
    document.select(&selector)
        .map(|element| element.html())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn extract_head_section(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("head").unwrap();
    document.select(&selector)
        .map(|element| element.html())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn replace_image_urls(html_content: &str) -> String {
    let img_re = Regex::new(r#"src="([^"]+)""#).unwrap();
    img_re.replace_all(html_content, |caps: &regex::Captures| {
        let url = &caps[1];
        if url.starts_with("http") {
            format!(r#"src="{}""#, url)
        } else {
            let path = if url.starts_with('/') {
                url.to_string()
            } else {
                format!("./{}", url)
            };
            println!("Image Path: {}", path);
            format!(r#"src="{}""#, path)
        }
    }).to_string()
}




pub fn replace_css_urls(html_content: &str, base_url: &str) -> String {
    let css_re = Regex::new(r#"href="([^"]+\.css)""#).unwrap();
    css_re.replace_all(html_content, |caps: &regex::Captures| {
        let url = &caps[1];
        if url.starts_with("http") {
            format!(r#"href="{}""#, url)
        } else {
            let absolute_url = crate::url_utils::make_absolute_url(base_url, url);
            format!(r#"href="{}""#, absolute_url)
        }
    }).to_string()
}

pub fn wrap_html(content: &str, head: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html lang="es">
        {}
        <body>
            {}
        </body>
        </html>"#,
        head,
        content
    )
}
