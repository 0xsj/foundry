// Plugin System with Shared State
//
// A plugin registry that loads plugins at startup, shares configuration,
// and allows plugins to reference the registry for dependency lookup.
//
// Run: rustc proposed.rs && ./proposed

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

// ---------- Config ----------

// ISSUE: PluginConfig wraps all fields in RefCell even though the config
// is constructed once and never mutated. RefCell adds runtime borrow tracking
// overhead and makes the API confusing ("can I mutate this? should I?").
#[derive(Debug)]
pub struct PluginConfig {
    name: RefCell<String>,
    version: RefCell<String>,
    settings: RefCell<HashMap<String, String>>,
}

impl PluginConfig {
    pub fn new(name: &str, version: &str) -> PluginConfig {
        PluginConfig {
            name: RefCell::new(name.to_string()),
            version: RefCell::new(version.to_string()),
            settings: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_setting(self, key: &str, value: &str) -> PluginConfig {
        self.settings.borrow_mut().insert(key.to_string(), value.to_string());
        self
    }

    // ISSUE: Returns String (allocated) when &str (borrowed) would work for
    // the name — the caller usually just needs to read it.
    pub fn name(&self) -> String {
        self.name.borrow().clone()
    }

    pub fn version(&self) -> String {
        self.version.borrow().clone()
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.settings.borrow().get(key).cloned()
    }
}

// ---------- Plugin ----------

// ISSUE: Plugin holds Rc<Registry> — a strong reference.
// Registry holds Vec<Rc<Plugin>> — also strong references.
// This is a cycle: Registry -> Plugin -> Registry.
// Neither will ever be freed.
#[derive(Debug)]
pub struct Plugin {
    pub config: Rc<PluginConfig>,
    // ISSUE: This should be Weak<Registry> to avoid the cycle.
    // Plugins are owned by the registry; they shouldn't own it back.
    pub registry: RefCell<Option<Rc<Registry>>>,
    pub state: RefCell<PluginState>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loaded,
    Failed(String),
}

impl Plugin {
    pub fn new(config: Rc<PluginConfig>) -> Rc<Plugin> {
        Rc::new(Plugin {
            config,
            registry: RefCell::new(None),
            state: RefCell::new(PluginState::Unloaded),
        })
    }

    pub fn load(&self) -> bool {
        println!("[{}] loading v{}", self.config.name(), self.config.version());
        *self.state.borrow_mut() = PluginState::Loaded;
        true
    }

    pub fn is_loaded(&self) -> bool {
        *self.state.borrow() == PluginState::Loaded
    }

    // ISSUE: get_registry_plugin_count accesses the registry through a strong Rc,
    // but it should be upgraded from a Weak.
    pub fn get_registry_plugin_count(&self) -> Option<usize> {
        self.registry.borrow().as_ref().map(|r| r.plugin_count())
    }
}

// ---------- Registry ----------

// ISSUE: The plugins Vec and config Map are wrapped in Arc<Mutex<>>,
// but the entire Registry is single-threaded (there's no thread::spawn anywhere,
// no Send requirement, nothing). Arc<Mutex<>> adds atomic overhead and
// makes every access more verbose for no benefit in this context.
// The right types here are Rc<RefCell<>> or just plain fields with &mut self methods.
#[derive(Debug)]
pub struct Registry {
    plugins: Arc<Mutex<Vec<Rc<Plugin>>>>,
    // Global config shared by all plugins
    shared_config: Arc<Mutex<HashMap<String, String>>>,
}

impl Registry {
    pub fn new() -> Rc<Registry> {
        Rc::new(Registry {
            plugins: Arc::new(Mutex::new(vec![])),
            shared_config: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn register(registry: &Rc<Registry>, plugin: Rc<Plugin>) {
        // ISSUE: Setting plugin.registry to Rc::clone(registry) creates the cycle.
        *plugin.registry.borrow_mut() = Some(Rc::clone(registry));
        registry.plugins.lock().unwrap().push(plugin);
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.lock().unwrap().len()
    }

    pub fn set_global_config(&self, key: &str, value: &str) {
        self.shared_config.lock().unwrap().insert(key.to_string(), value.to_string());
    }

    // ISSUE: Returns String when Option<&str> or Cow<str> would avoid the allocation.
    pub fn get_global_config(&self, key: &str) -> Option<String> {
        self.shared_config.lock().unwrap().get(key).cloned()
    }

    pub fn load_all(&self) {
        for plugin in self.plugins.lock().unwrap().iter() {
            plugin.load();
        }
    }

    // ISSUE: Returns a fully cloned Vec<String> of names. Returning
    // an iterator or references would avoid the allocations.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.lock().unwrap()
            .iter()
            .map(|p| p.config.name())  // name() already allocates
            .collect()
    }
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_load() {
        let registry = Registry::new();

        let config = Rc::new(PluginConfig::new("auth-plugin", "1.0.0"));
        let plugin = Plugin::new(Rc::clone(&config));

        Registry::register(&registry, Rc::clone(&plugin));
        registry.load_all();

        assert_eq!(registry.plugin_count(), 1);
        assert!(plugin.is_loaded());
    }

    #[test]
    fn test_global_config() {
        let registry = Registry::new();
        registry.set_global_config("log_level", "info");
        assert_eq!(registry.get_global_config("log_level"), Some(String::from("info")));
        assert_eq!(registry.get_global_config("missing"), None);
    }

    #[test]
    fn test_plugin_names() {
        let registry = Registry::new();

        for name in &["auth", "metrics", "tracing"] {
            let config = Rc::new(PluginConfig::new(name, "1.0.0"));
            let plugin = Plugin::new(config);
            Registry::register(&registry, plugin);
        }

        let names = registry.plugin_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&String::from("auth")));
        assert!(names.contains(&String::from("metrics")));
    }

    #[test]
    fn test_plugin_sees_registry_count() {
        let registry = Registry::new();

        let config = Rc::new(PluginConfig::new("cache-plugin", "2.1.0"));
        let plugin = Plugin::new(Rc::clone(&config));

        Registry::register(&registry, Rc::clone(&plugin));

        // After registration, plugin can query registry
        assert_eq!(plugin.get_registry_plugin_count(), Some(1));
    }

    #[test]
    fn test_plugin_config_settings() {
        let config = PluginConfig::new("mailer", "3.0.0")
            .with_setting("smtp_host", "mail.internal")
            .with_setting("smtp_port", "587");

        assert_eq!(config.get_setting("smtp_host"), Some(String::from("mail.internal")));
        assert_eq!(config.get_setting("missing"), None);
    }
}

fn main() {
    let registry = Registry::new();

    registry.set_global_config("environment", "production");
    registry.set_global_config("log_level", "warn");

    let plugins = vec![
        PluginConfig::new("auth", "2.0.0")
            .with_setting("jwt_secret", "s3cr3t")
            .with_setting("token_ttl_seconds", "3600"),
        PluginConfig::new("metrics", "1.3.1")
            .with_setting("export_interval_seconds", "15"),
        PluginConfig::new("feature-flags", "0.9.0"),
    ];

    for cfg in plugins {
        let plugin = Plugin::new(Rc::new(cfg));
        Registry::register(&registry, plugin);
    }

    registry.load_all();

    println!("\nLoaded plugins: {:?}", registry.plugin_names());
    println!("Global config 'log_level': {:?}", registry.get_global_config("log_level"));
    println!("Plugin count: {}", registry.plugin_count());
}
