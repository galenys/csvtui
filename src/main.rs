// #[macro_use]
// extern crate fstrings;

// use crossterm::{
//     event::{self, KeyCode, KeyEventKind},
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
//     ExecutableCommand,
// };
// use csv::Reader;
// use ratatui::{
//     prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect, Stylize, Terminal},
//     style::{Color, Style},
//     text::{Span, Spans},
//     widgets::{Block, Borders, Paragraph},
// };
// use std::env;
// use std::fs::{metadata, File};
// use std::io::{stdout, Result};

// fn main() -> Result<()> {
//     // Parse the command-line argument for the CSV file path
//     let args: Vec<String> = env::args().collect();
//     if args.len() < 2 {
//         eprintln!("Usage: {} <path to CSV file>", args[0]);
//         return Ok(());
//     }
//     let file_path = &args[1];

//     // Print the file path for debugging purposes
//     println!("Provided file path: {}", file_path);

//     // Check if the file exists
//     if metadata(file_path).is_err() {
//         eprintln!("Error: File not found: {}", file_path);
//         return Ok(());
//     }

//     // Read the CSV file
//     let file = File::open(file_path)?;
//     let mut rdr = Reader::from_reader(file);

//     // Collect the CSV data into a vector of records
//     let mut records = Vec::new();
//     for result in rdr.records() {
//         let record = result?;
//         records.push(record);
//     }

//     // Debug: Print the number of records read
//     println!("Number of records read: {}", records.len());

//     // Initial state is navigating with the starting position at (0, 0)
//     let mut state = AppState::Navigating(0, 0);

//     stdout().execute(EnterAlternateScreen)?;
//     enable_raw_mode()?;
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
//     terminal.clear()?;

//     loop {
//         terminal.draw(|frame| {
//             let area = frame.size();

//             // Create layout constraints dynamically based on the number of records
//             let constraints: Vec<Constraint> =
//                 records.iter().map(|_| Constraint::Length(3)).collect();
//             let rows: Vec<Rect> = Layout::default()
//                 .direction(Direction::Vertical)
//                 .constraints(constraints)
//                 .split(area)
//                 .to_vec();

//             for (i, row) in rows.iter().enumerate() {
//                 let row_data = &records[i];
//                 let spans: Vec<Span> = row_data
//                     .iter()
//                     .enumerate()
//                     .map(|(j, field)| {
//                         if let AppState::Navigating(selected_row, selected_col) = state {
//                             if i == selected_row && j == selected_col {
//                                 Span::styled(field.to_string(), Style::new().bg(Color::Blue))
//                             } else {
//                                 Span::raw(field.to_string())
//                             }
//                         } else {
//                             Span::raw(field.to_string())
//                         }
//                     })
//                     .collect();

//                 let paragraph = Paragraph::new(Spans::from(spans)).block(
//                     Block::default()
//                         .borders(Borders::ALL)
//                         .title(format!("Row {}", i + 1)),
//                 );
//                 frame.render_widget(paragraph, *row);
//             }
//         })?;

//         if event::poll(std::time::Duration::from_millis(16))? {
//             if let event::Event::Key(key) = event::read()? {
//                 if key.kind == KeyEventKind::Press {
//                     match state {
//                         AppState::Navigating(mut selected_row, mut selected_col) => {
//                             match key.code {
//                                 KeyCode::Char('q') => break,
//                                 KeyCode::Char('h') => {
//                                     if selected_col > 0 {
//                                         selected_col -= 1;
//                                     }
//                                 }
//                                 KeyCode::Char('j') => {
//                                     if selected_row < records.len() - 1 {
//                                         selected_row += 1;
//                                     }
//                                 }
//                                 KeyCode::Char('k') => {
//                                     if selected_row > 0 {
//                                         selected_row -= 1;
//                                     }
//                                 }
//                                 KeyCode::Char('l') => {
//                                     if selected_col < records[selected_row].len() - 1 {
//                                         selected_col += 1;
//                                     }
//                                 }
//                                 KeyCode::Enter => {
//                                     state = AppState::Editing(selected_row, selected_col)
//                                 }
//                                 _ => {}
//                             }
//                             state = AppState::Navigating(selected_row, selected_col);
//                         }
//                         AppState::Editing(row, col) => {
//                             match key.code {
//                                 KeyCode::Char('q') => break,
//                                 KeyCode::Enter => state = AppState::Navigating(row, col), // Exit editing mode on Enter key
//                                 _ => {}
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     stdout().execute(LeaveAlternateScreen)?;
//     disable_raw_mode()?;
//     Ok(())
// }

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
    let mut csv_view = csv_structs::CSVView::new(terminal, file_path);
    csv_view.run()?;
    Ok(())
}
