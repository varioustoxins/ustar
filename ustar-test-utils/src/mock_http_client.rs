//! Mock HTTP client for testing download functionality

use std::collections::HashMap;

/// Mock HTTP client for testing with pre-recorded responses
pub struct MockHttpClient {
    responses: HashMap<String, String>,
    binary_responses: HashMap<String, Vec<u8>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            binary_responses: HashMap::new(),
        }
    }

    /// Add a text response for a URL
    pub fn with_response(mut self, url: &str, response: &str) -> Self {
        self.responses.insert(url.to_string(), response.to_string());
        self
    }

    /// Add a binary response for a URL
    pub fn with_binary_response(mut self, url: &str, response: Vec<u8>) -> Self {
        self.binary_responses.insert(url.to_string(), response);
        self
    }

    /// Add a response from a file
    pub fn with_file_response(self, url: &str, file_path: &str) -> Self {
        let content = std::fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read mock file: {}", file_path));
        self.with_response(url, &content)
    }

    /// Get a text response for a URL
    pub fn get(&self, url: &str) -> Result<String, String> {
        self.responses
            .get(url)
            .cloned()
            .ok_or_else(|| format!("Mock: No response for URL: {}", url))
    }

    /// Get binary response for a URL
    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>, String> {
        // Try binary responses first
        if let Some(binary) = self.binary_responses.get(url) {
            return Ok(binary.clone());
        }

        // Fall back to text response as bytes
        self.responses
            .get(url)
            .map(|s| s.as_bytes().to_vec())
            .ok_or_else(|| format!("Mock: No response for URL: {}", url))
    }
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self::new()
    }
}
