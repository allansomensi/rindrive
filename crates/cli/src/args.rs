use clap::Parser;
use rindrive_core::engine::EngineType;

#[derive(Parser, Debug)]
#[command(author, version, about = "Rindrive")]
pub struct Args {
    /// Testing engine to use
    #[arg(short, long, value_enum, default_value_t = EngineType::SpotCheck)]
    pub engine: EngineType,

    /// Specific target (e.g., E: or /dev/sdb). If omitted, waits for USB connection.
    #[arg(short, long)]
    pub target: Option<String>,

    /// [Spot-check] Number of sections to divide the test
    #[arg(long, default_value_t = 576)]
    pub sections: usize,

    /// [Spot-check] Write buffer size in bytes
    #[arg(long, default_value_t = 4096)]
    pub buffer_size: usize,
}
