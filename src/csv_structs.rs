use chrono::{self};
use crossterm::event::{self, KeyCode, KeyEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use csv::{Reader, Writer};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Terminal;
use std::cmp::{max, min};
use std::fs::File;
use std::io::{stdout, Result};

#[derive(Debug, PartialEq)]
enum AppState {
    Navigating(usize, usize),
    Editing(usize, usize),
    EditingHeader(usize),
}

pub struct CSVModel {
    file_path: String,
    headers: Vec<String>,
    grid: Vec<Vec<String>>,
    state: AppState,
    running: bool,
    working_states: Vec<(Vec<String>, Vec<Vec<String>>)>,
    copy_buffer: Option<String>,
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
            working_states: Vec::new(),
            copy_buffer: None,
        })
    }

    fn get_current_row_and_col(&self) -> (usize, usize) {
        match self.state {
            AppState::Navigating(row, col) => (row, col),
            AppState::Editing(row, col) => (row, col),
            AppState::EditingHeader(col) => (0, col),
        }
    }

    fn insert_empty_row_after(&mut self, row: usize) {
        self.save_current_state();
        let empty_row = vec![String::new(); self.headers.len()];
        self.grid.insert(row + 1, empty_row);
    }

    fn insert_empty_row_before(&mut self, row: usize) {
        self.save_current_state();
        let empty_row = vec![String::new(); self.headers.len()];
        self.grid.insert(row, empty_row);
    }

    fn delete_row(&mut self, row: usize) {
        self.save_current_state();
        self.grid.remove(row);
    }

    fn insert_empty_col_after(&mut self, col: usize) {
        self.save_current_state();
        self.headers.insert(col + 1, String::new());
        for row in self.grid.iter_mut() {
            row.insert(col + 1, String::new());
        }
    }

    fn insert_empty_col_before(&mut self, col: usize) {
        self.save_current_state();
        self.headers.insert(col, String::new());
        for row in self.grid.iter_mut() {
            row.insert(col, String::new());
        }
    }

    fn delete_col(&mut self, col: usize) {
        self.save_current_state();
        self.headers.remove(col);
        for row in self.grid.iter_mut() {
            row.remove(col);
        }
    }

    fn save_current_state(&mut self) {
        let current_headers = self.headers.clone();
        let current_grid = self.grid.clone();
        self.working_states.push((current_headers, current_grid));
    }

    fn restore_last_state(&mut self) {
        if let Some((last_headers, last_grid)) = self.working_states.pop() {
            self.headers = last_headers;
            self.grid = last_grid;
            if self.get_current_row_and_col().0 >= self.grid.len() {
                self.state = AppState::Navigating(self.grid.len() - 1, 0);
            }
        }
    }

    fn copy_selected_cell_to_buffer(&mut self) {
        match self.state {
            AppState::Navigating(row, col) => {
                self.copy_buffer = Some(self.grid[row][col].clone());
            }
            _ => {}
        }
    }

    fn paste_from_buffer(&mut self) {
        match self.state {
            AppState::Navigating(row, col) => {
                if let Some(buffer) = &self.copy_buffer {
                    self.grid[row][col] = buffer.clone();
                }
            }
            _ => {}
        }
    }

    fn paste_date(&mut self) {
        match self.state {
            AppState::Navigating(row, col) => {
                self.grid[row][col] =
                    format!("{}", chrono::Local::now().format("%Y-%m-%d").to_string());
            }
            _ => {}
        }
    }

    fn save_changes_to_file(&self) -> Result<()> {
        let file = File::create(&self.file_path)?;
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
                // NAVIGATION
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
                KeyCode::Char('}') => {
                    self.state = AppState::Navigating(
                        min(self.grid.len() - 1, selected_row + 5),
                        selected_col,
                    )
                }
                KeyCode::Char('{') => {
                    if selected_row >= 5 {
                        self.state = AppState::Navigating(max(0, selected_row - 5), selected_col)
                    } else {
                        self.state = AppState::Navigating(0, selected_col)
                    }
                }
                KeyCode::Char('g') => {
                    self.state = AppState::Navigating(0, selected_col);
                }
                KeyCode::Char('G') => {
                    self.state = AppState::Navigating(self.grid.len() - 1, selected_col);
                }
                KeyCode::Char('I') => {
                    self.state = AppState::Navigating(selected_row, 0);
                }
                KeyCode::Char('A') => {
                    self.state = AppState::Navigating(selected_row, self.headers.len() - 1);
                }

                // STATE MANAGEMENT
                KeyCode::Char('u') => {
                    self.restore_last_state();
                }

                // CREATING AND DELETING ROWS AND COLUMNS
                KeyCode::Char('o') => {
                    self.insert_empty_row_after(selected_row);
                    self.state = AppState::Navigating(selected_row + 1, selected_col);
                }
                KeyCode::Char('O') => {
                    self.insert_empty_row_before(selected_row);
                    self.state = AppState::Navigating(selected_row, selected_col);
                }
                KeyCode::Char('d') => {
                    self.delete_row(selected_row);
                    if selected_row == self.grid.len() {
                        self.state = AppState::Navigating(selected_row - 1, selected_col);
                    } else {
                        self.state = AppState::Navigating(selected_row, selected_col);
                    }
                }
                KeyCode::Char('n') => {
                    self.insert_empty_col_after(selected_col);
                    self.state = AppState::Navigating(selected_row, selected_col + 1);
                }
                KeyCode::Char('N') => {
                    self.insert_empty_col_before(selected_col);
                    self.state = AppState::Navigating(selected_row, selected_col);
                }
                KeyCode::Char('D') => {
                    self.delete_col(selected_col);
                    if selected_col == self.grid.len() {
                        self.state = AppState::Navigating(selected_row, selected_col - 1);
                    } else {
                        self.state = AppState::Navigating(selected_row, selected_col);
                    }
                }

                // EDITING
                KeyCode::Char('i') => {
                    self.save_current_state();
                    self.state = AppState::Editing(selected_row, selected_col);
                }
                KeyCode::Enter => {
                    self.save_current_state();
                    self.state = AppState::Editing(selected_row, selected_col);
                }
                KeyCode::Char('r') => {
                    self.grid[selected_row][selected_col] = String::new();
                    self.state = AppState::Editing(selected_row, selected_col);
                }
                KeyCode::Char('H') => {
                    self.save_current_state();
                    self.state = AppState::EditingHeader(selected_col);
                }
                KeyCode::Char('y') => {
                    self.copy_selected_cell_to_buffer();
                }
                KeyCode::Char('p') => {
                    self.save_current_state();
                    self.paste_from_buffer();
                }
                KeyCode::Char('.') => {
                    self.paste_date();
                }

                // QUIT
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
                KeyCode::Backspace => {
                    self.grid[row][col].pop();
                }
                KeyCode::Char(char) => {
                    self.grid[row][col].push(char);
                }
                _ => {}
            },
            AppState::EditingHeader(col) => match key {
                KeyCode::Enter => {
                    self.state = AppState::Navigating(0, col);
                }
                KeyCode::Backspace => {
                    self.headers[col].pop();
                }
                KeyCode::Char(char) => {
                    self.headers[col].push(char);
                }
                _ => {}
            },
        }
    }
}

pub struct CSVView {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    model: CSVModel,
    scroll_offset: usize,
}

impl CSVView {
    pub fn new(
        terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
        file_path: &String,
    ) -> CSVView {
        CSVView {
            terminal,
            model: CSVModel::build_from_file_path(file_path).unwrap(),
            scroll_offset: 0,
        }
    }

    pub fn handle_keyboard_input(&mut self, key: KeyCode) {
        self.model.handle_keyboard_input(key);
        self.render_tui();
    }

    pub fn render_tui(&mut self) {
        let (selected_row, selected_col) = self.model.get_current_row_and_col();
        let _ = self.terminal.draw(|f| {
            let size = f.size();
            self.scroll_offset = max(0, selected_row as i32 - size.height as i32 + 10) as usize;

            let constraints = vec![Constraint::Length(5)]
                .into_iter()
                .chain(
                    std::iter::repeat(Constraint::Percentage(
                        (100 / (self.model.grid[0].len())) as u16,
                    ))
                    .take(self.model.grid[0].len()),
                )
                .collect::<Vec<_>>();

            let header_cells = std::iter::once(
                Cell::from("").style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .chain(self.model.headers.iter().enumerate().map(|(i, h)| {
                match self.model.state {
                    AppState::EditingHeader(col) if i == col => Cell::from(h.clone()).style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::Green),
                    ),
                    _ => Cell::from(h.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
                }
            }));
            let header_row = Row::new(header_cells).height(1);

            let rows = self
                .model
                .grid
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .map(|(i, item)| {
                    let row_number_cell =
                        Cell::from((i + 1).to_string()).style(Style::default().fg(Color::White));
                    let cells = item.iter().enumerate().map(|(j, c)| {
                        let mut cell = Cell::from(c.clone());
                        if i == selected_row && j == selected_col {
                            match self.model.state {
                                AppState::Navigating(_, _) => {
                                    cell = cell.style(Style::default().bg(Color::Blue));
                                }
                                AppState::Editing(_, _) => {
                                    cell = cell.style(Style::default().bg(Color::Green));
                                }
                                _ => {}
                            }
                        }
                        cell
                    });
                    let cells = std::iter::once(row_number_cell).chain(cells);
                    Row::new(cells).height(1)
                });

            let table = Table::new(rows, &constraints)
                .header(header_row)
                .block(Block::default().borders(Borders::ALL))
                .column_spacing(1);

            f.render_widget(table, size);
        });
    }

    pub fn run(&mut self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.terminal.clear()?;
        while self.model.running {
            self.render_tui();
            if let event::Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                if kind == event::KeyEventKind::Press {
                    self.handle_keyboard_input(code);
                }
            }
        }
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
