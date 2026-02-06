//! Enhanced SOCKS5 Proxy Validator
//!
//! Validates proxy connections and retrieves exit IP with geolocation information
//! Based on Eve-browser implementation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Proxy validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyValidationResult {
    /// Proxy address
    pub proxy: String,
    /// Whether the proxy is valid
    pub is_valid: bool,
    /// Error message if validation failed
    pub error: Option<String>,
    /// Exit IP address
    pub exit_ip: Option<String>,
    /// Geolocation information
    pub location: Option<LocationInfo>,
    /// Latency in milliseconds
    pub latency_ms: Option<f64>,
}

/// Geolocation information from IP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub city: String,
    pub timezone: String,
    pub isp: String,
}

impl LocationInfo {
    /// Format location as a short string (e.g., "US, CA, Los Angeles")
    pub fn format_short(&self) -> String {
        if self.city.is_empty() {
            format!("{}, {}", self.country_code, self.region)
        } else {
            format!("{}, {}, {}", self.country_code, self.region, self.city)
        }
    }

    /// Format location as a full string with ISP
    pub fn format_full(&self) -> String {
        format!("{} - {}", self.format_short(), self.isp)
    }
}

/// SOCKS5 Proxy Validator
pub struct ProxyValidator {
    timeout: Duration,
}

impl ProxyValidator {
    /// Create a new validator with specified timeout
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Validate a SOCKS5 proxy
    ///
    /// # Arguments
    /// * `host` - Proxy host address
    /// * `port` - Proxy port
    /// * `username` - Optional username for authentication
    /// * `password` - Optional password for authentication
    ///
    /// # Returns
    /// Detailed validation result including exit IP and geolocation
    pub fn validate(
        &self,
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
    ) -> ProxyValidationResult {
        let proxy_str = format!("{}:{}", host, port);

        // Step 1: Test SOCKS5 connection
        let start = Instant::now();
        match self.test_connection(host, port, username, password) {
            Ok(()) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;

                // Step 2: Get exit IP and location
                match self.get_exit_ip(host, port, username, password) {
                    Ok((ip, location)) => ProxyValidationResult {
                        proxy: proxy_str,
                        is_valid: true,
                        error: None,
                        exit_ip: Some(ip),
                        location,
                        latency_ms: Some(latency),
                    },
                    Err(e) => ProxyValidationResult {
                        proxy: proxy_str,
                        is_valid: true,
                        error: Some(format!("Cannot get exit IP: {}", e)),
                        exit_ip: None,
                        location: None,
                        latency_ms: Some(latency),
                    },
                }
            }
            Err(e) => ProxyValidationResult {
                proxy: proxy_str,
                is_valid: false,
                error: Some(e.to_string()),
                exit_ip: None,
                location: None,
                latency_ms: None,
            },
        }
    }

    /// Resolve host and port to socket address
    fn resolve_addr(&self, host: &str, port: u16) -> Result<SocketAddr> {
        let mut addrs = (host, port).to_socket_addrs()?;
        addrs
            .next()
            .ok_or_else(|| anyhow!("Cannot resolve: {}:{}", host, port))
    }

    /// Test SOCKS5 connection with authentication
    fn test_connection(
        &self,
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<()> {
        let addr = self.resolve_addr(host, port)?;
        let mut stream = TcpStream::connect_timeout(&addr, self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        // SOCKS5 handshake: version 5, 2 methods (no auth + username/password)
        stream.write_all(&[0x05, 0x02, 0x00, 0x02])?;

        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        if response[0] != 0x05 {
            return Err(anyhow!("Invalid SOCKS version: {}", response[0]));
        }

        if response[1] == 0xFF {
            return Err(anyhow!("No acceptable auth method"));
        }

        // Username/password authentication (method 0x02)
        if response[1] == 0x02 {
            let username = username.ok_or_else(|| anyhow!("Username required"))?;
            let password = password.ok_or_else(|| anyhow!("Password required"))?;

            let username_bytes = username.as_bytes();
            let password_bytes = password.as_bytes();

            let mut auth = Vec::with_capacity(3 + username_bytes.len() + password_bytes.len());
            auth.push(0x01); // Auth version
            auth.push(username_bytes.len() as u8);
            auth.extend_from_slice(username_bytes);
            auth.push(password_bytes.len() as u8);
            auth.extend_from_slice(password_bytes);

            stream.write_all(&auth)?;

            let mut auth_response = [0u8; 2];
            stream.read_exact(&mut auth_response)?;

            if auth_response[1] != 0x00 {
                return Err(anyhow!("Authentication failed"));
            }
        }

        // Test CONNECT command to google.com:80
        let target = b"google.com";
        let mut request = Vec::with_capacity(7 + target.len());
        request.push(0x05); // SOCKS5
        request.push(0x01); // CONNECT
        request.push(0x00); // Reserved
        request.push(0x03); // Domain name
        request.push(target.len() as u8);
        request.extend_from_slice(target);
        request.push(0x00); // Port high byte
        request.push(0x50); // Port low byte (80)

        stream.write_all(&request)?;

        let mut connect_response = [0u8; 10];
        stream.read_exact(&mut connect_response)?;

        if connect_response[1] != 0x00 {
            return Err(anyhow!("CONNECT failed: {}", connect_response[1]));
        }

        Ok(())
    }

    /// Get exit IP and geolocation through proxy
    fn get_exit_ip(
        &self,
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<(String, Option<LocationInfo>)> {
        let addr = self.resolve_addr(host, port)?;
        let mut stream = TcpStream::connect_timeout(&addr, self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        // SOCKS5 handshake
        stream.write_all(&[0x05, 0x02, 0x00, 0x02])?;
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;

        // Authentication if required
        if response[1] == 0x02 {
            let username = username.unwrap();
            let password = password.unwrap();

            let username_bytes = username.as_bytes();
            let password_bytes = password.as_bytes();

            let mut auth = Vec::with_capacity(3 + username_bytes.len() + password_bytes.len());
            auth.push(0x01);
            auth.push(username_bytes.len() as u8);
            auth.extend_from_slice(username_bytes);
            auth.push(password_bytes.len() as u8);
            auth.extend_from_slice(password_bytes);

            stream.write_all(&auth)?;

            let mut auth_response = [0u8; 2];
            stream.read_exact(&mut auth_response)?;

            if auth_response[1] != 0x00 {
                return Err(anyhow!("Authentication failed"));
            }
        }

        // Connect to ip-api.com:80
        let target = b"ip-api.com";
        let mut request = Vec::with_capacity(7 + target.len());
        request.push(0x05);
        request.push(0x01);
        request.push(0x00);
        request.push(0x03);
        request.push(target.len() as u8);
        request.extend_from_slice(target);
        request.push(0x00);
        request.push(0x50); // Port 80

        stream.write_all(&request)?;

        let mut connect_response = [0u8; 10];
        stream.read_exact(&mut connect_response)?;

        if connect_response[1] != 0x00 {
            return Err(anyhow!("CONNECT failed: {}", connect_response[1]));
        }

        // Send HTTP request to get IP and location info
        let http_request = "GET /json?fields=status,message,country,countryCode,region,regionName,city,timezone,isp,query HTTP/1.1\r\n\
                           Host: ip-api.com\r\n\
                           Connection: close\r\n\
                           User-Agent: clash-chain-patcher/1.0\r\n\r\n";

        stream.write_all(http_request.as_bytes())?;

        // Read HTTP response
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;

        let response_str = String::from_utf8_lossy(&response);

        // Parse JSON response
        if let Some(json_start) = response_str.find('{') {
            if let Some(json_end) = response_str.rfind('}') {
                let json_str = &response_str[json_start..=json_end];

                #[derive(Deserialize)]
                struct IpApiResponse {
                    status: String,
                    query: Option<String>,
                    country: Option<String>,
                    #[serde(rename = "countryCode")]
                    country_code: Option<String>,
                    #[serde(rename = "regionName")]
                    region_name: Option<String>,
                    city: Option<String>,
                    timezone: Option<String>,
                    isp: Option<String>,
                }

                let api_response: IpApiResponse = serde_json::from_str(json_str)?;

                if api_response.status == "success" {
                    let ip = api_response.query.unwrap_or_default();
                    let location = LocationInfo {
                        country: api_response.country.unwrap_or_default(),
                        country_code: api_response.country_code.unwrap_or_default(),
                        region: api_response.region_name.unwrap_or_default(),
                        city: api_response.city.unwrap_or_default(),
                        timezone: api_response.timezone.unwrap_or_default(),
                        isp: api_response.isp.unwrap_or_default(),
                    };

                    return Ok((ip, Some(location)));
                }
            }
        }

        Err(anyhow!("Cannot parse IP API response"))
    }

    /// Batch validate multiple proxies
    pub fn validate_batch(
        &self,
        proxies: &[(String, u16, Option<String>, Option<String>)],
    ) -> Vec<ProxyValidationResult> {
        proxies
            .iter()
            .enumerate()
            .map(|(i, (host, port, username, password))| {
                println!("Validating proxy [{}/{}]: {}:{}", i + 1, proxies.len(), host, port);
                self.validate(host, *port, username.as_deref(), password.as_deref())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_creation() {
        let result = ProxyValidationResult {
            proxy: "127.0.0.1:1080".to_string(),
            is_valid: true,
            error: None,
            exit_ip: Some("1.2.3.4".to_string()),
            location: Some(LocationInfo {
                country: "United States".to_string(),
                country_code: "US".to_string(),
                region: "California".to_string(),
                city: "Los Angeles".to_string(),
                timezone: "America/Los_Angeles".to_string(),
                isp: "Example ISP".to_string(),
            }),
            latency_ms: Some(100.5),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"is_valid\":true"));
        assert!(json.contains("\"exit_ip\":\"1.2.3.4\""));
    }

    #[test]
    fn test_location_format() {
        let location = LocationInfo {
            country: "United States".to_string(),
            country_code: "US".to_string(),
            region: "California".to_string(),
            city: "Los Angeles".to_string(),
            timezone: "America/Los_Angeles".to_string(),
            isp: "Example ISP".to_string(),
        };

        assert_eq!(location.format_short(), "US, California, Los Angeles");
        assert_eq!(location.format_full(), "US, California, Los Angeles - Example ISP");
    }

    #[test]
    fn test_validator_creation() {
        let validator = ProxyValidator::new(10);
        assert_eq!(validator.timeout, Duration::from_secs(10));
    }
}
