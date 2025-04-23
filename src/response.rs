use std::{fs::File, io::Write, process};

use crate::{
    args::Args,
    http2::parse_http2_response,
    utils::{decode_chunked_transfer, is_chunked_transfer, parse_status_line},
};

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
pub fn process_response(response: &[u8], args: &Args) {
    // Check if this is an HTTP/2 response
    if response.len() > 24 && &response[0..24] == b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
        process_http2_response(response, args);
        return;
    }

    // Otherwise, process as HTTP/1.1 response
    process_http1_response(response, args);
}

/// Process an HTTP/1.1 response.
///
/// This function takes a slice of bytes representing an HTTP/1.1 response and processes it.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP/1.1 response.
/// * `args` - A reference to an `Args` struct containing the request parameters.
fn process_http1_response(response: &[u8], args: &Args) {
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
    handle_response_body(&body, args);
}

/// Process an HTTP/2 response.
///
/// This function takes a slice of bytes representing an HTTP/2 response and processes it.
///
/// # Arguments
///
/// * `response` - A slice of bytes representing an HTTP/2 response.
/// * `args` - A reference to an `Args` struct containing the request parameters.
fn process_http2_response(response: &[u8], args: &Args) {
    // Parse HTTP/2 response and extract body data
    let body_data = parse_http2_response(response, args.verbose);

    // Handle the extracted body data
    handle_response_body(&body_data, args);
}

/// Handle the response body, either saving to a file or printing to stdout.
///
/// # Arguments
///
/// * `body` - A slice of bytes representing the response body.
/// * `args` - A reference to an `Args` struct containing the request parameters.
fn handle_response_body(body: &[u8], args: &Args) {
    if let Some(output_path) = &args.output {
        // Write to file
        match File::create(output_path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(body) {
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
        let body_str = String::from_utf8_lossy(body);
        println!("{}", body_str);
    }
}
