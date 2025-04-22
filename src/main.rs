#[cfg(test)]
mod tests;

use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    process,
};

struct Args {
    url: String,
    output: Option<String>,
    method: String,
    headers: Vec<String>,
    data: Option<String>,
}

impl Args {
    fn parse() -> Result<Self, &'static str> {
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            url: String::new(),
            output: None,
            method: "GET".to_string(),
            headers: Vec::new(),
            data: None,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
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

        if parsed.url.is_empty() {
            return Err("Missing URL");
        }

        Ok(parsed)
    }
}

fn parse_url(url: &str) -> Result<(String, u16, String), &'static str> {
    let url = url.trim_start_matches("http://");
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = if let Some((host, port)) = host.split_once(':') {
        (host, port.parse().map_err(|_| "Invalid port")?)
    } else {
        (host, 80)
    };
    Ok((host.to_string(), port, format!("/{}", path)))
}

fn build_http_request(args: &Args) -> Result<Vec<u8>, &'static str> {
    let (host, port, path) = parse_url(&args.url)?;
    
    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}:{}\r\n",
        args.method,
        path,
        host,
        port
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

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    let request_bytes = match build_http_request(&args) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    let (host, port, _) = match parse_url(&args.url) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    // Connect and send request
    let mut stream = match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("Connection error: {}", err);
            process::exit(1);
        }
    };

    if let Err(err) = stream.write_all(&request_bytes) {
        eprintln!("Write error: {}", err);
        process::exit(1);
    }

    // Read response
    let mut response = Vec::new();
    let mut buffer = [0u8; 1024]; // Fixed-size buffer for memory safety
    
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => response.extend_from_slice(&buffer[..n]),
            Err(err) => {
                eprintln!("Read error: {}", err);
                process::exit(1);
            }
        }
    }

    // Find the end of headers
    let header_end = match response.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(pos) => pos + 4,
        None => {
            eprintln!("Invalid HTTP response");
            process::exit(1);
        }
    };

    // Check status code
    match parse_status_line(&response) {
        Ok(status) if status >= 400 => {
            eprintln!("HTTP Error: {}", status);
            process::exit(1);
        }
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error parsing status: {}", err);
            process::exit(1);
        }
    }

    // Handle response
    if let Some(output_path) = args.output {
        // Write to file
        match File::create(output_path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(&response[header_end..]) {
                    eprintln!("Write error: {}", err);
                    process::exit(1);
                }
            }
            Err(err) => {
                eprintln!("File error: {}", err);
                process::exit(1);
            }
        }
    } else {
        // Print to stdout
        match String::from_utf8_lossy(&response[header_end..]) {
            body => println!("{}", body),
        }
    }
} 