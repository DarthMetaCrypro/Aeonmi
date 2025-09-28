use std::{fs, thread, time::Duration};

use anyhow::{Context, Result};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::cli_vault::VaultCommand;
use crate::vault::{describe_hijack_resilience, DomainQuantumVault, VaultDomainProfile};

pub fn dispatch(command: VaultCommand, use_tui: bool) -> Result<()> {
    let mut vault = DomainQuantumVault::open_default()?;

    match command {
        VaultCommand::Register(args) => {
            let metadata = read_metadata(args.metadata_path.as_ref())?;
            let secret = read_secret(&args.secret_path)?;
            let profile = VaultDomainProfile {
                domain: args.domain,
                registrar: args.registrar,
                expiration: args.expiration,
                auto_renew: args.auto_renew,
                dnssec_enabled: args.dnssec,
                registrar_lock: args.lock,
                blockchain_registry: args.blockchain,
                metadata,
            };
            vault.register_domain(profile, secret)?;
            println!("Domain registered and encrypted in quantum vault.");
        }
        VaultCommand::Transfer(args) => {
            let token = match args.auth_token_path {
                Some(path) => Some(read_secret(&path)?),
                None => None,
            };
            vault.transfer_domain(&args.domain, &args.new_registrar, token)?;
            println!("Domain transfer metadata recorded.");
        }
        VaultCommand::Fortify(args) => {
            if args.dnssec {
                vault.enable_dnssec(&args.domain)?;
            }
            if args.lock {
                vault.lock_domain(&args.domain, true)?;
            }
            if let Some(registry) = args.blockchain {
                vault.attach_blockchain_registry(&args.domain, &registry)?;
            }
            println!("Domain fortification updated.");
        }
        VaultCommand::QubeRun(args) => {
            let script = fs::read_to_string(&args.script).context("read QUBE script")?;
            let profile = args.profile.unwrap_or_else(|| "default".to_string());
            let report = vault.run_qube_policy(&script)?;
            println!("[QUBE:{}]\n{}", profile, report);
        }
        VaultCommand::VaultStatus(args) => {
            let status = vault.generate_status()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else if use_tui {
                render_tui(&status)?;
            } else {
                print_status(&status);
            }
        }
        VaultCommand::VaultAnalyze(args) => {
            if let Some(domain) = args.domain {
                let report = vault.run_simulation(&domain)?;
                print_simulation(&report, args.mitigate);
            } else {
                let map = describe_hijack_resilience(&vault)?;
                for (_domain, report) in map {
                    print_simulation(&report, args.mitigate);
                }
            }
        }
        VaultCommand::Add(args) => {
            let metadata = read_metadata(args.metadata_path.as_ref())?;
            let secret = read_secret(&args.secret_path)?;
            let profile = VaultDomainProfile {
                domain: args.domain,
                registrar: args.registrar,
                expiration: args.expiration,
                auto_renew: true,
                dnssec_enabled: false,
                registrar_lock: false,
                blockchain_registry: None,
                metadata,
            };
            vault.register_domain(profile, secret)?;
            println!("Asset added to vault.");
        }
        VaultCommand::Renew(args) => {
            vault.renew_domain(&args.domain, &args.expiration)?;
            println!("Domain renewal recorded.");
        }
        VaultCommand::Lock(args) => {
            vault.lock_domain(&args.domain, !args.unlock)?;
            println!(
                "Domain {} registrar lock {}.",
                args.domain,
                if args.unlock { "disabled" } else { "enabled" }
            );
        }
        VaultCommand::Audit(args) => {
            let detail = args
                .detail
                .as_ref()
                .map(|s| serde_json::from_str(s))
                .transpose()
                .context("parse audit detail")?
                .unwrap_or_else(|| serde_json::Value::String("manual audit".to_string()));
            vault.audit(&args.category, detail)?;
            println!("Audit entry appended.");
        }
        VaultCommand::Watch(args) => loop {
            let status = vault.generate_status()?;
            if use_tui {
                render_tui(&status)?;
            } else {
                print_status(&status);
            }
            if args.once {
                break;
            }
            thread::sleep(Duration::from_secs(args.interval));
        },
        VaultCommand::Sync => {
            vault.sync()?;
            println!("Vault synchronized.");
        }
        VaultCommand::Backup(args) => {
            vault.backup_to(args.path)?;
            println!("Vault backup complete.");
        }
        VaultCommand::Recover(args) => {
            vault.recover_from(args.path)?;
            println!("Vault recovered from backup.");
        }
        VaultCommand::ExportMerkle => {
            let json = vault.export_merkle_tree()?;
            println!("{}", json);
        }
    }

    Ok(())
}

fn read_metadata(path: Option<&std::path::PathBuf>) -> Result<serde_json::Value> {
    match path {
        Some(p) => {
            let text = fs::read_to_string(p).context("read metadata file")?;
            let value = serde_json::from_str(&text).context("parse metadata json")?;
            Ok(value)
        }
        None => Ok(serde_json::Value::Object(serde_json::Map::new())),
    }
}

fn read_secret(path: &std::path::PathBuf) -> Result<serde_json::Value> {
    let text = fs::read_to_string(path).context("read secret file")?;
    let value = serde_json::from_str(&text).context("parse secret json")?;
    Ok(value)
}

fn print_status(status: &crate::vault::VaultStatusReport) {
    println!("--- Domain Quantum Vault Status ---");
    println!("Total domains: {}", status.total_domains);
    println!("DNSSEC enabled: {}", status.dnssec_enabled);
    println!("Registrar lock: {}", status.registrar_locked);
    println!("Auto renew: {}", status.auto_renew_enabled);
    println!("Blockchain anchored: {}", status.blockchain_backed);
    println!("Merkle root: {}", status.merkle_root);
    if !status.upcoming_expirations.is_empty() {
        println!("Upcoming expirations:");
        for (domain, date) in &status.upcoming_expirations {
            println!("  - {} => {}", domain, date);
        }
    }
}

fn print_simulation(report: &crate::vault::VaultSimulationReport, show_actions: bool) {
    println!("=== Simulation for {} ===", report.domain);
    println!("Resilience score: {:.2}", report.titan_resilience_score);
    println!("Attack surface:");
    for item in &report.attack_surface {
        println!("  - {}", item);
    }
    if show_actions {
        println!("Mitigations:");
        for action in &report.mitigation_actions {
            println!("  * {}", action);
        }
    }
    if let Some(script) = &report.qube_script {
        println!("Suggested QUBE policy:\n{}", script);
    }
}

fn render_tui(status: &crate::vault::VaultStatusReport) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let expirations: Vec<ListItem> = status
        .upcoming_expirations
        .iter()
        .map(|(domain, date)| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{}", domain)),
                Span::styled(format!("  {}", date), Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect();

    let summary_lines = vec![
        Line::from(format!("Total domains: {}", status.total_domains)),
        Line::from(format!("DNSSEC enabled: {}", status.dnssec_enabled)),
        Line::from(format!("Registrar lock: {}", status.registrar_locked)),
        Line::from(format!("Auto renew: {}", status.auto_renew_enabled)),
        Line::from(format!("Blockchain anchored: {}", status.blockchain_backed)),
        Line::from(format!("Merkle root: {}", status.merkle_root)),
    ];

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(7), Constraint::Min(3)].as_ref())
            .split(f.size());

        let header = Paragraph::new(summary_lines.clone())
            .block(Block::default().borders(Borders::ALL).title("Vault Status"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        let list = List::new(expirations.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Expirations (press q to exit)"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(list, chunks[1]);
    })?;

    loop {
        if event::poll(Duration::from_millis(500))? {
            if let event::Event::Key(key) = event::read()? {
                if matches!(key.code, event::KeyCode::Char('q') | event::KeyCode::Esc) {
                    break;
                }
            }
        } else {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
