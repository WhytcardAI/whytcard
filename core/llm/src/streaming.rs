//! Token streaming for real-time generation

use crate::error::{LlmError, Result};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Events emitted during token streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Generation started
    Start {
        /// Prompt token count
        prompt_tokens: usize,
    },
    
    /// New token generated
    Token {
        /// The token text
        text: String,
        /// Token ID
        token_id: u32,
        /// Is this a special token
        is_special: bool,
    },
    
    /// Generation progress update
    Progress {
        /// Tokens generated so far
        tokens_generated: usize,
        /// Tokens per second
        tokens_per_second: f32,
    },
    
    /// Generation completed
    Done {
        /// Full generated text
        text: String,
        /// Total tokens generated
        tokens_generated: usize,
        /// Prompt tokens
        prompt_tokens: usize,
        /// Total time in milliseconds
        duration_ms: u64,
        /// Stop reason
        stop_reason: StopReason,
    },
    
    /// Error during generation
    Error {
        /// Error message
        message: String,
    },
}

/// Reason for stopping generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Reached max tokens
    MaxTokens,
    /// Hit end-of-generation token
    EndOfGeneration,
    /// Hit a stop sequence
    StopSequence,
    /// User cancelled
    Cancelled,
    /// Error occurred
    Error,
}

/// Token stream receiver
pub struct TokenStream {
    receiver: mpsc::Receiver<StreamEvent>,
    buffer: String,
    is_done: bool,
}

impl TokenStream {
    /// Create a new token stream
    pub(crate) fn new(receiver: mpsc::Receiver<StreamEvent>) -> Self {
        Self {
            receiver,
            buffer: String::new(),
            is_done: false,
        }
    }
    
    /// Receive the next event
    pub async fn next(&mut self) -> Option<StreamEvent> {
        if self.is_done {
            return None;
        }
        
        let event = self.receiver.recv().await?;
        
        // Track tokens in buffer
        if let StreamEvent::Token { ref text, .. } = event {
            self.buffer.push_str(text);
        }
        
        // Mark done
        if matches!(event, StreamEvent::Done { .. } | StreamEvent::Error { .. }) {
            self.is_done = true;
        }
        
        Some(event)
    }
    
    /// Collect all tokens into a string (blocks until done)
    pub async fn collect(mut self) -> Result<String> {
        while let Some(event) = self.next().await {
            match event {
                StreamEvent::Done { text, .. } => return Ok(text),
                StreamEvent::Error { message } => return Err(LlmError::GenerationError(message)),
                _ => continue,
            }
        }
        
        // If stream ended without Done event, return buffer
        Ok(self.buffer)
    }
    
    /// Get the current buffer content
    pub fn current_text(&self) -> &str {
        &self.buffer
    }
    
    /// Check if stream is done
    pub fn is_done(&self) -> bool {
        self.is_done
    }
}

/// Stream sender for the engine
pub struct StreamSender {
    sender: mpsc::Sender<StreamEvent>,
    tokens_generated: usize,
    prompt_tokens: usize,
    start_time: std::time::Instant,
}

impl StreamSender {
    /// Create a new stream channel
    pub fn channel(buffer_size: usize) -> (Self, TokenStream) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        
        let stream_sender = Self {
            sender,
            tokens_generated: 0,
            prompt_tokens: 0,
            start_time: std::time::Instant::now(),
        };
        
        let stream = TokenStream::new(receiver);
        
        (stream_sender, stream)
    }
    
    /// Send start event
    pub async fn send_start(&mut self, prompt_tokens: usize) -> Result<()> {
        self.prompt_tokens = prompt_tokens;
        self.sender
            .send(StreamEvent::Start { prompt_tokens })
            .await
            .map_err(|_| LlmError::ChannelError("Failed to send start event".into()))
    }
    
    /// Send a token
    pub async fn send_token(&mut self, text: String, token_id: u32, is_special: bool) -> Result<()> {
        self.tokens_generated += 1;
        
        self.sender
            .send(StreamEvent::Token {
                text,
                token_id,
                is_special,
            })
            .await
            .map_err(|_| LlmError::ChannelError("Failed to send token".into()))
    }
    
    /// Send progress update
    pub async fn send_progress(&self) -> Result<()> {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let tps = if elapsed > 0.0 {
            self.tokens_generated as f32 / elapsed
        } else {
            0.0
        };
        
        self.sender
            .send(StreamEvent::Progress {
                tokens_generated: self.tokens_generated,
                tokens_per_second: tps,
            })
            .await
            .map_err(|_| LlmError::ChannelError("Failed to send progress".into()))
    }
    
    /// Send done event
    pub async fn send_done(
        &self,
        text: String,
        prompt_tokens: usize,
        stop_reason: StopReason,
    ) -> Result<()> {
        self.sender
            .send(StreamEvent::Done {
                text,
                tokens_generated: self.tokens_generated,
                prompt_tokens,
                duration_ms: self.start_time.elapsed().as_millis() as u64,
                stop_reason,
            })
            .await
            .map_err(|_| LlmError::ChannelError("Failed to send done".into()))
    }
    
    /// Send error event
    pub async fn send_error(&self, message: String) -> Result<()> {
        self.sender
            .send(StreamEvent::Error { message })
            .await
            .map_err(|_| LlmError::ChannelError("Failed to send error".into()))
    }
    
    // Blocking variants for use in spawn_blocking
    
    /// Send start event (blocking)
    pub fn send_start_blocking(&mut self, prompt_tokens: usize) {
        self.prompt_tokens = prompt_tokens;
        let _ = self.sender.blocking_send(StreamEvent::Start { prompt_tokens });
    }
    
    /// Send a token (blocking) - returns false if receiver is closed
    pub fn send_token_blocking(&mut self, text: String, token_id: u32, is_special: bool) -> bool {
        self.tokens_generated += 1;
        self.sender.blocking_send(StreamEvent::Token {
            text,
            token_id,
            is_special,
        }).is_ok()
    }
    
    /// Send done event (blocking)
    pub fn send_done_blocking(&self, text: String, stop_reason: StopReason) {
        let _ = self.sender.blocking_send(StreamEvent::Done {
            text,
            tokens_generated: self.tokens_generated,
            prompt_tokens: self.prompt_tokens,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            stop_reason,
        });
    }
    
    /// Send error (blocking)
    pub fn send_error_blocking(&self, message: String) {
        let _ = self.sender.blocking_send(StreamEvent::Error { message });
    }
    
    /// Get tokens generated so far
    pub fn tokens_generated(&self) -> usize {
        self.tokens_generated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stream_channel() {
        let (mut sender, mut stream) = StreamSender::channel(10);
        
        // Send some events
        sender.send_start(10).await.unwrap();
        sender.send_token("Hello".into(), 1, false).await.unwrap();
        sender.send_token(" world".into(), 2, false).await.unwrap();
        sender.send_done("Hello world".into(), 10, StopReason::EndOfGeneration).await.unwrap();
        
        // Receive events
        let start = stream.next().await.unwrap();
        assert!(matches!(start, StreamEvent::Start { prompt_tokens: 10 }));
        
        let token1 = stream.next().await.unwrap();
        if let StreamEvent::Token { text, .. } = token1 {
            assert_eq!(text, "Hello");
        }
        
        let token2 = stream.next().await.unwrap();
        if let StreamEvent::Token { text, .. } = token2 {
            assert_eq!(text, " world");
        }
        
        let done = stream.next().await.unwrap();
        assert!(matches!(done, StreamEvent::Done { .. }));
        
        assert!(stream.is_done());
    }
    
    #[tokio::test]
    async fn test_stream_collect() {
        let (mut sender, stream) = StreamSender::channel(10);
        
        tokio::spawn(async move {
            sender.send_start(5).await.unwrap();
            sender.send_token("Test".into(), 1, false).await.unwrap();
            sender.send_done("Test".into(), 5, StopReason::MaxTokens).await.unwrap();
        });
        
        let result = stream.collect().await.unwrap();
        assert_eq!(result, "Test");
    }
    
    #[test]
    fn test_event_serialization() {
        let event = StreamEvent::Token {
            text: "hello".into(),
            token_id: 42,
            is_special: false,
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("token"));
        assert!(json.contains("hello"));
    }
}
