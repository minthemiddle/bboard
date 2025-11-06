use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affordance {
    pub id: u32,
    pub name: String,
    pub connects_to: Option<u32>, // Place ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub id: u32,
    pub name: String,
    pub group: Option<String>,
    pub affordances: Vec<Affordance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breadboard {
    pub name: String,
    pub created: String,
    pub places: Vec<Place>,
    #[serde(default = "default_next_place_id")]
    pub next_place_id: u32,
    #[serde(default = "default_next_affordance_id")]
    pub next_affordance_id: u32,
}

fn default_next_place_id() -> u32 {
    1
}

fn default_next_affordance_id() -> u32 {
    1
}

impl Breadboard {
    pub fn new(name: String) -> Self {
        Self {
            name,
            created: chrono::Utc::now().to_rfc3339(),
            places: Vec::new(),
            next_place_id: 1,
            next_affordance_id: 1,
        }
    }

    pub fn add_place(&mut self, place: Place) {
        self.places.push(place);
    }

    pub fn find_place(&self, id: &u32) -> Option<&Place> {
        self.places.iter().find(|p| &p.id == id)
    }

    pub fn find_place_mut(&mut self, id: &u32) -> Option<&mut Place> {
        self.places.iter_mut().find(|p| &p.id == id)
    }

    pub fn get_incoming_connections(&self, place_id: &u32) -> Vec<(&Place, &Affordance)> {
        self.places
            .iter()
            .flat_map(|place| {
                place.affordances.iter().filter_map(move |affordance| {
                    affordance.connects_to.as_ref().and_then(|dest| {
                        if dest == place_id {
                            Some((place, affordance))
                        } else {
                            None
                        }
                    })
                })
            })
            .collect()
    }

    pub fn generate_place_id(&mut self) -> u32 {
        let id = self.next_place_id;
        self.next_place_id += 1;
        id
    }

    pub fn generate_affordance_id(&mut self) -> u32 {
        let id = self.next_affordance_id;
        self.next_affordance_id += 1;
        id
    }

    // Sync ID counters after loading from file to ensure new IDs don't conflict
    pub fn sync_id_counters(&mut self) {
        let max_place_id = self.places.iter()
            .map(|p| p.id)
            .max()
            .unwrap_or(0);

        let max_affordance_id = self.places.iter()
            .flat_map(|p| p.affordances.iter())
            .map(|a| a.id)
            .max()
            .unwrap_or(0);

        self.next_place_id = max_place_id + 1;
        self.next_affordance_id = max_affordance_id + 1;
    }
}

impl Place {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            group: None,
            affordances: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_group(mut self, group: String) -> Self {
        self.group = Some(group);
        self
    }

    pub fn add_affordance(&mut self, affordance: Affordance) {
        self.affordances.push(affordance);
    }
}

impl Affordance {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            connects_to: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_connection(mut self, destination_place_id: u32) -> Self {
        self.connects_to = Some(destination_place_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_creation() {
        let place = Place::new(1, "Test Place".to_string());
        assert_eq!(place.id, 1);
        assert_eq!(place.name, "Test Place");
        assert_eq!(place.affordances.len(), 0);
        assert!(place.group.is_none());
    }

    #[test]
    fn test_place_with_group() {
        let place = Place::new(1, "Test Place".to_string()).with_group("web".to_string());
        assert_eq!(place.group, Some("web".to_string()));
    }

    #[test]
    fn test_affordance_creation() {
        let affordance = Affordance::new(1, "Click Me".to_string());
        assert_eq!(affordance.id, 1);
        assert_eq!(affordance.name, "Click Me");
        assert!(affordance.connects_to.is_none());
    }

    #[test]
    fn test_affordance_with_connection() {
        let dest_id = 2;
        let affordance = Affordance::new(1, "Click Me".to_string()).with_connection(dest_id);
        assert_eq!(affordance.connects_to, Some(dest_id));
    }

    #[test]
    fn test_add_affordance_to_place() {
        let mut place = Place::new(1, "Test Place".to_string());
        let affordance = Affordance::new(1, "Action".to_string());
        place.add_affordance(affordance);
        assert_eq!(place.affordances.len(), 1);
        assert_eq!(place.affordances[0].name, "Action");
    }

    #[test]
    fn test_breadboard_creation() {
        let breadboard = Breadboard::new("Test Board".to_string());
        assert_eq!(breadboard.name, "Test Board");
        assert_eq!(breadboard.places.len(), 0);
        assert_eq!(breadboard.next_place_id, 1);
        assert_eq!(breadboard.next_affordance_id, 1);
    }

    #[test]
    fn test_breadboard_add_place() {
        let mut breadboard = Breadboard::new("Test Board".to_string());
        let place = Place::new(1, "Test Place".to_string());
        let place_id = place.id;
        breadboard.add_place(place);
        assert_eq!(breadboard.places.len(), 1);
        assert_eq!(breadboard.places[0].id, place_id);
    }

    #[test]
    fn test_breadboard_find_place() {
        let mut breadboard = Breadboard::new("Test Board".to_string());
        let place = Place::new(1, "Test Place".to_string());
        let place_id = place.id;
        breadboard.add_place(place);

        let found = breadboard.find_place(&place_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Place");

        let not_found = breadboard.find_place(&999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_incoming_connections() {
        let mut breadboard = Breadboard::new("Test Board".to_string());

        let mut place1 = Place::new(1, "Place 1".to_string());
        let place2 = Place::new(2, "Place 2".to_string());
        let place2_id = place2.id;

        let affordance = Affordance::new(1, "Go to Place 2".to_string()).with_connection(place2_id);
        place1.add_affordance(affordance);

        breadboard.add_place(place1);
        breadboard.add_place(place2);

        let incoming = breadboard.get_incoming_connections(&place2_id);
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].0.name, "Place 1");
        assert_eq!(incoming[0].1.name, "Go to Place 2");
    }

    #[test]
    fn test_serialization() {
        let breadboard = Breadboard::new("Test Board".to_string());
        let toml_str = toml::to_string_pretty(&breadboard).unwrap();
        assert!(toml_str.contains("name = \"Test Board\""));
    }

    #[test]
    fn test_deserialization() {
        let toml_str = r#"
name = "Test Board"
created = "2025-01-15T10:00:00Z"

[[places]]
id = 1
name = "Test Place"

[[places.affordances]]
id = 1
name = "Test Action"
"#;
        let breadboard: Breadboard = toml::from_str(toml_str).unwrap();
        assert_eq!(breadboard.name, "Test Board");
        assert_eq!(breadboard.places.len(), 1);
        assert_eq!(breadboard.places[0].name, "Test Place");
        assert_eq!(breadboard.places[0].id, 1);
        assert_eq!(breadboard.places[0].affordances.len(), 1);
        assert_eq!(breadboard.places[0].affordances[0].name, "Test Action");
        assert_eq!(breadboard.places[0].affordances[0].id, 1);
    }
}