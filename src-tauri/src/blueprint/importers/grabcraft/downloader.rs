pub struct GrabCraftDownloader {
    client: reqwest::Client,
}

impl GrabCraftDownloader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MineMate/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn fetch_page(&self, url: &str) -> Result<String, String> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch page: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error: {}", status));
        }

        response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))
    }

    pub async fn download_image(&self, url: &str) -> Result<Vec<u8>, String> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to download image: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error: {}", status));
        }

        response.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| format!("Failed to read image: {}", e))
    }
}

impl Default for GrabCraftDownloader {
    fn default() -> Self {
        Self::new()
    }
}
