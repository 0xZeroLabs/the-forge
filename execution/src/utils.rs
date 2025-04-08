use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
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
    Url,
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
    pub submitter: Address,
}

pub fn parse_content_json(json_str: &str) -> Result<ContentSchema, Box<dyn Error>> {
    let content: ContentSchema = serde_json::from_str(json_str)?;
    Ok(content)
}

pub fn get_content_data(
    content: &VerificationResult,
    key: &str,
) -> Result<String, MainProcessError> {
    // splits the key and validate format first
    let (data_source, extraction_key) = parse_key(key)?;

    // get the appropriate content based on data source
    let content_str = match data_source.to_lowercase().as_str() {
        "received" => content.received_data.as_str(),
        "sent" => content.sent_data.as_str(),
        _ => {
            return Err(BadContentSchema(
                "Data source must be 'received' or 'sent'".into(),
            ))
        }
    };

    // use the parsed components to determine the extraction method
    if key.contains('|') {
        extract_header(content_str, extraction_key)
    } else {
        extract_json(content_str, extraction_key)
    }
}

// parse and validate the key format
fn parse_key(key: &str) -> Result<(&str, &str), MainProcessError> {
    let separator = if key.contains('|') {
        '|'
    } else if key.contains('>') {
        '>'
    } else {
        return Err(BadContentSchema(
            "Invalid key format - must contain either | or >".into(),
        ));
    };

    let parts: Vec<&str> = key.split(separator).collect();
    if parts.len() != 2 {
        return Err(BadContentSchema(
            "Invalid key format - must have exactly 2 parts".into(),
        ));
    }

    Ok((parts[0], parts[1]))
}

// parser utility function to extract headers
fn extract_header(content: &str, header_name: &str) -> Result<String, MainProcessError> {
    let normalized_header = header_name.trim().to_lowercase();
    let mut lines = content.lines();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            if key == normalized_header {
                let mut values = vec![value.trim().trim_end_matches(',').trim()];

                for next_line in lines.by_ref() {
                    // check original line before trimming
                    if !next_line.starts_with("                    ") {
                        break;
                    }

                    let cleaned = next_line.trim().trim_end_matches(',').trim();
                    if !cleaned.is_empty() {
                        values.push(cleaned);
                    }
                }

                return Ok(values.join(", "));
            }
        }
    }

    Err(BadContentSchema(format!(
        "Header '{}' not found in response",
        header_name
    )))
}
fn extract_json(content: &str, field_name: &str) -> Result<String, MainProcessError> {
    // finds JSON content more reliably
    let json_body = extract_json_body(content)?;

    // parse and extract the requested field
    let json_value: serde_json::Value = serde_json::from_str(&json_body)
        .map_err(|e| BadContentSchema(format!("Failed to parse JSON body: {}", e)))?;

    match json_value.get(field_name) {
        Some(value) => serialize_json_value(value),
        None => Err(BadContentSchema(format!(
            "Field '{}' not found in JSON response",
            field_name
        ))),
    }
}

fn extract_json_body(content: &str) -> Result<String, MainProcessError> {
    let mut in_headers = true;
    let mut json_lines = Vec::new();
    let mut empty_line_count = 0;

    for line in content.lines() {
        if line.trim().is_empty() {
            if in_headers {
                empty_line_count += 1;
                if empty_line_count >= 2 {
                    in_headers = false;
                }
            }
            continue;
        }

        if !in_headers {
            json_lines.push(line);
        } else if line.trim().starts_with('{') {
            // direct JSON content without headers
            json_lines.push(line);
            in_headers = false;
        }
    }

    if json_lines.is_empty() {
        return Err(BadContentSchema(
            "Could not find JSON body in response".into(),
        ));
    }

    Ok(json_lines.join(""))
}

// serializes JSON values consistently
fn serialize_json_value(value: &serde_json::Value) -> Result<String, MainProcessError> {
    match value {
        serde_json::Value::String(s) => Ok(s.clone()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Null => Ok("null".to_string()),
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => serde_json::to_string(value)
            .map_err(|e| BadContentSchema(format!("Failed to serialize JSON value: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use tlsn_core::connection::ServerName;

    use super::*;

    fn create_test_verification_result(received: &str, sent: &str) -> VerificationResult {
        VerificationResult {
            received_data: received.to_string(),
            sent_data: sent.to_string(),
            server_name: ServerName::new("api.x.com".to_string()),
            time: DateTime::to_utc(&Utc.with_ymd_and_hms(2024, 9, 19, 12, 23, 10).unwrap()),
        }
    }

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
            server_name: ServerName::new("api.x.com".to_string()),
            time: DateTime::to_utc(&Utc.with_ymd_and_hms(2024, 9, 19, 12, 23, 10).unwrap()),
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
            },
            "app_id": "ap3sd1234567890",
            "submitter": "0x0000000000000000000000000000000000000000"
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

    #[test]
    fn test_header_extraction() {
        let content = create_test_verification_result(
            r#"HTTP/1.1 200 OK
                Content-Type: application/json
                X-Custom-Header: test value
                Multi-Line-Header: first line,
                    second line,
                    third line"#,
            "",
        );

        // basic header extraction
        assert_eq!(
            get_content_data(&content, "received|content-type").unwrap(),
            "application/json"
        );

        // case insensitive header names
        assert_eq!(
            get_content_data(&content, "received|CONTENT-TYPE").unwrap(),
            "application/json"
        );

        // headers with multiple lines
        assert_eq!(
            get_content_data(&content, "received|multi-line-header").unwrap(),
            "first line, second line, third line"
        );

        // non-existent header
        assert!(get_content_data(&content, "received|nonexistent").is_err());
    }

    #[test]
    fn test_json_extraction() {
        let content = create_test_verification_result(
            r#"HTTP/1.1 200 OK
                Content-Type: application/json

                {
                    "string_field": "test",
                    "number_field": 123,
                    "bool_field": true,
                    "null_field": null,
                    "object_field": {"key": "value"},
                    "array_field": [1, 2, 3],
                    "nested": {
                        "deep": {
                            "deeper": "value"
                        }
                    }
                }"#,
            "",
        );

        // string field
        assert_eq!(
            get_content_data(&content, "received>string_field").unwrap(),
            "test"
        );

        // number field
        assert_eq!(
            get_content_data(&content, "received>number_field").unwrap(),
            "123"
        );

        // boolean field
        assert_eq!(
            get_content_data(&content, "received>bool_field").unwrap(),
            "true"
        );

        // null field
        assert_eq!(
            get_content_data(&content, "received>null_field").unwrap(),
            "null"
        );

        // object field
        assert_eq!(
            get_content_data(&content, "received>object_field").unwrap(),
            r#"{"key":"value"}"#
        );

        // array field
        assert_eq!(
            get_content_data(&content, "received>array_field").unwrap(),
            "[1,2,3]"
        );

        // Non-existent field
        assert!(get_content_data(&content, "received>nonexistent").is_err());
    }

    #[test]
    fn test_edge_cases() {
        // test direct JSON without headers
        let direct_json = create_test_verification_result(r#"{"key": "value"}"#, "");
        assert_eq!(
            get_content_data(&direct_json, "received>key").unwrap(),
            "value"
        );

        // test JSON with extra whitespace
        let whitespace_json = create_test_verification_result(
            r#"

                {
                    "key": "value"
                }

                "#,
            "",
        );
        assert_eq!(
            get_content_data(&whitespace_json, "received>key").unwrap(),
            "value"
        );

        // test malformed headers
        let malformed_headers = create_test_verification_result(
            r#"Invalid-Line
                Content-Type: application/json
                Malformed-Header
                {"key": "value"}"#,
            "",
        );
        assert_eq!(
            get_content_data(&malformed_headers, "received>key").unwrap(),
            "value"
        );
    }

    #[test]
    fn test_input_validation() {
        let content = create_test_verification_result("", "");

        // invalid key format
        assert!(get_content_data(&content, "invalid").is_err());
        assert!(get_content_data(&content, "invalid|").is_err());
        assert!(get_content_data(&content, "|invalid").is_err());
        assert!(get_content_data(&content, "a|b|c").is_err());

        // invalid data source
        assert!(get_content_data(&content, "invalid|header").is_err());
        assert!(get_content_data(&content, "invalid>field").is_err());
    }

    #[test]
    fn test_special_characters() {
        let content = create_test_verification_result(
            r#"HTTP/1.1 200 OK
                Content-Type: application/json

                {
                    "special:chars": "value:with:colons",
                    "nested": {
                        "field:with:colon": "test"
                    }
                }"#,
            "",
        );

        // test fields with colons
        assert_eq!(
            get_content_data(&content, "received>special:chars").unwrap(),
            "value:with:colons"
        );
    }

    #[test]
    fn test_large_json_response() {
        let mut large_json = String::from(
            r#"HTTP/1.1 200 OK
                Content-Type: application/json

                {"data": ["#,
        );

        // create a large array of items
        for i in 0..1000 {
            if i > 0 {
                large_json.push(',');
            }
            large_json.push_str(&format!(r#"{{"id": {}}}"#, i));
        }
        large_json.push_str("]}");

        let content = create_test_verification_result(&large_json, "");
        let result = get_content_data(&content, "received>data");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with('['));
    }
}
