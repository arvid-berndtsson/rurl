# rurl

A minimal HTTP client with no dependencies, similar to curl but written in Rust. It is designed to be simple, efficient, and easy to use for making HTTP requests from the command line.

> **Note:** The package is published as `rust-curl` on crates.io, but the command you run is still `rurl`

## Features

- HTTP and HTTPS support with proper TLS handling
- Custom headers
- Request body data (inline or from file)
- Various HTTP methods (GET, POST, HEAD, PUT, DELETE, etc.)
- Save response to file
- Include response headers in output
- Follow HTTP redirects automatically
- Basic authentication support
- Custom User-Agent strings
- Silent and verbose modes
- Fail fast on HTTP errors
- Intelligent response handling for Content-Length and chunked transfers
- Connection timeouts to prevent freezing or hanging
- Minimal memory usage with optimized buffer handling

## Installation

### From crates.io

```bash
cargo install rust-curl
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
- `-X, --request <METHOD>`: HTTP method to use (alias for -m)
- `-H, --header <HEADER>`: Add a header to the request
- `-d, --data <DATA>`: Add data to the request body (use @filename to read from file)
- `-i, --include`: Include response headers in output
- `-I, --head`: Fetch headers only (HEAD request)
- `-L, --location`: Follow HTTP redirects automatically
- `-s, --silent`: Silent mode (no progress output)
- `-f, --fail`: Fail silently on HTTP errors (exit code 22)
- `-A, --user-agent <NAME>`: Custom User-Agent string
- `-u, --user <USER:PASS>`: Server authentication credentials (Basic Auth)
- `-v, --verbose`: Enable verbose output with detailed status information
- `-h, --help`: Display help message
- `--tls-version <VERSION>`: Set minimum TLS version (1.0, 1.1, 1.2, 1.3)

### Examples

```bash
# Simple GET request
rurl https://arvid.tech

# Include response headers in output
rurl -i https://example.com

# Fetch only headers (HEAD request)
rurl -I https://example.com

# Follow redirects
rurl -L https://example.com/redirect

# Verbose output with connection and response details
rurl -v https://example.com

# Silent mode (suppress progress output)
rurl -s https://example.com

# POST request with JSON data
rurl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com

# POST data from a file
rurl -d @data.json https://api.example.com

# Custom User-Agent
rurl -A "MyApp/1.0" https://example.com

# Basic authentication
rurl -u username:password https://api.example.com

# Fail silently on HTTP errors
rurl -f https://example.com/might-not-exist

# Save response to file
rurl -o response.html https://arvid.tech

# Combine multiple options
rurl -L -i -A "MyApp/1.0" https://example.com
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
