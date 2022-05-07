mod app;
use app::App;
use app::Data;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    fs::{self},
    io,
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
};
use tui::widgets::Paragraph;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame, Terminal,
};

fn main() -> Result<(), io::Error> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(1000);
    let file = "config/app.json";
    let data_string = match fs::read_to_string(file) {
        Ok(data) => data,
        Err(_) => {
            println!("Unable to read file '{}'", file);
            String::from("{}")
        }
    };
    let data: Data = match serde_json::from_str(&data_string) {
        Ok(data) => data,
        Err(_) => {
            println!("Malformed JSON");
            Data {
                title: "default",
                systems: vec![],
            }
        }
    };
    let mut app: App = App::new(data.title, data.systems);
    app.state.select(Some(0));
    let res = run_app(&mut terminal, app, tick_rate);

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            update_system_status(&mut app.systems);
            last_tick = Instant::now();
        }
    }
}

fn update_system_status(systems: &mut Vec<app::System<'_>>) {
    for system in systems {
        for port in &system.ports {
            let sock: SocketAddr = format!("{}:{}", system.address, port).parse().unwrap();
            let timeout = Duration::from_millis(50);
            match TcpStream::connect_timeout(&sock, timeout) {
                Ok(_) => {
                    system.status = "UP";
                    break;
                }
                Err(_) => system.status = "DOWN",
            }
        }
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let up_style = Style::default().fg(Color::Green);
    let down_style = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);

    let rows = app.systems.iter().map(|s| {
        let style = if s.status == "UP" {
            up_style
        } else {
            down_style
        };
        Row::new(vec![s.name, s.address, s.status]).style(style)
    });

    let header = Row::new(vec!["Name", "Address", "Status"])
        .style(Style::default().fg(Color::Yellow))
        .bottom_margin(1);

    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Systems").borders(Borders::ALL))
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Length(30),
            Constraint::Length(15),
            Constraint::Length(10),
        ]);
    f.render_stateful_widget(table, chunks[0], &mut app.state);

    let selected_system = app
        .systems
        .get(
            app.state
                .selected()
                .expect("Expected at least one system by default"),
        )
        .expect("msg");
    let data = Paragraph::new(selected_system.name)
        .block(Block::default().title("Info").borders(Borders::ALL));
    f.render_widget(data, chunks[1]);
}
