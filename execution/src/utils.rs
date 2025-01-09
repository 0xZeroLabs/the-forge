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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_parse_content_json() {
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
        assert_eq!(content.metadata.property.key, "data.post_image_url");
        assert_eq!(
            content.metadata.property.value,
            "https://example.com/image.png"
        );
        assert!(matches!(
            content.metadata.property.property_type,
            PropertyType::URL
        ));
        assert_eq!(content.metadata.owner, "data.user_id_str");

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
