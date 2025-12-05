#!/usr/bin/env python
"""
Generate Vincent's persona system prompt.
"""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.research.extractor import PersonProfile
from src.avatar.prompt_generator import PromptGenerator


def main():
    # Paths
    profile_path = Path("data/subjects/vincent/research/profile.json")
    output_dir = Path("data/subjects/vincent")

    if not profile_path.exists():
        print(f"Profile not found: {profile_path}")
        return 1

    print("=" * 60)
    print("Generating Vincent's Persona")
    print("=" * 60)

    # Load profile
    print(f"\nLoading profile: {profile_path}")
    profile = PersonProfile.load(str(profile_path))

    # Create prompt generator
    generator = PromptGenerator()

    # Generate persona
    print("Generating system prompt...")
    persona = generator.generate(profile)

    # Save persona
    persona_path = output_dir / "persona.json"
    persona.save(str(persona_path))
    print(f"\nSaved persona: {persona_path}")

    # Also save as plain text for preview
    prompt_path = output_dir / "system_prompt.txt"
    with open(prompt_path, "w", encoding="utf-8") as f:
        f.write(persona.system_prompt)
    print(f"Saved prompt text: {prompt_path}")

    # Preview
    print("\n" + "=" * 60)
    print("System Prompt Preview (first 1000 chars):")
    print("=" * 60)
    print(persona.system_prompt[:1000])
    if len(persona.system_prompt) > 1000:
        print(f"\n... ({len(persona.system_prompt)} total characters)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
