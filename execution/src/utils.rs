use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use verifier::VerificationResult;

use crate::error::MainProcessError::{self, BadContentSchema};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub transcript_proof: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    URL,
    Json,
    File,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub key: String,
    pub description: String,
    pub mime: String,
    pub tags: Vec<String>,
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
    pub app_id: String,
}

pub fn parse_content_json(json_str: &str) -> Result<ContentSchema, Box<dyn Error>> {
    let content: ContentSchema = serde_json::from_str(json_str)?;
    Ok(content)
}

pub fn get_content_data(
    content: &VerificationResult,
    key: &str,
) -> Result<String, MainProcessError> {
    let array: Vec<&str> = if key.contains('|') {
        key.split_terminator('|').collect()
    } else if key.contains('>') {
        key.split_terminator('>').collect()
    } else {
        return Err(BadContentSchema(
            "Invalid key format - must contain either | or >".into(),
        ));
    };

    if array.len() < 2 {
        return Err(BadContentSchema(
            "Invalid key format - must have at least 2 parts".into(),
        ));
    }

    let ct = if array[0].to_lowercase() == "received" {
        content.received_data.as_str()
    } else if array[0].to_lowercase() == "sent" {
        content.sent_data.as_str()
    } else {
        return Err(BadContentSchema(
            "Invalid key format - must be a received or sent.".into(),
        ));
    };

    if key.contains('|') {
        // HTTP header extraction
        let header_name = array[1].to_lowercase();
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
        Err(BadContentSchema(
            format!("Header '{}' not found in response", header_name).into(),
        ))
    } else {
        // JSON extraction
        // Find the last header line or use the entire content if no headers are found
        let mut lines = ct.lines();
        let mut json_body = String::new();

        // Collect all lines that look like JSON (starting with '{' or ending with '}')
        while let Some(line) = lines.next() {
            if line.trim().starts_with('{') {
                json_body = line.to_string();
                // Collect any remaining lines
                json_body.push_str(&lines.collect::<Vec<&str>>().join(""));
                break;
            }
        }

        if json_body.is_empty() {
            return Err(BadContentSchema(
                "Could not find JSON body in response".into(),
            ));
        }

        let json_value: serde_json::Value = serde_json::from_str(&json_body)
            .map_err(|e| BadContentSchema(format!("Failed to parse JSON body: {}", e).into()))?;
        let field_name = array[1];

        match json_value.get(field_name) {
            Some(value) => match value {
                serde_json::Value::String(s) => Ok(s.clone()),
                serde_json::Value::Bool(b) => Ok(b.to_string()),
                serde_json::Value::Number(n) => Ok(n.to_string()),
                serde_json::Value::Null => Ok("null".to_string()),
                serde_json::Value::Object(o) => serde_json::to_string(o).map_err(|e| {
                    BadContentSchema(format!("Failed to serialize JSON object: {}", e).into())
                }),
                serde_json::Value::Array(a) => serde_json::to_string(a).map_err(|e| {
                    BadContentSchema(format!("Failed to serialize JSON array: {}", e).into())
                }),
            },
            None => Err(BadContentSchema(
                format!("Field '{}' not found in JSON response", field_name).into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use tlsn_core::ServerName;

    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_content_data() {
        let content = VerificationResult {
            received_data: String::from(
                r#"HTTP/1.1 200 OK
date: Thu, 19 Sep 2024 12:23:10 GMT
content-type: application/json;charset=utf-8
{"protected":false,"screen_name":"g_p_vlayer","sleep_time":{"enabled":false,"end_time":null,"start_time":null}}"#,
            ),
            sent_data: String::from(
                r#"HTTP/1.1 200 OK
date: Thu, 19 Sep 2024 12:23:10 GMT
perf: 7402827104
pragma: no-cache
server: tsa_o
status: 200 OK
expires: Tue, 31 Mar 1981 05:00:00 GMT
set-cookie: guest_id=v1%3A172674859049314397; Max-Age=34214400; Expires=Mon, 20 Oct 2025 12:23:10 GMT; Path=/; Domain=.x.com; Secure; SameSite=None
content-type: application/json;charset=utf-8
cache-control: no-cache, no-store, must-revalidate, pre-check=0, post-check=0
last-modified: Thu, 19 Sep 2024 12:23:10 GMT
x-transaction: bcdaba45f8cff3ed
content-length: 1078
x-access-level: read-write-directmessages
x-frame-options: SAMEORIGIN
x-transaction-id: bcdaba45f8cff3ed
x-xss-protection: 0
content-disposition: attachment; filename=json.json
x-client-event-enabled: true
x-content-type-options: nosniff
x-twitter-response-tags: BouncerCompliant
strict-transport-security: max-age=631138519
x-response-time: 124
x-connection-hash: 5a77fa2e596c5950ceff5a1c0207094a333aa663e4badcb2c8ce8b0b317accd6
connection: close

{"protected":false,"screen_name":"g_p_vlayer","always_use_https":true,"use_cookie_personalization":false,"sleep_time":{"enabled":false,"end_time":null,"start_time":null},"geo_enabled":false,"language":"en","discoverable_by_email":false,"discoverable_by_mobile_phone":false,"display_sensitive_media":false,"personalized_trends":true,"allow_media_tagging":"all","allow_contributor_request":"none","allow_ads_personalization":false,"allow_logged_out_device_personalization":false,"allow_location_history_personalization":false,"allow_sharing_data_for_third_party_personalization":false,"allow_dms_from":"following","always_allow_dms_from_subscribers":null,"allow_dm_groups_from":"following","translator_type":"none","country_code":"pl","address_book_live_sync_enabled":false,"universal_quality_filtering_enabled":"enabled","dm_receipt_setting":"all_enabled","allow_authenticated_periscope_requests":true,"protect_password_reset":false,"require_password_login":false,"requires_login_verification":false,"dm_quality_filter":"enabled","autoplay_disabled":false,"settings_metadata":{}}""#,
            ),
            server_name: ServerName::Dns("api.x.com".to_string()),
            time: DateTime::to_utc(&Utc.ymd(2024, 9, 19).and_hms(12, 23, 10)),
        };

        // Test HTTP header extraction
        let result = get_content_data(&content, "received|content-type");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "application/json;charset=utf-8");

        // Test JSON field extraction
        let result = get_content_data(&content, "received>screen_name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "g_p_vlayer");

        // Test nested object extraction
        let result = get_content_data(&content, "received>sleep_time");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r#"{"enabled":false,"end_time":null,"start_time":null}"#
        );

        // Test error cases
        let result = get_content_data(&content, "invalid");
        assert!(result.is_err());

        let result = get_content_data(&content, "received>nonexistent");
        assert!(result.is_err());

        let result = get_content_data(&content, "received|nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_content_json() {
        let valid_json = r#"{
            "category": "Social",
            "source": "TikTok",
            "url": "www.tiktok.com",
            "name": "Test",
            "address": "0x0000000000000000000000000000000000000000",
            "metadata": {
                "property": {
                    "key": "test",
                    "description": "test desc",
                    "mime": "text/plain",
                    "tags": ["test"],
                    "type": "url"
                },
                "owner": "test_owner"
            }
        }"#;

        let result = parse_content_json(valid_json);
        assert!(result.is_ok());

        let invalid_json = "{invalid}";
        let result = parse_content_json(invalid_json);
        assert!(result.is_err());

        let missing_fields_json = r#"{
            "category": "Social"
        }"#;
        let result = parse_content_json(missing_fields_json);
        assert!(result.is_err());
    }
}
