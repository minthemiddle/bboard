use crate::models::{Breadboard, Place, Affordance};
use crate::input::Mode;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    Place(Uuid),
    Affordance { place_id: Uuid, affordance_id: Uuid },
}

#[derive(Debug)]
pub struct AppState {
    pub mode: Mode,
    pub selection: Option<Selection>,
    pub collapsed: bool,
    pub filter: Option<String>,
    pub navigation_trail: Vec<Uuid>,
    pub edit_buffer: String,
    pub connection_search_buffer: String,
    pub connection_search_results: Vec<Uuid>,
    pub selected_connection_result: Option<usize>,
    pub file_list: Vec<String>,
    pub selected_file_index: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: Mode::Navigate,
            selection: None,
            collapsed: false,
            filter: None,
            navigation_trail: Vec::new(),
            edit_buffer: String::new(),
            connection_search_buffer: String::new(),
            connection_search_results: Vec::new(),
            selected_connection_result: None,
            file_list: Vec::new(),
            selected_file_index: None,
        }
    }
}

pub struct App {
    pub breadboard: Breadboard,
    pub state: AppState,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let breadboard = Breadboard::new("New Breadboard".to_string());
        let state = AppState::default();

        Self {
            breadboard,
            state,
            should_quit: false,
        }
    }

    pub fn new_place(&mut self, name: String) {
        let place = Place::new(name);
        self.breadboard.add_place(place);
    }

    pub fn add_affordance_to_place(&mut self, place_id: &Uuid, affordance: Affordance) {
        if let Some(place) = self.breadboard.find_place_mut(place_id) {
            place.add_affordance(affordance);
        }
    }

    pub fn get_selected_place(&self) -> Option<&Place> {
        match &self.state.selection {
            Some(Selection::Place(id)) => self.breadboard.find_place(id),
            Some(Selection::Affordance { place_id, .. }) => self.breadboard.find_place(place_id),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_selected_place_mut(&mut self) -> Option<&mut Place> {
        let id = match &self.state.selection {
            Some(Selection::Place(id)) | Some(Selection::Affordance { place_id: id, .. }) => *id,
            None => return None,
        };
        self.breadboard.find_place_mut(&id)
    }

    pub fn navigate_to_place(&mut self, place_id: Uuid) {
        if let Some(current_place) = self.get_selected_place() {
            self.state.navigation_trail.push(current_place.id);
        }
        self.state.selection = Some(Selection::Place(place_id));
    }

    pub fn navigate_back(&mut self) {
        if let Some(previous_id) = self.state.navigation_trail.pop() {
            self.state.selection = Some(Selection::Place(previous_id));
        }
    }

    pub fn toggle_collapsed(&mut self) {
        self.state.collapsed = !self.state.collapsed;
    }

    // Connection search methods
    const REMOVE_CONNECTION_ID: Uuid = Uuid::from_u128(0); // Special ID for remove connection option

    pub fn update_connection_search(&mut self) {
        // Start with the remove connection option
        let mut results = vec![Self::REMOVE_CONNECTION_ID];

        if self.state.connection_search_buffer.is_empty() {
            // Add all places
            results.extend(self.breadboard.places.iter().map(|p| p.id));
        } else {
            let search_lower = self.state.connection_search_buffer.to_lowercase();
            // Add matching places
            results.extend(self.breadboard.places.iter()
                .filter(|p| p.name.to_lowercase().contains(&search_lower))
                .map(|p| p.id));
        }

        self.state.connection_search_results = results;

        // Reset selection to first result (remove connection)
        self.state.selected_connection_result = Some(0);
    }

    pub fn start_connection_search(&mut self) {
        self.state.connection_search_buffer.clear();
        self.state.connection_search_results.clear();
        self.state.selected_connection_result = None;
        self.update_connection_search();
    }

    pub fn clear_connection_search(&mut self) {
        self.state.connection_search_buffer.clear();
        self.state.connection_search_results.clear();
        self.state.selected_connection_result = None;
    }

    pub fn get_selected_connection_place(&self) -> Option<&Place> {
        if let Some(selected_index) = self.state.selected_connection_result {
            if selected_index < self.state.connection_search_results.len() {
                let place_id = &self.state.connection_search_results[selected_index];
                // Check if this is the remove connection option
                if *place_id == Self::REMOVE_CONNECTION_ID {
                    return None; // This is the remove option, not a real place
                }
                self.breadboard.find_place(place_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_remove_connection_selected(&self) -> bool {
        if let Some(selected_index) = self.state.selected_connection_result {
            if selected_index < self.state.connection_search_results.len() {
                let place_id = &self.state.connection_search_results[selected_index];
                *place_id == Self::REMOVE_CONNECTION_ID
            } else {
                false
            }
        } else {
            false
        }
    }

    // File opening methods
    pub fn start_file_opening(&mut self, file_manager: &crate::file::FileManager) -> anyhow::Result<()> {
        self.state.file_list = file_manager.list_toml_files()?;
        self.state.selected_file_index = if self.state.file_list.is_empty() {
            None
        } else {
            Some(0)
        };
        Ok(())
    }

    pub fn get_selected_file(&self) -> Option<&String> {
        if let Some(selected_index) = self.state.selected_file_index {
            if selected_index < self.state.file_list.len() {
                Some(&self.state.file_list[selected_index])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear_file_selection(&mut self) {
        self.state.file_list.clear();
        self.state.selected_file_index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Mode;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.breadboard.name, "New Breadboard");
        assert_eq!(app.breadboard.places.len(), 0);
        assert!(!app.should_quit);
        assert_eq!(app.state.mode, Mode::Navigate);
        assert!(app.state.selection.is_none());
    }

    #[test]
    fn test_new_place() {
        let mut app = App::new();
        app.new_place("Test Place".to_string());
        assert_eq!(app.breadboard.places.len(), 1);
        assert_eq!(app.breadboard.places[0].name, "Test Place");
    }

    #[test]
    fn test_add_affordance_to_place() {
        let mut app = App::new();
        app.new_place("Test Place".to_string());

        let place_id = app.breadboard.places[0].id;
        let affordance = crate::models::Affordance::new("Test Action".to_string());
        app.add_affordance_to_place(&place_id, affordance);

        assert_eq!(app.breadboard.places[0].affordances.len(), 1);
        assert_eq!(app.breadboard.places[0].affordances[0].name, "Test Action");
    }

    #[test]
    fn test_get_selected_place() {
        let mut app = App::new();
        app.new_place("Test Place".to_string());

        // No selection initially
        assert!(app.get_selected_place().is_none());

        // Select the place
        let place_id = app.breadboard.places[0].id;
        app.state.selection = Some(Selection::Place(place_id));

        let selected = app.get_selected_place();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "Test Place");
    }

    #[test]
    fn test_navigate_to_place() {
        let mut app = App::new();
        app.new_place("Place 1".to_string());
        app.new_place("Place 2".to_string());

        let place1_id = app.breadboard.places[0].id;
        let place2_id = app.breadboard.places[1].id;

        // Start with first place selected
        app.state.selection = Some(Selection::Place(place1_id));
        assert_eq!(app.state.navigation_trail.len(), 0);

        // Navigate to second place
        app.navigate_to_place(place2_id);

        assert_eq!(app.state.selection, Some(Selection::Place(place2_id)));
        assert_eq!(app.state.navigation_trail.len(), 1);
        assert_eq!(app.state.navigation_trail[0], place1_id);
    }

    #[test]
    fn test_navigate_back() {
        let mut app = App::new();
        app.new_place("Place 1".to_string());
        app.new_place("Place 2".to_string());

        let place1_id = app.breadboard.places[0].id;
        let place2_id = app.breadboard.places[1].id;

        // Start with first place, navigate to second
        app.state.selection = Some(Selection::Place(place1_id));
        app.navigate_to_place(place2_id);

        assert_eq!(app.state.selection, Some(Selection::Place(place2_id)));
        assert_eq!(app.state.navigation_trail.len(), 1);

        // Navigate back
        app.navigate_back();

        assert_eq!(app.state.selection, Some(Selection::Place(place1_id)));
        assert_eq!(app.state.navigation_trail.len(), 0);
    }

    #[test]
    fn test_navigate_back_empty_trail() {
        let mut app = App::new();
        app.new_place("Place 1".to_string());

        let place1_id = app.breadboard.places[0].id;
        app.state.selection = Some(Selection::Place(place1_id));

        // Navigate back with empty trail should not panic
        app.navigate_back();

        // Selection should remain unchanged
        assert_eq!(app.state.selection, Some(Selection::Place(place1_id)));
        assert_eq!(app.state.navigation_trail.len(), 0);
    }

    #[test]
    fn test_toggle_collapsed() {
        let mut app = App::new();
        assert!(!app.state.collapsed);

        app.toggle_collapsed();
        assert!(app.state.collapsed);

        app.toggle_collapsed();
        assert!(!app.state.collapsed);
    }

    #[test]
    fn test_selection_with_affordance() {
        let mut app = App::new();
        app.new_place("Test Place".to_string());

        let place_id = app.breadboard.places[0].id;
        let affordance = crate::models::Affordance::new("Test Action".to_string());
        let affordance_id = affordance.id;
        app.add_affordance_to_place(&place_id, affordance);

        // Select the affordance
        app.state.selection = Some(Selection::Affordance { place_id, affordance_id });

        let selected = app.get_selected_place();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "Test Place");
    }
}