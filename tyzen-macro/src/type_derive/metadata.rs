use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

/// Global cache to avoid redundant file I/O during a single compilation session.
/// Since proc-macros can be executed in parallel, this uses a Mutex for thread-safety.
/// The structure is: EnumName -> VariantName -> AttributeKey -> AttributeValue.
static METADATA_CACHE: Lazy<Mutex<HashMap<String, HashMap<String, HashMap<String, String>>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Internal structure for JSON serialization of enum metadata.
#[derive(Serialize, Deserialize)]
struct EnumMetadata {
    variants: HashMap<String, HashMap<String, String>>,
}

/// Computes the path to the shadow metadata file in the `target` directory.
/// We store these in `target/tyzen/metadata/` to ensure they are cleaned by `cargo clean`.
fn get_metadata_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from("target/tyzen/metadata");
    path.push(format!("{}.json", name));
    path
}

/// Persists enum variant metadata to a shadow JSON file and updates the global cache.
/// This enables other types to "inherit" these attributes via the `apply` attribute.
pub fn save_enum_metadata(name: &str, variants: &[(String, Vec<(String, String)>)]) {
    let mut map = HashMap::new();
    for (v_name, attrs) in variants {
        let attrs_map: HashMap<_, _> = attrs.iter().cloned().collect();
        map.insert(v_name.clone(), attrs_map);
    }

    // Update global cache immediately to make it available for subsequent macro calls
    if let Ok(mut cache) = METADATA_CACHE.lock() {
        cache.insert(name.to_string(), map.clone());
    }

    let meta = EnumMetadata { variants: map };
    let path = get_metadata_path(name);
    
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Use pretty-printing for easier debugging of shadow files
    if let Ok(json) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(path, json);
    }
}

/// Loads enum variant metadata, checking the in-memory cache first before falling back to disk.
/// Returns None if the template metadata is not found.
pub fn load_enum_metadata(name: &str) -> Option<HashMap<String, HashMap<String, String>>> {
    // 1. Hot Path: Check global memory cache
    if let Ok(cache) = METADATA_CACHE.lock() {
        if let Some(meta) = cache.get(name) {
            return Some(meta.clone());
        }
    }

    // 2. Cold Path: Load from shadow JSON file on disk
    let path = get_metadata_path(name);
    let content = fs::read_to_string(path).ok()?;
    let meta: EnumMetadata = serde_json::from_str(&content).ok()?;

    // Update cache to avoid disk I/O next time this template is requested
    if let Ok(mut cache) = METADATA_CACHE.lock() {
        cache.insert(name.to_string(), meta.variants.clone());
    }

    Some(meta.variants)
}
