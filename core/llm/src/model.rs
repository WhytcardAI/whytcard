//! Model management - loading, unloading, and info

use crate::config::ModelConfig;
use crate::error::{LlmError, Result};

use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Information about a loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model file path
    pub path: PathBuf,
    
    /// Model name (from metadata or filename)
    pub name: String,
    
    /// Architecture (llama, mistral, qwen, etc.)
    pub architecture: Option<String>,
    
    /// Vocabulary size
    pub vocab_size: i32,
    
    /// Embedding dimension
    pub embedding_dim: i32,
    
    /// Training context length
    pub context_length: u32,
    
    /// Number of parameters
    pub n_params: u64,
    
    /// Model size in bytes
    pub size_bytes: u64,
    
    /// Whether model supports chat template
    pub has_chat_template: bool,
}

/// A loaded model with its backend reference
pub struct LoadedModel {
    /// The llama.cpp model
    pub(crate) model: LlamaModel,
    
    /// Model configuration used
    pub config: ModelConfig,
    
    /// Model information
    pub info: ModelInfo,
}

impl LoadedModel {
    /// Get model info
    pub fn info(&self) -> &ModelInfo {
        &self.info
    }
    
    /// Get the underlying model reference
    pub fn inner(&self) -> &LlamaModel {
        &self.model
    }
    
    /// Check if model has a chat template
    pub fn has_chat_template(&self) -> bool {
        self.model.chat_template(None).is_ok()
    }
    
    /// Get the chat template if available (returns None - templates are opaque)
    /// Use apply_chat_template directly with the model instead
    pub fn chat_template(&self) -> Option<String> {
        // LlamaChatTemplate is opaque - we can't extract the source
        // Just indicate if a template exists
        if self.has_chat_template() {
            Some("ChatML".to_string()) // Placeholder name
        } else {
            None
        }
    }
}

/// Manager for loading and caching models
pub struct ModelManager {
    /// Reference to the llama.cpp backend
    backend: Arc<LlamaBackend>,
    
    /// Models directory
    models_dir: PathBuf,
    
    /// Currently loaded models by name
    loaded_models: HashMap<String, Arc<LoadedModel>>,
    
    /// Default GPU layers to offload
    default_gpu_layers: u32,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(backend: Arc<LlamaBackend>, models_dir: impl Into<PathBuf>) -> Self {
        Self {
            backend,
            models_dir: models_dir.into(),
            loaded_models: HashMap::new(),
            default_gpu_layers: 1000,
        }
    }
    
    /// Set default GPU layers
    pub fn with_default_gpu_layers(mut self, layers: u32) -> Self {
        self.default_gpu_layers = layers;
        self
    }
    
    /// List available models in the models directory
    pub fn list_available(&self) -> Result<Vec<PathBuf>> {
        let mut models = Vec::new();
        
        if !self.models_dir.exists() {
            return Ok(models);
        }
        
        Self::scan_directory(&self.models_dir, &mut models)?;
        Ok(models)
    }
    
    /// Recursively scan directory for GGUF files
    fn scan_directory(dir: &Path, models: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                Self::scan_directory(&path, models)?;
            } else if let Some(ext) = path.extension() {
                if ext == "gguf" {
                    models.push(path);
                }
            }
        }
        Ok(())
    }
    
    /// Load a model from path
    pub fn load(&mut self, config: ModelConfig) -> Result<Arc<LoadedModel>> {
        let path = if config.path.is_absolute() {
            config.path.clone()
        } else {
            self.models_dir.join(&config.path)
        };
        
        if !path.exists() {
            return Err(LlmError::ModelNotFound(path.display().to_string()));
        }
        
        let model_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Check if already loaded
        if let Some(model) = self.loaded_models.get(&model_name) {
            debug!("Model {} already loaded", model_name);
            return Ok(Arc::clone(model));
        }
        
        info!("Loading model: {}", path.display());
        
        // Configure model parameters
        let gpu_layers = config.n_gpu_layers.unwrap_or(self.default_gpu_layers);
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers);
        
        // Load model
        let model = LlamaModel::load_from_file(&self.backend, &path, &model_params)
            .map_err(|e| LlmError::ModelLoadError(e.to_string()))?;
        
        // Extract model info
        let info = Self::extract_info(&model, &path);
        
        info!(
            "Loaded model: {} ({} params, {} ctx)",
            info.name, info.n_params, info.context_length
        );
        
        let loaded = Arc::new(LoadedModel {
            model,
            config,
            info,
        });
        
        self.loaded_models.insert(model_name.clone(), Arc::clone(&loaded));
        
        Ok(loaded)
    }
    
    /// Load a model by name (looks in models directory)
    pub fn load_by_name(&mut self, name: &str) -> Result<Arc<LoadedModel>> {
        // Check if already loaded
        if let Some(model) = self.loaded_models.get(name) {
            return Ok(Arc::clone(model));
        }
        
        // Search for model file
        let available = self.list_available()?;
        let model_path = available.into_iter()
            .find(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s == name || s.starts_with(name))
                    .unwrap_or(false)
            })
            .ok_or_else(|| LlmError::ModelNotFound(name.to_string()))?;
        
        self.load(ModelConfig::from_path(model_path))
    }
    
    /// Unload a model by name
    pub fn unload(&mut self, name: &str) -> bool {
        if self.loaded_models.remove(name).is_some() {
            info!("Unloaded model: {}", name);
            true
        } else {
            warn!("Model not found for unload: {}", name);
            false
        }
    }
    
    /// Get a loaded model by name
    pub fn get(&self, name: &str) -> Option<Arc<LoadedModel>> {
        self.loaded_models.get(name).cloned()
    }
    
    /// List loaded models
    pub fn list_loaded(&self) -> Vec<String> {
        self.loaded_models.keys().cloned().collect()
    }
    
    /// Extract model information
    fn extract_info(model: &LlamaModel, path: &Path) -> ModelInfo {
        let name = model.meta_val_str("general.name")
            .ok()
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());
        
        let architecture = model.meta_val_str("general.architecture").ok();
        
        let has_chat_template = model.chat_template(None).is_ok();
        
        ModelInfo {
            path: path.to_path_buf(),
            name,
            architecture,
            vocab_size: model.n_vocab(),
            embedding_dim: model.n_embd(),
            context_length: model.n_ctx_train(),
            n_params: model.n_params(),
            size_bytes: model.size(),
            has_chat_template,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_info_serialization() {
        let info = ModelInfo {
            path: PathBuf::from("test.gguf"),
            name: "test-model".to_string(),
            architecture: Some("llama".to_string()),
            vocab_size: 32000,
            embedding_dim: 4096,
            context_length: 4096,
            n_params: 7_000_000_000,
            size_bytes: 4_000_000_000,
            has_chat_template: true,
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ModelInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.name, "test-model");
        assert_eq!(parsed.vocab_size, 32000);
    }
}
