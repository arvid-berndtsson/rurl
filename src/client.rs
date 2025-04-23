use native_tls::TlsConnector;
use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    thread,
    time::Duration,
};

use crate::{args::Args, http::build_http_request, http2::build_http2_request, utils::parse_url};

/// Error type for request operations
#[derive(Debug)]
pub enum RequestError {
    ConnectionError(String),
    TlsError(String),
    WriteError(String),
    ReadError(String),
    NoResponseError,
}

/// Send an HTTP request and receive the response.
///
/// # Arguments
///
/// * `args` - A reference to an `Args` struct containing the request parameters.
///
/// # Returns
///
/// * `Result<Vec<u8>, RequestError>` - A vector of bytes representing the response, or an error if something went wrong.
pub fn send_request(args: &Args) -> Result<Vec<u8>, RequestError> {
    // Build request based on protocol version
    let request_bytes = if args.http2 {
        match build_http2_request(args) {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(RequestError::ConnectionError(err.to_string()));
            }
        }
    } else {
        match build_http_request(args) {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(RequestError::ConnectionError(err.to_string()));
            }
        }
    };

    let (host, port, _, is_https) = match parse_url(&args.url) {
        Ok(parsed) => parsed,
        Err(err) => {
            return Err(RequestError::ConnectionError(err.to_string()));
        }
    };

    // Connect and send request with timeout
    let addr = format!("{}:{}", host, port);
    let addrs = match addr.to_socket_addrs() {
        Ok(addrs) => addrs.collect::<Vec<_>>(),
        Err(err) => {
            return Err(RequestError::ConnectionError(format!(
                "DNS resolution error: {}",
                err
            )));
        }
    };

    if addrs.is_empty() {
        return Err(RequestError::ConnectionError(
            "No DNS records found".to_string(),
        ));
    }

    let mut stream = match TcpStream::connect_timeout(&addrs[0], Duration::from_secs(10)) {
        Ok(stream) => {
            // Set read/write timeouts
            if let Err(err) = stream.set_read_timeout(Some(Duration::from_secs(30))) {
                return Err(RequestError::ConnectionError(format!(
                    "Failed to set read timeout: {}",
                    err
                )));
            }
            if let Err(err) = stream.set_write_timeout(Some(Duration::from_secs(10))) {
                return Err(RequestError::ConnectionError(format!(
                    "Failed to set write timeout: {}",
                    err
                )));
            }
            stream
        }
        Err(err) => {
            return Err(RequestError::ConnectionError(format!(
                "Connection error: {} ({}:{})",
                err, host, port
            )));
        }
    };

    // Handle TLS if needed
    if is_https {
        handle_https_request(&mut stream, &request_bytes, args, &host)
    } else {
        handle_http_request(&mut stream, &request_bytes, args)
    }
}

/// Handle an HTTPS request.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TcpStream.
/// * `request_bytes` - A slice of bytes representing the request.
/// * `args` - A reference to an `Args` struct containing the request parameters.
/// * `host` - The hostname to use for TLS verification.
///
/// # Returns
///
/// * `Result<Vec<u8>, RequestError>` - A vector of bytes representing the response, or an error if something went wrong.
fn handle_https_request(
    stream: &mut TcpStream,
    request_bytes: &[u8],
    args: &Args,
    host: &str,
) -> Result<Vec<u8>, RequestError> {
    let mut builder = TlsConnector::builder();

    // Configure TLS
    builder
        .danger_accept_invalid_certs(false)
        .danger_accept_invalid_hostnames(false)
        .min_protocol_version(Some(native_tls::Protocol::Tlsv12));

    // Add ALPN protocol for HTTP/2 if requested
    if args.http2 {
        // Note: native-tls doesn't directly support ALPN configuration
        // In a real implementation, we'd need to use a library like rustls
        // For now, we'll just warn the user
        if args.verbose {
            println!("Warning: HTTP/2 over TLS with ALPN is not fully supported with native-tls");
            println!("The server might not negotiate HTTP/2");
        }
    }

    let connector = match builder.build() {
        Ok(connector) => connector,
        Err(err) => {
            return Err(RequestError::TlsError(format!("TLS error: {}", err)));
        }
    };

    if args.verbose {
        println!("Connecting to {} (HTTPS)...", host);
    }

    let mut tls_stream = match connector.connect(host, stream.try_clone().unwrap()) {
        Ok(stream) => stream,
        Err(err) => {
            return Err(RequestError::TlsError(format!(
                "TLS handshake error: {}",
                err
            )));
        }
    };

    if args.verbose {
        println!("Sending request...");
        println!("Waiting for response...");
    }

    // Use the TLS stream for communication
    if let Err(err) = tls_stream.write_all(request_bytes) {
        return Err(RequestError::WriteError(format!("Write error: {}", err)));
    }

    // Read response with a maximum size to prevent excessive memory usage
    read_response(&mut tls_stream, args)
}

/// Handle an HTTP request.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TcpStream.
/// * `request_bytes` - A slice of bytes representing the request.
/// * `args` - A reference to an `Args` struct containing the request parameters.
///
/// # Returns
///
/// * `Result<Vec<u8>, RequestError>` - A vector of bytes representing the response, or an error if something went wrong.
fn handle_http_request(
    stream: &mut TcpStream,
    request_bytes: &[u8],
    args: &Args,
) -> Result<Vec<u8>, RequestError> {
    if args.verbose {
        println!("Connecting to HTTP...");
    }

    if let Err(err) = stream.write_all(request_bytes) {
        return Err(RequestError::WriteError(format!("Write error: {}", err)));
    }

    if args.verbose {
        println!("Sending request...");
        println!("Waiting for response...");
    }

    // Read response
    read_response(stream, args)
}

/// Read an HTTP response.
///
/// # Arguments
///
/// * `stream` - A mutable reference to something that implements Read.
/// * `args` - A reference to an `Args` struct containing the request parameters.
///
/// # Returns
///
/// * `Result<Vec<u8>, RequestError>` - A vector of bytes representing the response, or an error if something went wrong.
fn read_response(stream: &mut impl Read, args: &Args) -> Result<Vec<u8>, RequestError> {
    // Read response with a maximum size to prevent excessive memory usage
    let mut response = Vec::with_capacity(1024 * 1024); // Start with 1MB capacity
    let mut buffer = [0u8; 8192]; // 8KB buffer for faster reading
    let mut total_read = 0;
    const MAX_SIZE: usize = 10 * 1024 * 1024; // 10 MB max response
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 50; // Limit attempts to avoid infinite loops

    // Read initial response headers
    while attempts < MAX_ATTEMPTS {
        match stream.read(&mut buffer) {
            Ok(0) => {
                if attempts > 0 || !response.is_empty() {
                    // End of stream after reading some data
                    break;
                }
                if args.verbose {
                    println!("No data received, retrying...");
                }
                thread::sleep(Duration::from_millis(100));
                attempts += 1;
                continue;
            }
            Ok(n) => {
                attempts = 0; // Reset attempts counter on successful read
                total_read += n;
                response.extend_from_slice(&buffer[..n]);

                // Try to find the end of headers to parse Content-Length
                if let Some(header_end) =
                    response.windows(4).position(|window| window == b"\r\n\r\n")
                {
                    let content_length_opt = {
                        // Use a block so the slice borrow ends
                        let headers = &response[..header_end + 4];
                        // Convert to string for easier parsing
                        if let Ok(headers_str) =
                            std::str::from_utf8(&headers[..std::cmp::min(headers.len(), 2048)])
                        {
                            // Find Content-Length header
                            headers_str
                                .lines()
                                .find(|line| line.to_lowercase().starts_with("content-length:"))
                                .and_then(|line| {
                                    line.split(':')
                                        .nth(1)
                                        .and_then(|val| val.trim().parse::<usize>().ok())
                                })
                        } else {
                            None
                        }
                    };

                    // Chunked transfer check
                    let is_chunked = {
                        let headers = &response[..header_end + 4];
                        if let Ok(headers_str) =
                            std::str::from_utf8(&headers[..std::cmp::min(headers.len(), 2048)])
                        {
                            headers_str.lines().any(|line| {
                                line.to_lowercase().starts_with("transfer-encoding:")
                                    && line.to_lowercase().contains("chunked")
                            })
                        } else {
                            false
                        }
                    };

                    // If Content-Length is present, use it to determine when to stop
                    if let Some(length) = content_length_opt {
                        if args.verbose {
                            println!("Response Content-Length: {} bytes", length);
                        }

                        // Calculate the total expected size
                        let expected_size = header_end + 4 + length;

                        // If we've read at least that much, we're done
                        if response.len() >= expected_size {
                            if args.verbose {
                                println!("Response complete based on Content-Length");
                            }
                            break;
                        }
                    } else if is_chunked {
                        // For chunked responses, look for the ending pattern 0\r\n\r\n
                        if response.windows(5).any(|window| window == b"0\r\n\r\n") {
                            if args.verbose {
                                println!("Chunked response complete");
                            }
                            break;
                        }
                    }
                    // If no content-length and not chunked, rely on connection close
                }

                if total_read > MAX_SIZE {
                    if args.verbose {
                        println!("Response too large, truncating at {} bytes", MAX_SIZE);
                    }
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                // On macOS, non-blocking read can return EAGAIN (Resource temporarily unavailable)
                if !response.is_empty() {
                    // We have some data already, check if we might be done
                    attempts += 1;
                    if attempts >= 5 {
                        if args.verbose {
                            println!(
                                "No more data after {} attempts, considering response complete",
                                attempts
                            );
                        }
                        break;
                    }
                }
                // Just retry after a short sleep
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(err) => {
                if !response.is_empty() {
                    if args.verbose {
                        println!(
                            "Read error, but processing partial response of {} bytes: {}",
                            response.len(),
                            err
                        );
                    }
                    break;
                }
                return Err(RequestError::ReadError(format!("Read error: {}", err)));
            }
        }
    }

    if attempts >= MAX_ATTEMPTS && response.is_empty() {
        return Err(RequestError::NoResponseError);
    }

    if args.verbose {
        println!("Received {} bytes", response.len());
    }

    Ok(response)
}
