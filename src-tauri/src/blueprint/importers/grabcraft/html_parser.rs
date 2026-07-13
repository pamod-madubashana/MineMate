use scraper::{Html, Selector};

pub struct GrabCraftPageData {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub layer_urls: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub length: Option<u32>,
}

pub fn parse_grabcraft_page(html: &str) -> Result<GrabCraftPageData, String> {
    let document = Html::parse_document(html);

    let name = extract_text(&document, "h1, .build-name, [class*='name']");
    let author = extract_text(&document, ".author, [class*='author']");
    let description = extract_text(&document, ".description, [class*='description']");

    let layer_urls = extract_layer_urls(&document)?;

    let width = extract_dimension(&document, "width");
    let height = extract_dimension(&document, "height");
    let length = extract_dimension(&document, "length");

    Ok(GrabCraftPageData {
        name,
        author,
        description,
        layer_urls,
        width,
        height,
        length,
    })
}

fn extract_text(document: &Html, selector: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    let element = document.select(&sel).next()?;
    Some(element.text().collect::<String>().trim().to_string())
}

fn extract_layer_urls(document: &Html) -> Result<Vec<String>, String> {
    let mut urls = Vec::new();

    let selectors = [
        "img[src*='layer']",
        "img[data-src*='layer']",
        ".layer img",
        "[class*='layer'] img",
        "img[src*='blueprint']",
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src")
                    .or_else(|| element.value().attr("data-src"))
                {
                    let url = if src.starts_with("http") {
                        src.to_string()
                    } else if src.starts_with("//") {
                        format!("https:{}", src)
                    } else {
                        format!("https://www.grabcraft.com{}", src)
                    };

                    if !urls.contains(&url) {
                        urls.push(url);
                    }
                }
            }
        }
    }

    if urls.is_empty() {
        for element in document.select(&Selector::parse("img").map_err(|e| e.to_string())?) {
            if let Some(src) = element.value().attr("src") {
                if src.contains("layer") || src.contains("blueprint") || src.contains("png") || src.contains("jpg") {
                    let url = if src.starts_with("http") {
                        src.to_string()
                    } else if src.starts_with("//") {
                        format!("https:{}", src)
                    } else {
                        format!("https://www.grabcraft.com{}", src)
                    };

                    if !urls.contains(&url) {
                        urls.push(url);
                    }
                }
            }
        }
    }

    urls.sort_by(|a, b| {
        let num_a = extract_number_from_url(a);
        let num_b = extract_number_from_url(b);
        num_a.cmp(&num_b)
    });

    Ok(urls)
}

fn extract_number_from_url(url: &str) -> u32 {
    url.split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse().ok())
        .max()
        .unwrap_or(0)
}

fn extract_dimension(document: &Html, dimension: &str) -> Option<u32> {
    let selectors = [
        &format!("[class*='{}']", dimension),
        &format!("span:contains('{}')", dimension),
        &format!("div:contains('{}')", dimension),
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>();
                if let Some(num) = text.split_whitespace()
                    .find_map(|s| s.parse().ok())
                {
                    return Some(num);
                }
            }
        }
    }

    None
}
