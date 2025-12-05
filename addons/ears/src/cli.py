"""
CLI interface for WhytCard Transcript
"""

import sys
from pathlib import Path
from typing import Optional
import argparse

from .config import settings
from .engine import get_engine
from .formatters import get_formatter


def transcribe_file(
    audio_path: str,
    output_path: Optional[str] = None,
    output_format: str = "markdown",
    language: Optional[str] = None,
    title: Optional[str] = None,
    model: Optional[str] = None,
    verbose: bool = False
) -> int:
    """
    Transcribe audio file and save result

    Returns:
        Exit code (0 for success)
    """
    audio_file = Path(audio_path)

    if not audio_file.exists():
        print(f"Error: File not found: {audio_path}")
        return 1

    # Get engine
    if verbose:
        print(f"Loading Whisper model: {model or settings.whisper.model}...")

    engine = get_engine()

    # Override model if specified
    if model:
        engine.model_name = model
        engine._model = None  # Force reload

    # Transcribe
    if verbose:
        print(f"Transcribing: {audio_file.name}...")

    try:
        result = engine.transcribe(str(audio_file), language=language)
    except Exception as e:
        print(f"Error during transcription: {e}")
        return 1

    if verbose:
        print(f"Detected language: {result.language}")
        print(f"Duration: {result.duration:.1f}s")

    # Format output
    formatter = get_formatter(output_format)

    # Determine output path
    if output_path:
        out_file = Path(output_path)
    else:
        ext_map = {
            "markdown": ".md",
            "md": ".md",
            "json": ".json",
            "pdf": ".pdf",
        }
        ext = ext_map.get(output_format.lower(), ".md")
        out_file = audio_file.with_suffix(ext)

    # Use audio filename as title if not specified
    if not title:
        title = audio_file.stem

    # Save
    try:
        formatter.save(result, str(out_file), title=title)
        print(f"Output saved: {out_file}")
    except Exception as e:
        print(f"Error saving output: {e}")
        return 1

    return 0


def run_server(
    host: str = "0.0.0.0",
    port: int = 8001,
    reload: bool = False
):
    """Start the API server"""
    try:
        import uvicorn
        uvicorn.run(
            "whytcard_transcript.server:app",
            host=host,
            port=port,
            reload=reload
        )
    except ImportError:
        print("Error: uvicorn not installed. Run: pip install uvicorn")
        sys.exit(1)


def main():
    """Main CLI entry point"""
    parser = argparse.ArgumentParser(
        prog="whytcard-transcript",
        description="WhytCard Transcript - Audio transcription addon"
    )

    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Transcribe command
    transcribe_parser = subparsers.add_parser(
        "transcribe",
        help="Transcribe an audio file"
    )
    transcribe_parser.add_argument(
        "audio_file",
        help="Path to audio file"
    )
    transcribe_parser.add_argument(
        "-o", "--output",
        help="Output file path"
    )
    transcribe_parser.add_argument(
        "-f", "--format",
        choices=["markdown", "md", "json", "pdf"],
        default="markdown",
        help="Output format (default: markdown)"
    )
    transcribe_parser.add_argument(
        "-l", "--language",
        help="Source language code (auto-detect if not specified)"
    )
    transcribe_parser.add_argument(
        "-t", "--title",
        help="Title for the transcription"
    )
    transcribe_parser.add_argument(
        "-m", "--model",
        choices=["tiny", "base", "small", "medium", "large"],
        help="Whisper model to use"
    )
    transcribe_parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose output"
    )

    # Server command
    server_parser = subparsers.add_parser(
        "serve",
        help="Start the API server"
    )
    server_parser.add_argument(
        "--host",
        default="0.0.0.0",
        help="Host to bind (default: 0.0.0.0)"
    )
    server_parser.add_argument(
        "--port",
        type=int,
        default=8001,
        help="Port to bind (default: 8001)"
    )
    server_parser.add_argument(
        "--reload",
        action="store_true",
        help="Enable auto-reload for development"
    )

    # Info command
    info_parser = subparsers.add_parser(
        "info",
        help="Show addon information"
    )

    args = parser.parse_args()

    if args.command == "transcribe":
        sys.exit(transcribe_file(
            audio_path=args.audio_file,
            output_path=args.output,
            output_format=args.format,
            language=args.language,
            title=args.title,
            model=args.model,
            verbose=args.verbose
        ))

    elif args.command == "serve":
        run_server(
            host=args.host,
            port=args.port,
            reload=args.reload
        )

    elif args.command == "info":
        print("WhytCard Transcript v0.1.0")
        print(f"  Model: {settings.whisper.model}")
        print(f"  Hub URL: {settings.hub.url}")
        print(f"  Auto-register: {settings.hub.auto_register}")
        print(f"  Output format: {settings.output.default_format}")

    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
