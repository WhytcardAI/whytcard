#!/usr/bin/env python
"""
WhytCard-Human CLI - Create AI avatars from real people.

Usage:
    python -m src.pipeline.cli create <name> [options]
    python -m src.pipeline.cli chat <subject_dir> [options]
    python -m src.pipeline.cli list
"""

import argparse
import sys
from pathlib import Path


def cmd_create(args):
    """Create a new human avatar."""
    from .orchestrator import create_human

    result = create_human(
        name=args.name,
        context=args.context,
        audio_url=args.audio_url,
        audio_file=args.audio_file,
        skip_research=args.skip_research,
    )

    if result.success:
        print(f"\nAvatar created successfully!")
        print(f"Directory: {result.subject_dir}")
        return 0
    else:
        print(f"\nFailed: {result.error}")
        return 1


def cmd_chat(args):
    """Start a chat session with an avatar."""
    from ..avatar.prompt_generator import PersonaPrompt
    from ..avatar.avatar import AvatarEngine

    subject_dir = Path(args.subject_dir)

    # Load persona
    persona_path = subject_dir / "persona.json"
    if not persona_path.exists():
        print(f"Persona not found: {persona_path}")
        return 1

    persona = PersonaPrompt.load(str(persona_path))
    print(f"Loaded persona: {persona.name}")

    # Find vocals
    vocals_path = None
    vocals_dir = subject_dir / "audio"
    if vocals_dir.exists():
        for f in vocals_dir.glob("*_vocals.wav"):
            vocals_path = str(f)
            break

    # Check model
    model_path = args.model
    if not Path(model_path).exists():
        print(f"Model not found: {model_path}")
        print("Please download a GGUF model first.")
        return 1

    # Create avatar
    print(f"Loading model: {model_path}")
    avatar = AvatarEngine(
        model_path=model_path,
        persona=persona,
        speaker_wav=vocals_path,
        enable_voice=args.voice and vocals_path is not None,
    )

    print(f"\n{'='*60}")
    print(f"Chat with {persona.name}")
    print(f"Type 'quit' to exit, 'reset' to clear history")
    print(f"{'='*60}\n")

    while True:
        try:
            user_input = input("You: ").strip()

            if not user_input:
                continue

            if user_input.lower() == "quit":
                break

            if user_input.lower() == "reset":
                avatar.reset_conversation()
                print("Conversation reset.\n")
                continue

            # Get response
            response = avatar.chat(
                user_input,
                synthesize_voice=args.voice and vocals_path is not None,
                language=args.language,
            )

            print(f"\n{persona.name}: {response.text}")

            if response.audio_path:
                print(f"[Audio: {response.audio_path}]")

            print()

        except KeyboardInterrupt:
            print("\n\nGoodbye!")
            break
        except Exception as e:
            print(f"Error: {e}")

    return 0


def cmd_list(args):
    """List all created subjects."""
    data_dir = Path("data/subjects")

    if not data_dir.exists():
        print("No subjects found.")
        return 0

    subjects = [d for d in data_dir.iterdir() if d.is_dir()]

    if not subjects:
        print("No subjects found.")
        return 0

    print(f"\n{'='*60}")
    print("Available Subjects")
    print(f"{'='*60}\n")

    for subject_dir in sorted(subjects):
        name = subject_dir.name

        # Check what's available
        has_profile = (subject_dir / "research" / "profile.json").exists()
        has_persona = (subject_dir / "persona.json").exists()
        has_vocals = any((subject_dir / "audio").glob("*_vocals.wav")) if (subject_dir / "audio").exists() else False

        status = []
        if has_profile:
            status.append("profile")
        if has_persona:
            status.append("persona")
        if has_vocals:
            status.append("voice")

        print(f"  {name}: [{', '.join(status) or 'empty'}]")
        print(f"    Path: {subject_dir}")

    print()
    return 0


def main():
    parser = argparse.ArgumentParser(
        description="WhytCard-Human - Create AI avatars from real people",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Create a new avatar
  python -m src.pipeline.cli create "John Doe" -c "software engineer"

  # Create with voice from YouTube
  python -m src.pipeline.cli create "John Doe" -u "https://youtube.com/watch?v=..."

  # Chat with an avatar
  python -m src.pipeline.cli chat data/subjects/john_doe

  # List all subjects
  python -m src.pipeline.cli list
""",
    )

    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # Create command
    create_parser = subparsers.add_parser("create", help="Create a new avatar")
    create_parser.add_argument("name", help="Person's name")
    create_parser.add_argument("-c", "--context", help="Additional context")
    create_parser.add_argument("-u", "--audio-url", help="YouTube/web URL for voice")
    create_parser.add_argument("-f", "--audio-file", help="Local audio file")
    create_parser.add_argument("-s", "--skip-research", action="store_true",
                               help="Skip research phase")
    create_parser.set_defaults(func=cmd_create)

    # Chat command
    chat_parser = subparsers.add_parser("chat", help="Chat with an avatar")
    chat_parser.add_argument("subject_dir", help="Subject directory")
    chat_parser.add_argument("-m", "--model",
                            default="models/gguf/qwen2.5-coder-7b-instruct-q4_k_m.gguf",
                            help="GGUF model path")
    chat_parser.add_argument("-v", "--voice", action="store_true",
                            help="Enable voice synthesis")
    chat_parser.add_argument("-l", "--language", default="fr",
                            help="Language for voice (default: fr)")
    chat_parser.set_defaults(func=cmd_chat)

    # List command
    list_parser = subparsers.add_parser("list", help="List all subjects")
    list_parser.set_defaults(func=cmd_list)

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
