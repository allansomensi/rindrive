use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rindrive_core::engine::spotcheck::Report;
use std::io::{self, Write};

/// Constant for converting raw bytes to Gibibytes (2^30).
const BYTES_PER_GIB: f64 = 1_073_741_824.0;

/// Creates and configures the main progress bar for the audit engine.
///
/// It maps the 0.0-1.0 float percentage to a 0-10000 integer range for smoother animation.
pub fn create_audit_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new(10000);

    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% - {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    pb
}

/// Creates an indeterminate spinner for waiting states (e.g., Watcher Mode).
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.yellow} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb
}

/// Prints a formatted informational message (Blue/Cyan theme).
pub fn print_info(header: &str, message: &str) {
    println!("{} {}", header.bold().blue(), message.cyan());
}

/// Prints a formatted success message (Green theme).
pub fn print_success(header: &str, message: &str) {
    println!("{} {}", header.bold().green(), message.white());
}

/// Prints a formatted error message to stderr (Red theme).
pub fn print_error(context: &str, error: &impl ToString) {
    eprintln!("{} {}", context.bold().red(), error.to_string().red());
}

/// Prints a dimmed message for low-priority or skipped events.
pub fn print_dimmed(message: &str) {
    println!("{}", message.dimmed());
}

/// Displays a styled "card" with the drive's mount point and capacity.
pub fn show_drive_details(mount_point: &str, size_bytes: u64) {
    let size_gib = size_bytes as f64 / BYTES_PER_GIB;

    println!();
    println!("   {}", " DRIVE INFORMATION ".on_blue().white().bold());
    println!("   {}", "─".repeat(45).blue().dimmed());
    println!("   {:<12} {}", "📂 Path:".bold(), mount_point.cyan().bold());
    println!(
        "   {:<12} {} {}",
        "💾 Size:".bold(),
        format!("{:.2} GiB", size_gib).yellow().bold(),
        format!("({} bytes)", size_bytes).dimmed().italic()
    );
    println!("   {}", "─".repeat(45).blue().dimmed());
    println!();
}

/// Renders the final audit report, including the integrity map if errors are found.
pub fn render_report(report: &Report) {
    println!("{}", "=".repeat(60).dimmed());
    println!("{:^60}", " AUDIT RESULTS ".on_blue().white().bold());
    println!("{}", "=".repeat(60).dimmed());

    let declared = report.declared_size_bytes as f64 / BYTES_PER_GIB;
    let validated = report.validated_size_bytes as f64 / BYTES_PER_GIB;

    println!(" {}:  {declared:.2} GiB", "📦 Declared Size".bold());
    println!(" {}:  {validated:.2} GiB", "🛡️ Validated Size".bold());

    println!("{}", "-".repeat(60).dimmed());

    if report.has_errors {
        println!(
            "{}",
            "❌ CRITICAL: FAKE OR DEFECTIVE DRIVE DETECTED!"
                .red()
                .bold()
                .blink()
        );
        println!(
            "\n{}",
            "Integrity Map ( . = OK | X = CORRUPTED ):".underline()
        );

        for (i, &ok) in report.integrity_map.iter().enumerate() {
            let symbol = if ok { ".".green() } else { "X".red().bold() };
            print!("{}", symbol);
            if (i + 1) % 64 == 0 {
                println!();
            }
        }
        println!();
    } else {
        println!(
            "{}",
            "✅ SUCCESS: DRIVE INTEGRITY VERIFIED (100% OK)"
                .green()
                .bold()
        );
    }
    println!("{}", "=".repeat(60).dimmed());
}

/// Prompts the user for a Yes/No confirmation via stdin.
pub fn confirm_action(prompt: &str) -> io::Result<bool> {
    print!("{} {}: ", prompt.bold(), "(y/N)".yellow());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
