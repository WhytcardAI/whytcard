//! Sampling strategies for token generation

use serde::{Deserialize, Serialize};

/// Sampling strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SamplingStrategy {
    /// Greedy sampling (always pick highest probability)
    Greedy,
    
    /// Temperature-based sampling
    Temperature {
        temperature: f32,
    },
    
    /// Top-k sampling
    TopK {
        k: i32,
        temperature: f32,
    },
    
    /// Top-p (nucleus) sampling
    TopP {
        p: f32,
        temperature: f32,
    },
    
    /// Min-p sampling
    MinP {
        p: f32,
        temperature: f32,
    },
    
    /// Combined strategy (typical for chat)
    Combined {
        temperature: f32,
        top_k: i32,
        top_p: f32,
        min_p: f32,
    },
    
    /// Mirostat sampling (dynamic entropy control)
    Mirostat {
        tau: f32,
        eta: f32,
        version: u8, // 1 or 2
    },
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        Self::Combined {
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.05,
        }
    }
}

impl SamplingStrategy {
    /// Create a greedy sampler
    pub fn greedy() -> Self {
        Self::Greedy
    }
    
    /// Create a temperature-only sampler
    pub fn temperature(temp: f32) -> Self {
        Self::Temperature { temperature: temp }
    }
    
    /// Create a top-k sampler
    pub fn top_k(k: i32, temp: f32) -> Self {
        Self::TopK { k, temperature: temp }
    }
    
    /// Create a top-p sampler
    pub fn top_p(p: f32, temp: f32) -> Self {
        Self::TopP { p, temperature: temp }
    }
    
    /// Create a combined sampler
    pub fn combined(temp: f32, top_k: i32, top_p: f32, min_p: f32) -> Self {
        Self::Combined {
            temperature: temp,
            top_k,
            top_p,
            min_p,
        }
    }
    
    /// Create Mirostat v2 sampler
    pub fn mirostat_v2(tau: f32, eta: f32) -> Self {
        Self::Mirostat {
            tau,
            eta,
            version: 2,
        }
    }
}

/// Penalty configuration for repetition control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyConfig {
    /// Repetition penalty multiplier
    pub repeat_penalty: f32,
    
    /// Number of tokens to look back
    pub repeat_last_n: i32,
    
    /// Frequency penalty (0.0 = disabled)
    pub frequency_penalty: f32,
    
    /// Presence penalty (0.0 = disabled)
    pub presence_penalty: f32,
}

impl Default for PenaltyConfig {
    fn default() -> Self {
        Self {
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
}

impl PenaltyConfig {
    /// No penalties
    pub fn none() -> Self {
        Self {
            repeat_penalty: 1.0,
            repeat_last_n: 0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
    
    /// Light penalties for creative text
    pub fn light() -> Self {
        Self {
            repeat_penalty: 1.05,
            repeat_last_n: 32,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
    
    /// Strong penalties to avoid repetition
    pub fn strong() -> Self {
        Self {
            repeat_penalty: 1.2,
            repeat_last_n: 128,
            frequency_penalty: 0.5,
            presence_penalty: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sampling_strategies() {
        let greedy = SamplingStrategy::greedy();
        assert!(matches!(greedy, SamplingStrategy::Greedy));
        
        let temp = SamplingStrategy::temperature(0.8);
        if let SamplingStrategy::Temperature { temperature } = temp {
            assert!((temperature - 0.8).abs() < f32::EPSILON);
        } else {
            panic!("Wrong strategy type");
        }
    }
    
    #[test]
    fn test_penalty_presets() {
        let none = PenaltyConfig::none();
        assert!((none.repeat_penalty - 1.0).abs() < f32::EPSILON);
        
        let strong = PenaltyConfig::strong();
        assert!((strong.repeat_penalty - 1.2).abs() < f32::EPSILON);
    }
}
