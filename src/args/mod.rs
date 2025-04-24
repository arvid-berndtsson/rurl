use std::env;

/// Represents command line arguments for the HTTP client
pub struct Args {
    pub url: String,
    pub output: Option<String>,
    pub method: String,
    pub headers: Vec<String>,
    pub data: Option<String>,
    pub help: bool,
    pub verbose: bool,
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
        };

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
                "-m" | "--method" => {
                    parsed.method = args.next().ok_or("Missing HTTP method")?.to_uppercase();
                }
                "-H" | "--header" => {
                    parsed.headers.push(args.next().ok_or("Missing header")?);
                }
                "-d" | "--data" => {
                    parsed.data = Some(args.next().ok_or("Missing data")?);
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
    println!("    -o, --output <FILE>     Save the response body to a file");
    println!("    -m, --method <METHOD>   HTTP method to use (default: GET)");
    println!("    -H, --header <HEADER>   Add a header to the request");
    println!("    -d, --data <DATA>       Add data to the request body");
    println!("    -v, --verbose           Enable verbose output");
    println!("    -h, --help              Display this help message");
    println!();
    println!("Examples:");
    println!("    rurl https://example.com");
    println!("    rurl -m POST -H \"Content-Type: application/json\" -d '{{\"key\":\"value\"}}' https://api.example.com");
    println!("    rurl -o response.html https://example.com");
}
