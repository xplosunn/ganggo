use std::{
  cmp::{max, min},
  error::Error,
  io::{self, BufRead},
  vec,
};

use termion::{
  async_stdin,
  event::{self, Key},
  raw::IntoRawMode,
};
use tui::{
  backend::TermionBackend,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
  Terminal,
};

use std::io::Read;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

struct AppState {
  matcher: SkimMatcherV2,
  raw_items: Vec<String>,
  filtered_items: Vec<String>,
  filter_masks: Vec<Vec<usize>>,
  out_selection: String,
  search_str: String,
}

impl AppState {
  fn filtered(&self) -> Vec<(usize, &String)> {
    match self.filter_masks.last() {
      Some(mask) => mask.iter().map(|&i| (i, &self.raw_items[i])).collect(),
      None => self.raw_items.iter().enumerate().collect(),
    }
  }
}

struct UiState {
  selection_state: ListState,
}

fn render_selection<'a>(raw_items: &Vec<String>) -> List<'a> {
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

fn selection_up(ui_state: &mut UiState, app_state: &mut AppState) {
  if let Some(selected) = ui_state.selection_state.selected() {
    let bounded_select = min(max(0, selected), app_state.filtered_items.len());
    let prev = if (bounded_select as i32) - 1 < 0 {
      app_state.filtered_items.len() - 1
    } else {
      bounded_select - 1
    };
    ui_state.selection_state.select(Some(prev));
  }
}

fn selection_down(ui_state: &mut UiState, app_state: &mut AppState) {
  if let Some(selected) = ui_state.selection_state.selected() {
    let bounded_select = min(max(0, selected), app_state.filtered_items.len());
    ui_state
      .selection_state
      .select(Some((bounded_select + 1) % app_state.filtered_items.len()));
  }
}

fn select_current(ui_state: &mut UiState, app_state: &mut AppState) {
  if let Some(selected) = ui_state.selection_state.selected() {
    let current = app_state.filtered();
    if selected < current.len() {
      let selected_item = &current[selected];
      app_state.out_selection = selected_item.1.clone();
    } else {
      app_state.out_selection = app_state.search_str.clone();
    }
  }
}

enum FilterUpdate {
  Append { c: char },
  Backspace,
}

fn update_filter(app_state: &mut AppState, update: FilterUpdate) {
  match update {
    FilterUpdate::Append { c } => {
      app_state.search_str.push(c);
      let updated_filter: Vec<(usize, &String)> = app_state
        .filtered()
        .into_iter()
        .filter(|&entry| {
          app_state
            .matcher
            .fuzzy_match(entry.1, &app_state.search_str)
            .is_some()
        })
        .collect();

      let idx: Vec<usize> = updated_filter.iter().map(|&entry| entry.0).collect();
      app_state.filtered_items = updated_filter
        .iter()
        .map(|&entry| entry.1.clone())
        .collect();

      app_state.filter_masks.push(idx);
    }
    FilterUpdate::Backspace => {
      app_state.search_str.pop();
      app_state.filter_masks.pop();
      let updated_filter: Vec<(usize, &String)> = app_state.filtered();
      app_state.filtered_items = updated_filter
        .iter()
        .map(|&entry| entry.1.clone())
        .collect();
    }
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  let init: Vec<String> = io::stdin().lock().lines().map(|s| s.unwrap()).collect();

  let mut app_state = AppState {
    matcher: SkimMatcherV2::default(),
    raw_items: init.clone(),
    filtered_items: init.clone(),
    filter_masks: vec![],
    out_selection: "".to_string(),
    search_str: "".to_string(),
  };

  let mut ui_state = UiState {
    selection_state: ListState::default(),
  };

  // Terminal initialization
  let mut stdin = async_stdin().bytes();
  let stdout = io::stdout().into_raw_mode()?;
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  ui_state.selection_state.select(Some(0));

  //clear the terminal on startup
  terminal.clear()?;

  let loop_out: Result<bool, Box<dyn Error>> = loop {
    terminal.draw(|f| {
      // Create two chunks with equal horizontal screen space
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(f.size());

      let search_chunk = chunks[0];
      let list_chunk = chunks[1];

      let search_input = Paragraph::new(app_state.search_str.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Search"));

      let selection = render_selection(&app_state.filtered_items);
      f.render_widget(search_input, search_chunk);
      f.render_stateful_widget(selection, list_chunk, &mut ui_state.selection_state);
    })?;

    let key_input_maybe = stdin.next();

    match key_input_maybe {
      Some(key_input_either) => match key_input_either {
        Ok(key_input) => {
          match termion::event::parse_event(key_input, &mut stdin) {
            Ok(event) => {
              match event {
		//For some reason esc doesn't parse properly
                event::Event::Key(Key::Esc) => {
                  break Ok(false);
                }
                event::Event::Key(Key::Down) => {
                  selection_down(&mut ui_state, &mut app_state);
                }
                event::Event::Key(Key::Up) => {
                  selection_up(&mut ui_state, &mut app_state);
                }
                event::Event::Key(Key::Char('\n')) => {
                  select_current(&mut ui_state, &mut app_state);
                  break Ok(true);
                }
                event::Event::Key(Key::Char(ch)) => {
                  update_filter(&mut app_state, FilterUpdate::Append { c: ch })
                }
                event::Event::Key(Key::Backspace) => {
                  update_filter(&mut app_state, FilterUpdate::Backspace)
                }
                _ => {}
              }
            }
	    //so we handle esc here 
            Err(key_parse_err) => {
	      if key_input == 27 {//ESC 
		break Ok(false);
	      } else {
		break Err(Box::new(key_parse_err));
	      }
	    }
          }
        }
        _ => {}
      },
      _ => {}
    }
  };

  //clear terminal and print selected item
  terminal.clear()?;
  if loop_out? {
    eprint!("{}", app_state.out_selection);
  }
  Ok(())
}
