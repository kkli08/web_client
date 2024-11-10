use reqwest::blocking::Client;
#[allow(unused_imports)]
use reqwest::Error as ReqwestError;
use structopt::StructOpt;
use url::{ParseError, Url};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(StructOpt, Debug)]
#[structopt(name = "curl", about = "A simple curl command-line tool in Rust")]
struct CurlArgs {
    #[structopt(name = "url")]
    url: String,
}

fn main() {
    let args = CurlArgs::from_args();
    let url_input = args.url;

    // Attempt to parse URL
    match Url::parse(&url_input) {
        Ok(url) => {
            println!("Requesting URL: {}", url);
            println!("Method: GET");

            // Check IP address and port number
            if let Err(e) = check_ip_address(&url) {
                println!("Error: {}", e);
            } else if let Err(e) = check_port_number(&url) {
                println!("Error: {}", e);
            } else {
                // Proceed to make GET request
                match make_get_request(&url) {
                    Ok(response) => {
                        println!("Response:\n{}", response);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Requesting URL: {}", url_input);
            println!("Method: GET");

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
                return Err("The URL contains an invalid IPv6 address.".to_string());
            } else {
                Ok(())
            }
        } else {
            // Check if host_str is numeric with dots (possible IPv4)
            if host_str.chars().all(|c| c.is_digit(10) || c == '.') {
                return Err("The URL contains an invalid IPv4 address.".to_string());
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
            return Err("The URL contains an invalid port number.".to_string());
        } else {
            Ok(())
        }
    } else {
        // No port number, skip the check
        Ok(())
    }
}

// Function to handle GET request using reqwest
fn make_get_request(url: &Url) -> Result<String, String> {
    let client = Client::new();
    let response = client.get(url.as_str()).send();

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                Err(format!("Request failed with status code: {}.", resp.status().as_u16()))
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
