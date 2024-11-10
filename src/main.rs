use reqwest::blocking::Client;
#[allow(unused_imports)]
use reqwest::Error as ReqwestError;
use serde_json::Value;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use structopt::StructOpt;
use url::{ParseError, Url};

#[derive(StructOpt, Debug)]
#[structopt(name = "curl", about = "A simple curl command-line tool in Rust")]
struct CurlArgs {
    #[structopt(name = "url")]
    url: String,

    /// HTTP method to use (GET, POST, etc.)
    #[structopt(short = "X", long = "request", default_value = "GET")]
    method: String,

    /// Data to send with POST request in the form 'key1=value1&key2=value2'
    #[structopt(short = "d", long = "data")]
    data: Option<String>,
}

fn main() {
    let args = CurlArgs::from_args();
    let url_input = args.url;
    let method = args.method.to_uppercase();
    let data = args.data;

    // Attempt to parse URL
    match Url::parse(&url_input) {
        Ok(url) => {
            println!("Requesting URL: {}", url);
            println!("Method: {}", method);

            if let Some(ref data_str) = data {
                println!("Data: {}", data_str);
            }

            // Check IP address and port number
            if let Err(e) = check_ip_address(&url) {
                println!("Error: {}", e);
            } else if let Err(e) = check_port_number(&url) {
                println!("Error: {}", e);
            } else {
                // Proceed to make request
                match make_request(&url, &method, data) {
                    Ok(response) => {
                        // Handle response
                        handle_response(&response);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Requesting URL: {}", url_input);
            println!("Method: {}", method);

            if let Some(ref data_str) = data {
                println!("Data: {}", data_str);
            }

            match e {
                ParseError::RelativeUrlWithoutBase => {
                    println!("Error: The URL does not have a valid base protocol.");
                }
                ParseError::InvalidIpv6Address => {
                    println!("Error: The URL contains an invalid IPv6 address.");
                }
                ParseError::InvalidIpv4Address => {
                    println!("Error: The URL contains an invalid IPv4 address.");
                }
                ParseError::InvalidPort => {
                    println!("Error: The URL contains an invalid port number.");
                }
                _ => {
                    println!("Error: Invalid URL: {}", e);
                }
            }
        }
    }
}

// Function to check IP address syntax
fn check_ip_address(url: &Url) -> Result<(), String> {
    if let Some(host_str) = url.host_str() {
        // Try to parse as IPv4 address
        if let Ok(_) = Ipv4Addr::from_str(host_str) {
            // Valid IPv4 address
            Ok(())
        } else if host_str.starts_with('[') && host_str.ends_with(']') {
            // Possible IPv6 address
            let ipv6_str = &host_str[1..host_str.len() - 1];
            if Ipv6Addr::from_str(ipv6_str).is_err() {
                Err("The URL contains an invalid IPv6 address.".to_string())
            } else {
                Ok(())
            }
        } else {
            // Check if host_str is numeric with dots (possible IPv4)
            if host_str.chars().all(|c| c.is_digit(10) || c == '.') {
                Err("The URL contains an invalid IPv4 address.".to_string())
            } else {
                // Not an IP address, skip checking
                Ok(())
            }
        }
    } else {
        // No host present
        Err("The URL does not contain a host.".to_string())
    }
}

// Function to check port number
#[allow(unused_comparisons)]
fn check_port_number(url: &Url) -> Result<(), String> {
    if let Some(port) = url.port() {
        if port > 65535 {
            Err("The URL contains an invalid port number.".to_string())
        } else {
            Ok(())
        }
    } else {
        // No port number, skip the check
        Ok(())
    }
}

// Function to make HTTP request
fn make_request(url: &Url, method: &str, data: Option<String>) -> Result<String, String> {
    let client = Client::new();

    let response = match method {
        "GET" => client.get(url.as_str()).send(),
        "POST" => {
            if let Some(data_str) = data {
                // Parse data into key-value pairs
                let params = parse_data(&data_str);
                client.post(url.as_str()).form(&params).send()
            } else {
                client.post(url.as_str()).send()
            }
        }
        _ => {
            return Err(format!("Unsupported HTTP method: {}", method));
        }
    };

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                Err(format!(
                    "Request failed with status code: {}.",
                    resp.status().as_u16()
                ))
            } else {
                match resp.text() {
                    Ok(text) => Ok(text),
                    Err(e) => Err(format!("Failed to read response text: {}", e)),
                }
            }
        }
        Err(e) => {
            if e.is_connect() {
                Err("Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.".to_string())
            } else {
                Err(format!("Request Error: {}", e))
            }
        }
    }
}

// Function to parse data string into key-value pairs
fn parse_data(data_str: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in data_str.split('&') {
        let mut key_value = pair.splitn(2, '=');
        let key = key_value.next().unwrap_or("").to_string();
        let value = key_value.next().unwrap_or("").to_string();
        params.insert(key, value);
    }
    params
}

// Function to handle response
fn handle_response(response: &str) {
    // Try to parse response as JSON
    match serde_json::from_str::<Value>(response) {
        Ok(json_value) => {
            // It's JSON, print with sorted keys
            println!("Response body (JSON with sorted keys):");
            print_json_sorted(&json_value);
        }
        Err(_) => {
            // Not JSON, print response directly
            println!("Response:\n{}", response);
        }
    }
}

// Function to print JSON with keys sorted alphabetically
fn print_json_sorted(value: &Value) {
    let sorted_json = sort_json(value);
    // Pretty-print the sorted JSON
    println!("{}", serde_json::to_string_pretty(&sorted_json).unwrap());
}

// Function to recursively sort JSON object
fn sort_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                let val = map.get(key).unwrap();
                sorted_map.insert(key.clone(), sort_json(val));
            }
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            let sorted_array: Vec<Value> = arr.iter().map(|v| sort_json(v)).collect();
            Value::Array(sorted_array)
        }
        _ => value.clone(),
    }
}
