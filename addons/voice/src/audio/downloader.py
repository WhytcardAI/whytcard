"""
Download audio from various sources (YouTube, web, etc.).
"""

import os
import subprocess
from pathlib import Path
from dataclasses import dataclass
from typing import Optional


@dataclass
class DownloadResult:
    """Result of an audio download."""
    success: bool
    source_url: str
    output_path: Optional[Path] = None
    title: Optional[str] = None
    duration: Optional[float] = None
    error: Optional[str] = None


class AudioDownloader:
    """
    Download audio from YouTube and other sources using yt-dlp.
    """

    def __init__(self, output_dir: str = "data/audio/raw"):
        """
        Initialize the downloader.

        Args:
            output_dir: Directory to save downloaded audio files
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Check if yt-dlp is available
        try:
            subprocess.run(["yt-dlp", "--version"], capture_output=True, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            raise RuntimeError("yt-dlp not found. Install with: pip install yt-dlp") from e

    def download(
        self,
        url: str,
        output_name: Optional[str] = None,
        audio_format: str = "wav",
        sample_rate: int = 44100,
    ) -> DownloadResult:
        """
        Download audio from a URL.

        Args:
            url: Source URL (YouTube, etc.)
            output_name: Output filename (without extension). If None, uses video title.
            audio_format: Output audio format (wav, mp3, etc.)
            sample_rate: Audio sample rate

        Returns:
            DownloadResult with download status and file path
        """
        # Build output template
        if output_name:
            output_template = str(self.output_dir / f"{output_name}.%(ext)s")
        else:
            output_template = str(self.output_dir / "%(title)s.%(ext)s")

        # yt-dlp command
        cmd = [
            "yt-dlp",
            "-x",  # Extract audio
            "--audio-format", audio_format,
            "--audio-quality", "0",  # Best quality
            "-o", output_template,
            "--no-playlist",  # Single video only
            url
        ]

        # Add ffmpeg postprocessor args for sample rate if WAV
        if audio_format == "wav":
            cmd.extend([
                "--postprocessor-args",
                f"ffmpeg:-ar {sample_rate} -ac 1"  # Mono for voice
            ])

        try:
            # Run download
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace"
            )

            if result.returncode != 0:
                return DownloadResult(
                    success=False,
                    source_url=url,
                    error=result.stderr
                )

            # Find the output file
            output_path = self._find_output_file(output_name, audio_format)

            if output_path:
                return DownloadResult(
                    success=True,
                    source_url=url,
                    output_path=output_path,
                    title=output_path.stem,
                )
            else:
                return DownloadResult(
                    success=False,
                    source_url=url,
                    error="Output file not found after download"
                )

        except Exception as e:
            return DownloadResult(
                success=False,
                source_url=url,
                error=str(e)
            )

    def _find_output_file(self, expected_name: Optional[str], audio_format: str) -> Optional[Path]:
        """Find the most recently created output file."""
        pattern = f"*.{audio_format}"
        files = list(self.output_dir.glob(pattern))

        if not files:
            return None

        # If we know the expected name, look for it
        if expected_name:
            for f in files:
                if f.stem == expected_name:
                    return f

        # Otherwise return most recent file
        return max(files, key=lambda f: f.stat().st_mtime)

    def download_youtube(
        self,
        url: str,
        output_name: Optional[str] = None,
    ) -> DownloadResult:
        """
        Download audio from YouTube video.

        Args:
            url: YouTube video URL
            output_name: Optional output filename

        Returns:
            DownloadResult
        """
        return self.download(url, output_name, audio_format="wav")

    def get_video_info(self, url: str) -> dict:
        """
        Get video information without downloading.

        Args:
            url: Video URL

        Returns:
            Dictionary with video info (title, duration, etc.)
        """
        cmd = [
            "yt-dlp",
            "--dump-json",
            "--no-download",
            url
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace"
            )

            if result.returncode == 0:
                import json
                return json.loads(result.stdout)
            else:
                return {"error": result.stderr}

        except Exception as e:
            return {"error": str(e)}


# CLI interface
if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python -m src.audio.downloader <url> [output_name]")
        sys.exit(1)

    url = sys.argv[1]
    output_name = sys.argv[2] if len(sys.argv) > 2 else None

    downloader = AudioDownloader()

    print(f"Downloading: {url}")
    result = downloader.download(url, output_name)

    if result.success:
        print(f"Success! Saved to: {result.output_path}")
    else:
        print(f"Failed: {result.error}")
