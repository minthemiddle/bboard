use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{EnableMouseCapture, DisableMouseCapture},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use std::io;

mod app;
mod models;
mod ui;
mod input;
mod file;

use app::{App, Selection};
use input::{InputHandler, Action, Mode};
use ui::UI;
use file::FileManager;
use anyhow::Result;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and UI
    let mut app = App::new();
    let mut ui = UI::new();
    let input_handler = InputHandler::new();
    let file_manager = FileManager::new();

    // Add sample breadboard data for testing
    let invoice_place = models::Place::new("Invoice".to_string());
    let invoice_id = invoice_place.id;
    app.breadboard.add_place(invoice_place);

    let setup_place = models::Place::new("Setup Autopay".to_string());
    let setup_id = setup_place.id;
    app.breadboard.add_place(setup_place);

    let confirm_place = models::Place::new("Confirm".to_string());
    let confirm_id = confirm_place.id;
    app.breadboard.add_place(confirm_place);

    // Add affordances with connections
    let turn_on_autopay = models::Affordance::new("Turn on Autopay".to_string())
        .with_connection(setup_id);
    app.add_affordance_to_place(&invoice_id, turn_on_autopay);

    let view_details = models::Affordance::new("View Details".to_string());
    app.add_affordance_to_place(&invoice_id, view_details);

    let cc_fields = models::Affordance::new("CC Fields".to_string())
        .with_connection(confirm_id);
    app.add_affordance_to_place(&setup_id, cc_fields);

    let cancel = models::Affordance::new("Cancel".to_string())
        .with_connection(invoice_id);
    app.add_affordance_to_place(&setup_id, cancel);

    let thank_you = models::Affordance::new("Thank You Message".to_string());
    app.add_affordance_to_place(&confirm_id, thank_you);

    // Set initial selection
    if let Some(first_place) = app.breadboard.places.first() {
        app.state.selection = Some(Selection::Place(first_place.id));
    }

    // Main event loop
    while !app.should_quit {
        terminal.draw(|f| ui.render::<CrosstermBackend<std::io::Stdout>>(f, &mut app))?;

        if let Ok(action) = input_handler.read_action(app.state.mode.clone()) {
            handle_action(&mut app, &file_manager, action)?;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_action(app: &mut App, file_manager: &FileManager, action: Action) -> Result<()> {
    match action {
        Action::Quit => app.should_quit = true,

        Action::NavigateUp => navigate_up(app),
        Action::NavigateDown => navigate_down(app),
        Action::NavigateRight => navigate_right(app),
        Action::NavigateLeft => navigate_left(app),

        Action::Select => handle_select(app, file_manager),
        Action::Back => handle_back(app),

        Action::NewPlace => handle_new_place(app),
        Action::NewAffordance => handle_new_affordance(app),
        Action::RemoveConnection => handle_remove_connection(app),

        Action::ToggleCollapsed => app.toggle_collapsed(),

        Action::Save => handle_save(app, file_manager)?,
        Action::Open => handle_enter_open_mode(app, file_manager)?,
        Action::EnterEditMode => handle_enter_edit_mode(app),
        Action::EnterConnectMode => handle_enter_connect_mode(app),
        Action::Delete => handle_delete(app),

        Action::Edit(text_change) => handle_edit(app, text_change),

        Action::Filter => {
            // Simple filter - toggle filtering by currently connected places
            app.state.filter = match app.state.filter.as_deref() {
                None => Some("connected".to_string()),
                Some("connected") => None,
                Some(_) => Some("connected".to_string()),
            };
        }

        Action::None => {}
    }

    Ok(())
}

fn navigate_up(app: &mut App) {
    match app.state.mode {
        Mode::Connect => {
            // Navigate up in connection search results
            if let Some(selected_index) = app.state.selected_connection_result {
                if selected_index > 0 {
                    app.state.selected_connection_result = Some(selected_index - 1);
                }
            }
        }
        Mode::OpenFile => {
            // Navigate up in file list
            if let Some(selected_index) = app.state.selected_file_index {
                if selected_index > 0 {
                    app.state.selected_file_index = Some(selected_index - 1);
                }
            }
        }
        Mode::Navigate => {
            if app.state.is_searching_places {
                // Navigate up in place search results
                if let Some(selected_index) = app.state.selected_place_result {
                    if selected_index > 0 {
                        app.state.selected_place_result = Some(selected_index - 1);
                    }
                }
            } else {
                // Up: Navigate within place hierarchy
                match &app.state.selection {
                    Some(Selection::Affordance { place_id, affordance_id }) => {
                        // If on an affordance, move to previous affordance or back to place
                        if let Some(place) = app.breadboard.find_place(place_id) {
                            if let Some(current_index) = place.affordances.iter().position(|a| &a.id == affordance_id) {
                                if current_index > 0 {
                                    // Move to previous affordance
                                    app.state.selection = Some(Selection::Affordance {
                                        place_id: *place_id,
                                        affordance_id: place.affordances[current_index - 1].id,
                                    });
                                } else {
                                    // Move back to the place itself
                                    app.state.selection = Some(Selection::Place(*place_id));
                                }
                            }
                        }
                    }
                    Some(Selection::Place(_)) => {
                        // On a place, up does nothing (or could go to previous place)
                    }
                    None => {
                        if let Some(first_place) = app.breadboard.places.first() {
                            app.state.selection = Some(Selection::Place(first_place.id));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn navigate_down(app: &mut App) {
    match app.state.mode {
        Mode::Connect => {
            // Navigate down in connection search results
            if let Some(selected_index) = app.state.selected_connection_result {
                if selected_index < app.state.connection_search_results.len() - 1 {
                    app.state.selected_connection_result = Some(selected_index + 1);
                }
            }
        }
        Mode::OpenFile => {
            // Navigate down in file list
            if let Some(selected_index) = app.state.selected_file_index {
                if selected_index < app.state.file_list.len() - 1 {
                    app.state.selected_file_index = Some(selected_index + 1);
                }
            }
        }
        Mode::Navigate => {
            if app.state.is_searching_places {
                // Navigate down in place search results
                if let Some(selected_index) = app.state.selected_place_result {
                    if selected_index < app.state.place_search_results.len() - 1 {
                        app.state.selected_place_result = Some(selected_index + 1);
                    }
                }
            } else {
                // Down: Navigate within place hierarchy
                match &app.state.selection {
                    Some(Selection::Place(place_id)) => {
                        // If on a place, move into first affordance if it exists
                        if let Some(place) = app.breadboard.find_place(place_id) {
                            if !place.affordances.is_empty() {
                                app.state.selection = Some(Selection::Affordance {
                                    place_id: *place_id,
                                    affordance_id: place.affordances[0].id,
                                });
                            }
                        }
                    }
                    Some(Selection::Affordance { place_id, affordance_id }) => {
                        // If on an affordance, move to next affordance
                        if let Some(place) = app.breadboard.find_place(place_id) {
                            if let Some(current_index) = place.affordances.iter().position(|a| &a.id == affordance_id) {
                                if current_index < place.affordances.len() - 1 {
                                    // Move to next affordance
                                    app.state.selection = Some(Selection::Affordance {
                                        place_id: *place_id,
                                        affordance_id: place.affordances[current_index + 1].id,
                                    });
                                }
                                // If at last affordance, stay there
                            }
                        }
                    }
                    None => {
                        if let Some(first_place) = app.breadboard.places.first() {
                            app.state.selection = Some(Selection::Place(first_place.id));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn navigate_right(app: &mut App) {
    // Tab: Move to next place
    let current_place_id = match &app.state.selection {
        Some(Selection::Place(id)) => Some(*id),
        Some(Selection::Affordance { place_id, .. }) => Some(*place_id),
        None => None,
    };

    if let Some(current_id) = current_place_id {
        let places = &app.breadboard.places;
        if let Some(current_index) = places.iter().position(|p| p.id == current_id) {
            if current_index < places.len() - 1 {
                // Move to next place
                app.state.selection = Some(Selection::Place(places[current_index + 1].id));
            }
        }
    } else {
        // No selection, go to first place
        if let Some(first_place) = app.breadboard.places.first() {
            app.state.selection = Some(Selection::Place(first_place.id));
        }
    }
}

fn navigate_left(app: &mut App) {
    // Shift+Tab: Move to previous place
    let current_place_id = match &app.state.selection {
        Some(Selection::Place(id)) => Some(*id),
        Some(Selection::Affordance { place_id, .. }) => Some(*place_id),
        None => None,
    };

    if let Some(current_id) = current_place_id {
        let places = &app.breadboard.places;
        if let Some(current_index) = places.iter().position(|p| p.id == current_id) {
            if current_index > 0 {
                // Move to previous place
                app.state.selection = Some(Selection::Place(places[current_index - 1].id));
            }
        }
    } else {
        // No selection, go to first place
        if let Some(first_place) = app.breadboard.places.first() {
            app.state.selection = Some(Selection::Place(first_place.id));
        }
    }
}

fn handle_select(app: &mut App, file_manager: &FileManager) {
    match app.state.mode {
        Mode::Navigate => {
            if app.state.is_searching_places {
                // In place search mode, jump to selected place
                if let Some(place) = app.get_selected_search_place() {
                    let place_id = place.id;
                    app.state.selection = Some(Selection::Place(place_id));
                    app.clear_place_search();
                }
            } else {
                // Normal navigation
                match &app.state.selection {
                    Some(Selection::Affordance { place_id, affordance_id }) => {
                        if let Some(place) = app.breadboard.find_place(place_id) {
                            if let Some(affordance) = place.affordances.iter().find(|a| &a.id == affordance_id) {
                                if let Some(dest_id) = &affordance.connects_to {
                                    app.navigate_to_place(*dest_id);
                                }
                            }
                        }
                    }
                    Some(Selection::Place(_)) => {
                        // Could enter edit mode when pressing Enter on a place
                    }
                    None => {}
                }
            }
        }
        Mode::Edit => {
            // Complete edit and save the changes
            let selection = app.state.selection.clone();
            let new_name = app.state.edit_buffer.clone();

            match selection {
                Some(Selection::Place(place_id)) => {
                    if let Some(place) = app.breadboard.find_place_mut(&place_id) {
                        place.name = new_name;
                    }
                }
                Some(Selection::Affordance { place_id, affordance_id }) => {
                    if let Some(place) = app.breadboard.find_place_mut(&place_id) {
                        if let Some(affordance) = place.affordances.iter_mut().find(|a| a.id == affordance_id) {
                            affordance.name = new_name;
                        }
                    }
                }
                None => {}
            }
            app.state.mode = Mode::Navigate;
            app.state.edit_buffer.clear();
        }
        Mode::Connect => {
            // Check what action to take before borrowing mutably
            let should_remove = app.is_remove_connection_selected();
            let selected_place_id = if !should_remove {
                app.get_selected_connection_place().map(|p| p.id)
            } else {
                None
            };

            if let Some(Selection::Affordance { place_id, affordance_id }) = &app.state.selection {
                if let Some(place) = app.breadboard.find_place_mut(place_id) {
                    if let Some(affordance) = place.affordances.iter_mut().find(|a| a.id == *affordance_id) {
                        if should_remove {
                            // Remove connection
                            affordance.connects_to = None;
                        } else if let Some(selected_place_id) = selected_place_id {
                            // Create connection with selected place
                            affordance.connects_to = Some(selected_place_id);
                        }
                    }
                }
            }
            // Exit connection mode
            app.state.mode = Mode::Navigate;
            app.clear_connection_search();
        }
        Mode::OpenFile => {
            // Open selected file
            if let Some(filename) = app.get_selected_file() {
                match file_manager.load_from_file(filename) {
                    Ok(breadboard) => {
                        app.breadboard = breadboard;
                        app.state.selection = None;
                        // Reset selection to first place if available
                        if let Some(first_place) = app.breadboard.places.first() {
                            app.state.selection = Some(Selection::Place(first_place.id));
                        }
                    }
                    Err(e) => {
                        // In a real app, you'd show an error message in the UI
                        eprintln!("Failed to load {}: {}", filename, e);
                    }
                }
            }
            // Exit file opening mode
            app.state.mode = Mode::Navigate;
            app.clear_file_selection();
        }
    }
}

fn handle_back(app: &mut App) {
    match app.state.mode {
        Mode::Edit => {
            app.state.mode = Mode::Navigate;
            app.state.edit_buffer.clear();
        }
        Mode::Connect => {
            app.state.mode = Mode::Navigate;
            app.clear_connection_search();
        }
        Mode::OpenFile => {
            app.state.mode = Mode::Navigate;
            app.clear_file_selection();
        }
        Mode::Navigate => {
            if app.state.is_searching_places {
                // Exit place search mode
                app.clear_place_search();
            } else {
                // Navigate back in trail
                app.navigate_back();
            }
        }
    }
}

fn handle_new_place(app: &mut App) {
    // For now, create a place with a default name
    let place_count = app.breadboard.places.len();
    app.new_place(format!("Place {}", place_count + 1));
}

fn handle_new_affordance(app: &mut App) {
    // Get the place ID whether we're on a place or an affordance
    let place_id = match app.state.selection {
        Some(Selection::Place(id)) => id,
        Some(Selection::Affordance { place_id, .. }) => place_id,
        None => return,
    };

    let affordance_count = app.breadboard.find_place(&place_id)
        .map(|p| p.affordances.len())
        .unwrap_or(0);

    let affordance = models::Affordance::new(format!("Action {}", affordance_count + 1));
    app.add_affordance_to_place(&place_id, affordance);
}


fn handle_remove_connection(app: &mut App) {
    // Remove connection from selected affordance ONLY
    // Safety check: Only proceed if we're definitely on an affordance
    let (place_id, affordance_id) = match &app.state.selection {
        Some(Selection::Affordance { place_id, affordance_id }) => (*place_id, *affordance_id),
        _ => {
            // Not on an affordance - do absolutely nothing
            return;
        }
    };

    // Find the specific place and affordance
    if let Some(place) = app.breadboard.find_place_mut(&place_id) {
        // Find only the affordance with the exact matching ID
        if let Some(affordance) = place.affordances.iter_mut().find(|a| a.id == affordance_id) {
            // Only modify this specific affordance's connection
            affordance.connects_to = None;
        }
        // If affordance not found, do nothing (shouldn't happen with valid selection)
    }
}

fn handle_save(app: &App, file_manager: &FileManager) -> Result<()> {
    let filename = "breadboard.toml";
    match file_manager.save_to_file(&app.breadboard, filename) {
        Ok(()) => {
            // In a real app, you'd show a success message
            println!("Saved to {}", filename);
        }
        Err(e) => {
            // In a real app, you'd show an error message in the UI
            eprintln!("Failed to save: {}", e);
        }
    }
    Ok(())
}


fn handle_enter_edit_mode(app: &mut App) {
    // Enter edit mode for the currently selected item
    if let Some(ref selection) = app.state.selection {
        app.state.mode = Mode::Edit;

        // Initialize edit buffer with current text
        match selection {
            Selection::Place(place_id) => {
                if let Some(place) = app.breadboard.find_place(place_id) {
                    app.state.edit_buffer = place.name.clone();
                }
            }
            Selection::Affordance { place_id, affordance_id } => {
                if let Some(place) = app.breadboard.find_place(place_id) {
                    if let Some(affordance) = place.affordances.iter().find(|a| &a.id == affordance_id) {
                        app.state.edit_buffer = affordance.name.clone();
                    }
                }
            }
        }
    }
}

fn handle_delete(app: &mut App) {
    // Delete the currently selected place or affordance
    match &app.state.selection {
        Some(Selection::Place(place_id)) => {
            // Remove the place
            app.breadboard.places.retain(|p| &p.id != place_id);
            // Clear selection
            app.state.selection = None;
        }
        Some(Selection::Affordance { place_id, affordance_id }) => {
            // Remove the affordance from its place
            if let Some(place) = app.breadboard.find_place_mut(place_id) {
                place.affordances.retain(|a| &a.id != affordance_id);
            }
            // Move selection back to the place
            app.state.selection = Some(Selection::Place(*place_id));
        }
        None => {
            // Nothing to delete
        }
    }
}

fn handle_edit(app: &mut App, text_change: String) {
    match app.state.mode {
        Mode::Edit => {
            // Handle text editing for regular edit mode
            if text_change == "backspace" {
                app.state.edit_buffer.pop();
            } else if text_change == "delete" {
                // Delete character at cursor position (simplified)
                if !app.state.edit_buffer.is_empty() {
                    app.state.edit_buffer.pop();
                }
            } else if text_change == "left" || text_change == "right" || text_change == "home" || text_change == "end" {
                // Cursor movement - simplified for now
            } else if !text_change.is_empty() {
                // Add character to buffer
                app.state.edit_buffer.push_str(&text_change);
            }
        }
        Mode::Connect => {
            // Handle connection search text editing
            if text_change == "backspace" {
                app.state.connection_search_buffer.pop();
                app.update_connection_search();
            } else if text_change == "delete" {
                // Delete character at cursor position (simplified)
                if !app.state.connection_search_buffer.is_empty() {
                    app.state.connection_search_buffer.pop();
                    app.update_connection_search();
                }
            } else if text_change == "left" || text_change == "right" || text_change == "home" || text_change == "end" {
                // Cursor movement - simplified for now
            } else if !text_change.is_empty() {
                // Add character to search buffer
                app.state.connection_search_buffer.push_str(&text_change);
                app.update_connection_search();
            }
        }
        Mode::OpenFile => {
            // No text editing in file opening mode
        }
        Mode::Navigate => {
            if app.state.is_searching_places {
                // Handle place search text editing
                if text_change == "backspace" {
                    app.state.place_search_buffer.pop();
                    app.update_place_search();
                } else if text_change == "delete" {
                    if !app.state.place_search_buffer.is_empty() {
                        app.state.place_search_buffer.pop();
                        app.update_place_search();
                    }
                } else if text_change == "left" || text_change == "right" || text_change == "home" || text_change == "end" {
                    // Cursor movement - simplified for now
                } else if !text_change.is_empty() {
                    // Add character to search buffer
                    app.state.place_search_buffer.push_str(&text_change);
                    app.update_place_search();
                }
            } else {
                // Start place search with first character
                if !text_change.is_empty() && text_change != "backspace" && text_change != "delete"
                   && text_change != "left" && text_change != "right" && text_change != "home" && text_change != "end" {
                    app.start_place_search();
                    app.state.place_search_buffer.push_str(&text_change);
                    app.update_place_search();
                }
            }
        }
    }
}

fn handle_enter_connect_mode(app: &mut App) {
    // Only allow connection mode when on an affordance
    if let Some(Selection::Affordance { .. }) = &app.state.selection {
        app.state.mode = Mode::Connect;
        app.start_connection_search();
    }
}

fn handle_enter_open_mode(app: &mut App, file_manager: &FileManager) -> Result<()> {
    app.state.mode = Mode::OpenFile;
    app.start_file_opening(file_manager)?;
    Ok(())
}