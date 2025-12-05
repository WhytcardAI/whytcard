#!/usr/bin/env python
"""
Script to separate vocals from Vincent's audio using Demucs.
"""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.audio.separator import VocalSeparator


def main():
    # Paths
    source_audio = Path("data/subjects/vincent/audio/source_frankenstein.wav")
    output_dir = Path("data/subjects/vincent/audio")

    if not source_audio.exists():
        print(f"Source audio not found: {source_audio}")
        return 1

    print("=" * 60)
    print("Vocal Separation with Demucs (htdemucs)")
    print("=" * 60)
    print(f"\nSource: {source_audio}")
    print(f"Output: {output_dir}")

    # Create separator with correct output directory
    separator = VocalSeparator(model_name="htdemucs", output_dir=str(output_dir))

    print("\nStarting separation...")
    print("(This may take a few minutes depending on audio length)")
    print()

    # Separate (output_name will be derived from source filename)
    result = separator.separate(str(source_audio))

    if result.success:
        print("\n" + "=" * 60)
        print("Separation Complete!")
        print("=" * 60)
        print(f"\nVocals: {result.vocals_path}")
        if result.accompaniment_path:
            print(f"Background: {result.accompaniment_path}")
        return 0
    else:
        print(f"\nSeparation failed: {result.error}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
