//! Main LLM engine - the heart of inference

use crate::config::{GenerationConfig, LlmConfig, ModelConfig};
use crate::error::{LlmError, Result};
use crate::model::{LoadedModel, ModelManager};
use crate::session::{ChatSession, MessageRole};
use crate::streaming::{StopReason, StreamSender, TokenStream};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, Special};
use llama_cpp_2::sampling::LlamaSampler;

use std::sync::Arc;
use tracing::{debug, info, warn};

/// Callback for streaming tokens
pub type TokenCallback = Box<dyn FnMut(&str, u32, bool) -> bool + Send>;

/// The main LLM inference engine
pub struct LlmEngine {
    /// llama.cpp backend
    backend: Arc<LlamaBackend>,
    
    /// Model manager
    model_manager: ModelManager,
    
    /// Engine configuration
    config: LlmConfig,
    
    /// Currently active model name
    active_model: Option<String>,
}

impl LlmEngine {
    /// Create a new LLM engine with default config
    pub fn new() -> Result<Self> {
        Self::with_config(LlmConfig::default())
    }
    
    /// Create engine with custom config
    pub fn with_config(config: LlmConfig) -> Result<Self> {
        // Initialize llama.cpp backend
        let backend = LlamaBackend::init()
            .map_err(|e| LlmError::BackendError(format!("Failed to init backend: {}", e)))?;
        
        let backend = Arc::new(backend);
        
        // Check capabilities
        if backend.supports_gpu_offload() {
            info!("GPU acceleration available");
        } else {
            warn!("GPU acceleration not available, using CPU");
        }
        
        let model_manager = ModelManager::new(Arc::clone(&backend), &config.models_dir)
            .with_default_gpu_layers(config.n_gpu_layers);
        
        Ok(Self {
            backend,
            model_manager,
            config,
            active_model: None,
        })
    }
    
    /// Load a model from path
    pub fn load_model(&mut self, path: impl Into<std::path::PathBuf>) -> Result<()> {
        let config = ModelConfig::from_path(path);
        let model = self.model_manager.load(config)?;
        self.active_model = Some(model.info.name.clone());
        Ok(())
    }
    
    /// Load a model with custom config
    pub fn load_model_with_config(&mut self, config: ModelConfig) -> Result<()> {
        let model = self.model_manager.load(config)?;
        self.active_model = Some(model.info.name.clone());
        Ok(())
    }
    
    /// Load a model by name
    pub fn load_model_by_name(&mut self, name: &str) -> Result<()> {
        let model = self.model_manager.load_by_name(name)?;
        self.active_model = Some(model.info.name.clone());
        Ok(())
    }
    
    /// Unload current model
    pub fn unload_model(&mut self) -> bool {
        if let Some(name) = self.active_model.take() {
            self.model_manager.unload(&name)
        } else {
            false
        }
    }
    
    /// Get active model
    pub fn active_model(&self) -> Option<Arc<LoadedModel>> {
        self.active_model.as_ref()
            .and_then(|name| self.model_manager.get(name))
    }
    
    /// Check if a model is loaded
    pub fn has_model(&self) -> bool {
        self.active_model.is_some()
    }
    
    /// List available models
    pub fn list_available_models(&self) -> Result<Vec<std::path::PathBuf>> {
        self.model_manager.list_available()
    }

    /// List loaded models
    pub fn list_loaded_models(&self) -> Vec<String> {
        self.model_manager.list_loaded()
    }

    /// Generate text from a prompt
    pub fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        self.generate_with_callback(prompt, config, None)
    }

    /// Generate text with streaming callback
    /// 
    /// The callback receives (token_text, token_id, is_special) and returns
    /// true to continue or false to stop generation.
    pub fn generate_with_callback(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        mut callback: Option<TokenCallback>,
    ) -> Result<String> {
        let model = self.active_model()
            .ok_or(LlmError::NoModelLoaded)?;
        
        // Create context
        let ctx_params = self.build_context_params(&model.config);
        let mut ctx = model.model.new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::ContextError(e.to_string()))?;
        
        // Build full prompt with system if present
        let full_prompt = if let Some(system) = &config.system_prompt {
            format!("{}\n\n{}", system, prompt)
        } else {
            prompt.to_string()
        };
        
        // Tokenize
        let tokens = model.model.str_to_token(&full_prompt, AddBos::Always)
            .map_err(|e| LlmError::TokenizationError(e.to_string()))?;
        
        debug!("Prompt tokens: {}", tokens.len());
        
        // Process prompt
        let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch.add(*token, i as i32, &[0], is_last)
                .map_err(|e| LlmError::GenerationError(e.to_string()))?;
        }
        
        ctx.decode(&mut batch)
            .map_err(|e| LlmError::GenerationError(e.to_string()))?;
        
        // Generate
        let mut sampler = Self::build_sampler(config);
        let mut output = String::new();
        let mut pos = tokens.len();
        
        for _ in 0..config.max_tokens {
            // Sample next token
            let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(new_token);
            
            // Check for end
            if model.model.is_eog_token(new_token) {
                break;
            }
            
            // Decode token to text
            let token_str = model.model.token_to_str(new_token, Special::Tokenize)
                .map_err(|e| LlmError::GenerationError(e.to_string()))?;
            
            let token_id = new_token.0 as u32;
            // Note: checking if token is special is complex with enumflags
            // For now, we assume non-special for user tokens
            let is_special = false;
            
            // Call streaming callback
            if let Some(ref mut cb) = callback {
                if !cb(&token_str, token_id, is_special) {
                    break; // Callback requested stop
                }
            }
            
            // Check stop sequences
            output.push_str(&token_str);
            if config.stop_sequences.iter().any(|s| output.ends_with(s)) {
                // Remove stop sequence from output
                for stop in &config.stop_sequences {
                    if output.ends_with(stop) {
                        output.truncate(output.len() - stop.len());
                        break;
                    }
                }
                break;
            }
            
            // Prepare next iteration
            batch.clear();
            batch.add(new_token, pos as i32, &[0], true)
                .map_err(|e| LlmError::GenerationError(e.to_string()))?;
            
            ctx.decode(&mut batch)
                .map_err(|e| LlmError::GenerationError(e.to_string()))?;
            
            pos += 1;
        }
        
        Ok(output)
    }

    /// Generate text with async streaming via channel
    /// 
    /// Spawns generation in a blocking task and sends tokens through a channel.
    pub fn generate_stream(
        &self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<TokenStream> {
        let model = self.active_model()
            .ok_or(LlmError::NoModelLoaded)?;
        
        let (sender, stream) = StreamSender::channel(256);
        
        // Clone what we need for the blocking task
        let model = Arc::clone(&model);
        let backend = Arc::clone(&self.backend);
        let config = config.clone();
        let prompt = prompt.to_string();
        let n_threads = self.config.n_threads;
        let n_threads_batch = self.config.n_threads_batch;
        
        // Spawn blocking task for generation
        tokio::task::spawn_blocking(move || {
            Self::generate_stream_blocking(
                &model,
                &backend,
                &prompt,
                &config,
                n_threads,
                n_threads_batch,
                sender,
            );
        });
        
        Ok(stream)
    }

    /// Blocking streaming generation (runs in spawn_blocking)
    fn generate_stream_blocking(
        model: &LoadedModel,
        backend: &LlamaBackend,
        prompt: &str,
        config: &GenerationConfig,
        n_threads: i32,
        n_threads_batch: i32,
        mut sender: StreamSender,
    ) {
        let result = (|| -> Result<(String, StopReason)> {
            // Create context
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(model.config.context_size)
                .with_n_batch(model.config.batch_size)
                .with_n_threads(n_threads)
                .with_n_threads_batch(n_threads_batch);
            
            let mut ctx = model.model.new_context(backend, ctx_params)
                .map_err(|e| LlmError::ContextError(e.to_string()))?;
            
            // Build prompt
            let full_prompt = if let Some(system) = &config.system_prompt {
                format!("{}\n\n{}", system, prompt)
            } else {
                prompt.to_string()
            };
            
            // Tokenize
            let tokens = model.model.str_to_token(&full_prompt, AddBos::Always)
                .map_err(|e| LlmError::TokenizationError(e.to_string()))?;
            
            // Send start (blocking)
            sender.send_start_blocking(tokens.len());
            
            // Process prompt
            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            for (i, token) in tokens.iter().enumerate() {
                let is_last = i == tokens.len() - 1;
                batch.add(*token, i as i32, &[0], is_last)
                    .map_err(|e| LlmError::GenerationError(e.to_string()))?;
            }
            
            ctx.decode(&mut batch)
                .map_err(|e| LlmError::GenerationError(e.to_string()))?;
            
            // Generate
            let mut sampler = Self::build_sampler(config);
            let mut output = String::new();
            let mut pos = tokens.len();
            let mut stop_reason = StopReason::MaxTokens;
            
            for _ in 0..config.max_tokens {
                // Sample
                let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
                sampler.accept(new_token);
                
                // Check end
                if model.model.is_eog_token(new_token) {
                    stop_reason = StopReason::EndOfGeneration;
                    break;
                }
                
                // Decode token
                let token_str = model.model.token_to_str(new_token, Special::Tokenize)
                    .map_err(|e| LlmError::GenerationError(e.to_string()))?;
                
                let token_id = new_token.0 as u32;
                
                // Send token (blocking)
                if !sender.send_token_blocking(token_str.clone(), token_id, false) {
                    stop_reason = StopReason::Cancelled;
                    break;
                }
                
                // Check stop sequences
                output.push_str(&token_str);
                let mut hit_stop = false;
                for stop in &config.stop_sequences {
                    if output.ends_with(stop) {
                        output.truncate(output.len() - stop.len());
                        stop_reason = StopReason::StopSequence;
                        hit_stop = true;
                        break;
                    }
                }
                if hit_stop {
                    break;
                }
                
                // Next iteration
                batch.clear();
                batch.add(new_token, pos as i32, &[0], true)
                    .map_err(|e| LlmError::GenerationError(e.to_string()))?;
                
                ctx.decode(&mut batch)
                    .map_err(|e| LlmError::GenerationError(e.to_string()))?;
                
                pos += 1;
            }
            
            Ok((output, stop_reason))
        })();
        
        match result {
            Ok((output, stop_reason)) => {
                sender.send_done_blocking(output, stop_reason);
            }
            Err(e) => {
                sender.send_error_blocking(e.to_string());
            }
        }
    }

    /// Chat completion with session
    pub fn chat(&self, session: &mut ChatSession, message: &str, config: &GenerationConfig) -> Result<String> {
        let model = self.active_model()
            .ok_or(LlmError::NoModelLoaded)?;
        
        // Add user message
        session.add_user_message(message);
        
        // Build prompt using chat template if available
        let prompt = self.build_chat_prompt(&model, session, config)?;
        
        // Generate
        let mut temp_config = config.clone();
        temp_config.system_prompt = None; // Already in chat template
        
        let response = self.generate(&prompt, &temp_config)?;
        
        // Add assistant response
        session.add_assistant_message(&response);
        
        Ok(response)
    }

    /// Chat with streaming
    pub fn chat_stream(
        &self,
        session: &mut ChatSession,
        message: &str,
        config: &GenerationConfig,
    ) -> Result<TokenStream> {
        let model = self.active_model()
            .ok_or(LlmError::NoModelLoaded)?;
        
        // Add user message
        session.add_user_message(message);
        
        // Build prompt
        let prompt = self.build_chat_prompt(&model, session, config)?;
        
        // Generate
        let mut temp_config = config.clone();
        temp_config.system_prompt = None;
        
        self.generate_stream(&prompt, &temp_config)
    }

    /// Build chat prompt from session
    fn build_chat_prompt(
        &self,
        model: &LoadedModel,
        session: &ChatSession,
        config: &GenerationConfig,
    ) -> Result<String> {
        // Try to use model's chat template
        if let Ok(template) = model.model.chat_template(None) {
            let messages: Vec<LlamaChatMessage> = session.get_messages_with_system()
                .iter()
                .filter_map(|m| {
                    LlamaChatMessage::new(
                        m.role.as_str().to_string(),
                        m.content.clone(),
                    ).ok()
                })
                .collect();
            
            if !messages.is_empty() {
                if let Ok(prompt) = model.model.apply_chat_template(&template, &messages, true) {
                    return Ok(prompt);
                }
            }
        }
        
        // Fallback: simple concatenation
        let mut prompt = String::new();
        
        if let Some(system) = session.system_prompt.as_ref().or(config.system_prompt.as_ref()) {
            prompt.push_str(&format!("System: {}\n\n", system));
        }
        
        for msg in session.get_messages() {
            let role = match msg.role {
                MessageRole::System => "System",
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
            };
            prompt.push_str(&format!("{}: {}\n", role, msg.content));
        }
        
        prompt.push_str("Assistant:");
        
        Ok(prompt)
    }

    /// Build context parameters
    fn build_context_params(&self, model_config: &ModelConfig) -> LlamaContextParams {
        LlamaContextParams::default()
            .with_n_ctx(model_config.context_size)
            .with_n_batch(model_config.batch_size)
            .with_n_ubatch(model_config.ubatch_size)
            .with_n_threads(self.config.n_threads)
            .with_n_threads_batch(self.config.n_threads_batch)
    }

    /// Build sampler from config
    fn build_sampler(config: &GenerationConfig) -> LlamaSampler {
        let seed = config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            (duration.as_nanos() % u32::MAX as u128) as u32
        });
        
        if config.temperature <= 0.0 {
            // Greedy sampling
            LlamaSampler::greedy()
        } else {
            // Build sampler chain
            let mut samplers = Vec::new();
            
            if config.top_k > 0 {
                samplers.push(LlamaSampler::top_k(config.top_k));
            }
            
            if config.top_p < 1.0 {
                samplers.push(LlamaSampler::top_p(config.top_p, 1));
            }
            
            if config.min_p > 0.0 {
                samplers.push(LlamaSampler::min_p(config.min_p, 1));
            }
            
            samplers.push(LlamaSampler::temp(config.temperature));
            samplers.push(LlamaSampler::dist(seed));
            
            LlamaSampler::chain_simple(samplers)
        }
    }

    /// Get backend capabilities
    pub fn supports_gpu(&self) -> bool {
        self.backend.supports_gpu_offload()
    }

    /// Get config
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generation_config_sampler() {
        let config = GenerationConfig::default();
        let sampler = LlmEngine::build_sampler(&config);
        // Just verify it doesn't panic
        drop(sampler);
    }
    
    #[test]
    fn test_greedy_sampler() {
        let config = GenerationConfig::greedy();
        let sampler = LlmEngine::build_sampler(&config);
        drop(sampler);
    }
}
