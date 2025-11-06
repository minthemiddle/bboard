use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, Selection};
use crate::input::Mode;

pub struct UI {
    list_state: ListState,
}

impl UI {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Status bar
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Mode line
            ])
            .split(frame.area());

        self.render_status_bar::<B>(frame, app, chunks[0]);
        self.render_main_content::<B>(frame, app, chunks[1]);
        self.render_mode_line::<B>(frame, app, chunks[2]);
    }

    fn render_status_bar<B: Backend>(&self, frame: &mut Frame, app: &App, area: Rect) {
        let status_text = if app.state.is_searching_places {
            vec![
                Span::styled("Jump to: ", Style::default().fg(Color::Green)),
                Span::styled(&app.state.place_search_buffer, Style::default().fg(Color::White)),
                Span::raw(" (type to filter, ↑/↓ to select, Enter to jump, Esc to cancel)"),
            ]
        } else {
            match app.state.mode {
                Mode::Edit => {
                    vec![
                        Span::styled("Editing: ", Style::default().fg(Color::Yellow)),
                        Span::styled(&app.state.edit_buffer, Style::default().fg(Color::White)),
                        Span::raw(" (Enter to save, Esc to cancel)"),
                    ]
                }
                Mode::Connect => {
                    vec![
                        Span::styled("Connect to: ", Style::default().fg(Color::Cyan)),
                        Span::styled(&app.state.connection_search_buffer, Style::default().fg(Color::White)),
                        Span::raw(" (↑/↓ to select, Enter to connect, Esc to cancel)"),
                    ]
                }
                Mode::OpenFile => {
                    vec![
                        Span::styled("Select file to open: ", Style::default().fg(Color::Magenta)),
                        Span::raw(" (↑/↓ to select, Enter to open, Esc to cancel)"),
                    ]
                }
                _ => {
                    vec![
                        Span::styled(
                            format!("Board: {} ", app.breadboard.name),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(
                            format!("Places: {} ", app.breadboard.places.len()),
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(
                            "(type to search) ",
                            Style::default().fg(Color::Gray),
                        ),
                    ]
                }
            }
        };

        let status_line = Line::from(status_text);
        let status_bar = Paragraph::new(status_line)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(status_bar, area);
    }

    fn render_main_content<B: Backend>(&mut self, frame: &mut Frame, app: &mut App, area: Rect) {
        if app.breadboard.places.is_empty() {
            self.render_empty_state::<B>(frame, area);
            return;
        }

        if app.state.mode == Mode::Connect {
            self.render_connection_search::<B>(frame, app, area);
        } else if app.state.mode == Mode::OpenFile {
            self.render_file_selection::<B>(frame, app, area);
        } else if app.state.is_searching_places {
            self.render_place_search::<B>(frame, app, area);
        } else if app.state.collapsed {
            self.render_collapsed_view::<B>(frame, app, area);
        } else {
            self.render_expanded_view::<B>(frame, app, area);
        }
    }

    fn render_empty_state<B: Backend>(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from("No places yet. Press Ctrl+N to create a place."),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("  Ctrl+N - New place"),
            Line::from("  Ctrl+O - Open file"),
            Line::from("  Ctrl+Q - Quit"),
        ];

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }

    fn render_expanded_view<B: Backend>(&mut self, frame: &mut Frame, app: &App, area: Rect) {
        let mut items = Vec::new();

        // Precompute all incoming connections once for performance
        let mut incoming_sources: std::collections::HashMap<uuid::Uuid, Vec<String>> = std::collections::HashMap::new();
        for place in &app.breadboard.places {
            for affordance in &place.affordances {
                if let Some(dest_id) = &affordance.connects_to {
                    incoming_sources.entry(*dest_id)
                        .or_insert_with(Vec::new)
                        .push(place.name.clone());
                }
            }
        }

        for (place_index, place) in app.breadboard.places.iter().enumerate() {
            let incoming_names = incoming_sources.get(&place.id);

            // Place header with incoming connections indicator
            let place_style = if app.state.selection == Some(Selection::Place(place.id)) {
                Style::default().bg(Color::Blue).fg(Color::Black)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let place_header = if let Some(names) = incoming_names {
                if names.is_empty() {
                    format!("┌─ {}", place.name)
                } else {
                    format!("┌─ {} (← {})", place.name, names.join(", "))
                }
            } else {
                format!("┌─ {}", place.name)
            };

            items.push(ListItem::new(Line::from(Span::styled(place_header, place_style))));

            // Affordances
            for affordance in &place.affordances {
                let affordance_style = if app.state.selection == Some(Selection::Affordance {
                    place_id: place.id,
                    affordance_id: affordance.id
                }) {
                    Style::default().bg(Color::Blue).fg(Color::Black)
                } else {
                    Style::default().fg(Color::White)
                };

                let affordance_text = if let Some(dest_id) = &affordance.connects_to {
                    if let Some(dest_place) = app.breadboard.find_place(dest_id) {
                        format!("├─ {} → {}", affordance.name, dest_place.name)
                    } else {
                        format!("├─ {} → [Unknown]", affordance.name)
                    }
                } else {
                    format!("├─ {}", affordance.name)
                };

                items.push(ListItem::new(Line::from(Span::styled(affordance_text, affordance_style))));
            }

            // Add spacing between places
            if place_index < app.breadboard.places.len() - 1 {
                items.push(ListItem::new(""));
            }
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Breadboard"))
            .highlight_style(Style::default());

        // Update list state for scrolling
        if let Some(selected_index) = app.get_selected_item_index() {
            self.list_state.select(Some(selected_index));
        }

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_collapsed_view<B: Backend>(&self, frame: &mut Frame, app: &App, area: Rect) {
        let mut items = Vec::new();

        // Determine which places to show based on filter
        let places_to_show: Vec<_> = if let Some("connected") = app.state.filter.as_deref() {
            // Show only places connected to the currently selected place
            // Get the place ID whether we're on a place or an affordance
            let selected_id = match app.state.selection {
                Some(Selection::Place(id)) => Some(id),
                Some(Selection::Affordance { place_id, .. }) => Some(place_id),
                None => None,
            };

            if let Some(selected_id) = selected_id {
                let mut connected_places = std::collections::HashSet::new();

                // Add outgoing connections
                if let Some(selected_place) = app.breadboard.find_place(&selected_id) {
                    for affordance in &selected_place.affordances {
                        if let Some(dest_id) = &affordance.connects_to {
                            connected_places.insert(*dest_id);
                        }
                    }
                }

                // Add incoming connections
                for incoming in app.breadboard.get_incoming_connections(&selected_id) {
                    connected_places.insert(incoming.0.id);
                }

                connected_places.insert(selected_id); // Include the selected place itself

                app.breadboard.places.iter()
                    .filter(|p| connected_places.contains(&p.id))
                    .collect()
            } else {
                app.breadboard.places.iter().collect()
            }
        } else {
            app.breadboard.places.iter().collect()
        };

        // Precompute incoming connection sources for performance
        let mut incoming_sources: std::collections::HashMap<uuid::Uuid, Vec<String>> = std::collections::HashMap::new();
        for place in &app.breadboard.places {
            for affordance in &place.affordances {
                if let Some(dest_id) = &affordance.connects_to {
                    incoming_sources.entry(*dest_id)
                        .or_insert_with(Vec::new)
                        .push(place.name.clone());
                }
            }
        }

        for place in places_to_show {
            let incoming_names = incoming_sources.get(&place.id);
            let outgoing_connections: Vec<_> = place.affordances.iter()
                .filter_map(|a| a.connects_to.as_ref())
                .filter_map(|dest_id| app.breadboard.find_place(dest_id))
                .collect();

            let place_style = if app.state.selection == Some(Selection::Place(place.id)) {
                Style::default().bg(Color::Blue).fg(Color::Black)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let mut place_info = format!("{} ({})", place.name, place.affordances.len());

            if let Some(names) = incoming_names {
                if !names.is_empty() {
                    place_info.push_str(&format!(" ← {}", names.join(", ")));
                }
            }

            if !outgoing_connections.is_empty() {
                let dest_names: Vec<_> = outgoing_connections.iter()
                    .map(|p| p.name.as_str())
                    .collect();
                place_info.push_str(&format!(" → {}", dest_names.join(", ")));
            }

            items.push(ListItem::new(Line::from(Span::styled(place_info, place_style))));
        }

        let title = if app.state.filter.is_some() {
            "Breadboard (Filtered)"
        } else {
            "Breadboard (Collapsed)"
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title));

        frame.render_widget(list, area);
    }

    fn render_mode_line<B: Backend>(&self, frame: &mut Frame, app: &App, area: Rect) {
        let mode_text = match app.state.mode {
            Mode::Navigate => "NAVIGATE",
            Mode::Edit => "EDIT",
            Mode::Connect => "CONNECT",
            Mode::OpenFile => "OPEN FILE",
        };

        let mode_style = match app.state.mode {
            Mode::Navigate => Style::default().fg(Color::Green),
            Mode::Edit => Style::default().fg(Color::Yellow),
            Mode::Connect => Style::default().fg(Color::Cyan),
            Mode::OpenFile => Style::default().fg(Color::Magenta),
        };

        let text = vec![
            Span::styled("Mode: ", Style::default().fg(Color::Gray)),
            Span::styled(mode_text, mode_style),
            Span::raw(" | "),
            Span::styled(
                if app.state.collapsed { "Collapsed" } else { "Expanded" },
                Style::default().fg(Color::Cyan),
            ),
        ];

        let mode_line = Line::from(text);
        let paragraph = Paragraph::new(mode_line);
        frame.render_widget(paragraph, area);
    }

    fn render_connection_search<B: Backend>(&self, frame: &mut Frame, app: &App, area: Rect) {
        let mut items = Vec::new();

        if app.state.connection_search_results.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "No places found",
                Style::default().fg(Color::Gray),
            ))));
        } else {
            for (index, place_id) in app.state.connection_search_results.iter().enumerate() {
                let is_selected = Some(index) == app.state.selected_connection_result;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                // Check if this is the remove connection option (using UUID 0)
                if *place_id == uuid::Uuid::from_u128(0) {
                    items.push(ListItem::new(Line::from(Span::styled(
                        "Remove connection",
                        style.fg(if is_selected { Color::White } else { Color::Red }),
                    ))));
                } else if let Some(place) = app.breadboard.find_place(place_id) {
                    items.push(ListItem::new(Line::from(Span::styled(
                        &place.name,
                        style,
                    ))));
                }
            }
        }

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Select place to connect to"));

        frame.render_widget(list, area);
    }

    fn render_file_selection<B: Backend>(&self, frame: &mut Frame, app: &App, area: Rect) {
        let mut items = Vec::new();

        if app.state.file_list.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "No TOML files found in current directory",
                Style::default().fg(Color::Gray),
            ))));
        } else {
            for (index, filename) in app.state.file_list.iter().enumerate() {
                let is_selected = Some(index) == app.state.selected_file_index;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                items.push(ListItem::new(Line::from(Span::styled(
                    filename,
                    style,
                ))));
            }
        }

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Select file to open"));

        frame.render_widget(list, area);
    }

    fn render_place_search<B: Backend>(&self, frame: &mut Frame, app: &App, area: Rect) {
        let mut items = Vec::new();

        if app.state.place_search_results.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "No places found",
                Style::default().fg(Color::Gray),
            ))));
        } else {
            for (index, place_id) in app.state.place_search_results.iter().enumerate() {
                let is_selected = Some(index) == app.state.selected_place_result;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                if let Some(place) = app.breadboard.find_place(place_id) {
                    items.push(ListItem::new(Line::from(Span::styled(
                        &place.name,
                        style,
                    ))));
                }
            }
        }

        let title = format!("Jump to place: {}", app.state.place_search_buffer);
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title));

        frame.render_widget(list, area);
    }
}