// src/banner.rs

/// Prints the application startup banner to the console.
pub fn print_banner() {
    // Using a raw string literal for the multi-line banner
    let banner = r#"
                   _                          
                  | |               _         
 _____ _   _ _____| | _   _ _____ _| |_ _____ 
| ___ | | | (____ | || | | (____ (_   _) ___ |
| ____|\ V // ___ | || |_| / ___ | | |_| ____|
|_____) \_/ \_____|\_)____/\_____|  \__)_____)
                                                                                                                                                                                                       

    LLM Evaluation & Testing Framework
"#;
    println!("{}", banner);
}