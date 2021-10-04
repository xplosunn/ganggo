use std::{
  error::Error,
  io, thread,
  time::{Duration, Instant},
  vec,
};

use termion::{
  async_stdin,
  event::{self, Key},
  raw::IntoRawMode,
};
use tui::{
  backend::TermionBackend,
  layout::{Constraint, Corner, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
  Terminal,
};

use std::io::{stdout, Read, Write};

use std::sync::mpsc;

/*enum Event<I> {
  Input(I),
  Tick,
} */

fn render_pets<'a>(raw_items: &Vec<&'a str>, selection_state: &ListState) -> List<'a> {
  let selection_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::White))
    .title("Selection")
    .border_type(tui::widgets::BorderType::Plain);

  let items: Vec<_> = raw_items
    .iter()
    .map(|item| {
      ListItem::new(Spans::from(vec![Span::styled(
        item.clone(),
        Style::default(),
      )]))
    })
    .collect();
  let list = List::new(items).block(selection_block).highlight_style(
    Style::default()
      .bg(Color::Yellow)
      .fg(Color::Black)
      .add_modifier(Modifier::BOLD),
  );

  list
}

fn main() -> Result<(), Box<dyn Error>> {
  // Terminal initialization
  let mut stdin = async_stdin().bytes();
  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut selection_state = ListState::default();
  selection_state.select(Some(0));

  //clear the terminal on startup
  terminal.clear();

  let raw_items = vec!["first", "second"];

  let mut out_selection = "";

  let loopOut: Result<(), Box<dyn Error>> = loop {
    terminal.draw(|f| {
      // Create two chunks with equal horizontal screen space
      let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

      let list_chunk = chunks[0];

      let selection = render_pets(&raw_items, &selection_state);
      f.render_stateful_widget(selection, list_chunk, &mut selection_state);
    })?;

    let key_input_maybe = stdin.next();

    match key_input_maybe {
      Some(key_input_either) => match key_input_either {
        Ok(key_input) => {
          let event = termion::event::parse_event(key_input, &mut stdin)?;
          match event {
            event::Event::Key(Key::Char('q')) => break Ok(()),
            event::Event::Key(Key::Down) => {
              if let Some(selected) = selection_state.selected() {
                if (selected < raw_items.len() - 1) {
                  selection_state.select(Some(selected + 1));
                } else {
                  selection_state.select(Some(0));
                }
              }
            }
            event::Event::Key(Key::Up) => {
              if let Some(selected) = selection_state.selected() {
                if (selected > 0) {
                  selection_state.select(Some(selected - 1));
                } else {
                  selection_state.select(Some(raw_items.len() - 1));
                }
              }
            }
            event::Event::Key(Key::Char('\n')) => {
              if let Some(selected) = selection_state.selected() {
                let selected_item = raw_items[selected];
                out_selection = selected_item;
                break Ok(());
              }
            }
            _ => {}
          }
        }
        _ => {}
      },
      _ => {}
    }
  };

  //clear terminal and print selected item 
  terminal.clear();
  eprint!("{}", out_selection);
  Ok(())
}
