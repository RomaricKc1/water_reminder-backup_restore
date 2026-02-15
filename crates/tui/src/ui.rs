use ratatui::Frame;
use ratatui::layout::{Constraint, HorizontalAlignment, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{self, Line, Span};
use ratatui::widgets::{
    Block, BorderType, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap,
};

use lib::{Focused, JobState, ServerState, TuiApp};

pub fn render(frame: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::vertical([Constraint::Min(0)]).split(frame.area());
    draw_main(frame, app, chunks[0])
}

fn draw_main(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Percentage(25),
        Constraint::Percentage(70),
        Constraint::Percentage(25),
    ])
    .split(area);
    draw_header(frame, app, chunks[0]);
    draw_main_view(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);
}

fn draw_main_view(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    draw_files_view(frame, app, chunks[0]);

    draw_content_view(frame, app, chunks[1]);
}

fn draw_header(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let mut logs_span = vec![];
    let mut log_lines: Vec<Line<'static>> = vec![];

    for entry in app.logs.iter().clone() {
        let span1 = entry.date.clone().blue();
        let span = span1 + entry.text.clone().white().add_modifier(Modifier::BOLD);
        logs_span.push(span);
    }
    for span in logs_span {
        log_lines.push(span);
    }

    let block = match app.focused {
        Focused::Logs => Block::bordered()
            .border_style(Style::default().fg(Color::Green))
            .border_type(BorderType::HeavyTripleDashed)
            .title(Span::styled(
                "=== App logs ===".to_string(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
        _ => Block::bordered()
            .border_type(BorderType::Thick)
            .title(Span::styled(
                "App logs".to_string(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )),
    };

    app.scroll_logs_state = app.scroll_logs_state.content_length(log_lines.len());
    let paragraph = Paragraph::new(log_lines)
        .block(block)
        .scroll((app.scroll_logs, 0))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.scroll_logs_state,
    );
}

fn draw_files_view(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let files_elm: Vec<ListItem> = app
        .loaded_files
        .items
        .iter()
        .map(|i| {
            if i.clone() == app.serving_file.clone() {
                ListItem::new(vec![(Span::raw(i.clone()) + Span::raw(" serving"))])
            } else {
                ListItem::new(vec![text::Line::from(Span::raw(i.clone()))])
            }
        })
        .collect();

    let block = match app.focused {
        Focused::Files => Block::bordered()
            .border_style(Style::default().fg(Color::Green))
            .border_type(BorderType::HeavyTripleDashed)
            .title(
                Span::styled(
                    "=== Loaded files ===".to_string(),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .add_modifier(Modifier::BOLD)
                .fg(Color::Blue),
            ),

        _ => Block::bordered()
            .border_type(BorderType::Thick)
            .title(Span::styled(
                "Loaded files".to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))
            .add_modifier(Modifier::BOLD)
            .fg(Color::Blue),
    };

    let files = List::new(files_elm)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Red))
        .highlight_symbol("> ");
    frame.render_stateful_widget(files, area, &mut app.loaded_files.state);
}

fn draw_content_view(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let file_content = app.get_file_content();
    let mut filename: String = "".into();
    if let Some(selected_file_idx) = app.loaded_files.state.selected() {
        filename = app.loaded_files.items[selected_file_idx].clone() + " ";
    }

    let block = match app.focused {
        Focused::Content => Block::bordered()
            .border_style(Style::default().fg(Color::Green))
            .border_type(BorderType::HeavyTripleDashed)
            .title(Span::styled(
                "=== Content ===".to_string() + filename.as_str(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),

        _ => Block::bordered()
            .border_type(BorderType::Thick)
            .title(Span::styled(
                "Content".to_string() + filename.as_str(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
    };

    app.scroll_content_state = app.scroll_content_state.content_length(file_content.len());
    let paragraph0 = Paragraph::new(file_content)
        .block(block)
        .alignment(HorizontalAlignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_content, 0));

    frame.render_widget(paragraph0, area);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.scroll_content_state,
    );
}

fn draw_footer(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let text = vec![
        text::Line::from(vec![
            Span::styled(
                "Configured address: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("http://{}:{}", app.addr.ip(), app.addr.port()),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
        ]),
        text::Line::from(vec![
            Span::styled(
                "Server state: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                match app.server_state {
                    ServerState::Running => "Running",
                    ServerState::Stopped => "Stopped",
                    ServerState::Start => "Init",
                },
                Style::default().add_modifier(Modifier::REVERSED),
            ),
        ]),
        text::Line::from(vec![
            Span::styled(
                "Server job state: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                match app.server_job {
                    JobState::BaseStart => "Base starting. Press any key to continue",
                    JobState::BaseRunning => "Base running",
                    JobState::FullStart => "Base and file server starting",
                    JobState::FullStop => "File server stopped",
                    JobState::FullRunning => "Base and file server running",
                },
                Style::default().add_modifier(Modifier::REVERSED),
            ),
        ]),
        text::Line::from(""),
        text::Line::from(vec![
            Span::styled("<j>", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(", down", Style::default().add_modifier(Modifier::REVERSED)),
            Span::styled(", <k> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("up", Style::default().add_modifier(Modifier::REVERSED)),
            Span::styled(", <l> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "focus pane right",
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(", <h> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "focus pane left",
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(", <G> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "End of file",
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(", <g> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "Beginning of file",
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(", <v> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "view file",
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(", <s> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                "serve file",
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::styled(", <q> ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("quit.", Style::default().add_modifier(Modifier::REVERSED)),
            Span::raw(" "),
        ]),
    ];

    let block = Block::bordered()
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Thick)
        .title(Span::styled(
            "Info",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
