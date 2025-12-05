#!/usr/bin/env python
"""
Test script for WhytCard-Human - Test with Vincent's data.
"""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.research.extractor import PersonProfile


def test_load_vincent_profile():
    """Test loading Vincent's profile from JSON."""
    profile_path = Path("data/subjects/vincent/research/profile.json")

    if not profile_path.exists():
        print(f"Profile not found: {profile_path}")
        return False

    profile = PersonProfile.load(str(profile_path))

    print("=" * 60)
    print("Vincent's Profile Loaded Successfully!")
    print("=" * 60)
    print(f"\nName: {profile.name}")
    print(f"Location: {profile.location}")
    print(f"Bio: {profile.bio[:100] if profile.bio else 'N/A'}...")

    print(f"\nProfessions ({len(profile.professions)}):")
    for occ in profile.professions[:5]:
        print(f"  - {occ}")

    print(f"\nFacettes ({len(profile.facettes)}):")
    for facette in profile.facettes:
        print(f"  - {facette.name}: {facette.symbol} ({facette.color})")

    print(f"\nSocial Profiles ({len(profile.social_profiles)}):")
    for social in profile.social_profiles[:5]:
        print(f"  - {social.platform}: {social.username}")

    print(f"\nKeywords/Traits: {', '.join(profile.keywords[:8])}")

    return True


def test_audio_exists():
    """Test that audio source exists."""
    audio_path = Path("data/subjects/vincent/audio/source_frankenstein.wav")

    if audio_path.exists():
        import os
        size_mb = os.path.getsize(audio_path) / (1024 * 1024)
        print(f"\nAudio Source: {audio_path}")
        print(f"Size: {size_mb:.2f} MB")
        return True
    else:
        print(f"\nAudio not found: {audio_path}")
        return False


def main():
    print("\n" + "=" * 60)
    print("WhytCard-Human - Vincent Test")
    print("=" * 60)

    results = []

    # Test 1: Load profile
    print("\n[Test 1] Loading Vincent's profile...")
    results.append(("Load Profile", test_load_vincent_profile()))

    # Test 2: Check audio
    print("\n[Test 2] Checking audio source...")
    results.append(("Audio Exists", test_audio_exists()))

    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    for name, passed in results:
        status = "PASS" if passed else "FAIL"
        print(f"  [{status}] {name}")

    all_passed = all(r[1] for r in results)
    print(f"\nOverall: {'All tests passed!' if all_passed else 'Some tests failed.'}")

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
