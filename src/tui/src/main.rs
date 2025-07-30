// crates/ofx-tui/src/main.rs
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ofx_parser::{adapters::ofx_parser::OfxParser, domain::entities::ofx::Transaction};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};
use std::{fs, io, path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[command(version, about = "OFX 1.02 transaction viewer in the terminal")]
struct Args {
    /// Path to OFX file (v1.02)
    path: PathBuf,
}

struct App {
    txs: Vec<Transaction>,
    state: TableState,
    scroll: usize,
}

impl App {
    fn new(txs: Vec<Transaction>) -> Self {
        let mut s = TableState::default();
        if !txs.is_empty() {
            s.select(Some(0));
        }
        Self {
            txs,
            state: s,
            scroll: 0,
        }
    }

    fn selected(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    fn select(&mut self, index: usize, viewport: usize) {
        let max = self.txs.len().saturating_sub(1);
        let idx = index.min(max);
        self.state.select(Some(idx));
        // maintain scroll so selected row stays in view
        if idx < self.scroll {
            self.scroll = idx;
        } else if idx >= self.scroll + viewport {
            self.scroll = idx + 1 - viewport;
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let content = fs::read_to_string(&args.path)?;
    let ofx: Vec<Transaction> = OfxParser::parse_string(content.as_ref()).map_or(Vec::new(), |d| {
        d.bank_msgs
            .iter()
            .flat_map(|i| {
                i.stmtrs
                    .banktranlist
                    .clone()
                    .map_or(Vec::new(), |t| t.transactions)
            })
            .collect()
    });

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = run_app(&mut terminal, App::new(ofx));

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(area);

            // Header
            let title = format!(
                "OFX Viewer — {} transactions (q=quit, ↑/↓/j/k=move, g/G=home/end)",
                app.txs.len()
            );
            let header = Block::default()
                .borders(Borders::BOTTOM)
                .title(Line::from(title).style(Style::default().add_modifier(Modifier::BOLD)));
            f.render_widget(header, layout[0]);

            // Table
            let rows_visible = layout[1].height.saturating_sub(2) as usize; // roughly
            let start = app.scroll.min(app.txs.len());
            let end = (start + rows_visible).min(app.txs.len());
            let visible_rows = app.txs[start..end].iter().map(|t| {
                Row::new(vec![
                    Cell::from(t.dtposted.to_string()),
                    Cell::from(t.trntype.clone()),
                    Cell::from(format_amount(t.trnamt)),
                    Cell::from(t.name.clone().unwrap_or("".to_string())),
                    Cell::from(t.memo.clone().unwrap_or("".to_string())),
                ])
            });

            let table = Table::default()
                .rows(visible_rows)
                .header(
                    Row::new(vec!["Date", "Type", "Amount", "Name", "Memo"])
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .bottom_margin(1),
                )
                .block(Block::default().borders(Borders::ALL).title("Transactions"))
                .widths([
                    Constraint::Length(10), // Date
                    Constraint::Length(10), // Type
                    Constraint::Length(12), // Amount
                    Constraint::Length(30), // Name
                    Constraint::Min(10),    // Memo
                ])
                .row_highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            // We render only the visible slice; highlight is relative to the slice.
            let mut rel_state = ratatui::widgets::TableState::default();
            if let Some(sel) = app.state.selected() {
                if sel >= start && sel < end {
                    rel_state.select(Some(sel - start));
                } else {
                    rel_state.select(None);
                }
            }
            f.render_stateful_widget(table, layout[1], &mut rel_state);

            // Footer/instructions
            let footer = Block::default().borders(Borders::TOP).title("OFX 1.02");
            f.render_widget(footer, layout[2]);
        })?;

        // Input handling
        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                // Ignore key repeats on some terminals
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => {
                        let sel = app.selected().saturating_add(1);
                        let viewport = terminal.size()?.height.saturating_sub(3) as usize;
                        app.select(sel, viewport);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let sel = app.selected().saturating_sub(1);
                        let viewport = terminal.size()?.height.saturating_sub(3) as usize;
                        app.select(sel, viewport);
                    }
                    KeyCode::Char('g') => {
                        let viewport = terminal.size()?.height.saturating_sub(3) as usize;
                        app.select(0, viewport);
                    }
                    KeyCode::Char('G') => {
                        if !app.txs.is_empty() {
                            let viewport = terminal.size()?.height.saturating_sub(3) as usize;
                            app.select(app.txs.len() - 1, viewport);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn format_amount(a: f64) -> String {
    if a < 0.0 {
        format!("-${:.2}", -a)
    } else {
        format!(" ${a:.2}")
    }
}

fn human_date(yyyymmdd: &str) -> String {
    if yyyymmdd.len() >= 8 {
        format!(
            "{}-{}-{}",
            &yyyymmdd[0..4],
            &yyyymmdd[4..6],
            &yyyymmdd[6..8]
        )
    } else {
        yyyymmdd.to_string()
    }
}
