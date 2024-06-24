use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use csv::Reader;
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    widgets::{Block, Borders, Paragraph},
};
use std::env;
use std::fs::{metadata, File};
use std::io::{stdout, Result};

fn main() -> Result<()> {
    // Parse the command-line argument for the CSV file path
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path to CSV file>", args[0]);
        return Ok(());
    }
    let file_path = &args[1];

    // Print the file path for debugging purposes
    println!("Provided file path: {}", file_path);

    // Check if the file exists
    if metadata(file_path).is_err() {
        eprintln!("Error: File not found: {}", file_path);
        return Ok(());
    }

    // Read the CSV file
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);

    // Collect the CSV data into a vector of records
    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(record);
    }

    // Debug: Print the number of records read
    println!("Number of records read: {}", records.len());

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.size();

            // Create layout constraints dynamically based on the number of records
            let constraints: Vec<Constraint> =
                records.iter().map(|_| Constraint::Length(3)).collect();
            let rows: Vec<Rect> = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area)
                .to_vec();

            for (i, row) in rows.iter().enumerate() {
                if i < records.len() {
                    let row_data = &records[i];
                    let row_text = row_data
                        .iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<String>>()
                        .join(" | ");
                    let paragraph = Paragraph::new(row_text.clone()).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Row {}", i + 1)),
                    );
                    frame.render_widget(paragraph, *row);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
