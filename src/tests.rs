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
        let response = if request.contains("POST") {
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"success\"}"
        } else if request.contains("Authorization: Bearer token") {
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"authenticated\":true}"
        } else {
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!"
        };

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