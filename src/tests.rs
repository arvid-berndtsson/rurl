use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

// Mock HTTP server for testing
struct MockServer {
    listener: TcpListener,
}

impl MockServer {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        Self { listener }
    }

    fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0u8; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer);

        let (content_type, body) = if request.contains("POST") {
            ("application/json", "{\"status\":\"success\"}")
        } else if request.contains("Authorization: Bearer token") {
            ("application/json", "{\"authenticated\":true}")
        } else if request.contains("chunked") {
            ("text/plain", "Hello, Chunked World!")
        } else {
            ("text/plain", "Hello, World!")
        };

        // Add proper Content-Length and other headers for HTTP/1.1
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            content_type,
            body.len(),
            body
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn run(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        Self::handle_connection(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}

// HTTP/2 Mock Server
struct MockHttp2Server {
    listener: TcpListener,
}

impl MockHttp2Server {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        Self { listener }
    }

    fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0u8; 1024];
        stream.read(&mut buffer).unwrap();

        let request = &buffer[..1024];

        // Check if it's an HTTP/2 request by looking for the connection preface
        let is_http2 = request.starts_with(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");

        if is_http2 {
            // HTTP/2 response with a simple settings frame and headers + data frames
            // This is a very simplified mock of HTTP/2 framing

            // Connection preface
            let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
            stream.write_all(preface).unwrap();

            // SETTINGS frame (empty settings)
            let settings_frame = [
                0x00, 0x00, 0x00, // Length: 0
                0x04, // Type: SETTINGS
                0x00, // Flags: none
                0x00, 0x00, 0x00, 0x00, // Stream ID: 0
            ];
            stream.write_all(&settings_frame).unwrap();

            // HEADERS frame
            let headers_content = b"HTTP/2.0 200 OK\r\nContent-Type: application/json\r\n\r\n";
            let headers_length = headers_content.len() as u32;
            let headers_frame = [
                ((headers_length >> 16) & 0xFF) as u8, // Length (high byte)
                ((headers_length >> 8) & 0xFF) as u8,  // Length (middle byte)
                (headers_length & 0xFF) as u8,         // Length (low byte)
                0x01,                                  // Type: HEADERS
                0x04,                                  // Flags: END_HEADERS
                0x00,
                0x00,
                0x00,
                0x01, // Stream ID: 1
            ];
            stream.write_all(&headers_frame).unwrap();
            stream.write_all(headers_content).unwrap();

            // DATA frame with JSON payload
            let json_body = b"{\"protocol\":\"HTTP/2\",\"message\":\"Hello from HTTP/2\"}";
            let data_length = json_body.len() as u32;
            let data_frame = [
                ((data_length >> 16) & 0xFF) as u8, // Length (high byte)
                ((data_length >> 8) & 0xFF) as u8,  // Length (middle byte)
                (data_length & 0xFF) as u8,         // Length (low byte)
                0x00,                               // Type: DATA
                0x01,                               // Flags: END_STREAM
                0x00,
                0x00,
                0x00,
                0x01, // Stream ID: 1
            ];
            stream.write_all(&data_frame).unwrap();
            stream.write_all(json_body).unwrap();
        } else {
            // Regular HTTP/1.1 response
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 56\r\n\r\n{\"protocol\":\"HTTP/1.1\",\"message\":\"Hello from HTTP/1.1\"}";
            stream.write_all(response.as_bytes()).unwrap();
        }

        stream.flush().unwrap();
    }

    fn run(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        Self::handle_connection(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}

#[test]
fn test_basic_get_request() {
    let server = MockServer::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args(["run", "--", &format!("http://127.0.0.1:{}", port)])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Hello, World!"));
}

#[test]
fn test_verbose_output() {
    let server = MockServer::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args(["run", "--", "-v", &format!("http://127.0.0.1:{}", port)])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("Hello, World!"));
    assert!(stdout.contains("Connecting to"));
    assert!(stdout.contains("Sending request"));
    assert!(stdout.contains("Content-Length"));
    assert!(stdout.contains("Status: HTTP/1.1 200 OK"));
}

#[test]
fn test_save_to_file() {
    let server = MockServer::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output_file = "test_output.txt";
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("http://127.0.0.1:{}", port),
            "-o",
            output_file,
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.len() > 0); // Should see "Response body saved to..." message

    let file_content = std::fs::read_to_string(output_file).unwrap();
    assert!(file_content.contains("Hello, World!"));

    // Cleanup
    std::fs::remove_file(output_file).unwrap();
}

#[test]
fn test_post_request() {
    let server = MockServer::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("http://127.0.0.1:{}", port),
            "-m",
            "POST",
            "-d",
            "{\"key\":\"value\"}",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("success"));
}

#[test]
fn test_custom_headers() {
    let server = MockServer::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("http://127.0.0.1:{}", port),
            "-H",
            "Authorization: Bearer token",
            "-H",
            "Content-Type: application/json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("authenticated"));
}

#[test]
fn test_help_flag() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rurl - A minimal HTTP client"));
    assert!(stdout.contains("-v, --verbose"));
}

#[test]
fn test_invalid_url() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "not-a-valid-url"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    // Check that it reports the error about URL format
    assert!(String::from_utf8_lossy(&output.stderr).contains("URL must start with http://"));
}

#[test]
fn test_malformed_url() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "http://"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    // Check for the specific error
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid host"));
}

#[test]
fn test_invalid_port() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "http://localhost:99999"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid port"));
}

#[test]
fn test_missing_url() {
    let output = std::process::Command::new("cargo")
        .args(["run"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Missing URL"));
}

#[test]
fn test_connection_timeout() {
    // Use a non-routable IP to test connection timeout
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "http://192.168.255.255:12345"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain connection error or timeout
    assert!(
        stderr.contains("Connection") || stderr.contains("timeout") || stderr.contains("timed out"),
        "Expected connection error or timeout, got: {}",
        stderr
    );
}

#[test]
fn test_tls_connection_attempt() {
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    // Use example.com as the test domain
    let mut child = Command::new("cargo")
        .args(["run", "--", "https://example.com"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Give it some time to attempt the connection
    thread::sleep(Duration::from_secs(5));

    // Check if it's completed
    match child.try_wait() {
        Ok(Some(status)) => {
            // If it completed, check if it succeeded
            let output = child.wait_with_output().unwrap();

            if status.success() {
                // TLS worked - check output
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Check for HTML content that would indicate successful response
                assert!(
                    stdout.contains("<html")
                        || stdout.contains("<body")
                        || stdout.contains("<!DOCTYPE"),
                    "Expected HTML response, got: {}",
                    stdout
                );
            } else {
                // TLS connection failed but didn't hang indefinitely
                let stderr = String::from_utf8_lossy(&output.stderr);
                // TLS error should be present in stderr
                assert!(
                    stderr.contains("TLS")
                        || stderr.contains("SSL")
                        || stderr.contains("handshake")
                        || stderr.contains("Connection"),
                    "Expected TLS error, got: {}",
                    stderr
                );
            }
        }
        Ok(None) => {
            // Still running - kill it
            let _ = child.kill();
            let _ = child.wait();
            panic!("Test timed out - request is taking too long to complete");
        }
        Err(e) => panic!("Error waiting for process: {}", e),
    }
}

#[test]
fn test_invalid_tls_hostname() {
    // Testing with valid HTTPS protocol but invalid hostname
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "https://invalid-hostname-that-doesnt-exist.example",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain either a DNS error or TLS error
    assert!(
        stderr.contains("DNS")
            || stderr.contains("TLS")
            || stderr.contains("not found")
            || stderr.contains("unknown")
            || stderr.contains("Connection")
            || stderr.contains("connect"),
        "Expected connection error, got: {}",
        stderr
    );
}

#[test]
fn test_http2_request() {
    let server = MockHttp2Server::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "--http2",
            &format!("http://127.0.0.1:{}", port),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Check that we got an HTTP/2 response
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HTTP/2"));
    assert!(stdout.contains("Hello from HTTP/2"));
}

#[test]
fn test_http2_verbose_output() {
    let server = MockHttp2Server::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "--http2",
            "-v",
            &format!("http://127.0.0.1:{}", port),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Check for HTTP/2 specific verbose output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HTTP/2 response received"));
    assert!(stdout.contains("Frame: type="));
}

#[test]
fn test_http2_post_request() {
    let server = MockHttp2Server::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "--http2",
            "-m",
            "POST",
            "-d",
            "{\"test\":\"data\"}",
            &format!("http://127.0.0.1:{}", port),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Check that we got an HTTP/2 response
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HTTP/2"));
}

#[test]
fn test_http2_with_headers() {
    let server = MockHttp2Server::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "--http2",
            "-H",
            "X-Test-Header: test-value",
            "-H",
            "User-Agent: rurl/1.0",
            &format!("http://127.0.0.1:{}", port),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Check that we got an HTTP/2 response
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HTTP/2"));
}

#[test]
fn test_fallback_to_http1() {
    // This test uses a regular HTTP/1.1 server but makes the request with --http2
    // This simulates a server that doesn't support HTTP/2
    let server = MockServer::new();
    let port = server.port();
    thread::spawn(move || server.run());

    // Give the server time to start
    thread::sleep(Duration::from_millis(100));

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "--http2",
            &format!("http://127.0.0.1:{}", port),
        ])
        .output()
        .unwrap();

    // The request should still succeed, but with HTTP/1.1
    assert!(output.status.success());

    // Check that we got an HTTP/1.1 response
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"));
}
