use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::watch;

pub(crate) mod ui;

use lib::{AsyncComm, JobState, TuiApp, TuiError, route_now};

pub async fn run(
    tick_rate: Duration,
    ip: Vec<u8>,
    port: u16,
    path: String,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    match TuiApp::new("any title", ip, port, path) {
        Ok(app) => {
            let app_result = run_app(&mut terminal, app, tick_rate).await;

            // restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            if let Err(err) = app_result {
                println!("{err:?}");
            }
        }

        Err(err) => {
            // restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            match err {
                TuiError::BadIp => println!("Bad ip."),
                _ => {
                    println!("{err:?}");
                }
            }
        }
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: TuiApp,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    let (stop_tx, stop_rx) = watch::channel(false);

    let mut comm = AsyncComm {
        stop_tx,
        stop_rx,
        job_handle: None,
    };

    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if !event::poll(timeout)? {
            app.on_tick();
            last_tick = Instant::now();
            continue;
        }
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('h') | KeyCode::Left => app.on_left(),
                KeyCode::Char('j') | KeyCode::Down => app.on_down(),
                KeyCode::Char('k') | KeyCode::Up => app.on_up(),
                KeyCode::Char('l') | KeyCode::Right => app.on_right(),
                KeyCode::Enter => app.on_enter(),
                KeyCode::Tab => app.on_tab(),
                KeyCode::Char(c) => app.on_key(c),

                _ => {}
            }
        }
        if app.should_quit {
            return Ok(());
        }

        let serving_file = app.serving_file.clone();

        match app.server_job {
            JobState::BaseStart => {
                app.add_log("Starting the base server.".to_string());

                match route_now(app.addr, None, comm.stop_rx.clone()).await {
                    Ok(handle) => {
                        comm.job_handle = Some(handle);
                        app.server_job = JobState::BaseRunning;
                    }
                    Err(_) => {
                        app.add_log(
                            "Cannot start base server. Bind failed. Restarting...".to_string(),
                        );
                        app.server_job = JobState::BaseStart;
                    }
                }
            }

            JobState::FullStart => {
                // stop and restart with filesharing
                if let Some(ref job_handle) = comm.job_handle {
                    job_handle.abort();
                    app.add_log("Starting the base and file server.".to_string());

                    match route_now(app.addr, serving_file.into(), comm.stop_rx.clone()).await {
                        Ok(handle) => {
                            comm.job_handle = Some(handle);
                            app.server_job = JobState::FullRunning;
                        }
                        Err(_) => {
                            app.add_log(
                                "Cannot star full server. Bind failed. Restarting...".to_string(),
                            );
                            app.server_job = JobState::FullStart;
                        }
                    }
                } else {
                    app.add_log("could not find handle. File server not started".to_string());
                }
            }

            JobState::FullStop => {
                // stop filesharing and restart base
                if let Some(ref job_handle) = comm.job_handle {
                    job_handle.abort();
                    app.add_log("file serving job stopped.".to_string());
                    app.server_job = JobState::BaseStart;
                } else {
                    app.add_log(
                        "could not find handle. File server not stoped and base not running?"
                            .to_string(),
                    );
                }
            }

            JobState::BaseRunning | JobState::FullRunning => {}
        };
    }
}
