use crate::args::Args;
use std::fs::File;
use std::io::Write;

/// Extract the Content-Length header value from an HTTP response.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP response.
///
/// # Returns
///
/// * `Option<usize>` - The Content-Length value if found, otherwise None.
pub fn get_content_length(response: &[u8]) -> Option<usize> {
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
pub fn is_chunked_transfer(response: &[u8]) -> bool {
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
pub fn parse_status_line(response: &[u8]) -> Result<u16, &'static str> {
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
pub fn decode_chunked_transfer(body: &[u8]) -> Vec<u8> {
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
pub fn process(response: &[u8], args: &Args) {
    // Find the end of headers
    let header_end = match response.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(pos) => pos + 4,
        None => {
            eprintln!("Invalid HTTP response");
            std::process::exit(1);
        }
    };

    // Check status code
    let status = match parse_status_line(response) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("Error parsing status: {}", err);
            std::process::exit(1);
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
        std::process::exit(1);
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
                    std::process::exit(1);
                }
                println!("Response body saved to '{}'", output_path);
            }
            Err(err) => {
                eprintln!("File error: {}", err);
                std::process::exit(1);
            }
        }
    } else {
        // Print to stdout
        let body_str = String::from_utf8_lossy(&body);
        println!("{}", body_str);
    }
}
