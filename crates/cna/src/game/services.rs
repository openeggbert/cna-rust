#![allow(non_snake_case, clippy::missing_errors_doc)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{CnaError, Result};

/// Per-game XNA service registry using Rust runtime type tokens.
pub struct GameServiceContainer {
    services: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl GameServiceContainer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Mutex::new(HashMap::new()),
        }
    }

    pub fn AddService(&self, r#type: TypeId, provider: Arc<dyn Any + Send + Sync>) -> Result<()> {
        let mut services = self
            .services
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if services.contains_key(&r#type) {
            return Err(CnaError::InvalidInput(
                "a service is already registered for this type token",
            ));
        }
        services.insert(r#type, provider);
        Ok(())
    }

    pub fn RemoveService(&self, r#type: TypeId) {
        self.services
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&r#type);
    }

    #[must_use]
    pub fn GetService(&self, r#type: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
        self.services
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&r#type)
            .cloned()
    }
}

impl Default for GameServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Rust projection of `System.IServiceProvider` for XNA service containers.
pub trait ServiceProvider {
    fn GetService(&self, r#type: TypeId) -> Option<Arc<dyn Any + Send + Sync>>;
}

impl ServiceProvider for GameServiceContainer {
    fn GetService(&self, r#type: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
        Self::GetService(self, r#type)
    }
}

/// Per-game string launch-parameter dictionary.
pub struct LaunchParameters {
    values: Mutex<HashMap<String, String>>,
}

impl LaunchParameters {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for LaunchParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// Inherited dictionary behavior kept outside the declared XNA type surface.
pub trait LaunchParametersExt {
    fn Add(&self, key: &str, value: &str) -> Result<()>;
    fn ContainsKey(&self, key: &str) -> bool;
    fn Item(&self, key: &str) -> Option<String>;
    fn Remove(&self, key: &str) -> bool;
    fn Count(&self) -> usize;
    fn Entries(&self) -> Vec<(String, String)>;
}

impl LaunchParametersExt for LaunchParameters {
    fn Add(&self, key: &str, value: &str) -> Result<()> {
        let mut values = self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if values.contains_key(key) {
            return Err(CnaError::InvalidInput(
                "a launch parameter with this key already exists",
            ));
        }
        values.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    fn ContainsKey(&self, key: &str) -> bool {
        self.values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(key)
    }

    fn Item(&self, key: &str) -> Option<String> {
        self.values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key)
            .cloned()
    }

    fn Remove(&self, key: &str) -> bool {
        self.values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(key)
            .is_some()
    }

    fn Count(&self) -> usize {
        self.values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    fn Entries(&self) -> Vec<(String, String)> {
        self.values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }
}
