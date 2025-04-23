#[cfg(test)]
mod tests;

use native_tls::TlsConnector;
use std::{
    env,
    fs::File,
    io::{ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    process, thread,
    time::Duration,
};

struct Args {
    url: String,
    output: Option<String>,
    method: String,
    headers: Vec<String>,
    data: Option<String>,
    help: bool,
    verbose: bool,
}

impl Args {
    /// Parse command line arguments.
    ///
    /// This function parses command line arguments and returns an `Args` struct.
    ///
    /// # Returns
    ///
    /// * `Result<Self, &'static str>` - An `Args` struct if successful, or an error message if unsuccessful.
    fn parse() -> Result<Self, &'static str> {
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            url: String::new(),
            output: None,
            method: "GET".to_string(),
            headers: Vec::new(),
            data: None,
            help: false,
            verbose: false,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    parsed.help = true;
                    return Ok(parsed);
                }
                "-v" | "--verbose" => {
                    parsed.verbose = true;
                }
                "-o" | "--output" => {
                    parsed.output = Some(args.next().ok_or("Missing output file")?);
                }
                "-m" | "--method" => {
                    parsed.method = args.next().ok_or("Missing HTTP method")?.to_uppercase();
                }
                "-H" | "--header" => {
                    parsed.headers.push(args.next().ok_or("Missing header")?);
                }
                "-d" | "--data" => {
                    parsed.data = Some(args.next().ok_or("Missing data")?);
                }
                _ if arg.starts_with('-') => {
                    return Err("Unknown option");
                }
                _ => {
                    parsed.url = arg;
                }
            }
        }

        if parsed.url.is_empty() && !parsed.help {
            return Err("Missing URL");
        }

        Ok(parsed)
    }
}

/// Parse a URL into its components.
///
/// This function takes a URL string and parses it into its components: host, port, path, and protocol.
///
/// # Arguments
///
/// * `url` - A string slice representing the URL to parse.
///
/// # Returns
///
/// * `Result<(String, u16, String, bool), &'static str>` - A tuple containing the host, port, path, and protocol if successful, or an error message if unsuccessful.
fn parse_url(url: &str) -> Result<(String, u16, String, bool), &'static str> {
    let (protocol, rest) = if url.starts_with("https://") {
        (true, url.trim_start_matches("https://"))
    } else if url.starts_with("http://") {
        (false, url.trim_start_matches("http://"))
    } else {
        return Err("URL must start with http:// or https://");
    };

    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = if let Some((host, port)) = host.split_once(':') {
        (host, port.parse().map_err(|_| "Invalid port")?)
    } else {
        (host, if protocol { 443 } else { 80 })
    };

    if host.is_empty() {
        return Err("Invalid host");
    }

    Ok((host.to_string(), port, format!("/{}", path), protocol))
}

/// Build an HTTP request from the given arguments.
///
/// This function takes an `Args` struct and builds an HTTP request string.
///
/// # Arguments
///
/// * `args` - A reference to an `Args` struct containing the request parameters.
///
/// # Returns
///
/// * `Result<Vec<u8>, &'static str>` - A vector of bytes representing the HTTP request if successful, or an error message if unsuccessful.
fn build_http_request(args: &Args) -> Result<Vec<u8>, &'static str> {
    let (host, _port, path, _) = parse_url(&args.url)?;

    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        args.method, path, host
    );

    // Add headers
    for header in &args.headers {
        request.push_str(&format!("{}\r\n", header));
    }

    // Add content length if there's a body
    if let Some(data) = &args.data {
        request.push_str(&format!("Content-Length: {}\r\n", data.len()));
    }

    // End headers
    request.push_str("\r\n");

    // Add body if present
    let mut request_bytes = request.into_bytes();
    if let Some(data) = &args.data {
        request_bytes.extend_from_slice(data.as_bytes());
    }

    Ok(request_bytes)
}

/// Extract the Content-Length header value from an HTTP response.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP response.
///
/// # Returns
///
/// * `Option<usize>` - The Content-Length value if found, otherwise None.
fn get_content_length(response: &[u8]) -> Option<usize> {
    // Convert to string for easier parsing
    let headers = std::str::from_utf8(&response[..std::cmp::min(response.len(), 2048)]).ok()?;

    for line in headers.lines() {
        let line = line.trim().to_lowercase();
        if line.starts_with("content-length:") {
            // Extract the value part
            let value = line.split(':').nth(1)?.trim().parse::<usize>().ok()?;
            return Some(value);
        }
    }

    None
}

/// Check if the response is using chunked transfer encoding.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP response.
///
/// # Returns
///
/// * `bool` - Whether the response is using chunked transfer encoding.
fn is_chunked_transfer(response: &[u8]) -> bool {
    // Convert to string for easier parsing
    if let Ok(headers) = std::str::from_utf8(&response[..std::cmp::min(response.len(), 2048)]) {
        for line in headers.lines() {
            let line = line.trim().to_lowercase();
            if line.starts_with("transfer-encoding:") && line.contains("chunked") {
                return true;
            }
        }
    }

    false
}

/// Parse the status line of an HTTP response.
///
/// This function takes a slice of bytes representing an HTTP response and parses the status line.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP response.
///
/// # Returns
///
/// * `Result<u16, &'static str>` - The status code of the response if successful, or an error message if unsuccessful.
fn parse_status_line(response: &[u8]) -> Result<u16, &'static str> {
    let status_line = match response.split(|&b| b == b'\r').next() {
        Some(line) => line,
        None => return Err("Invalid response format"),
    };

    let status_line = match std::str::from_utf8(status_line) {
        Ok(line) => line,
        Err(_) => return Err("Invalid UTF-8 in status line"),
    };

    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or("Missing status code")?
        .parse::<u16>()
        .map_err(|_| "Invalid status code")?;

    Ok(status_code)
}

/// Decode a chunked transfer encoded response body
///
/// # Arguments
///
/// * `body` - A slice of bytes containing the chunked response body
///
/// # Returns
///
/// * `Vec<u8>` - The decoded response body
fn decode_chunked_transfer(body: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < body.len() {
        // Find the end of the chunk size line
        let chunk_size_end = match &body[i..].windows(2).position(|w| w == b"\r\n") {
            Some(pos) => i + pos,
            None => break, // Malformed chunked encoding
        };

        if chunk_size_end == i {
            break; // No more chunks
        }

        // Parse the chunk size from hex
        let chunk_size_line = std::str::from_utf8(&body[i..chunk_size_end]).unwrap_or("");
        let chunk_size = match usize::from_str_radix(chunk_size_line.trim(), 16) {
            Ok(size) => size,
            Err(_) => break, // Invalid hex
        };

        // Check if this is the last chunk (zero size)
        if chunk_size == 0 {
            break;
        }

        // Skip the CRLF after the chunk size
        let chunk_start = chunk_size_end + 2;

        // Ensure we don't go beyond the buffer
        if chunk_start + chunk_size > body.len() {
            break;
        }

        // Append the chunk data to the result
        result.extend_from_slice(&body[chunk_start..chunk_start + chunk_size]);

        // Move index to the next chunk, skipping the CRLF after the chunk data
        i = chunk_start + chunk_size + 2;
    }

    result
}

/// A simple HTTP client that can send requests and receive responses.
///
/// This program supports:
/// - HTTP and HTTPS requests
/// - Custom headers
/// - Request body data
/// - Various HTTP methods (GET, POST, etc.)
///
/// Usage:
///     rurl [OPTIONS] <URL>
///
/// Options:
///     -o, --output <FILE>     Save the response body to a file
///     -m, --method <METHOD>   HTTP method to use (default: GET)
///     -H, --header <HEADER>   Add a header to the request
///     -d, --data <DATA>       Add data to the request body
///
/// Examples:
///     rurl https://example.com
///     rurl -m POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com
///     rurl -o response.html https://example.com
fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!("Usage: rurl [OPTIONS] <URL>");
            eprintln!("Try 'rurl --help' for more information.");
            process::exit(1);
        }
    };

    // Display help if requested
    if args.help {
        println!("rurl - A minimal HTTP client");
        println!();
        println!("Usage:");
        println!("    rurl [OPTIONS] <URL>");
        println!();
        println!("Options:");
        println!("    -o, --output <FILE>     Save the response body to a file");
        println!("    -m, --method <METHOD>   HTTP method to use (default: GET)");
        println!("    -H, --header <HEADER>   Add a header to the request");
        println!("    -d, --data <DATA>       Add data to the request body");
        println!("    -v, --verbose           Enable verbose output");
        println!("    -h, --help              Display this help message");
        println!();
        println!("Examples:");
        println!("    rurl https://example.com");
        println!("    rurl -m POST -H \"Content-Type: application/json\" -d '{{\"key\":\"value\"}}' https://api.example.com");
        println!("    rurl -o response.html https://example.com");
        process::exit(0);
    }

    let request_bytes = match build_http_request(&args) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    let (host, port, _, is_https) = match parse_url(&args.url) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    // Connect and send request with timeout
    let addr = format!("{}:{}", host, port);
    let addrs = match addr.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(err) => {
            eprintln!("DNS resolution error: {}", err);
            process::exit(1);
        }
    };

    let mut stream =
        match TcpStream::connect_timeout(&addrs.collect::<Vec<_>>()[0], Duration::from_secs(10)) {
            Ok(stream) => {
                // Set read/write timeouts
                if let Err(err) = stream.set_read_timeout(Some(Duration::from_secs(30))) {
                    eprintln!("Failed to set read timeout: {}", err);
                    process::exit(1);
                }
                if let Err(err) = stream.set_write_timeout(Some(Duration::from_secs(10))) {
                    eprintln!("Failed to set write timeout: {}", err);
                    process::exit(1);
                }
                stream
            }
            Err(err) => {
                eprintln!("Connection error: {} ({}:{})", err, host, port);
                process::exit(1);
            }
        };

    // Handle TLS if needed
    if is_https {
        let connector = match TlsConnector::builder()
            .danger_accept_invalid_certs(false)
            .danger_accept_invalid_hostnames(false)
            .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
            .build()
        {
            Ok(connector) => connector,
            Err(err) => {
                eprintln!("TLS error: {}", err);
                process::exit(1);
            }
        };

        if args.verbose {
            println!("Connecting to {} (HTTPS)...", host);
        }

        let mut tls_stream = match connector.connect(&host, stream) {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("TLS handshake error: {}", err);
                process::exit(1);
            }
        };

        if args.verbose {
            println!("Sending request...");
            println!("Waiting for response...");
        }

        // Use the TLS stream for communication
        if let Err(err) = tls_stream.write_all(&request_bytes) {
            eprintln!("Write error: {}", err);
            process::exit(1);
        }

        // Read response with a maximum size to prevent excessive memory usage
        let mut response = Vec::with_capacity(1024 * 1024); // Start with 1MB capacity
        let mut buffer = [0u8; 8192]; // 8KB buffer for faster reading
        let mut total_read = 0;
        const MAX_SIZE: usize = 10 * 1024 * 1024; // 10 MB max response
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 50; // Limit attempts to avoid infinite loops

        // Read initial response headers
        while attempts < MAX_ATTEMPTS {
            match tls_stream.read(&mut buffer) {
                Ok(0) => {
                    if attempts > 0 {
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
                        let content_length = get_content_length(&response[..header_end + 4]);

                        // If Content-Length is present, use it to determine when to stop
                        if let Some(length) = content_length {
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
                        } else if is_chunked_transfer(&response[..header_end + 4]) {
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
                        eprintln!("Response too large, truncating at {} bytes", MAX_SIZE);
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
                    eprintln!("Read error: {}", err);
                    if !response.is_empty() {
                        if args.verbose {
                            println!("Processing partial response of {} bytes", response.len());
                        }
                        break;
                    }
                    process::exit(1);
                }
            }
        }

        if attempts >= MAX_ATTEMPTS && response.is_empty() {
            eprintln!("No response received after maximum attempts");
            process::exit(1);
        }

        if args.verbose {
            println!("Received {} bytes", response.len());
        }
        // Process response
        process_response(&response, &args);
    } else {
        // HTTP connection handling (existing code)
        if args.verbose {
            println!("Connecting to {} (HTTP)...", host);
        }

        if let Err(err) = stream.write_all(&request_bytes) {
            eprintln!("Write error: {}", err);
            process::exit(1);
        }

        if args.verbose {
            println!("Sending request...");
            println!("Waiting for response...");
        }

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
                        let content_length = get_content_length(&response[..header_end + 4]);

                        // If Content-Length is present, use it to determine when to stop
                        if let Some(length) = content_length {
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
                        } else if is_chunked_transfer(&response[..header_end + 4]) {
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
                        eprintln!("Response too large, truncating at {} bytes", MAX_SIZE);
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
                    eprintln!("Read error: {}", err);
                    if !response.is_empty() {
                        if args.verbose {
                            println!("Processing partial response of {} bytes", response.len());
                        }
                        break;
                    }
                    process::exit(1);
                }
            }
        }

        if attempts >= MAX_ATTEMPTS && response.is_empty() {
            eprintln!("No response received after maximum attempts");
            process::exit(1);
        }

        if args.verbose {
            println!("Received {} bytes", response.len());
        }

        // Process response
        process_response(&response, &args);
    }
}

/// Process an HTTP response.
///
/// This function takes a slice of bytes representing an HTTP response and processes it.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP response.
/// * `args` - A reference to an `Args` struct containing the request parameters.
///
/// # Returns
///
/// * `()` - This function does not return a value.
fn process_response(response: &[u8], args: &Args) {
    // Find the end of headers
    let header_end = match response.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(pos) => pos + 4,
        None => {
            eprintln!("Invalid HTTP response");
            process::exit(1);
        }
    };

    // Check status code
    let status = match parse_status_line(response) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("Error parsing status: {}", err);
            process::exit(1);
        }
    };

    // Print status line and essential headers
    if args.verbose {
        if let Ok(headers) = std::str::from_utf8(&response[..header_end]) {
            let status_line = headers.lines().next().unwrap_or("Unknown status");
            println!("Status: {}", status_line);

            // Print some important headers
            let mut content_type = None;
            let mut content_length = None;
            let mut transfer_encoding = None;

            for line in headers.lines().skip(1) {
                let lower_line = line.to_lowercase();
                if lower_line.starts_with("content-type:") {
                    content_type = Some(line);
                } else if lower_line.starts_with("content-length:") {
                    content_length = Some(line);
                } else if lower_line.starts_with("transfer-encoding:") {
                    transfer_encoding = Some(line);
                }
            }

            if let Some(ct) = content_type {
                println!("{}", ct);
            }
            if let Some(cl) = content_length {
                println!("{}", cl);
            }
            if let Some(te) = transfer_encoding {
                println!("{}", te);
            }
            println!();
        }
    }

    // Check for error status
    if status >= 400 {
        eprintln!("HTTP Error: {}", status);
        if let Ok(body) = std::str::from_utf8(&response[header_end..]) {
            eprintln!("Response body: {}", body);
        }
        process::exit(1);
    }

    // Handle chunked transfer encoding
    let body = if is_chunked_transfer(&response[..header_end]) {
        decode_chunked_transfer(&response[header_end..])
    } else {
        response[header_end..].to_vec()
    };

    // Handle response body
    if let Some(output_path) = &args.output {
        // Write to file
        match File::create(output_path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(&body) {
                    eprintln!("Write error: {}", err);
                    process::exit(1);
                }
                println!("Response body saved to '{}'", output_path);
            }
            Err(err) => {
                eprintln!("File error: {}", err);
                process::exit(1);
            }
        }
    } else {
        // Print to stdout
        let body_str = String::from_utf8_lossy(&body);
        println!("{}", body_str);
    }
}
