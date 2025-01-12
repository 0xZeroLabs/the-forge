use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub transcript_proof: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    URL,
    Text,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub property_type: PropertyType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub property: Property,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentSchema {
    pub category: String,
    pub source: String,
    pub url: String,
    pub metadata: Metadata,
    pub name: String,
    pub address: Address,
}

pub fn parse_content_json(json_str: &str) -> Result<ContentSchema, Box<dyn Error>> {
    let content: ContentSchema = serde_json::from_str(json_str)?;
    Ok(content)
}

pub fn get_content_data(ct: &str, key: &str) -> Result<String, Box<dyn Error>> {
    let array: Vec<&str> = key.split_terminator('|').collect();

    if array.len() < 2 {
        return Err("Invalid key format - must have at least 2 parts separated by |".into());
    }

    // Get the header name we're looking for from the second part of the key
    let header_name = array[1].to_lowercase();

    // Split response into lines and look for the header
    for line in ct.lines() {
        let line = line.trim();
        let parts: Vec<&str> = line.split(": ").collect();
        if parts.len() == 2 {
            let current_header = parts[0].to_lowercase();
            if current_header == header_name {
                return Ok(parts[1].to_string());
            }
        }
    }

    Err(format!("Header '{}' not found in response", header_name).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_content_data() {
        let http_response = r#"HTTP/1.1 200 OK
  date: Thu, 19 Sep 2024 12:23:10 GMT
  content-type: application/json;charset=utf-8
  server: tsa_o
  status: 200 OK
  x-transaction-id: bcdaba45f8cff3ed"#;

        // Test successful extraction
        let result = get_content_data(http_response, "received|content-type");
        match &result {
            Ok(_) => println!("Extraction successful"),
            Err(e) => println!("Extraction error: {}", e),
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "application/json;charset=utf-8");

        // Test case-insensitive matching
        let result = get_content_data(http_response, "received|CONTENT-TYPE");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "application/json;charset=utf-8");

        // Test header not found
        let result = get_content_data(http_response, "received|nonexistent-header");
        assert!(result.is_err());

        // Test invalid key format
        let result = get_content_data(http_response, "invalid_key");
        assert!(result.is_err());
    }

    use alloy::primitives::address;

    #[tokio::test]
    async fn test_parse_content_json() {
        // Create a test JSON string with valid data
        let json_str = r#"{
            "category": "Social",
            "source": "TikTok",
            "url": "www.tiktok.com",
            "name": "Test Name",
            "address": "0x0000000000000000000000000000000000000000",
            "metadata": {
                "property": {
                    "key": "received|post_image_url",
                    "value": "https://example.com/image.png",
                    "type": "url"
                },
                "owner": "received|user_id_str"
            }
        }"#;

        // Test successful parsing
        let result = parse_content_json(json_str);
        match &result {
            Ok(_) => println!("Parsing successful"),
            Err(e) => println!("Parsing error: {}", e),
        }
        assert!(result.is_ok());

        let content = result.unwrap();
        assert_eq!(content.category, "Social");
        assert_eq!(content.source, "TikTok");
        assert_eq!(content.url, "www.tiktok.com");
        assert_eq!(content.name, "Test Name");
        assert_eq!(
            content.address,
            address!("0x0000000000000000000000000000000000000000")
        );
        assert_eq!(content.metadata.property.key, "received|post_image_url");
        assert_eq!(
            content.metadata.property.value,
            "https://example.com/image.png"
        );
        assert!(matches!(
            content.metadata.property.property_type,
            PropertyType::URL
        ));
        assert_eq!(content.metadata.owner, "received|user_id_str");

        // Test invalid JSON
        let invalid_json = r#"{
            "category": "Social",
            "invalid_json"
        }"#;
        let result = parse_content_json(invalid_json);
        assert!(result.is_err());

        // Test missing required fields
        let missing_fields = r#"{
            "category": "Social",
            "source": "TikTok"
        }"#;
        let result = parse_content_json(missing_fields);
        assert!(result.is_err());

        // Test invalid property type
        let invalid_type = r#"{
            "category": "Social",
            "source": "TikTok",
            "url": "www.tiktok.com",
            "name": "Test Name",
            "address": "0x0000000000000000000000000000000000000000",
            "metadata": {
                "property": {
                    "key": "data.post_image_url",
                    "value": "https://example.com/image.png",
                    "type": "INVALID_TYPE"
                },
                "owner": "data.user_id_str"
            }
        }"#;
        let result = parse_content_json(invalid_type);
        assert!(result.is_err());

        // Test invalid address format
        let invalid_address = r#"{
            "category": "Social",
            "source": "TikTok",
            "url": "www.tiktok.com",
            "name": "Test Name",
            "address": "invalid_address",
            "metadata": {
                "property": {
                    "key": "data.post_image_url",
                    "value": "https://example.com/image.png",
                    "type": "URL"
                },
                "owner": "data.user_id_str"
            }
        }"#;
        let result = parse_content_json(invalid_address);
        assert!(result.is_err());
    }
}
