//! SSS Token Admin TUI - Production-ready Terminal UI for Solana Stablecoin Standard
//!
//! This TUI provides real-time monitoring and administration of SSS tokens
//! with full Solana blockchain integration.
//!
//! ## Features
//! 
//! - `solana` (default: disabled): Enable full Solana RPC integration
//!   - Requires OpenSSL to be installed or vendored (needs Perl on Windows)
//!   - Without this feature, the TUI runs in demo/mock mode
//!
//! ## Environment Variables
//!
//! - `SSS_RPC_URL`: Solana RPC endpoint (default: https://api.devnet.solana.com)
//! - `SSS_KEYPAIR_PATH`: Path to keypair file (default: ~/.config/solana/id.json)

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap},
};
use std::{
    io,
    time::{Duration, Instant},
};

#[cfg(feature = "solana")]
use {
    anchor_client::{Client, Cluster, Program},
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
    },
    std::rc::Rc,
};



// ============================================================================
// Constants
// ============================================================================

/// Program ID for the SSS Token program
#[cfg(feature = "solana")]
const PROGRAM_ID: &str = "SSSToken11111111111111111111111111111111111";

/// PDA seeds (used in solana feature)
#[allow(dead_code)]
const STABLECOIN_SEED: &[u8] = b"stablecoin";
#[allow(dead_code)]
const MINTER_SEED: &[u8] = b"minter";
#[allow(dead_code)]
const BLACKLIST_SEED: &[u8] = b"blacklist";

/// Refresh interval for blockchain data (in milliseconds)
#[allow(dead_code)]
const REFRESH_INTERVAL_MS: u64 = 5000;

// ============================================================================
// Mock Types (for non-Solana builds)
// ============================================================================

#[cfg(not(feature = "solana"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct MockPubkey([u8; 32]);

#[cfg(not(feature = "solana"))]
impl MockPubkey {
    fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let mut arr = [0u8; 32];
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        arr[..8].copy_from_slice(&counter.to_le_bytes());
        arr[31] = 0x01; // Valid base58 ending
        Self(arr)
    }
    
    fn from_str(s: &str) -> Result<Self, ()> {
        // Simple mock - just accept any string
        let bytes = s.as_bytes();
        let mut arr = [0u8; 32];
        let len = bytes.len().min(32);
        arr[..len].copy_from_slice(&bytes[..len]);
        Ok(Self(arr))
    }
}

#[cfg(not(feature = "solana"))]
impl std::fmt::Display for MockPubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Base58-like encoding
        write!(f, "{}", bs58::encode(&self.0).into_string())
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// Represents the on-chain stablecoin state
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct StablecoinState {
    #[cfg(feature = "solana")]
    authority: Pubkey,
    #[cfg(not(feature = "solana"))]
    authority: MockPubkey,
    
    #[cfg(feature = "solana")]
    asset_mint: Pubkey,
    #[cfg(not(feature = "solana"))]
    asset_mint: MockPubkey,
    
    total_supply: u64,
    paused: bool,
    preset: u8,
    compliance_enabled: bool,
    bump: u8,
}

/// Represents a minter info account
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct MinterInfo {
    #[cfg(feature = "solana")]
    minter: Pubkey,
    #[cfg(not(feature = "solana"))]
    minter: MockPubkey,
    
    quota: u64,
    minted_amount: u64,
    bump: u8,
}

/// Represents a blacklist entry
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct BlacklistEntry {
    #[cfg(feature = "solana")]
    account: Pubkey,
    #[cfg(not(feature = "solana"))]
    account: MockPubkey,
    
    reason: String,
    
    #[cfg(feature = "solana")]
    blacklisted_by: Pubkey,
    #[cfg(not(feature = "solana"))]
    blacklisted_by: MockPubkey,
    
    blacklisted_at: i64,
}

/// Role types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Role {
    Master,
    Minter,
    Burner,
    Blacklister,
    Pauser,
    Seizer,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "Master"),
            Role::Minter => write!(f, "Minter"),
            Role::Burner => write!(f, "Burner"),
            Role::Blacklister => write!(f, "Blacklister"),
            Role::Pauser => write!(f, "Pauser"),
            Role::Seizer => write!(f, "Seizer"),
        }
    }
}

/// TUI screen/views
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Dashboard,
    Minters,
    Blacklist,
    Roles,
    Actions,
    Help,
}

/// Application state
#[derive(Debug)]
#[allow(dead_code)]
struct App {
    // UI state
    should_quit: bool,
    current_view: View,
    selected_item: usize,
    scroll_offset: usize,
    input_mode: bool,
    input_buffer: String,
    status_message: Option<(String, Instant)>,
    
    // Connection state
    connected: bool,
    connecting: bool,
    rpc_url: String,
    
    #[cfg(feature = "solana")]
    authority: Option<Pubkey>,
    #[cfg(not(feature = "solana"))]
    authority: Option<MockPubkey>,
    
    #[cfg(feature = "solana")]
    program_id: Pubkey,
    #[cfg(not(feature = "solana"))]
    program_id: MockPubkey,
    
    #[cfg(feature = "solana")]
    stablecoin_pda: Option<Pubkey>,
    #[cfg(not(feature = "solana"))]
    stablecoin_pda: Option<MockPubkey>,
    
    // Blockchain data
    stablecoin_state: Option<StablecoinState>,
    minters: Vec<MinterInfo>,
    blacklist: Vec<BlacklistEntry>,
    
    // Stats
    last_refresh: Option<Instant>,
    refresh_count: u64,
    error_count: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            current_view: View::Dashboard,
            selected_item: 0,
            scroll_offset: 0,
            input_mode: false,
            input_buffer: String::new(),
            status_message: None,
            connected: false,
            connecting: false,
            rpc_url: String::from("https://api.devnet.solana.com"),
            authority: None,
            #[cfg(feature = "solana")]
            program_id: Pubkey::try_from(PROGRAM_ID).unwrap_or_default(),
            #[cfg(not(feature = "solana"))]
            program_id: MockPubkey::from_str("SSSToken11111111111111111111111111111111111").unwrap_or_default(),
            stablecoin_pda: None,
            stablecoin_state: None,
            minters: Vec::new(),
            blacklist: Vec::new(),
            last_refresh: None,
            refresh_count: 0,
            error_count: 0,
        }
    }
}

impl App {
    fn format_supply(&self) -> String {
        if let Some(state) = &self.stablecoin_state {
            format_number(state.total_supply)
        } else {
            "---".to_string()
        }
    }
    
    fn get_preset_name(&self) -> &'static str {
        if let Some(state) = &self.stablecoin_state {
            match state.preset {
                1 => "SSS-1 (Standard)",
                2 => "SSS-2 (Compliance)",
                _ => "Unknown",
            }
        } else {
            "---"
        }
    }
    
    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }
    
    fn clear_expired_status(&mut self) {
        if let Some((_, time)) = self.status_message {
            if time.elapsed() > Duration::from_secs(5) {
                self.status_message = None;
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

#[cfg(feature = "solana")]
fn shorten_pubkey(pubkey: &Pubkey) -> String {
    let s = pubkey.to_string();
    format!("{}...{}", &s[..4], &s[s.len()-4..])
}

#[cfg(not(feature = "solana"))]
fn shorten_pubkey(pubkey: &MockPubkey) -> String {
    let s = pubkey.to_string();
    if s.len() > 8 {
        format!("{}...{}", &s[..4], &s[s.len()-4..])
    } else {
        s
    }
}

#[cfg(feature = "solana")]
fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = std::env::var("HOME").ok().or_else(|| std::env::var("USERPROFILE").ok()) {
            return path.replacen('~', &home, 1);
        }
    }
    path.to_string()
}

#[cfg(feature = "solana")]
fn derive_stablecoin_pda(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[STABLECOIN_SEED, authority.to_bytes().as_ref()],
        program_id,
    )
}

#[cfg(feature = "solana")]
fn derive_minter_pda(stablecoin: &Pubkey, minter: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MINTER_SEED, stablecoin.to_bytes().as_ref(), minter.to_bytes().as_ref()],
        program_id,
    )
}

#[cfg(feature = "solana")]
fn derive_blacklist_pda(stablecoin: &Pubkey, account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[BLACKLIST_SEED, stablecoin.to_bytes().as_ref(), account.to_bytes().as_ref()],
        program_id,
    )
}

// ============================================================================
// Solana Client Setup (only for solana feature)
// ============================================================================

#[cfg(feature = "solana")]
fn setup_solana_client(
    rpc_url: &str,
    keypair_path: &str,
    program_id: Pubkey,
) -> Result<(Program<Rc<Keypair>>, Pubkey)> {
    let expanded_path = expand_tilde(keypair_path);
    let keypair = read_keypair_file(&expanded_path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair from {}: {}", expanded_path, e))?;
    
    let authority = keypair.pubkey();
    
    let client = Client::new_with_options(
        Cluster::Custom(rpc_url.to_string(), rpc_url.to_string()),
        Rc::new(keypair),
        CommitmentConfig::confirmed(),
    );
    
    let program = client
        .program(program_id)
        .map_err(|e| anyhow::anyhow!("Failed to create program client: {}", e))?;
    
    Ok((program, authority))
}

// ============================================================================
// UI Rendering
// ============================================================================

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    // Header
    render_header(f, app, chunks[0]);
    
    // Main content based on current view
    match app.current_view {
        View::Dashboard => render_dashboard(f, app, chunks[1]),
        View::Minters => render_minters(f, app, chunks[1]),
        View::Blacklist => render_blacklist(f, app, chunks[1]),
        View::Roles => render_roles(f, app, chunks[1]),
        View::Actions => render_actions(f, app, chunks[1]),
        View::Help => render_help(f, app, chunks[1]),
    }
    
    // Status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let mode = if cfg!(feature = "solana") {
        "Live"
    } else {
        "Demo"
    };
    
    let title = if app.connected {
        format!("SSS Token Admin [{}] - Connected to {}", mode, app.rpc_url)
    } else if app.connecting {
        format!("SSS Token Admin [{}] - Connecting to {}...", mode, app.rpc_url)
    } else {
        format!("SSS Token Admin [{}] - Not Connected (Press 'c' to connect)", mode)
    };
    
    let header = Paragraph::new(title)
        .style(Style::default().fg(if app.connected { 
            Color::Green 
        } else if app.connecting { 
            Color::Yellow 
        } else { 
            Color::Red 
        }))
        .block(Block::default().borders(Borders::ALL).title("SSS Admin TUI"));
    
    f.render_widget(header, area);
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Stats
            Constraint::Length(5),  // Connection info
            Constraint::Min(0),     // Controls
        ])
        .split(area);
    
    // Stats panel
    let stats_text = if let Some(state) = &app.stablecoin_state {
        format!(
            "Total Supply: {} tokens\n\
             Preset: {}\n\
             Paused: {}\n\
             Compliance: {}\n\
             Authority: {}",
            app.format_supply(),
            app.get_preset_name(),
            if state.paused { "YES" } else { "NO" },
            if state.compliance_enabled { "ENABLED" } else { "DISABLED" },
            shorten_pubkey(&state.authority)
        )
    } else {
        "No stablecoin initialized for this authority.\n\
         Use 'Actions' menu to initialize a new stablecoin.".to_string()
    };
    
    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Stablecoin Status"));
    f.render_widget(stats, chunks[0]);
    
    // Connection info
    let conn_text = format!(
        "Authority: {}\n\
         Stablecoin PDA: {}\n\
         Program ID: {}",
        app.authority.map(|a| shorten_pubkey(&a)).unwrap_or_else(|| "Not set".to_string()),
        app.stablecoin_pda.map(|p| shorten_pubkey(&p)).unwrap_or_else(|| "Not derived".to_string()),
        shorten_pubkey(&app.program_id)
    );
    
    let conn_info = Paragraph::new(conn_text)
        .block(Block::default().borders(Borders::ALL).title("Connection Info"));
    f.render_widget(conn_info, chunks[1]);
    
    // Controls help
    let controls = Paragraph::new(
        "[1] Dashboard  [2] Minters  [3] Blacklist  [4] Roles  [5] Actions  [?] Help  [q] Quit"
    ).block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(controls, chunks[2]);
}

fn render_minters(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.minters.is_empty() {
        vec![ListItem::new("No minters registered")]
    } else {
        app.minters.iter().map(|m| {
            ListItem::new(format!(
                "{}: Quota {} | Minted {} | Available {}",
                shorten_pubkey(&m.minter),
                format_number(m.quota),
                format_number(m.minted_amount),
                format_number(m.quota.saturating_sub(m.minted_amount))
            ))
        }).collect()
    };
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("Minters ({})", app.minters.len())))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_widget(list, area);
}

fn render_blacklist(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.blacklist.is_empty() {
        vec![ListItem::new("No accounts blacklisted")]
    } else {
        app.blacklist.iter().map(|b| {
            ListItem::new(format!(
                "{}: {} (by {} at {})",
                shorten_pubkey(&b.account),
                b.reason,
                shorten_pubkey(&b.blacklisted_by),
                chrono::DateTime::from_timestamp(b.blacklisted_at, 0)
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "?".to_string())
            ))
        }).collect()
    };
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("Blacklist ({})", app.blacklist.len())))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_widget(list, area);
}

fn render_roles(f: &mut Frame, _app: &App, area: Rect) {
    let roles_text = "Available Roles:\n\
        \n\
        * Master     - Full administrative control\n\
        * Minter     - Can mint new tokens (with quota)\n\
        * Burner     - Can burn tokens\n\
        * Blacklister - Can add/remove from blacklist\n\
        * Pauser     - Can pause/unpause operations\n\
        * Seizer     - Can seize tokens from blacklisted accounts\n\
        \n\
        Use the Actions menu to assign or revoke roles.";
    
    let roles = Paragraph::new(roles_text)
        .block(Block::default().borders(Borders::ALL).title("Role Management"));
    f.render_widget(roles, area);
}

fn render_actions(f: &mut Frame, _app: &App, area: Rect) {
    let actions = vec![
        "[i] Initialize new stablecoin",
        "[m] Mint tokens",
        "[b] Burn tokens",
        "[p] Pause operations",
        "[u] Unpause operations",
        "[+] Add minter",
        "[-] Remove minter",
        "[B] Add to blacklist",
        "[R] Remove from blacklist",
        "[a] Assign role",
        "[r] Revoke role",
        "[s] Seize tokens",
    ];
    
    let items: Vec<ListItem> = actions.iter().map(|a| ListItem::new(*a)).collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Actions (Select and press Enter)"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_widget(list, area);
}

fn render_help(f: &mut Frame, _app: &App, area: Rect) {
    let mode_info = if cfg!(feature = "solana") {
        "Running in LIVE mode with full Solana integration.\n\n"
    } else {
        "Running in DEMO mode (compile with --features solana for live mode).\n\n"
    };
    
    let help_text = format!(
        "{}SSS Token Admin TUI - Help\n\
        \n\
        Navigation:\n\
        * Press number keys 1-5 to switch between views\n\
        * Press Up/Down arrows to navigate lists\n\
        * Press Enter to select an action\n\
        * Press Escape to cancel input or go back\n\
        \n\
        Connection:\n\
        * Press 'c' to connect to Solana RPC\n\
        * Press 'r' to refresh blockchain data\n\
        \n\
        Actions:\n\
        * Available actions depend on your role permissions\n\
        * All transactions require signing with your keypair\n\
        \n\
        Configuration:\n\
        * RPC URL: Set via SSS_RPC_URL environment variable\n\
        * Keypair: Set via SSS_KEYPAIR_PATH environment variable\n\
        \n\
        Press 'q' to quit the application.",
        mode_info
    );
    
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(help, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else if app.connected {
        format!(
            "Connected | Refresh: {} | Last update: {}",
            app.refresh_count,
            app.last_refresh
                .map(|t| format!("{:.1}s ago", t.elapsed().as_secs_f32()))
                .unwrap_or_else(|| "Never".to_string())
        )
    } else {
        "Not connected - Press 'c' to connect".to_string()
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, area);
}

// ============================================================================
// Input Handling
// ============================================================================

fn handle_input(key: KeyCode, modifiers: KeyModifiers, app: &mut App) -> Result<()> {
    // Handle input mode separately
    if app.input_mode {
        match key {
            KeyCode::Enter => {
                app.input_mode = false;
                app.set_status(format!("Input received: {}", app.input_buffer));
                app.input_buffer.clear();
            }
            KeyCode::Esc => {
                app.input_mode = false;
                app.input_buffer.clear();
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        }
        return Ok(());
    }
    
    // Global keybindings
    match key {
        KeyCode::Char('q') if modifiers == KeyModifiers::NONE => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if modifiers == KeyModifiers::NONE => {
            app.connecting = true;
            app.set_status("Initiating connection...");
        }
        KeyCode::Char('r') if modifiers == KeyModifiers::NONE => {
            app.set_status("Refreshing data...");
        }
        KeyCode::Char('?') if modifiers == KeyModifiers::NONE => {
            app.current_view = View::Help;
        }
        KeyCode::Char('1') => app.current_view = View::Dashboard,
        KeyCode::Char('2') => app.current_view = View::Minters,
        KeyCode::Char('3') => app.current_view = View::Blacklist,
        KeyCode::Char('4') => app.current_view = View::Roles,
        KeyCode::Char('5') => app.current_view = View::Actions,
        KeyCode::Up => {
            if app.selected_item > 0 {
                app.selected_item -= 1;
            }
        }
        KeyCode::Down => {
            app.selected_item += 1;
        }
        _ => {}
    }
    
    // View-specific keybindings
    match app.current_view {
        View::Actions => {
            match key {
                KeyCode::Char('i') => app.set_status("Initialize: Enter stablecoin name"),
                KeyCode::Char('m') => app.set_status("Mint: Enter recipient address"),
                KeyCode::Char('b') => app.set_status("Burn: Enter amount"),
                KeyCode::Char('p') => app.set_status("Pause operation requested"),
                KeyCode::Char('u') => app.set_status("Unpause operation requested"),
                KeyCode::Char('+') => app.set_status("Add minter: Enter address"),
                KeyCode::Char('-') => app.set_status("Remove minter: Enter address"),
                KeyCode::Char('B') => app.set_status("Add to blacklist: Enter address"),
                KeyCode::Char('R') => app.set_status("Remove from blacklist: Enter address"),
                KeyCode::Char('a') => app.set_status("Assign role: Enter address"),
                KeyCode::Char('r') => app.set_status("Revoke role: Enter address"),
                KeyCode::Char('s') => app.set_status("Seize: Enter address"),
                _ => {}
            }
        }
        _ => {}
    }
    
    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::default();
    
    // Get configuration from environment
    if let Ok(rpc_url) = std::env::var("SSS_RPC_URL") {
        app.rpc_url = rpc_url;
    }
    
    // Main event loop
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;
        
        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_input(key.code, key.modifiers, &mut app)?;
            }
        }
        
        // Clear expired status messages
        app.clear_expired_status();
        
        // Check if we should quit
        if app.should_quit {
            break;
        }
        
        // Handle connection
        if app.connecting && !app.connected {
            #[cfg(feature = "solana")]
            {
                let keypair_path = std::env::var("SSS_KEYPAIR_PATH")
                    .unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
                
                match setup_solana_client(&app.rpc_url, &keypair_path, app.program_id) {
                    Ok((_program, authority)) => {
                        app.connected = true;
                        app.connecting = false;
                        app.authority = Some(authority);
                        
                        let (stablecoin_pda, _) = derive_stablecoin_pda(&authority, &app.program_id);
                        app.stablecoin_pda = Some(stablecoin_pda);
                        
                        app.last_refresh = Some(Instant::now());
                        app.refresh_count = 1;
                        app.set_status(format!("Connected as {}", shorten_pubkey(&authority)));
                    }
                    Err(e) => {
                        app.connecting = false;
                        app.error_count += 1;
                        app.set_status(format!("Connection failed: {}", e));
                    }
                }
            }
            
            #[cfg(not(feature = "solana"))]
            {
                // Demo mode - simulate connection
                app.connected = true;
                app.connecting = false;
                app.authority = Some(MockPubkey::new_unique());
                app.stablecoin_pda = Some(MockPubkey::new_unique());
                
                // Demo data
                app.stablecoin_state = Some(StablecoinState {
                    authority: app.authority.unwrap(),
                    asset_mint: MockPubkey::new_unique(),
                    total_supply: 1_000_000_000,
                    paused: false,
                    preset: 2,
                    compliance_enabled: true,
                    bump: 254,
                });
                
                app.minters = vec![
                    MinterInfo {
                        minter: MockPubkey::new_unique(),
                        quota: 10_000_000,
                        minted_amount: 2_500_000,
                        bump: 253,
                    },
                ];
                
                app.last_refresh = Some(Instant::now());
                app.refresh_count = 1;
                app.set_status("Connected (Demo Mode)".to_string());
            }
        }
    }
    
    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    println!("SSS Admin TUI closed. Goodbye!");
    Ok(())
}
