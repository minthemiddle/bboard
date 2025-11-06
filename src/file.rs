use crate::models::Breadboard;
use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

pub struct FileManager;

impl FileManager {
    pub fn new() -> Self {
        Self
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, breadboard: &Breadboard, path: P) -> Result<()> {
        let toml_string = toml::to_string_pretty(breadboard)
            .context("Failed to serialize breadboard to TOML")?;

        fs::write(path, toml_string)
            .context("Failed to write TOML to file")?;

        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<Breadboard> {
        let content = fs::read_to_string(path)
            .context("Failed to read TOML file")?;

        let breadboard: Breadboard = toml::from_str(&content)
            .context("Failed to parse TOML as Breadboard")?;

        Ok(breadboard)
    }

    pub fn list_toml_files(&self) -> Result<Vec<String>> {
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;

        let mut toml_files = Vec::new();

        for entry in fs::read_dir(current_dir)
            .context("Failed to read current directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "toml" {
                        if let Some(filename) = path.file_name() {
                            if let Some(filename_str) = filename.to_str() {
                                toml_files.push(filename_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        toml_files.sort();
        Ok(toml_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_file_manager_new() {
        let _fm = FileManager::new();
        // Just testing that it doesn't panic
    }

    #[test]
    fn test_save_and_load_breadboard() -> Result<()> {
        let fm = FileManager::new();
        let mut breadboard = Breadboard::new("Test Board".to_string());

        let place = crate::models::Place::new("Test Place".to_string());
        let place_id = place.id;
        breadboard.add_place(place);

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        // Save the breadboard
        fm.save_to_file(&breadboard, path)?;

        // Load it back
        let loaded = fm.load_from_file(path)?;

        assert_eq!(loaded.name, "Test Board");
        assert_eq!(loaded.places.len(), 1);
        assert_eq!(loaded.places[0].name, "Test Place");
        assert_eq!(loaded.places[0].id, place_id);

        Ok(())
    }

    #[test]
    fn test_load_nonexistent_file() {
        let fm = FileManager::new();
        let result = fm.load_from_file("non_existent_file_12345.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_toml() -> Result<()> {
        let fm = FileManager::new();
        let mut temp_file = NamedTempFile::new()?;

        // Write invalid TOML
        writeln!(temp_file, "invalid toml content [[[")?;

        let result = fm.load_from_file(temp_file.path());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_save_complex_breadboard() -> Result<()> {
        let fm = FileManager::new();
        let mut breadboard = Breadboard::new("Complex Board".to_string());

        // Create places with affordances and connections
        let mut place1 = crate::models::Place::new("Place 1".to_string());
        let place1_id = place1.id;

        let place2 = crate::models::Place::new("Place 2".to_string());
        let place2_id = place2.id;

        let affordance = crate::models::Affordance::new("Go to Place 2".to_string())
            .with_connection(place2_id);
        place1.add_affordance(affordance);

        breadboard.add_place(place1);
        breadboard.add_place(place2);

        let temp_file = NamedTempFile::new()?;
        fm.save_to_file(&breadboard, temp_file.path())?;

        // Verify the file content contains expected data
        let content = fs::read_to_string(temp_file.path())?;
        assert!(content.contains("Complex Board"));
        assert!(content.contains("Place 1"));
        assert!(content.contains("Place 2"));
        assert!(content.contains("Go to Place 2"));

        // Load and verify structure
        let loaded = fm.load_from_file(temp_file.path())?;
        assert_eq!(loaded.name, "Complex Board");
        assert_eq!(loaded.places.len(), 2);

        let loaded_place1 = loaded.find_place(&place1_id).unwrap();
        assert_eq!(loaded_place1.affordances.len(), 1);
        assert_eq!(loaded_place1.affordances[0].connects_to, Some(place2_id));

        Ok(())
    }
}