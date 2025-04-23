# rurl

A minimal HTTP client with no dependencies, similar to curl but written in Rust. It is designed to be simple, efficient, and easy to use for making HTTP requests from the command line.

## Features

- HTTP and HTTPS support with proper TLS handling
- Custom headers
- Request body data
- Various HTTP methods (GET, POST, etc.)
- Save response to file
- Intelligent response handling for Content-Length and chunked transfers
- Connection timeouts to prevent freezing or hanging
- Verbose mode for debugging
- Minimal memory usage with optimized buffer handling

## Installation

### From crates.io

```bash
cargo install rurl
```

### From source

```bash
git clone https://github.com/arvid-berndtsson/rurl.git
cd rurl
cargo build --release && cargo install --path .
```

## Usage

```
rurl [OPTIONS] <URL>
```

### Options

- `-o, --output <FILE>`: Save the response body to a file
- `-m, --method <METHOD>`: HTTP method to use (default: GET)
- `-H, --header <HEADER>`: Add a header to the request
- `-d, --data <DATA>`: Add data to the request body
- `-v, --verbose`: Enable verbose output with detailed status information
- `-h, --help`: Display help message

### Examples

```bash
# Simple GET request
rurl https://arvid.tech

# Verbose output with connection and response details
rurl -v https://example.com

# POST request with JSON data
rurl -m POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com

# Save response to file
rurl -o response.html https://arvid.tech
```

## Features and Behavior

- Automatically follows the HTTP protocol rules for HTTP/1.1
- Properly handles chunked transfer encoding
- Adds 'Connection: close' to requests to ensure proper connection termination
- Implements timeouts to prevent hanging during network issues
- Limits maximum response size to prevent memory exhaustion
- Provides detailed progress information in verbose mode

## License

[MIT](LICENSE)

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any bugs, features, or improvements.

## Acknowledgments

This project is inspired by the simplicity and power of `curl`, but aims to be written in a more idiomatic Rust style with a focus on minimalism and ease of use.
