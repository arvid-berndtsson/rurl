use std::env;
use std::path::Path;

/// Represents command line arguments for the HTTP client
pub struct Args {
    pub url: String,
    pub output: Option<String>,
    pub method: String,
    pub headers: Vec<String>,
    pub data: Option<String>,
    pub help: bool,
    pub verbose: bool,
    pub tls_version: Option<String>,
}

impl Args {
    /// Parse command line arguments.
    ///
    /// This function parses command line arguments and returns an `Args` struct.
    ///
    /// # Returns
    ///
    /// * `Result<Self, &'static str>` - An `Args` struct if successful, or an error message if unsuccessful.
    pub fn parse() -> Result<Self, &'static str> {
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            url: String::new(),
            output: None,
            method: "GET".to_string(),
            headers: Vec::new(),
            data: None,
            help: false,
            verbose: false,
            tls_version: None,
        };

        // Check environment variable for TLS version
        if let Ok(tls_version) = env::var("RURL_TLS_VERSION") {
            parsed.tls_version = Some(tls_version);
        }

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    parsed.help = true;
                    return Ok(parsed);
                }
                "-v" | "--verbose" => {
                    parsed.verbose = true;
                }
                "-o" | "--output" => {
                    let output_path = args.next().ok_or("Missing output file")?;
                    validate_output_path(&output_path)?;
                    parsed.output = Some(output_path);
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
                "--tls-version" => {
                    parsed.tls_version = Some(args.next().ok_or("Missing TLS version")?);
                }
                _ if arg.starts_with('-') => {
                    return Err("Unknown option");
                }
                _ => {
                    parsed.url = arg;
                }
            }
        }

        if parsed.url.is_empty() && !parsed.help {
            return Err("Missing URL");
        }

        Ok(parsed)
    }
}

/// Validate output file path to prevent path traversal attacks
///
/// This function checks for potentially dangerous path patterns that could
/// lead to path traversal vulnerabilities.
///
/// # Arguments
///
/// * `path` - The file path to validate
///
/// # Returns
///
/// * `Result<(), &'static str>` - Ok if the path is safe, or an error message if unsafe
fn validate_output_path(path: &str) -> Result<(), &'static str> {
    // Check for null bytes which can be used for path traversal
    if path.contains('\0') {
        return Err("Invalid output path: contains null bytes");
    }

    // Parse the path to normalize it
    let path_obj = Path::new(path);
    
    // Check for absolute paths pointing to sensitive system directories
    if path_obj.is_absolute() {
        // Use to_string_lossy to handle any encoding, then check
        let path_str = path_obj.to_string_lossy();
        let path_lower = path_str.to_lowercase();
        
        // Check for common sensitive system directories (case-insensitive for Windows)
        let sensitive_dirs = [
            "/etc/", "/sys/", "/proc/", "/dev/", "/root/", "/boot/",
            "c:\\windows\\", "c:\\program files\\", "c:\\program files (x86)\\",
            "c:\\windows\\system32\\", "c:\\programdata\\",
        ];
        
        for sensitive_dir in &sensitive_dirs {
            if path_lower.starts_with(sensitive_dir) {
                return Err("Invalid output path: cannot write to system directories");
            }
        }
    }

    // Check for parent directory traversal attempts
    // Use proper component checking to avoid false positives with filenames like "my..file.txt"
    use std::path::Component;
    for component in path_obj.components() {
        if matches!(component, Component::ParentDir) {
            return Err("Invalid output path: suspicious path traversal pattern");
        }
    }

    Ok(())
}

/// Print usage information
pub fn print_help() {
    println!("rurl - A minimal HTTP client");
    println!();
    println!("Usage:");
    println!("    rurl [OPTIONS] <URL>");
    println!();
    println!("Options:");
    println!("    -o, --output <FILE>     Save the response body to a file");
    println!("    -m, --method <METHOD>   HTTP method to use (default: GET)");
    println!("    -H, --header <HEADER>   Add a header to the request");
    println!("    -d, --data <DATA>       Add data to the request body");
    println!("    -v, --verbose           Enable verbose output");
    println!("    -h, --help              Display this help message");
    println!("    --tls-version <VERSION> Set TLS version (1.0, 1.1, 1.2, 1.3)");
    println!();
    println!("Environment Variables:");
    println!("    RURL_TLS_VERSION        Set TLS version (overridden by --tls-version)");
    println!();
    println!("Examples:");
    println!("    rurl https://example.com");
    println!("    rurl -m POST -H \"Content-Type: application/json\" -d '{{\"key\":\"value\"}}' https://api.example.com");
    println!("    rurl -o response.html https://example.com");
    println!("    rurl --tls-version 1.2 https://example.com");
    println!("    RURL_TLS_VERSION=1.3 rurl https://example.com");
}
