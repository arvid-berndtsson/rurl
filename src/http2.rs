use crate::args::Args;
use crate::utils::parse_url;

/// HTTP/2 Frame Types
#[repr(u8)]
pub enum Http2FrameType {
    Data = 0x0,
    Headers = 0x1,
    Priority = 0x2,
    RstStream = 0x3,
    Settings = 0x4,
    PushPromise = 0x5,
    Ping = 0x6,
    GoAway = 0x7,
    WindowUpdate = 0x8,
    Continuation = 0x9,
}

/// HTTP/2 Frame Flags
#[repr(u8)]
pub enum Http2FrameFlag {
    EndStream = 0x1,
    EndHeaders = 0x4,
    Padded = 0x8,
    Priority = 0x20,
}

/// Create an HTTP/2 SETTINGS frame
pub fn create_http2_settings_frame() -> Vec<u8> {
    let mut frame = Vec::new();

    // Length (6 bytes for 3 settings)
    frame.extend_from_slice(&[0, 0, 6]);

    // Type (SETTINGS = 0x4)
    frame.push(Http2FrameType::Settings as u8);

    // Flags (none)
    frame.push(0);

    // Stream Identifier (0 for connection-level)
    frame.extend_from_slice(&[0, 0, 0, 0]);

    // Settings Payload
    // SETTINGS_MAX_CONCURRENT_STREAMS (0x3) = 100
    frame.extend_from_slice(&[0, 3, 0, 0, 0, 100]);

    frame
}

/// Create an HTTP/2 HEADERS frame for a request
pub fn create_http2_headers_frame(
    method: &str,
    path: &str,
    host: &str,
    headers: &[String],
    has_body: bool,
) -> Vec<u8> {
    // This is a very simplified implementation that doesn't actually do HPACK encoding
    // In a real implementation, we would need to properly encode the headers using HPACK

    // For now, we'll just create a basic frame structure
    let mut frame = Vec::new();

    // We'll fill in the length later
    frame.extend_from_slice(&[0, 0, 0]);

    // Type (HEADERS = 0x1)
    frame.push(Http2FrameType::Headers as u8);

    // Flags (END_HEADERS = 0x4, maybe END_STREAM = 0x1 if no body)
    let flags = Http2FrameFlag::EndHeaders as u8
        | if !has_body {
            Http2FrameFlag::EndStream as u8
        } else {
            0
        };
    frame.push(flags);

    // Stream Identifier (1 for first stream)
    frame.extend_from_slice(&[0, 0, 0, 1]);

    // Headers payload (simplified - in reality this should be HPACK encoded)
    // We'll just use a placeholder for now
    let payload = format!("{} {} HTTP/2.0\r\nHost: {}\r\n", method, path, host);

    // Add custom headers
    let headers_str = headers.join("\r\n");

    frame.extend_from_slice(payload.as_bytes());
    frame.extend_from_slice(headers_str.as_bytes());
    frame.extend_from_slice(b"\r\n");

    // Now update the length
    let payload_len = frame.len() - 9; // 9 bytes for frame header
    frame[0] = ((payload_len >> 16) & 0xFF) as u8;
    frame[1] = ((payload_len >> 8) & 0xFF) as u8;
    frame[2] = (payload_len & 0xFF) as u8;

    frame
}

/// Create an HTTP/2 DATA frame for the request body
pub fn create_http2_data_frame(data: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();

    // Length
    let len = data.len();
    frame.extend_from_slice(&[(len >> 16) as u8, (len >> 8) as u8, len as u8]);

    // Type (DATA = 0x0)
    frame.push(Http2FrameType::Data as u8);

    // Flags (END_STREAM = 0x1)
    frame.push(Http2FrameFlag::EndStream as u8);

    // Stream Identifier (1 for first stream)
    frame.extend_from_slice(&[0, 0, 0, 1]);

    // Data payload
    frame.extend_from_slice(data);

    frame
}

/// Build an HTTP/2 request
///
/// This function takes an `Args` struct and builds an HTTP/2 request.
///
/// # Arguments
///
/// * `args` - A reference to an `Args` struct containing the request parameters.
///
/// # Returns
///
/// * `Result<Vec<u8>, &'static str>` - A vector of bytes representing the HTTP/2 request if successful, or an error message if unsuccessful.
pub fn build_http2_request(args: &Args) -> Result<Vec<u8>, &'static str> {
    let (host, _port, path, _) = parse_url(&args.url)?;

    // HTTP/2 connection preface
    let mut request = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".to_vec();

    // Add SETTINGS frame
    request.extend_from_slice(&create_http2_settings_frame());

    // Add HEADERS frame
    let has_body = args.data.is_some();
    request.extend_from_slice(&create_http2_headers_frame(
        &args.method,
        &path,
        &host,
        &args.headers,
        has_body,
    ));

    // Add DATA frame if body is present
    if let Some(data) = &args.data {
        request.extend_from_slice(&create_http2_data_frame(data.as_bytes()));
    }

    Ok(request)
}

/// Process an HTTP/2 response
///
/// This function parses an HTTP/2 response and extracts the body data.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP/2 response.
/// * `verbose` - Whether to print verbose output.
///
/// # Returns
///
/// * `Vec<u8>` - The extracted response body.
pub fn parse_http2_response(response: &[u8], verbose: bool) -> Vec<u8> {
    if verbose {
        println!("HTTP/2 response received ({} bytes)", response.len());
        println!("Note: HTTP/2 response parsing is simplified");
    }

    // This is a very simplified HTTP/2 response parser
    // In a real implementation, we would need to properly parse the frame structure

    let mut i = 0;
    let mut body_data = Vec::new();

    // Skip connection preface in the response if present
    if response.len() > 24 && &response[0..24] == b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
        i = 24;
    }

    while i + 9 <= response.len() {
        // 9 is the frame header size
        // Parse frame header
        let length = ((response[i] as usize) << 16)
            | ((response[i + 1] as usize) << 8)
            | (response[i + 2] as usize);
        let frame_type = response[i + 3];
        let flags = response[i + 4];
        let stream_id = ((response[i + 5] as u32 & 0x7F) << 24)
            | ((response[i + 6] as u32) << 16)
            | ((response[i + 7] as u32) << 8)
            | (response[i + 8] as u32);

        if verbose {
            println!(
                "Frame: type={}, length={}, flags={:02x}, stream_id={}",
                frame_type, length, flags, stream_id
            );
        }

        // Make sure we have the full frame
        if i + 9 + length > response.len() {
            if verbose {
                println!("Incomplete frame, stopping parsing");
            }
            break;
        }

        // Process different frame types
        match frame_type {
            0 => {
                // DATA frame
                // Extract DATA payload
                body_data.extend_from_slice(&response[i + 9..i + 9 + length]);
                if verbose {
                    println!("DATA frame: {} bytes", length);
                }
            }
            1 => {
                // HEADERS frame
                if verbose {
                    println!("HEADERS frame: {} bytes", length);
                    // In a real implementation, we would decode the HPACK-encoded headers
                    if let Ok(headers) = std::str::from_utf8(&response[i + 9..i + 9 + length]) {
                        println!("Headers (not properly decoded): {}", headers);
                    }
                }
            }
            // Other frame types could be handled here
            _ => {
                if verbose {
                    println!("Unhandled frame type: {}", frame_type);
                }
            }
        }

        // Move to the next frame
        i += 9 + length;
    }

    body_data
}
