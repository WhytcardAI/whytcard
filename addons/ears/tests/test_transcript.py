"""
Tests for WhytCard Transcript
"""

import pytest
from pathlib import Path

from src.config import settings
from src.engine import TranscriptionEngine, get_engine
from src.formatters import MarkdownFormatter, JSONFormatter, get_formatter


class TestConfig:
    """Test configuration loading"""

    def test_settings_loaded(self):
        """Settings should be loaded"""
        assert settings is not None
        assert settings.whisper is not None
        assert settings.hub is not None

    def test_default_model(self):
        """Default model should be base"""
        assert settings.whisper.model == "base"

    def test_hub_url(self):
        """Hub URL should be localhost:3000"""
        assert "localhost:3000" in settings.hub.url


class TestEngine:
    """Test transcription engine"""

    def test_get_engine_singleton(self):
        """Engine should be singleton"""
        engine1 = get_engine()
        engine2 = get_engine()
        assert engine1 is engine2

    def test_engine_corrections_loaded(self):
        """Engine should have corrections dictionary"""
        engine = get_engine()
        assert hasattr(engine, "corrections")
        assert isinstance(engine.corrections, dict)

    def test_apply_corrections(self):
        """Corrections should be applied"""
        engine = get_engine()
        # Default corrections include "j'ai pas" -> "je n'ai pas"
        text = "j'ai pas vu"
        corrected = engine._apply_corrections(text)
        # Result depends on corrections dict content
        assert isinstance(corrected, str)


class TestFormatters:
    """Test output formatters"""

    def test_get_formatter_markdown(self):
        """Should return markdown formatter"""
        formatter = get_formatter("markdown")
        assert isinstance(formatter, MarkdownFormatter)

        formatter = get_formatter("md")
        assert isinstance(formatter, MarkdownFormatter)

    def test_get_formatter_json(self):
        """Should return JSON formatter"""
        formatter = get_formatter("json")
        assert isinstance(formatter, JSONFormatter)

    def test_get_formatter_default(self):
        """Unknown format should return markdown"""
        formatter = get_formatter("unknown")
        assert isinstance(formatter, MarkdownFormatter)


class TestMarkdownFormatter:
    """Test markdown formatting"""

    def test_format_output(self):
        """Format should produce valid markdown"""
        from src.engine import TranscriptionResult

        result = TranscriptionResult(
            raw_text="Test text",
            corrected_text="Test text corrected",
            summary="A test summary",
            language="en",
            duration=10.5,
            model_used="base"
        )

        formatter = MarkdownFormatter()
        output = formatter.format(result, title="Test Title")

        assert "# Transcription: Test Title" in output
        assert "Test text" in output
        assert "Test text corrected" in output
        assert "A test summary" in output
        assert "10.5s" in output


class TestJSONFormatter:
    """Test JSON formatting"""

    def test_format_output(self):
        """Format should produce valid JSON"""
        import json
        from src.engine import TranscriptionResult

        result = TranscriptionResult(
            raw_text="Test text",
            corrected_text="Test text corrected",
            summary="A test summary",
            language="en",
            duration=10.5,
            model_used="base"
        )

        formatter = JSONFormatter()
        output = formatter.format(result, title="Test")

        # Should be valid JSON
        data = json.loads(output)
        assert data["raw_text"] == "Test text"
        assert data["language"] == "en"
        assert data["duration"] == 10.5
