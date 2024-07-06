use crossterm::event::{self, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use csv::{Reader, Writer};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::fs::File;
use std::io::{stdout, Result};

#[derive(Debug, PartialEq)]
enum AppState {
    Navigating(usize, usize), // Holds the current row and column
    Editing(usize, usize),    // Holds the row and column being edited
}

pub struct CSVModel {
    file_path: String,
    headers: Vec<String>,
    grid: Vec<Vec<String>>,
    state: AppState,
    running: bool,
}

impl CSVModel {
    pub fn build_from_file_path(file_path: &String) -> Result<CSVModel> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        let mut grid = Vec::new();

        let headers = rdr
            .headers()?
            .iter()
            .map(|field| field.to_string())
            .collect();

        for result in rdr.records() {
            let record = result?;
            grid.push(record.iter().map(|field| field.to_string()).collect());
        }

        Ok(CSVModel {
            file_path: file_path.clone(),
            headers,
            grid,
            state: AppState::Navigating(0, 0),
            running: true,
        })
    }

    fn get_current_row_and_col(&self) -> (usize, usize) {
        match self.state {
            AppState::Navigating(row, col) => (row, col),
            AppState::Editing(row, col) => (row, col),
        }
    }

    fn append_char_to_record(&mut self, c: char) {
        if let AppState::Editing(row, col) = self.state {
            self.grid[row][col].push(c);
        }
    }

    fn save_changes_to_file(&self) -> Result<()> {
        let file = File::create(&self.file_path)?;
        // TODO: may need to erase the file first, we'll see
        let mut wtr = Writer::from_writer(file);

        wtr.write_record(self.headers.iter())?;
        for row in self.grid.iter() {
            wtr.write_record(row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn handle_keyboard_input(&mut self, key: KeyCode) {
        match self.state {
            AppState::Navigating(selected_row, selected_col) => match key {
                KeyCode::Char('j') => {
                    if selected_row < self.grid.len() - 1 {
                        self.state = AppState::Navigating(selected_row + 1, selected_col);
                    }
                }
                KeyCode::Char('k') => {
                    if selected_row > 0 {
                        self.state = AppState::Navigating(selected_row - 1, selected_col);
                    }
                }
                KeyCode::Char('h') => {
                    if selected_col > 0 {
                        self.state = AppState::Navigating(selected_row, selected_col - 1);
                    }
                }
                KeyCode::Char('l') => {
                    if selected_col < self.grid[selected_row].len() - 1 {
                        self.state = AppState::Navigating(selected_row, selected_col + 1);
                    }
                }
                KeyCode::Enter => {
                    self.state = AppState::Editing(selected_row, selected_col);
                }
                KeyCode::Char('q') => {
                    self.save_changes_to_file().unwrap();
                    self.running = false;
                }
                _ => {}
            },
            AppState::Editing(row, col) => match key {
                KeyCode::Enter => {
                    self.state = AppState::Navigating(row, col);
                }
                KeyCode::Char(char) => {
                    self.append_char_to_record(char);
                }
                _ => {}
            },
        }
    }
}

pub struct CSVView {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    model: CSVModel,
}

impl CSVView {
    pub fn new(
        terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
        file_path: &String,
    ) -> CSVView {
        CSVView {
            terminal,
            model: CSVModel::build_from_file_path(file_path).unwrap(),
        }
    }

    pub fn handle_keyboard_input(&mut self, key: KeyCode) {
        self.model.handle_keyboard_input(key);
    }

    pub fn render_tui(&mut self) {
        let _ = self.terminal.draw(|frame| {
            let chunks = frame.size();
            let (selected_row, selected_col) = self.model.get_current_row_and_col();
            let mut text = String::new();
            for (i, row) in self.model.grid.iter().enumerate() {
                for (j, col) in row.iter().enumerate() {
                    if i == selected_row && j == selected_col {
                        text.push_str(&format!("{}*", col));
                    } else {
                        text.push_str(col);
                    }
                    text.push_str("\t");
                }
                text.push_str("\n");
            }
            let paragraph = Paragraph::new(text.as_str());
            frame.render_widget(paragraph, chunks);
        });
    }

    pub fn run(&mut self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.terminal.clear()?;
        while self.model.running {
            self.render_tui();
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press {
                        self.handle_keyboard_input(key.code);
                    }
                }
            }
        }
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
