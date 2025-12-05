"""
Voice cloning using Coqui XTTS v2.

XTTS is a multi-lingual text-to-speech model that can clone voices
from just a few seconds of audio.
"""

import os
from pathlib import Path
from dataclasses import dataclass
from typing import Optional, Union

import torch

try:
    from TTS.api import TTS
    HAS_TTS = True
except ImportError:
    HAS_TTS = False


@dataclass
class CloneResult:
    """Result of voice cloning synthesis."""
    success: bool
    text: str
    output_path: Optional[Path] = None
    duration: Optional[float] = None
    error: Optional[str] = None


class VoiceCloner:
    """
    Clone a voice and synthesize speech using Coqui XTTS v2.

    XTTS can clone a voice from just 3-10 seconds of reference audio.
    Supports multiple languages including French, English, Spanish, etc.
    """

    SUPPORTED_LANGUAGES = [
        "en", "es", "fr", "de", "it", "pt", "pl", "tr", "ru",
        "nl", "cs", "ar", "zh-cn", "ja", "hu", "ko"
    ]

    def __init__(
        self,
        model_name: str = "tts_models/multilingual/multi-dataset/xtts_v2",
        output_dir: str = "data/voices",
        device: Optional[str] = None,
    ):
        """
        Initialize the voice cloner.

        Args:
            model_name: XTTS model identifier
            output_dir: Directory to save synthesized audio
            device: Device to use (cuda, cpu). Auto-detected if None.
        """
        if not HAS_TTS:
            raise ImportError("TTS not installed. Run: pip install TTS")

        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Auto-detect device
        if device is None:
            self.device = "cuda" if torch.cuda.is_available() else "cpu"
        else:
            self.device = device

        print(f"Loading XTTS model on {self.device}...")
        print("(This may take a few minutes on first run to download the model)")

        self.tts = TTS(model_name).to(self.device)
        print("XTTS model loaded successfully!")

        # Cache for speaker embeddings
        self._speaker_cache: dict[str, dict] = {}

    def clone_voice(
        self,
        text: str,
        speaker_wav: Union[str, list[str]],
        language: str = "fr",
        output_path: Optional[str] = None,
        speed: float = 1.0,
    ) -> CloneResult:
        """
        Synthesize speech using a cloned voice.

        Args:
            text: Text to synthesize
            speaker_wav: Path(s) to reference audio file(s) for voice cloning
            language: Target language code (fr, en, etc.)
            output_path: Path for output audio. If None, auto-generated.
            speed: Speech speed multiplier (0.5-2.0)

        Returns:
            CloneResult with synthesized audio path
        """
        # Validate language
        if language not in self.SUPPORTED_LANGUAGES:
            return CloneResult(
                success=False,
                text=text,
                error=f"Unsupported language: {language}. Supported: {self.SUPPORTED_LANGUAGES}"
            )

        # Handle speaker_wav as list or string
        if isinstance(speaker_wav, str):
            speaker_wav = [speaker_wav]

        # Validate reference audio exists
        for wav in speaker_wav:
            if not Path(wav).exists():
                return CloneResult(
                    success=False,
                    text=text,
                    error=f"Reference audio not found: {wav}"
                )

        # Generate output path if not provided
        if output_path is None:
            # Create filename from text (first 30 chars)
            safe_text = "".join(c for c in text[:30] if c.isalnum() or c in " -_").strip()
            safe_text = safe_text.replace(" ", "_")
            output_path = self.output_dir / f"{safe_text}.wav"
        else:
            output_path = Path(output_path)

        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)

        try:
            print(f"Synthesizing: '{text[:50]}{'...' if len(text) > 50 else ''}'")
            print(f"Using voice from: {speaker_wav}")

            # Synthesize with voice cloning
            self.tts.tts_to_file(
                text=text,
                speaker_wav=speaker_wav,
                language=language,
                file_path=str(output_path),
                speed=speed,
            )

            if output_path.exists():
                # Get duration
                import soundfile as sf
                audio_data, sr = sf.read(str(output_path))
                duration = len(audio_data) / sr

                return CloneResult(
                    success=True,
                    text=text,
                    output_path=output_path,
                    duration=duration,
                )
            else:
                return CloneResult(
                    success=False,
                    text=text,
                    error="Output file was not created"
                )

        except Exception as e:
            import traceback
            return CloneResult(
                success=False,
                text=text,
                error=f"{str(e)}\n{traceback.format_exc()}"
            )

    def clone_conversation(
        self,
        texts: list[str],
        speaker_wav: Union[str, list[str]],
        language: str = "fr",
        output_dir: Optional[str] = None,
        pause_between: float = 0.5,
    ) -> list[CloneResult]:
        """
        Synthesize multiple texts with the same cloned voice.

        Args:
            texts: List of texts to synthesize
            speaker_wav: Reference audio for voice cloning
            language: Target language
            output_dir: Directory for outputs
            pause_between: Pause duration between segments (not used in individual files)

        Returns:
            List of CloneResults
        """
        if output_dir is None:
            output_dir = self.output_dir / "conversation"
        else:
            output_dir = Path(output_dir)

        output_dir.mkdir(parents=True, exist_ok=True)

        results = []
        for i, text in enumerate(texts):
            output_path = output_dir / f"{i:03d}.wav"
            result = self.clone_voice(
                text=text,
                speaker_wav=speaker_wav,
                language=language,
                output_path=str(output_path),
            )
            results.append(result)

            if result.success:
                print(f"[{i+1}/{len(texts)}] Synthesized: {result.output_path}")
            else:
                print(f"[{i+1}/{len(texts)}] Failed: {result.error}")

        return results

    def test_voice(
        self,
        speaker_wav: Union[str, list[str]],
        language: str = "fr",
    ) -> CloneResult:
        """
        Quick test of voice cloning with a sample phrase.

        Args:
            speaker_wav: Reference audio
            language: Target language

        Returns:
            CloneResult with test audio
        """
        test_phrases = {
            "fr": "Bonjour, ceci est un test de clonage vocal.",
            "en": "Hello, this is a voice cloning test.",
            "es": "Hola, esta es una prueba de clonacion de voz.",
            "de": "Hallo, dies ist ein Stimmklontest.",
        }

        text = test_phrases.get(language, test_phrases["en"])

        return self.clone_voice(
            text=text,
            speaker_wav=speaker_wav,
            language=language,
            output_path=str(self.output_dir / f"test_{language}.wav"),
        )


# CLI interface
if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        print("Usage: python -m src.voice.cloner <speaker_wav> <text> [language]")
        print("Example: python -m src.voice.cloner voice.wav 'Bonjour!' fr")
        sys.exit(1)

    speaker_wav = sys.argv[1]
    text = sys.argv[2]
    language = sys.argv[3] if len(sys.argv) > 3 else "fr"

    cloner = VoiceCloner()
    result = cloner.clone_voice(text, speaker_wav, language)

    if result.success:
        print(f"\nSuccess!")
        print(f"Output: {result.output_path}")
        print(f"Duration: {result.duration:.2f}s")
    else:
        print(f"\nFailed: {result.error}")
