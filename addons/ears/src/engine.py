"""
Transcription engine using OpenAI Whisper
"""

import re
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional

import whisper

from .config import settings


@dataclass
class TranscriptionResult:
    """Result of audio transcription"""
    raw_text: str
    corrected_text: str
    summary: str
    language: str
    duration: float
    model_used: str
    timestamp: datetime = field(default_factory=datetime.now)
    segments: list[dict] = field(default_factory=list)
    confidence: Optional[float] = None
    source_file: Optional[str] = None


class TranscriptionEngine:
    """Audio transcription engine using Whisper"""

    # Common transcription corrections for French
    CORRECTIONS_FR = {
        r"\bvies\b": "vois",
        r"\bvenements\b": "paiements",
        r"\bbietrie\b": "bijouterie",
        r"\bau pied\b": "au pire",
        r"\bformagnac\b": "Forminator",
        r"\baccrustation\b": "incrustation",
        r"\bd'eau\b": "d'abord",
        r"\bennun\b": "quelqu'un",
        r"\bburesque\b": "burlesque",
        r"\bca te date\b": "ca te va",
        r"\btu m'as coups\b": "tu me coupes",
        r"\bmontraveillant-sembre\b": "mode de travail ensemble",
        r"\bconfrondes\b": "confrontes",
    }

    def __init__(self, model_name: Optional[str] = None, device: Optional[str] = None):
        """Initialize the transcription engine"""
        self.model_name = model_name or settings.whisper.model
        self.device = device or settings.whisper.device
        self._model = None

    @property
    def model(self):
        """Lazy load the Whisper model"""
        if self._model is None:
            print(f"Loading Whisper model: {self.model_name}")
            self._model = whisper.load_model(self.model_name)
        return self._model

    def transcribe(
        self,
        audio_path: str | Path,
        language: Optional[str] = None,
        include_timestamps: bool = False
    ) -> TranscriptionResult:
        """
        Transcribe an audio file

        Args:
            audio_path: Path to the audio file
            language: Language code (fr, en, de, etc.)
            include_timestamps: Include word-level timestamps

        Returns:
            TranscriptionResult with raw and corrected text
        """
        audio_path = Path(audio_path)
        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        language = language or settings.whisper.language

        # Transcribe
        result = self.model.transcribe(
            str(audio_path),
            language=language,
            word_timestamps=include_timestamps
        )

        raw_text = result["text"].strip()

        # Apply corrections
        corrected_text = self._apply_corrections(raw_text, language)

        # Generate summary
        summary = self._generate_summary(corrected_text)

        # Extract segments if available
        segments = []
        if "segments" in result:
            segments = [
                {
                    "start": seg["start"],
                    "end": seg["end"],
                    "text": seg["text"]
                }
                for seg in result["segments"]
            ]

        return TranscriptionResult(
            raw_text=raw_text,
            corrected_text=corrected_text,
            summary=summary,
            language=language,
            duration=result.get("duration", 0.0) if isinstance(result, dict) else 0.0,
            model_used=self.model_name,
            segments=segments,
            source_file=str(audio_path)
        )

    def _apply_corrections(self, text: str, language: str) -> str:
        """Apply language-specific corrections to transcription"""
        corrected = text

        if language == "fr":
            for pattern, replacement in self.CORRECTIONS_FR.items():
                corrected = re.sub(pattern, replacement, corrected, flags=re.IGNORECASE)

        # General cleanup
        corrected = re.sub(r'\s+', ' ', corrected)  # Multiple spaces
        corrected = corrected.strip()

        return corrected

    def _generate_summary(self, text: str, max_points: int = 5) -> str:
        """Generate a simple summary from the transcription"""
        # Split into sentences
        sentences = re.split(r'[.!?]+', text)
        sentences = [s.strip() for s in sentences if s.strip()]

        if not sentences:
            return "Aucun contenu a resumer."

        # For now, return first few sentences as summary
        # In a real implementation, this would use NLP or LLM
        summary_sentences = sentences[:max_points]

        summary_points = []
        for i, sentence in enumerate(summary_sentences, 1):
            if len(sentence) > 20:  # Only include meaningful sentences
                summary_points.append(f"- {sentence}")

        if not summary_points:
            return "Contenu trop court pour resumer."

        return "\n".join(summary_points)


# Global engine instance
_engine: Optional[TranscriptionEngine] = None


def get_engine() -> TranscriptionEngine:
    """Get or create the global transcription engine"""
    global _engine
    if _engine is None:
        _engine = TranscriptionEngine()
    return _engine


def transcribe_audio(
    audio_path: str | Path,
    language: Optional[str] = None
) -> TranscriptionResult:
    """Convenience function to transcribe audio"""
    return get_engine().transcribe(audio_path, language)
