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
pub fn parse(url: &str) -> Result<(String, u16, String, bool), &'static str> {
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
