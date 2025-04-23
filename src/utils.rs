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
pub fn parse_url(url: &str) -> Result<(String, u16, String, bool), &'static str> {
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
