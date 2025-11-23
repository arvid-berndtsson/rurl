use crate::args::Args;
use crate::http::url;

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
pub fn build(args: &Args) -> Result<Vec<u8>, &'static str> {
    let (host, _port, path, _) = url::parse(&args.url)?;

    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        args.method, path, host
    );

    // Add User-Agent header if specified
    if let Some(user_agent) = &args.user_agent {
        request.push_str(&format!("User-Agent: {}\r\n", user_agent));
    }

    // Add Basic Authentication if specified
    if let Some(user) = &args.user {
        let encoded = base64_encode(user.as_bytes());
        request.push_str(&format!("Authorization: Basic {}\r\n", encoded));
    }

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

/// Base64 encode a byte slice
fn base64_encode(data: &[u8]) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        result.push(BASE64_CHARS[(b1 >> 2) as usize] as char);
        result.push(BASE64_CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        
        if i + 1 < data.len() {
            result.push(BASE64_CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        
        if i + 2 < data.len() {
            result.push(BASE64_CHARS[(b3 & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}
