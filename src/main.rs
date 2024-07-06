use std::env;
use std::error::Error;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod csv_structs;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse the command-line argument for the CSV file path
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path to CSV file>", args[0]);
        return Err("Invalid number of arguments".into());
    }
    let file_path = &args[1];
    let terminal: Terminal<CrosstermBackend<std::io::Stdout>> =
        Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut tui_app = csv_structs::CSVView::new(terminal, file_path);
    tui_app.run()?;
    Ok(())
}
