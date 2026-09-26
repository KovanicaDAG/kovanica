use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;

use crate::api::Client;
use crate::Wallet;
use kovanica_state::Address;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    Head,
    P2p,
    Bootstrap,
    State,
    Blocks,
    Balance,
    Keygen,
    Address,
    Send,
    Htlc,
    Offer,
    Rwa,
    Nft,
    Quit,
}

impl MenuItem {
    fn label(&self) -> &'static str {
        match self {
            MenuItem::Head => "1. Chain Head",
            MenuItem::P2p => "2. P2P Info",
            MenuItem::Bootstrap => "3. Bootstrap",
            MenuItem::State => "4. Node State",
            MenuItem::Blocks => "5. Blocks",
            MenuItem::Balance => "6. Balance",
            MenuItem::Keygen => "7. Keygen",
            MenuItem::Address => "8. Address",
            MenuItem::Send => "9. Send",
            MenuItem::Htlc => "10. HTLC",
            MenuItem::Offer => "11. Offer",
            MenuItem::Rwa => "12. RWA",
            MenuItem::Nft => "13. NFT",
            MenuItem::Quit => "Q. Quit",
        }
    }

    fn all() -> &'static [MenuItem] {
        &[
            MenuItem::Head,
            MenuItem::P2p,
            MenuItem::Bootstrap,
            MenuItem::State,
            MenuItem::Blocks,
            MenuItem::Balance,
            MenuItem::Keygen,
            MenuItem::Address,
            MenuItem::Send,
            MenuItem::Htlc,
            MenuItem::Offer,
            MenuItem::Rwa,
            MenuItem::Nft,
            MenuItem::Quit,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    BalanceAddress,
    AddressKeyPath,
}

struct App {
    menu_state: ListState,
    client: Client,
    output: String,
    show_output: bool,
    input_buffer: String,
    input_prompt: Option<String>,
    input_mode: InputMode,
}

impl App {
    fn new(client: Client) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        Self {
            menu_state,
            client,
            output: String::new(),
            show_output: false,
            input_buffer: String::new(),
            input_prompt: None,
            input_mode: InputMode::Normal,
        }
    }

    fn next(&mut self) {
        if self.input_prompt.is_none() {
            let i = self.menu_state.selected().unwrap_or(0);
            if i < MenuItem::all().len() - 1 {
                self.menu_state.select(Some(i + 1));
            }
        }
    }

    fn previous(&mut self) {
        if self.input_prompt.is_none() {
            let i = self.menu_state.selected().unwrap_or(0);
            if i > 0 {
                self.menu_state.select(Some(i - 1));
            }
        }
    }

    fn start_input(&mut self, prompt: String, mode: InputMode) {
        self.input_prompt = Some(prompt);
        self.input_buffer.clear();
        self.input_mode = mode;
    }

    fn handle_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    fn handle_input_backspace(&mut self) {
        self.input_buffer.pop();
    }

    async fn submit_input(&mut self) -> Result<()> {
        let input = std::mem::take(&mut self.input_buffer);
        let mode = std::mem::replace(&mut self.input_mode, InputMode::Normal);
        self.input_prompt = None;

        match mode {
            InputMode::Normal => {}
            InputMode::BalanceAddress => {
                let addr = Address::parse(&input).map_err(|e| anyhow::anyhow!("invalid address: {e}"))?;
                let result = self.client.utxos(&addr.to_hex())?;
                self.output = serde_json::to_string_pretty(&result)?;
                self.show_output = true;
            }
            InputMode::AddressKeyPath => {
                let path = if input.trim().is_empty() { "kovanica.key".to_string() } else { input };
                let wallet = crate::Wallet::load(&PathBuf::from(&path))?;
                let addr = wallet.address();
                self.output = format!("address (kvnc): {}\naddress (hex):  {}", addr.to_kvnc(), addr.to_hex());
                self.show_output = true;
            }
        }
        Ok(())
    }

    async fn execute(&mut self, item: MenuItem) -> Result<()> {
        self.output.clear();
        self.show_output = true;

        match item {
            MenuItem::Head => {
                let result = self.client.head()?;
                self.output = serde_json::to_string_pretty(&result)?;
            }
            MenuItem::P2p => {
                let result = self.client.p2p()?;
                self.output = serde_json::to_string_pretty(&result)?;
            }
            MenuItem::Bootstrap => {
                let result = self.client.bootstrap()?;
                self.output = serde_json::to_string_pretty(&result)?;
            }
            MenuItem::State => {
                let result = self.client.state()?;
                self.output = serde_json::to_string_pretty(&result)?;
            }
            MenuItem::Blocks => {
                let result = self.client.blocks()?;
                self.output = serde_json::to_string_pretty(&result)?;
            }
            MenuItem::Balance => {
                self.start_input(
                    "Enter address (kvnc...dag or 64-hex): ".to_string(),
                    InputMode::BalanceAddress,
                );
            }
            MenuItem::Keygen => {
                let wallet = crate::Wallet::generate()?;
                let key_path = "kovanica.key";
                wallet.save(&PathBuf::from("kovanica.key"), false)?;
                let addr = wallet.address();
                let mut output = String::new();
                output.push_str(&format!("Wrote key to {key_path} (keep it secret)\n"));
                output.push_str(&format!("address (kvnc): {}\n", addr.to_kvnc()));
                output.push_str(&format!("address (hex):  {}", addr.to_hex()));
                self.output = output;
                self.show_output = true;
            }
            MenuItem::Address => {
                self.start_input(
                    "Enter key file path (default: kovanica.key): ".to_string(),
                    InputMode::AddressKeyPath,
                );
            }
            MenuItem::Send => {
                self.output = "Send requires multi-step input. Use CLI: kovanica send --key <key> --to <addr> --amount <atoms>".to_string();
                self.show_output = true;
            }
            MenuItem::Htlc => {
                self.output = "HTLC - use CLI: kovanica htlc create/redeem/refund/balance".to_string();
                self.show_output = true;
            }
            MenuItem::Offer => {
                self.output = "Offer - use CLI: kovanica offer create/verify".to_string();
                self.show_output = true;
            }
            MenuItem::Rwa => {
                self.output = "RWA - use CLI: kovanica rwa derive/issue/burn/info".to_string();
                self.show_output = true;
            }
            MenuItem::Nft => {
                self.output = "NFT - use CLI: kovanica nft info/collection".to_string();
                self.show_output = true;
            }
            MenuItem::Quit => {
                return Err(anyhow::anyhow!("quit"));
            }
        }
        Ok(())
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.area());

    let items: Vec<ListItem> = MenuItem::all()
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let style = if Some(idx) == app.menu_state.selected() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(item.label(), style)))
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Kovanica TUI"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(menu, chunks[0], &mut app.menu_state.clone());

    let (output_title, output_text) = if app.input_prompt.is_some() {
        ("Input".to_string(), Text::from(format!("{}{}", app.input_prompt.as_ref().unwrap(), app.input_buffer)))
    } else if app.show_output {
        ("Output".to_string(), Text::from(app.output.clone()))
    } else {
        ("Output".to_string(), Text::from("Select a command from the menu (Enter to execute, Q to quit)"))
    };

    let output_block = Block::default().borders(Borders::ALL).title(output_title);
    let output = Paragraph::new(output_text)
        .block(output_block)
        .wrap(Wrap { trim: true });
    f.render_widget(output, chunks[1]);

    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(Rect {
            x: f.area().x,
            y: f.area().y + f.area().height - 4,
            width: f.area().width,
            height: 4,
        });

    let help_text = if app.input_prompt.is_some() {
        "Type input | Enter: Submit | Esc: Cancel"
    } else {
        "↑/↓: Navigate | Enter: Execute | Q: Quit"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help, footer_chunks[0]);

    let contact = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Contact: ", Style::default().fg(Color::Cyan)),
            Span::styled("dev@kovanica.online", Style::default().fg(Color::White)),
            Span::styled("  |  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "security@kovanica.online",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("by: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "github.com/BetterCallDzuks",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
    ])
    .alignment(Alignment::Center);
    f.render_widget(contact, footer_chunks[1]);

    let version = Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(version, footer_chunks[2]);
}

pub fn run(client: Client) -> Result<()> {
    let rt = Runtime::new()?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client);
    let result = rt.block_on(run_app(&mut terminal, &mut app));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if app.input_prompt.is_some() {
                    match key.code {
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => { app.input_buffer.pop(); }
                        KeyCode::Enter => {
                            if let Err(e) = app.submit_input().await {
                                if e.to_string() != "quit" {
                                    app.output = format!("Error: {}", e);
                                    app.show_output = true;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.input_prompt = None;
                            app.input_buffer.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Up => app.previous(),
                        KeyCode::Down => app.next(),
                        KeyCode::Enter => {
                            if let Some(selected) = app.menu_state.selected() {
                                let item = MenuItem::all()[selected];
                                if item == MenuItem::Quit {
                                    return Ok(());
                                }
                                if let Err(e) = app.execute(item).await {
                                    if e.to_string() != "quit" {
                                        app.output = format!("Error: {}", e);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}