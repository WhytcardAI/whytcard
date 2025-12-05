//! Configuration types for the LLM engine

use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::path::PathBuf;

/// Main LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Directory for storing models
    pub models_dir: PathBuf,
    
    /// Default model to load on startup
    pub default_model: Option<String>,
    
    /// Number of threads for generation
    pub n_threads: i32,
    
    /// Number of threads for batch processing
    pub n_threads_batch: i32,
    
    /// Enable GPU acceleration
    pub use_gpu: bool,
    
    /// Number of layers to offload to GPU
    pub n_gpu_layers: u32,
    
    /// Enable logging
    pub enable_logging: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        let num_cpus = std::thread::available_parallelism()
            .map(|p| p.get() as i32)
            .unwrap_or(4);
        
        Self {
            models_dir: PathBuf::from("data/models/llm"),
            default_model: None,
            n_threads: num_cpus / 2,
            n_threads_batch: num_cpus,
            use_gpu: true,
            n_gpu_layers: 1000, // Offload all layers by default
            enable_logging: false,
        }
    }
}

/// Model-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to the GGUF file
    pub path: PathBuf,
    
    /// Context window size (None = use model default)
    pub context_size: Option<NonZeroU32>,
    
    /// Batch size for prompt processing
    pub batch_size: u32,
    
    /// Micro-batch size
    pub ubatch_size: u32,
    
    /// Number of GPU layers to offload
    pub n_gpu_layers: Option<u32>,
    
    /// Use memory mapping
    pub use_mmap: bool,
    
    /// Lock model in memory
    pub use_mlock: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            context_size: NonZeroU32::new(4096),
            batch_size: 512,
            ubatch_size: 256,
            n_gpu_layers: None, // Use engine default
            use_mmap: true,
            use_mlock: false,
        }
    }
}

impl ModelConfig {
    /// Create config from a model path
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
    
    /// Set context size
    pub fn with_context_size(mut self, size: u32) -> Self {
        self.context_size = NonZeroU32::new(size);
        self
    }
    
    /// Set GPU layers
    pub fn with_gpu_layers(mut self, layers: u32) -> Self {
        self.n_gpu_layers = Some(layers);
        self
    }
}

/// Generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Maximum tokens to generate
    pub max_tokens: u32,
    
    /// Temperature for sampling (0.0 = deterministic)
    pub temperature: f32,
    
    /// Top-k sampling (0 = disabled)
    pub top_k: i32,
    
    /// Top-p (nucleus) sampling
    pub top_p: f32,
    
    /// Min-p sampling threshold
    pub min_p: f32,
    
    /// Repetition penalty
    pub repeat_penalty: f32,
    
    /// Tokens to look back for repetition penalty
    pub repeat_last_n: i32,
    
    /// Frequency penalty
    pub frequency_penalty: f32,
    
    /// Presence penalty
    pub presence_penalty: f32,
    
    /// Random seed (None = random)
    pub seed: Option<u32>,
    
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    
    /// System prompt to prepend
    pub system_prompt: Option<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.05,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            seed: None,
            stop_sequences: vec![],
            system_prompt: None,
        }
    }
}

impl GenerationConfig {
    /// Create a deterministic (greedy) config
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_k: 1,
            top_p: 1.0,
            min_p: 0.0,
            ..Default::default()
        }
    }
    
    /// Create a creative config
    pub fn creative() -> Self {
        Self {
            temperature: 0.9,
            top_k: 0,
            top_p: 0.95,
            min_p: 0.0,
            ..Default::default()
        }
    }
    
    /// Create a balanced config for coding
    pub fn coding() -> Self {
        Self {
            temperature: 0.2,
            top_k: 40,
            top_p: 0.9,
            min_p: 0.05,
            repeat_penalty: 1.05,
            ..Default::default()
        }
    }
    
    /// Set max tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }
    
    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }
    
    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
    
    /// Add stop sequence
    pub fn with_stop_sequence(mut self, seq: impl Into<String>) -> Self {
        self.stop_sequences.push(seq.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert!(config.use_gpu);
        assert_eq!(config.n_gpu_layers, 1000);
    }
    
    #[test]
    fn test_generation_presets() {
        let greedy = GenerationConfig::greedy();
        assert_eq!(greedy.temperature, 0.0);
        
        let creative = GenerationConfig::creative();
        assert_eq!(creative.temperature, 0.9);
        
        let coding = GenerationConfig::coding();
        assert_eq!(coding.temperature, 0.2);
    }
    
    #[test]
    fn test_model_config_builder() {
        let config = ModelConfig::from_path("test.gguf")
            .with_context_size(8192)
            .with_gpu_layers(32);
        
        assert_eq!(config.context_size, NonZeroU32::new(8192));
        assert_eq!(config.n_gpu_layers, Some(32));
    }
}
