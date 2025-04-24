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
fn test_tls_version_argument() {
    // This test uses a real HTTPS server
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--",
            "-v",
            "--tls-version",
            "1.2",
            "https://httpbin.org/get",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Using minimum TLS version: 1.2"));
}

#[test]
fn test_tls_version_environment() {
    // This test uses a real HTTPS server
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["run", "--", "-v", "https://httpbin.org/get"]);

    cmd.env("RURL_TLS_VERSION", "1.3");

    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Using minimum TLS version: 1.3"));
}
