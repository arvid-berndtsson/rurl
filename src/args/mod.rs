use std::env;

/// Represents command line arguments for the HTTP client
#[derive(Clone)]
pub struct Args {
    pub url: String,
    pub output: Option<String>,
    pub method: String,
    pub headers: Vec<String>,
    pub data: Option<String>,
    pub help: bool,
    pub verbose: bool,
    pub tls_version: Option<String>,
    pub include_headers: bool,
    pub head_only: bool,
    pub follow_redirects: bool,
    pub silent: bool,
    pub user_agent: Option<String>,
    pub user: Option<String>,
    pub fail_fast: bool,
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
            include_headers: false,
            head_only: false,
            follow_redirects: false,
            silent: false,
            user_agent: None,
            user: None,
            fail_fast: false,
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
                    parsed.output = Some(args.next().ok_or("Missing output file")?);
                }
                "-m" | "--method" | "-X" | "--request" => {
                    parsed.method = args.next().ok_or("Missing HTTP method")?.to_uppercase();
                }
                "-H" | "--header" => {
                    parsed.headers.push(args.next().ok_or("Missing header")?);
                }
                "-d" | "--data" => {
                    let data_arg = args.next().ok_or("Missing data")?;
                    // Check if data starts with @ to read from file
                    if let Some(filename) = data_arg.strip_prefix('@') {
                        let file_content = std::fs::read_to_string(filename)
                            .map_err(|_| "Failed to read data file")?;
                        parsed.data = Some(file_content);
                    } else {
                        parsed.data = Some(data_arg);
                    }
                    // If data is provided without explicit method, default to POST
                    if parsed.method == "GET" {
                        parsed.method = "POST".to_string();
                    }
                }
                "--tls-version" => {
                    parsed.tls_version = Some(args.next().ok_or("Missing TLS version")?);
                }
                "-i" | "--include" => {
                    parsed.include_headers = true;
                }
                "-I" | "--head" => {
                    parsed.head_only = true;
                    parsed.method = "HEAD".to_string();
                }
                "-L" | "--location" => {
                    parsed.follow_redirects = true;
                }
                "-s" | "--silent" => {
                    parsed.silent = true;
                }
                "-A" | "--user-agent" => {
                    parsed.user_agent = Some(args.next().ok_or("Missing user agent")?);
                }
                "-u" | "--user" => {
                    parsed.user = Some(args.next().ok_or("Missing user credentials")?);
                }
                "-f" | "--fail" => {
                    parsed.fail_fast = true;
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

/// Print usage information
pub fn print_help() {
    println!("rurl - A minimal HTTP client");
    println!();
    println!("Usage:");
    println!("    rurl [OPTIONS] <URL>");
    println!();
    println!("Options:");
    println!("    -o, --output <FILE>       Save the response body to a file");
    println!("    -m, --method <METHOD>     HTTP method to use (default: GET)");
    println!("    -X, --request <METHOD>    HTTP method to use (alias for -m)");
    println!("    -H, --header <HEADER>     Add a header to the request");
    println!("    -d, --data <DATA>         Add data to the request body");
    println!("                              Use @filename to read from file");
    println!("    -i, --include             Include response headers in output");
    println!("    -I, --head                Fetch headers only (HEAD request)");
    println!("    -L, --location            Follow redirects");
    println!("    -s, --silent              Silent mode (no progress output)");
    println!("    -f, --fail                Fail silently on HTTP errors");
    println!("    -A, --user-agent <NAME>   Custom User-Agent string");
    println!("    -u, --user <USER:PASS>    Server authentication credentials");
    println!("    -v, --verbose             Enable verbose output");
    println!("    -h, --help                Display this help message");
    println!("    --tls-version <VERSION>   Set TLS version (1.0, 1.1, 1.2, 1.3)");
    println!();
    println!("Environment Variables:");
    println!("    RURL_TLS_VERSION          Set TLS version (overridden by --tls-version)");
    println!();
    println!("Examples:");
    println!("    rurl https://example.com");
    println!("    rurl -i https://example.com");
    println!("    rurl -I https://example.com");
    println!("    rurl -L https://example.com/redirect");
    println!("    rurl -A \"Mozilla/5.0\" https://example.com");
    println!("    rurl -u user:pass https://api.example.com");
    println!("    rurl -X POST -H \"Content-Type: application/json\" -d '{{\"key\":\"value\"}}' https://api.example.com");
    println!("    rurl -d @data.json https://api.example.com");
    println!("    rurl -o response.html https://example.com");
    println!("    rurl --tls-version 1.2 https://example.com");
    println!("    RURL_TLS_VERSION=1.3 rurl https://example.com");
}
