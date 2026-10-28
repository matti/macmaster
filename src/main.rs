use clap::{ArgAction, Parser};
use macmaster::apply::{Mode, apply};
use macmaster::hid::HidSystem;
use macmaster::prefs::CfPrefs;
use macmaster::report;
use macmaster::settings::SETTINGS;
use std::process::{Command, ExitCode};

/// Säätää tämän Macin valmiiksi macmasterin sisäänrakennetuilla asetuksilla.
///
/// Ilman argumentteja kaikki asetukset otetaan käyttöön nykyiselle
/// käyttäjälle. Ajon voi toistaa turvallisesti. Mitään ei tarvitse konfiguroida.
#[derive(Parser)]
#[command(
    version,
    override_usage = "macmaster [VALINNAT]",
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = "{about-with-newline}\nKäyttö: {usage}\n\nValinnat:\n{options}"
)]
struct Cli {
    /// Näytä mitä muuttuisi, kirjoittamatta mitään.
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Näytä tämä ohje.
    #[arg(short, long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Näytä versio.
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,
}

fn restart(process: &str) -> bool {
    // killall exits 1 when the process is not running; launchd relaunches Dock-like agents itself.
    match Command::new("/usr/bin/killall").arg(process).status() {
        Ok(_) => true,
        Err(err) => {
            eprintln!("VIRHE: {process} uudelleenkäynnistys epäonnistui: {err}");
            false
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mode = if cli.dry_run {
        Mode::DryRun
    } else {
        Mode::Apply
    };
    let outcome = apply(&mut CfPrefs, &mut HidSystem::new(), SETTINGS, mode);

    for entry in &outcome.entries {
        println!("{}", report::line(entry));
    }
    println!();
    for line in report::summary(&outcome) {
        println!("{line}");
    }

    let restarts_ok =
        mode == Mode::DryRun || outcome.processes_to_restart().into_iter().all(restart);
    if outcome.has_failures() || !restarts_ok {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
