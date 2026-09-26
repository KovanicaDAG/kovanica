use anyhow::{Result, Context};
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
enum SendStep {
    KeyPath,
    ToAddress,
    Amount,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum HtlcStep {
    SubCommand,
    KeyPath,
    Amount,
    RecipientPk,
    PreimageHash,
    Timeout,
    AssetId,
    OutpointTx,
    OutpointIndex,
    Script,
    Preimage,
    ToAddress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OfferStep {
    SubCommand,
    Maker,
    GiveAsset,
    GiveAmount,
    TakeAsset,
    TakeAmount,
    PreimageHash,
    Timeout,
    ExpiresAt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RwaStep {
    SubCommand,
    Issuer,
    Class,
    Id,
    Version,
    KeyPath,
    AssetId,
    Amount,
    MetadataPath,
    CollectionId,
    ToAddress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NftStep {
    SubCommand,
    AssetId,
    CollectionId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    BalanceAddress,
    AddressKeyPath,
    Send(SendStep),
    Htlc(HtlcStep),
    Offer(OfferStep),
    Rwa(RwaStep),
    Nft(NftStep),
}

struct SendState {
    key_path: Option<String>,
    to_address: Option<String>,
    amount: Option<u64>,
}

struct HtlcState {
    sub_command: Option<String>,
    key_path: Option<String>,
    amount: Option<u64>,
    recipient_pk: Option<String>,
    preimage_hash: Option<String>,
    timeout: Option<u32>,
    asset_id: Option<String>,
    outpoint_tx: Option<String>,
    outpoint_index: Option<u32>,
    script: Option<String>,
    preimage: Option<String>,
    to_address: Option<String>,
}

struct OfferState {
    sub_command: Option<String>,
    maker: Option<String>,
    give_asset: Option<String>,
    give_amount: Option<u64>,
    take_asset: Option<String>,
    take_amount: Option<u64>,
    preimage_hash: Option<String>,
    timeout: Option<u32>,
    expires_at: Option<String>,
}

struct RwaState {
    sub_command: Option<String>,
    issuer: Option<String>,
    class: Option<String>,
    id: Option<String>,
    version: Option<u8>,
    key_path: Option<String>,
    asset_id: Option<String>,
    amount: Option<u64>,
    metadata_path: Option<String>,
    collection_id: Option<String>,
    to_address: Option<String>,
}

struct NftState {
    sub_command: Option<String>,
    asset_id: Option<String>,
    collection_id: Option<String>,
}

struct App {
    menu_state: ListState,
    client: Client,
    output: String,
    show_output: bool,
    input_buffer: String,
    input_prompt: Option<String>,
    input_mode: InputMode,
    send_state: SendState,
    htlc_state: HtlcState,
    offer_state: OfferState,
    rwa_state: RwaState,
    nft_state: NftState,
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
            send_state: SendState::default(),
            htlc_state: HtlcState::default(),
            offer_state: OfferState::default(),
            rwa_state: RwaState::default(),
            nft_state: NftState::default(),
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

    async fn submit_input(&mut self) -> Result<()> {
        let input = std::mem::take(&mut self.input_buffer);
        let mode = std::mem::replace(&mut self.input_mode, InputMode::Normal);
        self.input_prompt = None;

        match mode {
            InputMode::BalanceAddress => {
                let addr = Address::parse(&self.input_buffer).map_err(|e| anyhow::anyhow!("invalid address: {e}"))?;
                let result = self.client.utxos(&addr.to_hex())?;
                self.output = serde_json::to_string_pretty(&result)?;
                self.show_output = true;
            }
            InputMode::AddressKeyPath => {
                let path = if self.input_buffer.trim().is_empty() { "kovanica.key".to_string() } else { self.input_buffer.clone() };
                let wallet = crate::Wallet::load(&PathBuf::from(&path))?;
                let addr = wallet.address();
                self.output = format!("address (kvnc): {}\naddress (hex):  {}", addr.to_kvnc(), addr.to_hex());
                self.show_output = true;
            }
            InputMode::Send(step) => {
                match step {
                    SendStep::KeyPath => {
                        self.send_state.key_path = Some(if self.input_buffer.trim().is_empty() { "kovanica.key".to_string() } else { self.input_buffer.clone() });
                        self.start_input("Enter recipient address (kvnc...dag or hex): ".to_string(), InputMode::Send(SendStep::ToAddress));
                    }
                    SendStep::ToAddress => {
                        self.send_state.to_address = Some(self.input_buffer.clone());
                        self.start_input("Enter amount in atoms (1 KVNC = 100000000): ".to_string(), InputMode::Send(SendStep::Amount));
                    }
                    SendStep::Amount => {
                        let amount = self.input_buffer.parse::<u64>().map_err(|_| anyhow::anyhow!("invalid amount"))?;
                        self.send_state.amount = Some(amount);
                        self.execute_send().await?;
                    }
                }
            }
            InputMode::Htlc(step) => {
                // HTLC implementation would go here
                self.output = "HTLC multi-step input not yet fully implemented".to_string();
                self.show_output = true;
            }
            InputMode::Offer(step) => {
                // Offer implementation would go here
                self.output = "Offer multi-step input not yet fully implemented".to_string();
                self.show_output = true;
            }
            InputMode::Rwa(step) => {
                // RWA implementation would go here
                self.output = "RWA multi-step input not yet fully implemented".to_string();
                self.show_output = true;
            }
            InputMode::Nft(step) => {
                // NFT implementation would go here
                self.output = "NFT multi-step input not yet fully implemented".to_string();
                self.show_output = true;
            }
            InputMode::Normal => {}
        }
        Ok(())
    }

    async fn execute_send(&mut self) -> Result<()> {
        let key_path = self.send_state.key_path.clone().unwrap_or_else(|| "kovanica.key".to_string());
        let to_address = self.send_state.to_address.clone().unwrap();
        let amount = self.send_state.amount.unwrap();

        let wallet = crate::Wallet::load(&PathBuf::from(&key_path))?;
        let from = wallet.address().to_hex();
        let to_addr = Address::parse(&to_address).map_err(|e| anyhow::anyhow!("invalid address: {e}"))?.to_hex();

        let prepared = self.client.prepare(&from, &to_addr, amount)?;
        let sighash_hex = prepared.get("sighash").and_then(|v| v.as_str()).context("missing sighash")?;
        let sighash = hex::decode(sighash_hex.trim()).context("sighash not hex")?;

        let wallet = crate::Wallet::load(&PathBuf::from(&key_path))?;
        let sig = wallet.keypair().sign(&sighash);
        let sig_hex = hex::encode(sig);

        let result = self.client.submit(&from, &to_addr, amount, &sig_hex)?;
        self.output = format!("Sent {} atoms ({} KVNC) to {}", amount, amount / 100_000_000, to_address);
        self.show_output = true;
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
                self.send_state = SendState::default();
                self.start_input(
                    "Enter key file path (default: kovanica.key): ".to_string(),
                    InputMode::Send(SendStep::KeyPath),
                );
            }
            MenuItem::Htlc => {
                self.htlc_state = HtlcState::default();
                self.start_input(
                    "HTLC subcommand (create/redeem/refund/balance): ".to_string(),
                    InputMode::Htlc(HtlcStep::SubCommand),
                );
            }
            MenuItem::Offer => {
                self.offer_state = OfferState::default();
                self.start_input(
                    "Offer subcommand (create/verify): ".to_string(),
                    InputMode::Offer(OfferStep::SubCommand),
                );
            }
            MenuItem::Rwa => {
                self.rwa_state = RwaState::default();
                self.start_input(
                    "RWA subcommand (derive/issue/burn/info): ".to_string(),
                    InputMode::Rwa(RwaStep::SubCommand),
                );
            }
            MenuItem::Nft => {
                self.nft_state = NftState::default();
                self.start_input(
                    "NFT subcommand (info/collection): ".to_string(),
                    InputMode::Nft(NftStep::SubCommand),
                );
            }
            MenuItem::Quit => {
                return Err(anyhow::anyhow!("quit"));
            }
        }
        Ok(())
    }
}

// Default implementations for state structs
impl Default for SendState {
    fn default() -> Self {
        Self { key_path: None, to_address: None, amount: None }
    }
}

impl Default for HtlcState {
    fn default() -> Self {
        Self { 
            sub_command: None, key_path: None, amount: None, recipient_pk: None,
            preimage_hash: None, timeout: None, asset_id: None, outpoint_tx: None,
            outpoint_index: None, script: None, preimage: None, to_address: None 
        }
    }
}

impl Default for OfferState {
    fn default() -> Self {
        Self { sub_command: None, maker: None, give_asset: None, give_amount: None, take_asset: None, take_amount: None, preimage_hash: None, timeout: None, expires_at: None }
    }
}

impl Default for RwaState {
    fn default() -> Self {
        Self { sub_command: None, issuer: None, class: None, id: None, version: None, key_path: None, asset_id: None, amount: None, metadata_path: None, collection_id: None, to_address: None }
    }
}

impl Default for NftState {
    fn default() -> Self {
        Self { sub_command: None, asset_id: None, collection_id: None }
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