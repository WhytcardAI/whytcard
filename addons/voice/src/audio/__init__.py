"""
Audio processing module - Download and separate vocals.
"""

from .downloader import AudioDownloader
from .separator import VocalSeparator
from .converter import AudioConverter

__all__ = ["AudioDownloader", "VocalSeparator", "AudioConverter"]
