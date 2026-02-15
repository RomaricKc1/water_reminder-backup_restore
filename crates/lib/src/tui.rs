use chrono::prelude::*;
use ratatui::style::{Modifier, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{ListState, ScrollbarState};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::errors::{RouteListenErr, TuiError};
use crate::{RouteDetails, csv_ops, read_files_wrapper, route_details};

/// TUI app
pub struct TuiApp {
    pub should_quit: bool,
    pub loaded_files: StatefulList<String>,
    pub logs: Vec<LogElm>,
    pub focused: Focused,
    pub scroll_content: u16,
    pub scroll_content_state: ScrollbarState,
    pub scroll_logs: u16,
    pub scroll_logs_state: ScrollbarState,
    pub current_file_lines: Vec<String>,
    pub file_state: FileState,
    pub server_state: ServerState,
    pub serving_file: String,
    pub addr: SocketAddr,
    pub server_job: JobState,
}

#[derive(Default)]
pub struct LogElm {
    pub date: String,
    pub text: String,
}

#[derive(Default)]
pub struct LoadFilesArgs {
    pub current_path: String,
    pub msg_success: String,
    pub msg_failure: String,
}

pub enum JobState {
    BaseStart,
    BaseRunning,
    FullRunning,
    FullStart,
    FullStop,
}

pub enum ServerState {
    Running,
    Stopped,
    Start,
}

pub enum FileState {
    Loaded(String),
    Error(String),
    Unknown,
}

pub enum Focused {
    Logs,
    Files,
    Content,
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl<'a> TuiApp {
    pub fn new(
        title: &'a str,
        ip: Vec<u8>,
        port: u16,
        loading_path: String,
    ) -> Result<Self, TuiError> {
        _ = title;
        if ip.len() != 4 {
            return Err(TuiError::BadIp);
        }

        if port == 0 {
            return Err(TuiError::BadPort);
        }

        let octets: [u8; 4] = [ip[0], ip[1], ip[2], ip[3]];
        let ip = Ipv4Addr::from(octets);

        let loaded_files = match read_files_wrapper(loading_path) {
            Ok(files) => files.to_vec(),
            Err(_) => [
                "file1.csv".into(),
                "./test/test.csv".into(),
                "file3.csv".into(),
            ]
            .to_vec(),
        };
        // println!("{:#?}", loaded_files);

        Ok(TuiApp {
            should_quit: false,
            loaded_files: StatefulList::with_items(loaded_files),
            addr: SocketAddr::from((ip, port)),
            logs: [LogElm::default(), LogElm::default(), LogElm::default()].into(),
            focused: Focused::Files,
            scroll_content: 0,
            scroll_content_state: ScrollbarState::default(),
            scroll_logs_state: ScrollbarState::default(),
            scroll_logs: 0,
            current_file_lines: vec![],
            file_state: FileState::Unknown,
            server_state: ServerState::Start,
            serving_file: "".into(),
            server_job: JobState::BaseStart,
        })
    }

    pub fn any_file_load(&mut self, args: LoadFilesArgs) {
        match csv_ops::read_csv_to_vec(args.current_path.clone()) {
            Ok(res) => {
                self.current_file_lines = res;
                self.file_state = FileState::Loaded(args.current_path.clone());
                self.add_log(args.msg_success.to_string());
            }
            Err(e) => {
                self.add_log(format!("{}: {}", args.msg_failure, e));
                self.file_state = FileState::Error(args.current_path);
                self.current_file_lines = vec![];
            }
        }
    }

    pub fn load_current_file_content(&mut self) {
        let current_path = self.get_selected_file();

        match &self.file_state {
            FileState::Loaded(_file) => {
                let file = _file.clone();
                if file != current_path {
                    self.any_file_load(LoadFilesArgs {
                        current_path: current_path.clone(),
                        msg_success: format!("l: Load successfull. Viewing {}", current_path),
                        msg_failure: format!("l: Error occured while reading {}", current_path),
                    });
                }
            }
            FileState::Unknown => {
                let path = self.get_selected_file();
                self.any_file_load(LoadFilesArgs {
                    current_path: path.clone(),
                    msg_success: format!("u: Load successfull. Viewing {}", path),
                    msg_failure: format!("u: Error occured while reading {}", path),
                });
            }
            FileState::Error(_file) => {
                let file = _file.clone();
                if file != current_path {
                    self.any_file_load(LoadFilesArgs {
                        current_path: current_path.clone(),
                        msg_success: format!("e: Load successfull. Viewing {}", current_path),
                        msg_failure: format!("e: Error occured while reading {}", current_path),
                    });
                }
            }
        }
    }

    pub fn serve_current_file(&mut self) {
        match self.server_state {
            ServerState::Running => {
                self.add_log("Were running. Stopping.".to_string());
                self.server_state = ServerState::Stopped;
                self.serving_file = "".into();
                self.server_job = JobState::FullStop;
            }

            ServerState::Stopped | ServerState::Start => {
                self.add_log("Were stopped (+init, or stopped). Reserving.".to_string());

                let current_path = self.get_selected_file();

                match &self.file_state {
                    FileState::Loaded(_file) => {
                        self.add_log(format!("Successfully serving {current_path}."));
                        self.serving_file = current_path;
                        self.server_state = ServerState::Running;
                        self.server_job = JobState::FullStart;
                    }
                    _ => {
                        self.add_log(format!("File not loaded {current_path}, serve failed"));
                        self.server_state = ServerState::Stopped;
                        self.serving_file = "".into();
                    }
                }
            }
        }
    }

    pub fn get_selected_file(&mut self) -> String {
        let mut filename: String = "".into();
        if let Some(selected_file_idx) = self.loaded_files.state.selected() {
            filename = self.loaded_files.items[selected_file_idx].clone();
        }

        filename
    }

    pub fn add_log(&mut self, text: String) {
        let now = Local::now();
        let fmt_datetime = format!("{} :: ", now.format("%d/%B/%Y %H:%M:%S"));

        let logelm = LogElm {
            date: fmt_datetime,
            text,
        };

        self.logs.push(logelm);

        let len_elm = self.logs.len() as u16;
        if self.scroll_logs < len_elm {
            self.scroll_logs = self.scroll_logs.wrapping_add(1);
        }
        self.scroll_logs_state = self.scroll_logs_state.position(self.scroll_logs as usize);
    }

    pub fn get_file_content(&mut self) -> Vec<Line<'static>> {
        let current_path = self.get_selected_file();
        if !current_path.is_empty() {
            self.load_current_file_content();
        }

        let mut lines_elm = vec![];

        for line in self.current_file_lines.clone() {
            let mut lines_spans = vec![];
            let data: Vec<&str> = line.split(",").collect();
            if data.len() != 2 {
                self.add_log("Error. Expected csv files. eg: 1,42".into());
                return vec![Line::from(vec![])];
            }

            let (idx, timestamp) = (data[0].to_string(), data[1].to_string());

            let mut datetime: Option<String> = None;
            if let Some(date) =
                DateTime::from_timestamp_millis(timestamp.parse::<i64>().unwrap_or(0))
            {
                datetime = Some(format!("{}", date.format("%d/%B/%Y %H:%M:%S")));
            }

            lines_spans.push(idx.blue());
            lines_spans.push(" ".black());

            if let Some(datetime) = datetime {
                lines_spans.push(datetime.white().add_modifier(Modifier::BOLD))
            }
            lines_spans.push(" ".black());

            lines_elm.push(Line::from(lines_spans.clone()));
        }

        let mut ret = vec![];
        for line in lines_elm {
            ret.push(line);
        }

        ret
    }

    pub fn on_up(&mut self) {
        match self.focused {
            Focused::Logs => {
                if self.scroll_logs > 0 {
                    self.scroll_logs = self.scroll_logs.wrapping_sub(1);
                }
                self.scroll_logs_state = self.scroll_logs_state.position(self.scroll_logs as usize);
            }

            Focused::Content => {
                if self.scroll_content > 0 {
                    self.scroll_content = self.scroll_content.wrapping_sub(1);
                }
                self.scroll_content_state = self
                    .scroll_content_state
                    .position(self.scroll_content as usize);
            }

            Focused::Files => {
                self.loaded_files.previous();
            }
        }
    }

    pub fn on_enter(&mut self) {
        match self.focused {
            Focused::Logs => {
                //
            }
            Focused::Content => {
                //
            }

            Focused::Files => {
                //
            }
        }
    }

    pub fn on_down(&mut self) {
        match self.focused {
            Focused::Logs => {
                let len_elm = self.logs.len() as u16;
                if self.scroll_logs < len_elm {
                    self.scroll_logs = self.scroll_logs.wrapping_add(1);
                }
                self.scroll_logs_state = self.scroll_logs_state.position(self.scroll_logs as usize);
            }
            Focused::Content => {
                let len_elm = self.current_file_lines.len() as u16;
                if self.scroll_content < len_elm {
                    self.scroll_content = self.scroll_content.wrapping_add(1);
                }
                self.scroll_content_state = self
                    .scroll_content_state
                    .position(self.scroll_content as usize);
            }

            Focused::Files => {
                self.loaded_files.next();
            }
        }
    }

    pub fn on_right(&mut self) {
        match self.focused {
            Focused::Logs => {
                self.focused = Focused::Files;
            }
            Focused::Content => {
                self.focused = Focused::Logs;
            }

            Focused::Files => {
                self.focused = Focused::Content;
            }
        }
    }

    pub fn on_left(&mut self) {
        match self.focused {
            Focused::Logs => {
                self.focused = Focused::Content;
            }
            Focused::Content => {
                self.focused = Focused::Files;
            }

            Focused::Files => {
                self.focused = Focused::Logs;
            }
        }
    }

    pub fn on_tab(&mut self) {
        match self.focused {
            Focused::Logs => {
                self.focused = Focused::Files;
            }
            Focused::Content => {
                self.focused = Focused::Logs;
            }

            Focused::Files => {
                self.focused = Focused::Content;
            }
        }
    }

    pub fn on_scroll_end(&mut self) {
        match self.focused {
            Focused::Logs => {
                self.scroll_logs = self.logs.len() as u16;
                self.scroll_logs_state = self.scroll_logs_state.position(self.scroll_logs as usize);
            }
            Focused::Content => {
                self.scroll_content = self.current_file_lines.len() as u16;
                self.scroll_content_state = self
                    .scroll_content_state
                    .position(self.scroll_content as usize);
            }

            Focused::Files => {
                //
            }
        }
    }

    pub fn on_scroll_begin(&mut self) {
        match self.focused {
            Focused::Logs => {
                self.scroll_logs = 0;
                self.scroll_logs_state = self.scroll_logs_state.position(self.scroll_logs as usize);
            }
            Focused::Content => {
                self.scroll_content = 0;
                self.scroll_content_state = self
                    .scroll_content_state
                    .position(self.scroll_content as usize);
            }

            Focused::Files => {
                //
            }
        }
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            's' => {
                self.serve_current_file();
            }
            'G' => {
                self.on_scroll_end();
            }
            'g' => {
                self.on_scroll_begin();
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        //
    }
}

pub async fn route_now(
    addr: SocketAddr,
    serving_file: Option<String>,
    stop_rx: watch::Receiver<bool>,
) -> Result<JoinHandle<()>, RouteListenErr> {
    match route_details(addr, serving_file).await {
        Ok(details) => Ok(tokio::spawn(async move {
            serve_axum_app(stop_rx, details).await;
        })),
        Err(_) => Err(RouteListenErr::BindError),
    }
}

pub async fn serve_axum_app(stop_rx: watch::Receiver<bool>, details: RouteDetails) {
    let _ = axum::serve(details.listener, details.app).await;
    loop {
        if *stop_rx.borrow() {
            println!("router: received stop signal, exiting.");
            break;
        }
    }
}
