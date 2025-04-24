use native_tls::TlsConnector;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

use crate::args::Args;
use crate::http::response;

/// Set up TCP stream with appropriate timeouts
pub fn setup_tcp_stream(host: &str, port: u16) -> Result<TcpStream, String> {
    let addr = format!("{}:{}", host, port);
    let addrs = match addr.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(err) => {
            return Err(format!("DNS resolution error: {}", err));
        }
    };

    let stream =
        match TcpStream::connect_timeout(&addrs.collect::<Vec<_>>()[0], Duration::from_secs(10)) {
            Ok(stream) => {
                // Set read/write timeouts
                if let Err(err) = stream.set_read_timeout(Some(Duration::from_secs(30))) {
                    return Err(format!("Failed to set read timeout: {}", err));
                }
                if let Err(err) = stream.set_write_timeout(Some(Duration::from_secs(10))) {
                    return Err(format!("Failed to set write timeout: {}", err));
                }
                stream
            }
            Err(err) => {
                return Err(format!("Connection error: {} ({}:{})", err, host, port));
            }
        };

    Ok(stream)
}

/// Read HTTP response from any type of stream that implements Read
pub fn read_http_response<T: Read>(stream: &mut T, verbose: bool) -> Result<Vec<u8>, String> {
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
                if attempts > 0 {
                    // End of stream after reading some data
                    break;
                }
                if verbose {
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
                    let content_length = response::get_content_length(&response[..header_end + 4]);

                    // If Content-Length is present, use it to determine when to stop
                    if let Some(length) = content_length {
                        if verbose {
                            println!("Response Content-Length: {} bytes", length);
                        }

                        // Calculate the total expected size
                        let expected_size = header_end + 4 + length;

                        // If we've read at least that much, we're done
                        if response.len() >= expected_size {
                            if verbose {
                                println!("Response complete based on Content-Length");
                            }
                            break;
                        }
                    } else if response::is_chunked_transfer(&response[..header_end + 4]) {
                        // For chunked responses, look for the ending pattern 0\r\n\r\n
                        if response.windows(5).any(|window| window == b"0\r\n\r\n") {
                            if verbose {
                                println!("Chunked response complete");
                            }
                            break;
                        }
                    }
                    // If no content-length and not chunked, rely on connection close
                }

                if total_read > MAX_SIZE {
                    return Err(format!(
                        "Response too large, truncating at {} bytes",
                        MAX_SIZE
                    ));
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                // On macOS, non-blocking read can return EAGAIN (Resource temporarily unavailable)
                if !response.is_empty() {
                    // We have some data already, check if we might be done
                    attempts += 1;
                    if attempts >= 5 {
                        if verbose {
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
                    if verbose {
                        println!("Processing partial response of {} bytes", response.len());
                    }
                    break;
                }
                return Err(format!("Read error: {}", err));
            }
        }
    }

    if attempts >= MAX_ATTEMPTS && response.is_empty() {
        return Err("No response received after maximum attempts".to_string());
    }

    if verbose {
        println!("Received {} bytes", response.len());
    }

    Ok(response)
}

/// Handle HTTPS connections
pub fn handle_https_connection(
    stream: TcpStream,
    host: &str,
    request_bytes: &[u8],
    args: &Args,
) -> Result<(), String> {
    let connector = match TlsConnector::builder()
        .danger_accept_invalid_certs(false)
        .danger_accept_invalid_hostnames(false)
        .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
        .build()
    {
        Ok(connector) => connector,
        Err(err) => {
            return Err(format!("TLS error: {}", err));
        }
    };

    if args.verbose {
        println!("Connecting to {} (HTTPS)...", host);
    }

    let mut tls_stream = match connector.connect(host, stream) {
        Ok(stream) => stream,
        Err(err) => {
            return Err(format!("TLS handshake error: {}", err));
        }
    };

    if args.verbose {
        println!("Sending request...");
        println!("Waiting for response...");
    }

    // Use the TLS stream for communication
    if let Err(err) = tls_stream.write_all(request_bytes) {
        return Err(format!("Write error: {}", err));
    }

    // Read response
    match read_http_response(&mut tls_stream, args.verbose) {
        Ok(response_bytes) => {
            // Process response
            response::process(&response_bytes, args);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

/// Handle HTTP connections
pub fn handle_http_connection(
    mut stream: TcpStream,
    host: &str,
    request_bytes: &[u8],
    args: &Args,
) -> Result<(), String> {
    if args.verbose {
        println!("Connecting to {} (HTTP)...", host);
    }

    if let Err(err) = stream.write_all(request_bytes) {
        return Err(format!("Write error: {}", err));
    }

    if args.verbose {
        println!("Sending request...");
        println!("Waiting for response...");
    }

    // Read response
    match read_http_response(&mut stream, args.verbose) {
        Ok(response_bytes) => {
            // Process response
            response::process(&response_bytes, args);
            Ok(())
        }
        Err(err) => Err(err),
    }
}
