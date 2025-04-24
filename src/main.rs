#[cfg(test)]
mod tests;

mod args;
mod http;

use std::process;

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
    // Parse arguments
    let args = match args::Args::parse() {
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
        args::print_help();
        process::exit(0);
    }

    // Build HTTP request
    let request_bytes = match http::request::build(&args) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    // Parse URL
    let (host, port, _, is_https) = match http::url::parse(&args.url) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    // Setup TCP stream
    let stream = match http::client::setup_tcp_stream(&host, port) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    // Handle HTTP or HTTPS connection
    let result = if is_https {
        http::client::handle_https_connection(stream, &host, &request_bytes, &args)
    } else {
        http::client::handle_http_connection(stream, &host, &request_bytes, &args)
    };

    // Handle any errors
    if let Err(err) = result {
        eprintln!("{}", err);
        process::exit(1);
    }
}
