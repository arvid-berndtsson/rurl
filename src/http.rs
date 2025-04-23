use crate::args::Args;
use crate::utils::parse_url;

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
pub fn build_http_request(args: &Args) -> Result<Vec<u8>, &'static str> {
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
