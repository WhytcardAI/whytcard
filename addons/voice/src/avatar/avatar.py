"""
Avatar Engine - Combines LLM with voice cloning for interactive AI persona.

Uses llama.cpp for local LLM inference and XTTS for voice synthesis.
"""

import os
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional, Generator

try:
    from llama_cpp import Llama
    HAS_LLAMA = True
except ImportError:
    HAS_LLAMA = False

from .prompt_generator import PersonaPrompt
from ..voice.cloner import VoiceCloner


@dataclass
class AvatarResponse:
    """Response from the avatar."""
    text: str
    audio_path: Optional[Path] = None
    tokens_used: int = 0
    generation_time: float = 0.0


@dataclass
class ConversationMessage:
    """A message in a conversation."""
    role: str  # "user" or "assistant"
    content: str


class AvatarEngine:
    """
    Interactive AI avatar with personality and voice.

    Combines:
    - LLM (llama.cpp) for text generation with persona prompt
    - XTTS for voice synthesis with cloned voice
    """

    def __init__(
        self,
        model_path: str,
        persona: PersonaPrompt,
        speaker_wav: Optional[str] = None,
        output_dir: str = "data/voices/conversations",
        n_ctx: int = 4096,
        n_gpu_layers: int = -1,  # -1 = all layers on GPU
        enable_voice: bool = True,
    ):
        """
        Initialize the avatar.

        Args:
            model_path: Path to GGUF model file
            persona: PersonaPrompt with system prompt and personality
            speaker_wav: Path to reference audio for voice cloning
            output_dir: Directory for voice outputs
            n_ctx: Context window size
            n_gpu_layers: Number of layers to offload to GPU (-1 = all)
            enable_voice: Whether to enable voice synthesis
        """
        if not HAS_LLAMA:
            raise ImportError("llama-cpp-python not installed. Run: pip install llama-cpp-python")

        self.persona = persona
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.enable_voice = enable_voice

        # Load LLM
        print(f"Loading LLM model: {model_path}")
        self.llm = Llama(
            model_path=model_path,
            n_ctx=n_ctx,
            n_gpu_layers=n_gpu_layers,
            verbose=False,
        )
        print("LLM loaded successfully!")

        # Load voice cloner if enabled and speaker provided
        self.voice_cloner = None
        self.speaker_wav = speaker_wav

        if enable_voice and speaker_wav:
            print("Initializing voice cloner...")
            self.voice_cloner = VoiceCloner(output_dir=str(self.output_dir))
            print("Voice cloner ready!")

        # Conversation history
        self.history: list[ConversationMessage] = []
        self.response_counter = 0

    def _build_prompt(self, user_message: str) -> str:
        """Build the full prompt with system prompt, history, and user message."""
        # Format: ChatML style
        prompt_parts = []

        # System prompt
        prompt_parts.append(f"<|im_start|>system\n{self.persona.system_prompt}<|im_end|>")

        # Conversation history
        for msg in self.history:
            prompt_parts.append(f"<|im_start|>{msg.role}\n{msg.content}<|im_end|>")

        # Current user message
        prompt_parts.append(f"<|im_start|>user\n{user_message}<|im_end|>")

        # Assistant prefix
        prompt_parts.append("<|im_start|>assistant\n")

        return "\n".join(prompt_parts)

    def chat(
        self,
        message: str,
        max_tokens: int = 512,
        temperature: float = 0.7,
        top_p: float = 0.9,
        synthesize_voice: bool = True,
        language: str = "fr",
    ) -> AvatarResponse:
        """
        Send a message and get a response.

        Args:
            message: User message
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            top_p: Top-p sampling
            synthesize_voice: Whether to synthesize voice for this response
            language: Language for voice synthesis

        Returns:
            AvatarResponse with text and optional audio
        """
        import time
        start_time = time.time()

        # Build prompt
        prompt = self._build_prompt(message)

        # Generate response
        response = self.llm(
            prompt,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            stop=["<|im_end|>", "<|im_start|>"],
        )

        # Extract text
        text = response["choices"][0]["text"].strip()
        tokens_used = response["usage"]["total_tokens"]

        # Update history
        self.history.append(ConversationMessage(role="user", content=message))
        self.history.append(ConversationMessage(role="assistant", content=text))

        generation_time = time.time() - start_time

        # Synthesize voice if enabled
        audio_path = None
        if synthesize_voice and self.voice_cloner and self.speaker_wav:
            self.response_counter += 1
            audio_output = self.output_dir / f"response_{self.response_counter:04d}.wav"

            voice_result = self.voice_cloner.clone_voice(
                text=text,
                speaker_wav=self.speaker_wav,
                language=language,
                output_path=str(audio_output),
            )

            if voice_result.success:
                audio_path = voice_result.output_path

        return AvatarResponse(
            text=text,
            audio_path=audio_path,
            tokens_used=tokens_used,
            generation_time=generation_time,
        )

    def chat_stream(
        self,
        message: str,
        max_tokens: int = 512,
        temperature: float = 0.7,
    ) -> Generator[str, None, None]:
        """
        Stream a response token by token.

        Args:
            message: User message
            max_tokens: Maximum tokens
            temperature: Sampling temperature

        Yields:
            Tokens as they are generated
        """
        prompt = self._build_prompt(message)

        full_response = ""

        for output in self.llm(
            prompt,
            max_tokens=max_tokens,
            temperature=temperature,
            stop=["<|im_end|>", "<|im_start|>"],
            stream=True,
        ):
            token = output["choices"][0]["text"]
            full_response += token
            yield token

        # Update history after streaming completes
        self.history.append(ConversationMessage(role="user", content=message))
        self.history.append(ConversationMessage(role="assistant", content=full_response.strip()))

    def reset_conversation(self) -> None:
        """Clear conversation history."""
        self.history = []
        self.response_counter = 0

    def get_conversation_summary(self) -> str:
        """Get a summary of the conversation."""
        if not self.history:
            return "No conversation yet."

        lines = []
        for msg in self.history:
            prefix = "User:" if msg.role == "user" else f"{self.persona.name}:"
            lines.append(f"{prefix} {msg.content[:100]}{'...' if len(msg.content) > 100 else ''}")

        return "\n".join(lines)

    def save_conversation(self, path: str) -> None:
        """Save conversation to file."""
        import json

        data = {
            "persona_name": self.persona.name,
            "messages": [
                {"role": msg.role, "content": msg.content}
                for msg in self.history
            ],
        }

        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    def load_conversation(self, path: str) -> None:
        """Load conversation from file."""
        import json

        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.history = [
            ConversationMessage(role=msg["role"], content=msg["content"])
            for msg in data["messages"]
        ]


# CLI interface
if __name__ == "__main__":
    import sys

    print("AvatarEngine - Interactive AI persona with voice")
    print("")
    print("Example usage:")
    print("  from src.avatar.prompt_generator import PromptGenerator, PersonaPrompt")
    print("  from src.avatar.avatar import AvatarEngine")
    print("  from src.research.extractor import PersonProfile")
    print("")
    print("  # Load or create persona")
    print("  persona = PersonaPrompt.load('data/subjects/john/persona.json')")
    print("")
    print("  # Create avatar")
    print("  avatar = AvatarEngine(")
    print("      model_path='models/gguf/qwen2.5-coder-7b-instruct-q4_k_m.gguf',")
    print("      persona=persona,")
    print("      speaker_wav='data/audio/processed/john_vocals.wav',")
    print("  )")
    print("")
    print("  # Chat")
    print("  response = avatar.chat('Bonjour, comment vas-tu?')")
    print("  print(response.text)")
    print("  # Audio saved to response.audio_path")
