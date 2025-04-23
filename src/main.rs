#[cfg(test)]
mod tests;

use rurl::{
    args::Args,
    client::{send_request, RequestError},
    process_response,
};
use std::process;

/// rurl - A minimal HTTP client
fn main() {
    // Parse command line arguments
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
        Args::print_help();
        process::exit(0);
    }

    // Send the request
    let response = match send_request(&args) {
        Ok(response) => response,
        Err(RequestError::ConnectionError(err)) => {
            eprintln!("Connection error: {}", err);
            process::exit(1);
        }
        Err(RequestError::TlsError(err)) => {
            eprintln!("TLS error: {}", err);
            process::exit(1);
        }
        Err(RequestError::WriteError(err)) => {
            eprintln!("Write error: {}", err);
            process::exit(1);
        }
        Err(RequestError::ReadError(err)) => {
            eprintln!("Read error: {}", err);
            process::exit(1);
        }
        Err(RequestError::NoResponseError) => {
            eprintln!("No response received after maximum attempts");
            process::exit(1);
        }
    };

    // Process the response
    process_response(&response, &args);
}
