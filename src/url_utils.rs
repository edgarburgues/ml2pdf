use url::Url;
use regex::Regex;

pub fn make_absolute_url(base_url: &str, relative_url: &str) -> String {
    let base = Url::parse(base_url).expect("Invalid base URL");
    let joined = base.join(relative_url).expect("Invalid relative URL");
    joined.to_string()
}

pub fn make_absolute_image_url(base_url: &str, relative_url: &str) -> String {
    if relative_url.starts_with("http") {
        relative_url.to_string()
    } else {
        let base = Url::parse(base_url).expect("Invalid base URL");
        let base_parts: Vec<&str> = base.path_segments().map(|c| c.collect()).unwrap_or(vec![]);
        let locale = base_parts.get(0).unwrap_or(&"");
        let adjusted_path = if relative_url.starts_with("../") {
            format!("/{}/training/{}", locale, relative_url.trim_start_matches("../"))
        } else {
            format!("/{}/training{}", locale, relative_url)
        };
        let mut joined = base.clone();
        joined.set_path(&adjusted_path);
        joined.to_string()
    }
}

pub fn convert_course_url(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/courses/([^/]+)").unwrap();
    re.captures(url).map(|caps| {
        let locale = &caps[1];
        let course_code = &caps[2];
        (locale.to_string(), format!("https://learn.microsoft.com/api/lists/studyguide/course/course.{}?locale={}", course_code, locale))
    })
}

pub fn extract_course_code(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let course_code_re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/courses/([^/]+)").unwrap();
    match course_code_re.captures(url) {
        Some(caps) => Ok(caps[2].to_string()),
        None => Err(Box::from("Invalid URL")),
    }
}

pub fn transform_module_url(url: &str) -> Option<String> {
    let re = Regex::new(r"https://learn\.microsoft\.com/([^/]+)/training/modules/([^/]+)/?").unwrap();
    re.captures(url).map(|caps| {
        let locale = &caps[1];
        let module_id = &caps[2];
        format!("https://learn.microsoft.com/api/hierarchy/modules/learn.wwl.{}?locale={}", module_id, locale)
    })
}
