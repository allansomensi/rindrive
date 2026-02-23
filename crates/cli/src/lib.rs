use crate::args::Args;
use crate::error::CliError;
use clap::Parser;
use rindrive_core::engine::EngineType;
use rindrive_core::{detector, engine, platforms};
use rindrive_i18n::fl;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod args;
mod display;
mod error;

pub fn run() {
    let args = Args::parse();

    let result = match &args.target {
        Some(target) => run_direct_mode(target, &args),
        None => run_watcher_mode(&args),
    };

    if let Err(e) = result {
        match e {
            CliError::UserAbort => display::print_dimmed(&fl!("cli-op-cancelled")),
            _ => display::print_error(&fl!("cli-fatal-error"), &e.to_string()),
        }

        process::exit(e.exit_code());
    }
}

fn run_direct_mode(target: &str, args: &Args) -> Result<(), CliError> {
    if !display::confirm_action(&fl!("cli-confirm-audit", target = target))? {
        return Err(CliError::UserAbort);
    }
    perform_audit(target, args)
}

fn run_watcher_mode(args: &Args) -> Result<(), CliError> {
    let spinner = display::create_spinner(&fl!("cli-watcher-active"));
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut watcher = detector::DriveWatcher::new();
        watcher.watch(tx);
    });

    loop {
        if let Ok(detector::DriveEvent::Connected(mount_point)) =
            rx.recv_timeout(Duration::from_millis(100))
        {
            spinner.finish_and_clear();
            display::print_success(&fl!("cli-drive-detected"), &mount_point);

            if display::confirm_action(&fl!("cli-confirm-test"))? {
                perform_audit(&mount_point, args)?;
                return Ok(());
            } else {
                display::print_dimmed(&fl!("cli-ignored"));
                spinner.reset();
            }
        } else {
            spinner.tick();
        }
    }
}

fn perform_audit(mount_point: &str, args: &Args) -> Result<(), CliError> {
    display::print_info(&fl!("cli-opening-drive"), mount_point);

    let mut drive = platforms::open_drive(mount_point)
        .map_err(|e| rindrive_core::error::AuditError::Platform(e.to_string()))?;

    display::show_drive_details(mount_point, drive.size());

    match args.engine {
        EngineType::SpotCheck => {
            display::print_info(
                &fl!("cli-engine-label"),
                &fl!(
                    "cli-engine-info",
                    sections = (args.sections as i64),
                    buffer = (args.buffer_size as i64)
                ),
            );
            println!();

            let pb = display::create_audit_progress_bar();
            pb.enable_steady_tick(Duration::from_millis(100));

            let report_result =
                engine::spotcheck::run(&mut *drive, args.sections, args.buffer_size, |msg, pct| {
                    pb.set_message(msg.to_string());
                    let position = (pct * 10000.0) as u64;
                    pb.set_position(position);

                    true
                });

            pb.finish_and_clear();

            let report = report_result.map_err(|e| {
                rindrive_core::error::AuditError::Platform(fl!(
                    "cli-engine-failure",
                    error = e.to_string()
                ))
            })?;

            display::render_report(&report);
        }
        EngineType::FullScan => {
            todo!()
        }
    }

    Ok(())
}
