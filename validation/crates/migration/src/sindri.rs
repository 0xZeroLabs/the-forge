use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::Value;
use std::{fs::File, io::Read, time::Duration};

const API_URL: &'static str = "https://sindri.app/api/v1/";

// This function proves the circuit using the input data provided by the user.
pub async fn prove_guest_code(json_input: &str, header: HeaderMap) -> Value {
    println!("Reading circuit details locally");
    let mut file = File::open("./data/compile_out.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let circuit_data: Value = serde_json::from_str(&data).unwrap();
    let circuit_id = &circuit_data["circuit_id"].as_str().unwrap();
    let circuit_id = circuit_id;

    // Initiate proof generation.
    println!("Reading proof input from input.json file");
    let proof_input = json_input.to_string();
    let map = serde_json::json!({"proof_input": proof_input});

    println!("Requesting a proof");
    let client = Client::new();
    let response = client
        .post(format!("{API_URL}circuit/{circuit_id}/prove"))
        .headers(header.clone())
        .json(&map)
        .send()
        .await
        .unwrap();
    assert_eq!(&response.status().as_u16(), &201u16, "Expected status code 201");
    let response_body = response.json::<Value>().await.unwrap();
    let proof_id = response_body["proof_id"].as_str().unwrap();

    // Poll proof detail until it has a status of Ready or Failed.
    let proof_data = poll_proof_status(header, proof_id).await;
    if proof_data["status"].as_str().unwrap().contains("Failed") {
        println!("Proving failed.");
        std::process::exit(1);
    }

    proof_data
}

// This function creates a header map with the API key and sets Accept header to
// application/json.
pub fn headers_json(api_key: &str) -> HeaderMap {
    let mut headers_json = HeaderMap::new();
    headers_json.insert("Accept", "application/json".parse().unwrap());
    headers_json.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {api_key}").to_string()).unwrap(),
    );
    headers_json
}

// This function polls the status of the proof until it is Ready or Failed.
async fn poll_proof_status(header: HeaderMap, proof_id: &str) -> Value {
    let endpoint = format!("proof/{proof_id}/detail");
    let timeout = 600;
    let return_value = poll_status(&endpoint, &API_URL, header, timeout).await;
    return_value
}

// Poll the status of the endpoint until it is Ready or Failed.
// The function will return the data in JSON for either case.
// If the status is ready, the function will return a JSON file containing
// circuit or proof data. If the status is failed, the function will return a
// JSON file containing an error message.
async fn poll_status(endpoint: &str, api_url: &str, header: HeaderMap, timeout: i64) -> Value {
    let client = Client::new();
    for _i in 1..timeout {
        let response = client
            .get(format!("{api_url}{endpoint}"))
            .headers(header.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(&response.status().as_u16(), &200u16, "Expected status code 201");

        // If the response is Ready or Failed, break the loop and return the data.
        let data = response.json::<Value>().await.unwrap();
        let status = &data["status"].to_string();
        if ["Ready", "Failed"].iter().any(|&s| status.as_str().contains(s)) {
            return data;
        }

        // Wait for 1 second before polling again.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Polling timed out after {} seconds", timeout);
    std::process::exit(1);
}
