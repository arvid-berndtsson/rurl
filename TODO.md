# TODO: Project Improvement Tasks

This document outlines tasks and improvements for rurl, a Rust-based minimal HTTP client similar to curl.

## 🚀 High Priority Features

### Core HTTP Functionality
- [ ] Add support for HTTP/2 protocol
- [ ] Implement HTTP/3 (QUIC) support
- [ ] Add support for following redirects (301, 302, 303, 307, 308)
  - [ ] Add `--location` / `-L` flag to follow redirects
  - [ ] Add `--max-redirects` option to limit redirect chains
  - [ ] Track and display redirect chain in verbose mode
- [ ] Implement request retry logic with exponential backoff
  - [ ] Add `--retry` option for number of retries
  - [ ] Add `--retry-delay` for delay between retries
  - [ ] Add `--retry-max-time` for maximum retry duration
- [ ] Add support for Range requests (partial content/resumable downloads)
  - [ ] Implement `--range` / `-r` option
  - [ ] Support resuming interrupted downloads with `--continue-at` / `-C`
- [ ] Implement proper cookie handling
  - [ ] Add `--cookie` / `-b` option to send cookies
  - [ ] Add `--cookie-jar` / `-c` option to save cookies
  - [ ] Parse and store Set-Cookie headers
- [ ] Add proxy support
  - [ ] HTTP proxy (`--proxy` / `-x`)
  - [ ] HTTPS proxy
  - [ ] SOCKS4/SOCKS5 proxy support
  - [ ] Proxy authentication
  - [ ] Respect `http_proxy` and `https_proxy` environment variables
- [ ] Support for compressed responses
  - [ ] Gzip compression (`Accept-Encoding: gzip`)
  - [ ] Deflate compression
  - [ ] Brotli compression
  - [ ] Add `--compressed` flag
- [ ] Add authentication methods
  - [ ] Basic authentication (`--user` / `-u`)
  - [ ] Digest authentication
  - [ ] Bearer token authentication (already supports via headers)
  - [ ] OAuth 2.0 flow support
  - [ ] mTLS (mutual TLS) with client certificates

### Request Customization
- [ ] Add support for form data submission
  - [ ] URL-encoded forms (`--data-urlencode`)
  - [ ] Multipart form data (`--form` / `-F`)
  - [ ] File uploads in form data
- [ ] Implement request body from file
  - [ ] Add `--data-binary` for binary data
  - [ ] Add `--data-ascii` for text data
  - [ ] Support reading from stdin with `-d @-`
- [ ] Add more HTTP methods support
  - [ ] PATCH method
  - [ ] OPTIONS method
  - [ ] TRACE method
  - [ ] CONNECT method
- [ ] Implement custom User-Agent
  - [ ] Add `--user-agent` / `-A` flag
  - [ ] Default User-Agent with version info
- [ ] Add referer header support (`--referer` / `-e`)
- [ ] Support for request rate limiting
  - [ ] `--limit-rate` option to throttle bandwidth

### Response Handling
- [ ] Implement response filters
  - [ ] Headers only mode (`--head` / `-I`)
  - [ ] Show only headers (`--include` / `-i`)
  - [ ] Silent mode (`--silent` / `-s`)
  - [ ] Show transfer progress (`--progress-bar`)
- [ ] Add output formatting options
  - [ ] JSON pretty-printing for JSON responses
  - [ ] XML formatting
  - [ ] HTML rendering/cleaning
- [ ] Response validation
  - [ ] Check response status codes
  - [ ] Add `--fail` / `-f` flag to fail silently on HTTP errors
  - [ ] Add `--fail-with-body` to show body on failures
- [ ] Save response metadata
  - [ ] Save headers to separate file (`--dump-header` / `-D`)
  - [ ] Save timing information
  - [ ] Save certificate information for HTTPS

### Performance & Optimization
- [ ] Implement connection pooling/reuse
  - [ ] Keep-Alive connection support
  - [ ] Connection caching for multiple requests
- [ ] Add parallel download support
  - [ ] Multiple concurrent connections for single file
  - [ ] Parallel downloads of multiple files
- [ ] Optimize memory usage
  - [ ] Streaming large responses to disk
  - [ ] Configurable buffer sizes
  - [ ] Memory-mapped file I/O for large files
- [ ] Add benchmarking mode
  - [ ] Measure request/response timings
  - [ ] Report bandwidth statistics
  - [ ] Connection timing breakdown (DNS, TCP, TLS, etc.)

## 🔒 Security Enhancements

### TLS/SSL Improvements
- [ ] Add certificate verification options
  - [ ] `--insecure` / `-k` flag to skip certificate verification
  - [ ] `--cacert` option for custom CA certificate
  - [ ] `--cert` option for client certificate
  - [ ] `--key` option for client private key
  - [ ] Certificate pinning support
- [ ] Improve TLS configuration
  - [ ] Support for different cipher suites
  - [ ] SNI (Server Name Indication) support
  - [ ] OCSP stapling verification
  - [ ] Session resumption/caching
- [ ] Add support for custom TLS versions per request
- [ ] Implement certificate chain validation logging

### Security Features
- [ ] Add request signing capabilities
  - [ ] HMAC signing
  - [ ] AWS Signature V4
- [ ] Implement secure credential storage
  - [ ] Keychain/keyring integration
  - [ ] Encrypted credential files
- [ ] Add security headers validation
  - [ ] HSTS checking
  - [ ] CSP validation
  - [ ] X-Frame-Options checking
- [ ] Implement DNS over HTTPS (DoH) support

## 📊 Developer Experience

### Error Handling
- [ ] Improve error messages with suggestions
- [ ] Add error codes for programmatic handling
- [ ] Implement detailed error context with stack traces
- [ ] Add `--show-error` flag for detailed error output
- [ ] Better timeout error messages

### Debugging & Diagnostics
- [ ] Enhanced verbose mode
  - [ ] Request/response headers display
  - [ ] Timing information (DNS, connect, TLS, transfer)
  - [ ] Color-coded output
  - [ ] Hex dump mode (`--trace` / `--trace-ascii`)
- [ ] Add `--trace-time` for timestamps in trace
- [ ] Implement `--write-out` for custom output format
  - [ ] Support variables like `%{http_code}`, `%{time_total}`, etc.
- [ ] Add network diagnostic tools
  - [ ] DNS lookup information
  - [ ] Connection path tracing
  - [ ] MTU discovery
- [ ] Implement dry-run mode (`--dry-run`)
  - [ ] Show request that would be sent without sending it

### Configuration
- [ ] Add configuration file support
  - [ ] `.rurlrc` in home directory
  - [ ] Project-specific `.rurl` files
  - [ ] JSON/TOML format support
- [ ] Environment variable support for all options
- [ ] Profile support (dev, staging, production)
- [ ] Config file generation command

### Testing
- [ ] Expand test coverage
  - [ ] Unit tests for all modules
  - [ ] Integration tests for HTTP/HTTPS
  - [ ] Edge case testing (large files, slow connections, etc.)
  - [ ] Property-based testing
- [ ] Add fuzzing tests for parser robustness
- [ ] Performance regression tests
- [ ] Add mock server utilities for testing
- [ ] Continuous integration improvements
  - [ ] Test on multiple platforms (Linux, macOS, Windows)
  - [ ] Test with different Rust versions

## 📚 Documentation

### User Documentation
- [ ] Create comprehensive man page
- [ ] Add usage examples for common scenarios
  - [ ] REST API interaction examples
  - [ ] File upload examples
  - [ ] Authentication examples
  - [ ] Proxy configuration examples
- [ ] Create tutorial/guide documentation
- [ ] Add troubleshooting guide
- [ ] Document differences from curl
- [ ] Create comparison table with curl features

### Developer Documentation
- [ ] Add inline code documentation (rustdoc)
- [ ] Create architecture documentation
- [ ] Add contributing guidelines (CONTRIBUTING.md)
- [ ] Document code style and conventions
- [ ] Add module-level documentation
- [ ] Create API documentation for library usage
- [ ] Add sequence diagrams for complex flows

### Examples
- [ ] Create examples directory with common use cases
- [ ] Add script examples for automation
- [ ] Provide integration examples with popular APIs
  - [ ] GitHub API examples
  - [ ] REST API examples
  - [ ] GraphQL examples

## 🛠️ Code Quality & Maintenance

### Code Structure
- [ ] Refactor into library + CLI binary structure
  - [ ] Expose core functionality as library crate
  - [ ] Separate CLI interface from core logic
- [ ] Implement plugin system for extensibility
- [ ] Add middleware/interceptor support
- [ ] Create modular request/response processors
- [ ] Implement trait-based abstractions for different protocols

### Code Quality
- [ ] Add comprehensive rustdoc comments
- [ ] Implement clippy linting in CI
- [ ] Add rustfmt configuration
- [ ] Set up cargo-deny for dependency checking
- [ ] Implement cargo-audit for security auditing
- [ ] Add code coverage reporting
- [ ] Static analysis with additional tools

### Dependencies
- [ ] Evaluate and minimize dependencies
- [ ] Consider replacing native-tls with rustls for pure Rust implementation
- [ ] Add feature flags for optional dependencies
- [ ] Keep dependencies up to date with dependabot
- [ ] Regular security audits of dependencies

### Build & Distribution
- [ ] Cross-compilation support
- [ ] Create binary releases for major platforms
- [ ] Set up automated release process
- [ ] Add checksums and signatures for releases
- [ ] Docker container image
- [ ] Package for major package managers
  - [ ] Homebrew formula
  - [ ] apt/deb package
  - [ ] RPM package
  - [ ] Chocolatey package (Windows)
  - [ ] Scoop package (Windows)

## 🌟 Advanced Features

### Scripting & Automation
- [ ] Add scripting/automation support
  - [ ] Request chaining
  - [ ] Variable substitution
  - [ ] Environment variable expansion
- [ ] JSON/YAML request file support
- [ ] Batch request processing
- [ ] Request template system
- [ ] Response assertion/validation

### API Testing Features
- [ ] Add JSON path queries for responses
- [ ] XML path queries
- [ ] Response schema validation
- [ ] Test assertion framework
- [ ] Load testing capabilities
- [ ] API mocking/stubbing

### Monitoring & Observability
- [ ] OpenTelemetry integration
- [ ] Prometheus metrics export
- [ ] Structured logging (JSON output)
- [ ] Request/response logging to file
- [ ] Statistics collection and reporting

### Internationalization
- [ ] i18n support for error messages
- [ ] Multi-language documentation
- [ ] Unicode URL support (IDN)
- [ ] Proper charset handling

## 🎯 Performance Features

### Caching
- [ ] Implement HTTP cache (RFC 7234)
  - [ ] Respect Cache-Control headers
  - [ ] ETag support
  - [ ] Last-Modified support
- [ ] DNS cache
- [ ] Connection cache
- [ ] TLS session cache

### Async/Concurrency
- [ ] Consider async/await implementation
  - [ ] Evaluate tokio runtime integration
  - [ ] async-std alternative
- [ ] Thread pool for parallel requests
- [ ] Concurrent connection limits

## 🔄 Compatibility & Interoperability

### Standards Compliance
- [ ] Full HTTP/1.1 compliance
- [ ] WebSocket support
- [ ] Server-Sent Events (SSE) support
- [ ] Content negotiation (Accept headers)

### Integration
- [ ] Shell completion scripts
  - [ ] Bash completion
  - [ ] Zsh completion
  - [ ] Fish completion
  - [ ] PowerShell completion
- [ ] Editor integration
  - [ ] VS Code extension
  - [ ] Vim plugin
- [ ] CI/CD integration examples

## 📱 Platform-Specific Features

### Cross-Platform
- [ ] Windows-specific features
  - [ ] Windows certificate store integration
  - [ ] NTLM authentication
- [ ] macOS-specific features
  - [ ] Keychain integration
  - [ ] macOS system proxy detection
- [ ] Linux-specific features
  - [ ] systemd integration
  - [ ] Linux kernel features optimization

## 🎨 User Interface

### Output Formatting
- [ ] Color output for better readability
  - [ ] Syntax highlighting for JSON/XML/HTML
  - [ ] Color-coded HTTP status codes
  - [ ] ANSI color support
- [ ] Progress indicators
  - [ ] Progress bar for downloads
  - [ ] Spinner for requests
  - [ ] ETA calculations
- [ ] Table output format
- [ ] Tree view for nested data

### Interactive Mode
- [ ] Interactive REPL mode
- [ ] Request history browsing
- [ ] Tab completion for URLs and options
- [ ] Request editing and replay

## 🐛 Bug Fixes & Improvements

### Known Issues
- [ ] Handle edge cases in URL parsing
- [ ] Improve timeout handling across different scenarios
- [ ] Better handling of malformed responses
- [ ] Fix potential memory leaks in long-running scenarios
- [ ] Handle extremely large headers
- [ ] Proper handling of connection interruptions

### Performance Issues
- [ ] Optimize buffer allocation
- [ ] Reduce memory copies
- [ ] Improve string handling efficiency
- [ ] Profile and optimize hot paths

## 📈 Metrics & Analytics

### Usage Metrics
- [ ] Add telemetry (opt-in)
  - [ ] Usage statistics
  - [ ] Feature usage tracking
  - [ ] Error reporting
- [ ] Performance metrics collection
- [ ] Success/failure rate tracking

## 🔮 Future Considerations

### Experimental Features
- [ ] GraphQL native support
- [ ] gRPC support
- [ ] MQTT protocol support
- [ ] WebRTC support
- [ ] IPv6 priority support
- [ ] QUIC protocol experimentation

### Research & Innovation
- [ ] Machine learning for request optimization
- [ ] Predictive connection pooling
- [ ] Smart retry strategies
- [ ] Adaptive compression

---

## Priority Levels

**P0 (Critical)**: Essential for basic functionality
- Redirect following
- Better error messages
- Basic authentication

**P1 (High)**: Important for common use cases
- Cookie handling
- Proxy support
- Compression support
- Form data support

**P2 (Medium)**: Nice to have features
- Advanced TLS options
- Response formatting
- Configuration files

**P3 (Low)**: Future enhancements
- HTTP/2 and HTTP/3
- Advanced features
- Experimental protocols

---

## Contributing

When working on items from this TODO list:

1. Check if the item is already being worked on
2. Create an issue referencing this TODO item
3. Update this file to mark items in progress with your GitHub handle
4. Submit a PR referencing the issue
5. Update this file to mark items as complete with PR number

Format for marking in progress:
- [ ] Task name [WIP - @username - #issue]

Format for completed:
- [x] Task name [Done - #PR]

---

Last Updated: 2025-11-29
Version: 1.0
