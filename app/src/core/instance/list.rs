//! Instance list management

use std::path::PathBuf;
use std::collections::HashMap;
use crate::core::error::{OxideError, Result};
use super::Instance;

/// List of all instances
#[derive(Debug, Clone)]
pub struct InstanceList {
    /// All instances, keyed by ID
    pub instances: Vec<Instance>,
    
    /// Groups and their collapsed state
    pub groups: HashMap<String, bool>,
}

impl InstanceList {
    /// Create a new empty instance list
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            groups: HashMap::new(),
        }
    }

    /// Load instances from the instances directory
    pub fn load(instances_dir: &PathBuf) -> Result<Self> {
        let mut list = Self::new();
        
        // Create instances directory if it doesn't exist
        if !instances_dir.exists() {
            std::fs::create_dir_all(instances_dir)?;
            return Ok(list);
        }

        // Iterate over directories in instances_dir
        for entry in std::fs::read_dir(instances_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Try to load instance from this directory
                match Instance::load(&path) {
                    Ok(instance) => {
                        // Track group
                        if let Some(ref group) = instance.group {
                            list.groups.entry(group.clone()).or_insert(false);
                        }
                        list.instances.push(instance);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load instance from {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by name
        list.instances.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        // Load groups state
        list.load_groups_state(instances_dir);

        Ok(list)
    }

    /// Save all instances
    pub fn save_all(&self) -> Result<()> {
        for instance in &self.instances {
            instance.save()?;
        }
        Ok(())
    }

    /// Get an instance by ID
    pub fn get(&self, id: &str) -> Option<&Instance> {
        self.instances.iter().find(|i| i.id == id)
    }

    /// Get a mutable instance by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Instance> {
        self.instances.iter_mut().find(|i| i.id == id)
    }

    /// Add an instance
    pub fn add(&mut self, instance: Instance) {
        // Track group
        if let Some(ref group) = instance.group {
            self.groups.entry(group.clone()).or_insert(false);
        }
        
        self.instances.push(instance);
        self.sort();
    }

    /// Update an existing instance
    pub fn update(&mut self, instance: Instance) {
        if let Some(existing) = self.get_mut(&instance.id) {
            *existing = instance;
            let _ = existing.save();
        }
    }

    /// Remove an instance by ID
    pub fn remove(&mut self, id: &str) -> Option<Instance> {
        if let Some(pos) = self.instances.iter().position(|i| i.id == id) {
            let instance = self.instances.remove(pos);
            
            // Try to delete the instance directory
            if let Err(e) = std::fs::remove_dir_all(&instance.path) {
                tracing::warn!("Failed to delete instance directory: {}", e);
            }
            
            Some(instance)
        } else {
            None
        }
    }

    /// Get all instances in a group
    pub fn get_group(&self, group: &str) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| i.group.as_deref() == Some(group))
            .collect()
    }

    /// Get all ungrouped instances
    pub fn get_ungrouped(&self) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| i.group.is_none())
            .collect()
    }

    /// Get all unique group names
    pub fn get_groups(&self) -> Vec<String> {
        let mut groups: Vec<_> = self.groups.keys().cloned().collect();
        groups.sort();
        groups
    }

    /// Check if a group is collapsed
    pub fn is_group_collapsed(&self, group: &str) -> bool {
        *self.groups.get(group).unwrap_or(&false)
    }

    /// Set group collapsed state
    pub fn set_group_collapsed(&mut self, group: &str, collapsed: bool) {
        self.groups.insert(group.to_string(), collapsed);
    }

    /// Rename a group
    pub fn rename_group(&mut self, old_name: &str, new_name: &str) {
        for instance in &mut self.instances {
            if instance.group.as_deref() == Some(old_name) {
                instance.group = Some(new_name.to_string());
                let _ = instance.save();
            }
        }
        
        if let Some(collapsed) = self.groups.remove(old_name) {
            self.groups.insert(new_name.to_string(), collapsed);
        }
    }

    /// Delete a group (moves instances to ungrouped)
    pub fn delete_group(&mut self, group_name: &str) {
        for instance in &mut self.instances {
            if instance.group.as_deref() == Some(group_name) {
                instance.group = None;
                let _ = instance.save();
            }
        }
        
        self.groups.remove(group_name);
    }

    /// Sort instances by name
    fn sort(&mut self) {
        self.instances.sort_by(|a, b| {
            // First sort by group (ungrouped last)
            match (&a.group, &b.group) {
                (Some(ag), Some(bg)) => ag.cmp(bg).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }

    /// Get total play time across all instances
    pub fn total_play_time(&self) -> u64 {
        self.instances.iter().map(|i| i.total_played_seconds).sum()
    }

    /// Iterate over instances
    pub fn iter(&self) -> std::slice::Iter<'_, Instance> {
        self.instances.iter()
    }

    /// Number of instances
    pub fn count(&self) -> usize {
        self.instances.len()
    }

    /// Filter instances by search query (case-insensitive substring on name)
    pub fn get_filtered(&self, query: &str) -> Vec<&Instance> {
        if query.trim().is_empty() {
            return self.instances.iter().collect();
        }

        let q = query.to_lowercase();
        self.instances
            .iter()
            .filter(|i| i.name.to_lowercase().contains(&q))
            .collect()
    }

    /// Load groups collapsed state from file
    fn load_groups_state(&mut self, instances_dir: &PathBuf) {
        let groups_file = instances_dir.join("groups.json");
        if groups_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&groups_file) {
                if let Ok(groups) = serde_json::from_str::<HashMap<String, bool>>(&content) {
                    for (group, collapsed) in groups {
                        self.groups.entry(group).or_insert(collapsed);
                    }
                }
            }
        }
    }

    /// Save groups collapsed state to file
    pub fn save_groups_state(&self, instances_dir: &PathBuf) -> Result<()> {
        let groups_file = instances_dir.join("groups.json");
        let content = serde_json::to_string_pretty(&self.groups)?;
        std::fs::write(groups_file, content)?;
        Ok(())
    }

    /// Find instances by Minecraft version
    pub fn find_by_version(&self, version: &str) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| i.minecraft_version == version)
            .collect()
    }

    /// Find instances by mod loader
    pub fn find_by_mod_loader(&self, loader: super::ModLoaderType) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| {
                i.mod_loader
                    .as_ref()
                    .map(|l| l.loader_type == loader)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get instance count
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

impl Default for InstanceList {
    fn default() -> Self {
        Self::new()
    }
}
