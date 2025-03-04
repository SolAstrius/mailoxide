use clap::Parser;
use rayon::prelude::*;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(author, version, about = "Converts single or multiple eml files into an mbox", long_about = None)]
struct Args {
    /// Eml file or folder to be parsed
    #[arg(default_value = "input")]
    input: String,

    /// Destination folder of the final mbox file
    #[arg(default_value_t = String::from("."))]
    output: String,
}

#[derive(Error, Debug)]
enum MailoxideError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("No .eml files found in the provided directory")]
    NoEmlFiles,
}

type Result<T> = std::result::Result<T, MailoxideError>;

/// Check if path is absolute or relative and return the absolute path
fn path_check(path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        path_buf
    } else {
        std::env::current_dir().unwrap().join(path_buf)
    }
}

/// Generate an mbox From line with current time
fn generate_from_line() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Get current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Convert to struct tm (simplified version)
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    
    // Simple calculation for day of week (0-6), month (0-11), day (1-31), etc.
    let days_since_epoch = now / 86400;
    let day_of_week = days[(days_since_epoch % 7) as usize];
    
    let month = months[((now / 2628000) % 12) as usize]; // Approximation
    let day = ((now / 86400) % 31) + 1;
    let hour = (now / 3600) % 24;
    let min = (now / 60) % 60;
    let sec = now % 60;
    
    format!("From MAILER-DAEMON {} {} {} {:02}:{:02}:{:02} {}\n", 
        day_of_week, month, day, hour, min, sec, 1970 + (now / 31536000)
    ).into_bytes()
}

/// Create an mbox, or append to an existing one, from a single eml file
fn create_mbox_from_single_eml(eml_file: &Path, output_path: &Path) -> Result<()> {
    let dest_path = output_path.join("output.mbox");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dest_path)?;
    
    // Use a buffered writer for better performance
    let mut writer = io::BufWriter::with_capacity(64 * 1024, file); // 64KB buffer
    
    let mut eml_content = Vec::new();
    File::open(eml_file)?.read_to_end(&mut eml_content)?;
    
    // Ensure content starts with "From " line for mbox format
    if !starts_with_from_line(&eml_content) {
        // Add a proper From line with timestamp
        writer.write_all(&generate_from_line())?;
    }
    
    writer.write_all(&eml_content)?;
    
    // Ensure the file ends with a newline
    if !eml_content.ends_with(b"\n") {
        writer.write_all(b"\n")?;
    }
    
    // Ensure all data is written
    writer.flush()?;
    
    println!("Successfully converted 1 email to mbox format");
    println!("Output saved to {}", dest_path.display());
    
    Ok(())
}

/// Check if the content starts with a "From " line
fn starts_with_from_line(content: &[u8]) -> bool {
    content.starts_with(b"From ")
}

/// Create an mbox, or append to an existing one, from multiple eml files
fn create_mbox_from_multiple_emls(eml_path: &Path, output_path: &Path) -> Result<()> {
    let dest_path = output_path.join("output.mbox");
    
    // Collect all .eml files
    let eml_files: Vec<_> = fs::read_dir(eml_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(ext) = entry.path().extension() {
                ext == "eml"
            } else {
                false
            }
        })
        .map(|entry| entry.path())
        .collect();
    
    let total_files = eml_files.len();
    
    // Check if any eml files were found
    if total_files == 0 {
        return Err(MailoxideError::NoEmlFiles);
    }
    
    // Create/truncate the mbox file first
    let mbox_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&dest_path)?;
    
    // Use a buffered writer for better performance
    let mut writer = io::BufWriter::with_capacity(1024 * 1024, mbox_file); // 1MB buffer
    
    // Show start message with total files
    if total_files > 1000 {
        println!("Starting conversion of {} email files...", total_files);
    }
    
    // Create a channel for parallel processing with controlled memory usage
    const CHUNK_SIZE: usize = 100; // Process files in batches to control memory usage
    let mut files_processed = 0;
    
    // Process files in chunks to avoid excessive memory usage
    for chunk in eml_files.chunks(CHUNK_SIZE) {
        // Process current chunk in parallel
        let results: Vec<_> = chunk.par_iter()
            .map(|file_path| {
                let mut content = Vec::new();
                
                // Read the file content
                File::open(file_path)
                    .map_err(MailoxideError::from)
                    .and_then(|mut file| {
                        file.read_to_end(&mut content)
                            .map_err(MailoxideError::from)
                    })?;
                
                // Ensure content has proper mbox format
                let mut formatted_content = Vec::new();
                if !starts_with_from_line(&content) {
                    formatted_content.extend_from_slice(&generate_from_line());
                }
                
                formatted_content.extend_from_slice(&content);
                
                // Ensure content ends with newline
                if !formatted_content.ends_with(b"\n") {
                    formatted_content.push(b'\n');
                }
                
                Ok(formatted_content)
            })
            .collect();
        
        // Write current chunk's results
        for result in results {
            match result {
                Ok(content) => {
                    writer.write_all(&content)?;
                }
                Err(e) => return Err(e),
            }
        }
        
        // Update progress
        files_processed += chunk.len();
        if total_files > 1000 && files_processed % 1000 == 0 {
            println!("Progress: {}/{} files processed", files_processed, total_files);
        }
    }
    
    // Ensure all data is written
    writer.flush()?;
    
    println!("Successfully converted {} email(s) to mbox format", total_files);
    println!("Output saved to {}", dest_path.display());
    
    Ok(())
}

fn main() -> Result<()> {
    print_banner();
    
    let args = Args::parse();
    
    // Check input file
    let eml_path = path_check(&args.input);
    
    // Check if output is the default value ("input") and matches the input path
    let output_path = if args.output == "input" && args.input == "input" {
        // Use current directory instead to avoid confusion
        path_check(".")
    } else {
        path_check(&args.output)
    };
    
    // Ensure output directory exists
    if !output_path.exists() {
        fs::create_dir_all(&output_path)?;
    }

    if eml_path.is_dir() {
        // If it's a folder, process multiple files
        create_mbox_from_multiple_emls(&eml_path, &output_path)?;
    } else if eml_path.is_file() {
        // Check if it's an .eml file
        if eml_path.extension().is_some_and(|ext| ext == "eml") {
            create_mbox_from_single_eml(&eml_path, &output_path)?;
        } else {
            return Err(MailoxideError::InvalidPath(
                "Provided file is not an .eml file".to_string(),
            ));
        }
    } else {
        return Err(MailoxideError::InvalidPath(
            "Provide a folder or an eml file".to_string(),
        ));
    }
    
    Ok(())
}

fn print_banner() {
    println!(r#"

  ______   __  __   _          _______    ____      __  __   ____     ____   __   __
 |  ____| |  \/  | | |        |__   __|  / __ \    |  \/  | |  _ \   / __ \  \ \ / /
 | |__    | \  / | | |           | |    | |  | |   | \  / | | |_) | | |  | |  \ V / 
 |  __|   | |\/| | | |           | |    | |  | |   | |\/| | |  _ <  | |  | |   > <  
 | |____  | |  | | | |____       | |    | |__| |   | |  | | | |_) | | |__| |  / . \ 
 |______| |_|  |_| |______|      |_|     \____/    |_|  |_| |____/   \____/  /_/ \_\
                                                                                    
                                                                                    
 Convert your eml files into an mbox folder.
 https://github.com/SolAstrius/mailoxide
    "#);
}