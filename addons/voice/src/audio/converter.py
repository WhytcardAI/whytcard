"""
Audio format conversion utilities.
"""

import subprocess
from pathlib import Path
from dataclasses import dataclass
from typing import Optional


@dataclass
class ConversionResult:
    """Result of audio conversion."""
    success: bool
    input_path: Path
    output_path: Optional[Path] = None
    error: Optional[str] = None


class AudioConverter:
    """
    Convert audio between formats using FFmpeg.
    """

    def __init__(self):
        """Initialize the converter."""
        # Check if ffmpeg is available
        try:
            result = subprocess.run(
                ["ffmpeg", "-version"],
                capture_output=True,
                check=True
            )
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            raise RuntimeError("ffmpeg not found. Please install ffmpeg.") from e

    def convert(
        self,
        input_path: str,
        output_path: str,
        sample_rate: Optional[int] = None,
        channels: Optional[int] = None,
        bitrate: Optional[str] = None,
    ) -> ConversionResult:
        """
        Convert audio file to another format.

        Args:
            input_path: Path to input audio file
            output_path: Path for output file (format determined by extension)
            sample_rate: Target sample rate (e.g., 44100, 22050)
            channels: Number of channels (1=mono, 2=stereo)
            bitrate: Bitrate for lossy formats (e.g., "192k")

        Returns:
            ConversionResult
        """
        input_path = Path(input_path)
        output_path = Path(output_path)

        if not input_path.exists():
            return ConversionResult(
                success=False,
                input_path=input_path,
                error=f"Input file not found: {input_path}"
            )

        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Build ffmpeg command
        cmd = ["ffmpeg", "-y", "-i", str(input_path)]

        if sample_rate:
            cmd.extend(["-ar", str(sample_rate)])

        if channels:
            cmd.extend(["-ac", str(channels)])

        if bitrate:
            cmd.extend(["-b:a", bitrate])

        cmd.append(str(output_path))

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace"
            )

            if result.returncode != 0:
                return ConversionResult(
                    success=False,
                    input_path=input_path,
                    error=result.stderr
                )

            if output_path.exists():
                return ConversionResult(
                    success=True,
                    input_path=input_path,
                    output_path=output_path,
                )
            else:
                return ConversionResult(
                    success=False,
                    input_path=input_path,
                    error="Output file not created"
                )

        except Exception as e:
            return ConversionResult(
                success=False,
                input_path=input_path,
                error=str(e)
            )

    def to_wav(
        self,
        input_path: str,
        output_path: Optional[str] = None,
        sample_rate: int = 22050,
        mono: bool = True,
    ) -> ConversionResult:
        """
        Convert audio to WAV format (optimal for voice processing).

        Args:
            input_path: Path to input audio
            output_path: Path for output WAV. If None, uses input name with .wav
            sample_rate: Target sample rate (22050 is good for XTTS)
            mono: If True, convert to mono

        Returns:
            ConversionResult
        """
        input_path = Path(input_path)

        if output_path is None:
            output_path = input_path.with_suffix(".wav")

        return self.convert(
            str(input_path),
            str(output_path),
            sample_rate=sample_rate,
            channels=1 if mono else None,
        )

    def get_audio_info(self, path: str) -> dict:
        """
        Get audio file information.

        Args:
            path: Path to audio file

        Returns:
            Dictionary with format, duration, sample_rate, channels, etc.
        """
        cmd = [
            "ffprobe",
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            str(path)
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
                info = json.loads(result.stdout)

                # Extract relevant info
                audio_stream = None
                for stream in info.get("streams", []):
                    if stream.get("codec_type") == "audio":
                        audio_stream = stream
                        break

                fmt = info.get("format", {})

                return {
                    "duration": float(fmt.get("duration", 0)),
                    "format": fmt.get("format_name"),
                    "sample_rate": int(audio_stream.get("sample_rate", 0)) if audio_stream else 0,
                    "channels": audio_stream.get("channels", 0) if audio_stream else 0,
                    "codec": audio_stream.get("codec_name") if audio_stream else None,
                    "bit_rate": int(fmt.get("bit_rate", 0)),
                }
            else:
                return {"error": result.stderr}

        except Exception as e:
            return {"error": str(e)}


# CLI interface
if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python -m src.audio.converter <input> [output]")
        sys.exit(1)

    input_path = sys.argv[1]
    output_path = sys.argv[2] if len(sys.argv) > 2 else None

    converter = AudioConverter()

    # Show info first
    info = converter.get_audio_info(input_path)
    print(f"Input info: {info}")

    if output_path:
        result = converter.convert(input_path, output_path)
        if result.success:
            print(f"Converted to: {result.output_path}")
        else:
            print(f"Failed: {result.error}")
