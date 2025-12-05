//! Text chunking for RAG.
//!
//! Provides intelligent text splitting that respects:
//! - Sentence boundaries
//! - Paragraph boundaries
//! - Code block boundaries
//! - UTF-8 character boundaries (safe for multi-byte characters)

use crate::config::ChunkingConfig;
use crate::error::Result;
use crate::types::{Chunk, Document};

/// Find a valid UTF-8 character boundary at or before the given byte index.
/// This ensures we never slice in the middle of a multi-byte character.
fn find_char_boundary(s: &str, byte_index: usize) -> usize {
    if byte_index >= s.len() {
        return s.len();
    }
    // Walk backwards from byte_index until we find a char boundary
    let mut idx = byte_index;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Chunking strategy.
#[derive(Debug, Clone, Copy, Default)]
pub enum ChunkingStrategy {
    /// Split on sentences/paragraphs
    #[default]
    Semantic,
    /// Fixed size chunks
    FixedSize,
    /// Split on code boundaries (functions, classes)
    Code,
}

/// Text chunker.
pub struct Chunker {
    config: ChunkingConfig,
    strategy: ChunkingStrategy,
}

impl Chunker {
    /// Create a new chunker with default config.
    pub fn new() -> Self {
        Self {
            config: ChunkingConfig::default(),
            strategy: ChunkingStrategy::default(),
        }
    }

    /// Create chunker with custom config.
    pub fn with_config(config: ChunkingConfig) -> Self {
        Self {
            config,
            strategy: ChunkingStrategy::default(),
        }
    }

    /// Set chunking strategy.
    pub fn with_strategy(mut self, strategy: ChunkingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Chunk a document into smaller pieces.
    pub fn chunk(&self, document: &Document) -> Result<Vec<Chunk>> {
        let text = &document.content;

        if text.is_empty() {
            return Ok(vec![]);
        }

        let chunks = match self.strategy {
            ChunkingStrategy::Semantic => self.chunk_semantic(text),
            ChunkingStrategy::FixedSize => self.chunk_fixed(text),
            ChunkingStrategy::Code => self.chunk_code(text),
        };

        // Convert raw chunks to Chunk structs
        let result: Vec<Chunk> = chunks
            .into_iter()
            .enumerate()
            .filter(|(_, (text, _, _))| text.len() >= self.config.min_chunk_size)
            .map(|(index, (text, start, end))| {
                let mut chunk = Chunk::new(&document.id, index, text, start, end);
                chunk.metadata = document.metadata.clone();
                chunk
            })
            .collect();

        Ok(result)
    }

    /// Semantic chunking: split on paragraph/sentence boundaries.
    fn chunk_semantic(&self, text: &str) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start: usize = 0;
        let mut byte_pos: usize = 0;

        // Split by paragraphs first
        for paragraph in text.split("\n\n") {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                byte_pos += 2; // Account for \n\n
                continue;
            }

            // If adding this paragraph exceeds chunk size, save current and start new
            if !current_chunk.is_empty()
                && current_chunk.len() + paragraph.len() > self.config.chunk_size
            {
                chunks.push((
                    current_chunk.clone(),
                    current_start,
                    current_start + current_chunk.len(),
                ));

                // Start new chunk with overlap (safe UTF-8 boundary)
                let target_overlap = current_chunk
                    .len()
                    .saturating_sub(self.config.chunk_overlap);
                let overlap_start = find_char_boundary(&current_chunk, target_overlap);
                let overlap_text = &current_chunk[overlap_start..];
                current_start = byte_pos.saturating_sub(overlap_text.len());
                current_chunk = overlap_text.to_string();
            }

            // Add paragraph to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(paragraph);

            byte_pos += paragraph.len() + 2; // +2 for \n\n (bytes)

            // If single paragraph is too large, split by sentences
            if current_chunk.len() > self.config.chunk_size {
                let sentence_chunks = self.split_by_sentences(&current_chunk, current_start);
                chunks.extend(sentence_chunks);
                current_chunk.clear();
                current_start = byte_pos;
            }
        }

        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            chunks.push((
                current_chunk.clone(),
                current_start,
                current_start + current_chunk.len(),
            ));
        }

        chunks
    }

    /// Split text by sentences when paragraphs are too large.
    fn split_by_sentences(&self, text: &str, base_offset: usize) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = base_offset;
        let mut byte_offset: usize = 0;

        // Simple sentence splitting on . ! ?
        for sentence in split_sentences(text) {
            if !current_chunk.is_empty()
                && current_chunk.len() + sentence.len() > self.config.chunk_size
            {
                chunks.push((
                    current_chunk.clone(),
                    current_start,
                    current_start + current_chunk.len(),
                ));

                // Overlap (safe UTF-8 boundary)
                let target_overlap = current_chunk
                    .len()
                    .saturating_sub(self.config.chunk_overlap);
                let overlap_start = find_char_boundary(&current_chunk, target_overlap);
                let overlap_text = &current_chunk[overlap_start..];
                current_start = base_offset + byte_offset.saturating_sub(overlap_text.len());
                current_chunk = overlap_text.to_string();
            }

            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(&sentence);
            byte_offset += sentence.len() + 1;
        }

        if !current_chunk.is_empty() {
            chunks.push((
                current_chunk.clone(),
                current_start,
                current_start + current_chunk.len(),
            ));
        }

        chunks
    }

    /// Fixed size chunking.
    fn chunk_fixed(&self, text: &str) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;

        // Ensure we make progress (step must be > 0)
        let step = self.config.chunk_size.saturating_sub(self.config.chunk_overlap).max(1);

        while start < chars.len() {
            let end = (start + self.config.chunk_size).min(chars.len());
            let chunk_text: String = chars[start..end].iter().collect();

            chunks.push((chunk_text, start, end));

            // Move forward by step
            start += step;
        }

        chunks
    }

    /// Code-aware chunking (split on function/class boundaries).
    fn chunk_code(&self, text: &str) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut char_pos = 0;

        // Split on common code boundaries
        let code_boundaries = [
            "\nfn ",
            "\npub fn ",
            "\nasync fn ",
            "\npub async fn ",
            "\nimpl ",
            "\nstruct ",
            "\nenum ",
            "\ntrait ",
            "\nclass ",
            "\ndef ",
            "\nasync def ",
            "\nfunction ",
            "\nconst ",
            "\nexport ",
        ];

        let lines: Vec<&str> = text.lines().collect();

        for line in lines.iter() {
            let is_boundary = code_boundaries
                .iter()
                .any(|b| line.starts_with(b.trim_start()));

            if is_boundary && !current_chunk.is_empty() {
                // Save current chunk
                chunks.push((
                    current_chunk.clone(),
                    current_start,
                    current_start + current_chunk.len(),
                ));
                current_chunk.clear();
                current_start = char_pos;
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
            char_pos += line.len() + 1;

            // If chunk is too big, force split
            if current_chunk.len() > self.config.chunk_size * 2 {
                chunks.push((
                    current_chunk.clone(),
                    current_start,
                    current_start + current_chunk.len(),
                ));
                current_chunk.clear();
                current_start = char_pos;
            }
        }

        if !current_chunk.is_empty() {
            chunks.push((
                current_chunk.clone(),
                current_start,
                current_start + current_chunk.len(),
            ));
        }

        chunks
    }
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new()
    }
}

/// Split text into sentences.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for c in text.chars() {
        current.push(c);

        if c == '.' || c == '!' || c == '?' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    // Remaining text
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(content: &str) -> Document {
        Document::new(content)
    }

    #[test]
    fn test_empty_document() {
        let chunker = Chunker::new();
        let doc = make_doc("");
        let chunks = chunker.chunk(&doc).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_small_document() {
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 100,
            chunk_overlap: 10,
            min_chunk_size: 5,
        });
        let doc = make_doc("Hello world. This is a test.");
        let chunks = chunker.chunk(&doc).unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hello world. This is a test.");
    }

    #[test]
    fn test_large_document_splits() {
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 50,
            chunk_overlap: 10,
            min_chunk_size: 10,
        });

        let content = "This is paragraph one with some content.\n\n\
                       This is paragraph two with more content.\n\n\
                       This is paragraph three with even more content.";

        let doc = make_doc(content);
        let chunks = chunker.chunk(&doc).unwrap();

        assert!(chunks.len() > 1);

        // All chunks should have content
        for chunk in &chunks {
            assert!(!chunk.text.is_empty());
            assert!(chunk.token_count > 0);
        }
    }

    #[test]
    fn test_fixed_size_chunking() {
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 20,
            chunk_overlap: 5,
            min_chunk_size: 5,
        })
        .with_strategy(ChunkingStrategy::FixedSize);

        let doc = make_doc("This is a test document with some content that should be split.");
        let chunks = chunker.chunk(&doc).unwrap();

        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_code_chunking() {
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 100,
            chunk_overlap: 10,
            min_chunk_size: 10,
        })
        .with_strategy(ChunkingStrategy::Code);

        let code = r#"
fn hello() {
    println!("Hello");
}

fn world() {
    println!("World");
}

fn main() {
    hello();
    world();
}
"#;

        let doc = make_doc(code);
        let chunks = chunker.chunk(&doc).unwrap();

        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_chunk_metadata_inheritance() {
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 500,
            chunk_overlap: 50,
            min_chunk_size: 10,
        });
        let doc = Document::new("Hello world content here with enough text to pass minimum size")
            .with_metadata(serde_json::json!({"key": "value"}));

        let chunks = chunker.chunk(&doc).unwrap();

        assert!(!chunks.is_empty());
        assert_eq!(
            chunks[0].metadata,
            Some(serde_json::json!({"key": "value"}))
        );
    }

    #[test]
    fn test_split_sentences() {
        let sentences = split_sentences("Hello world. How are you? I am fine!");
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
        assert_eq!(sentences[2], "I am fine!");
    }

    #[test]
    fn test_chunk_indices() {
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 50,
            chunk_overlap: 0,
            min_chunk_size: 5,
        });

        let doc = make_doc("First chunk content.\n\nSecond chunk content.");
        let chunks = chunker.chunk(&doc).unwrap();

        // Verify indices are sequential
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_utf8_french_accents() {
        // Test that UTF-8 multi-byte characters (French accents) don't cause panics
        let chunker = Chunker::with_config(ChunkingConfig {
            chunk_size: 30,
            chunk_overlap: 10,
            min_chunk_size: 5,
        });

        // Text with French accents (é = 2 bytes in UTF-8)
        let content = "Voici un texte en français avec des accents : é, è, ê, à, ù, ô, î, ç.\n\n\
                       Ceci est un deuxième paragraphe également accentué.";

        let doc = make_doc(content);
        let result = chunker.chunk(&doc);

        // Should not panic and should produce valid chunks
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(!chunks.is_empty());

        // Verify all chunks are valid UTF-8 strings
        for chunk in &chunks {
            assert!(chunk.text.is_ascii() || chunk.text.chars().count() > 0);
        }
    }

    #[test]
    fn test_find_char_boundary() {
        // Test the helper function
        // "Café" = C(1) a(1) f(1) é(2) = 5 bytes total
        // Indices: C=0, a=1, f=2, é=3-4
        let s = "Café";

        // Index 0 (at 'C') should stay at 0
        assert_eq!(super::find_char_boundary(s, 0), 0);

        // Index 1 (at 'a') should stay at 1
        assert_eq!(super::find_char_boundary(s, 1), 1);

        // Index 2 (at 'f') should stay at 2
        assert_eq!(super::find_char_boundary(s, 2), 2);

        // Index 3 (start of 'é') should stay at 3
        assert_eq!(super::find_char_boundary(s, 3), 3);

        // Index 4 (middle of 'é', second byte) should go back to 3
        assert_eq!(super::find_char_boundary(s, 4), 3);

        // Index 5 (end of string) should return 5
        assert_eq!(super::find_char_boundary(s, 5), 5);

        // Index beyond string length should return string length
        assert_eq!(super::find_char_boundary(s, 100), s.len());
    }
}
