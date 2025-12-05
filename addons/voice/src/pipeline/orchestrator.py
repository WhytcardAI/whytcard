"""
Full pipeline orchestrator for creating an AI human avatar.

Orchestrates the complete workflow:
1. Research: Search and extract information about a person
2. Documentation: Generate structured documentation
3. Audio: Download and process audio for voice cloning
4. Voice: Clone the person's voice
5. Avatar: Create interactive AI persona
"""

import os
import json
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime


@dataclass
class PipelineConfig:
    """Configuration for the pipeline."""
    # Subject info
    subject_name: str
    subject_context: Optional[str] = None

    # Directories
    data_dir: str = "data/subjects"
    models_dir: str = "models"

    # Audio source
    audio_url: Optional[str] = None
    audio_file: Optional[str] = None

    # Model paths
    gguf_model: str = "models/gguf/qwen2.5-coder-7b-instruct-q4_k_m.gguf"

    # Options
    enable_voice: bool = True
    language: str = "fr"

    def get_subject_dir(self) -> Path:
        """Get the subject's data directory."""
        safe_name = self.subject_name.lower().replace(" ", "_")
        return Path(self.data_dir) / safe_name


@dataclass
class PipelineResult:
    """Result of the pipeline execution."""
    success: bool
    subject_name: str
    subject_dir: Path
    profile_path: Optional[Path] = None
    persona_path: Optional[Path] = None
    vocals_path: Optional[Path] = None
    error: Optional[str] = None
    steps_completed: list[str] = field(default_factory=list)


class HumanPipeline:
    """
    Full pipeline for creating an AI human avatar.

    Orchestrates all modules:
    - Research: Web search and info extraction
    - Audio: Download and vocal separation
    - Voice: Voice cloning
    - Avatar: LLM persona with voice
    """

    def __init__(self, config: PipelineConfig):
        """
        Initialize the pipeline.

        Args:
            config: Pipeline configuration
        """
        self.config = config
        self.subject_dir = config.get_subject_dir()
        self.subject_dir.mkdir(parents=True, exist_ok=True)

        # Create subdirectories
        (self.subject_dir / "research").mkdir(exist_ok=True)
        (self.subject_dir / "audio").mkdir(exist_ok=True)
        (self.subject_dir / "voice").mkdir(exist_ok=True)
        (self.subject_dir / "docs").mkdir(exist_ok=True)

        self.result = PipelineResult(
            success=False,
            subject_name=config.subject_name,
            subject_dir=self.subject_dir,
        )

    def run_research(self, api_key: Optional[str] = None) -> bool:
        """
        Run the research phase.

        Args:
            api_key: Tavily API key (optional, uses env var if not provided)

        Returns:
            True if successful
        """
        print(f"\n{'='*60}")
        print(f"PHASE 1: RESEARCH - {self.config.subject_name}")
        print(f"{'='*60}\n")

        try:
            from ..research.searcher import PersonSearcher
            from ..research.extractor import InfoExtractor

            # Search for information
            print("Searching for information...")
            searcher = PersonSearcher(api_key=api_key)
            results = searcher.search(
                self.config.subject_name,
                context=self.config.subject_context,
            )

            print(f"Found {len(results.results)} results")

            # Extract structured profile
            print("Extracting profile...")
            extractor = InfoExtractor()
            profile = extractor.extract_profile(results)

            # Save profile
            profile_path = self.subject_dir / "research" / "profile.json"
            profile.save(str(profile_path))
            print(f"Profile saved: {profile_path}")

            self.result.profile_path = profile_path
            self.result.steps_completed.append("research")
            return True

        except Exception as e:
            print(f"Research failed: {e}")
            self.result.error = str(e)
            return False

    def run_audio(self) -> bool:
        """
        Run the audio phase (download and separation).

        Returns:
            True if successful
        """
        print(f"\n{'='*60}")
        print(f"PHASE 2: AUDIO PROCESSING")
        print(f"{'='*60}\n")

        try:
            audio_path = None

            # Download if URL provided
            if self.config.audio_url:
                from ..audio.downloader import AudioDownloader

                print(f"Downloading audio from: {self.config.audio_url}")
                downloader = AudioDownloader(
                    output_dir=str(self.subject_dir / "audio")
                )
                result = downloader.download(
                    self.config.audio_url,
                    output_name="source",
                )

                if not result.success:
                    raise Exception(f"Download failed: {result.error}")

                audio_path = result.output_path
                print(f"Downloaded: {audio_path}")

            elif self.config.audio_file:
                audio_path = Path(self.config.audio_file)
                if not audio_path.exists():
                    raise FileNotFoundError(f"Audio file not found: {audio_path}")

            if not audio_path:
                print("No audio source provided, skipping audio phase")
                return True

            # Separate vocals
            from ..audio.separator import VocalSeparator

            print("Separating vocals...")
            separator = VocalSeparator(
                output_dir=str(self.subject_dir / "audio"),
            )
            result = separator.separate(
                str(audio_path),
                output_name="separated",
            )

            if not result.success:
                raise Exception(f"Separation failed: {result.error}")

            print(f"Vocals extracted: {result.vocals_path}")
            self.result.vocals_path = result.vocals_path
            self.result.steps_completed.append("audio")
            return True

        except Exception as e:
            print(f"Audio processing failed: {e}")
            self.result.error = str(e)
            return False

    def run_documentation(self) -> bool:
        """
        Run the documentation phase.

        Returns:
            True if successful
        """
        print(f"\n{'='*60}")
        print(f"PHASE 3: DOCUMENTATION")
        print(f"{'='*60}\n")

        try:
            from ..research.extractor import PersonProfile
            from ..documentation.generator import DocumentGenerator

            # Load profile
            profile_path = self.subject_dir / "research" / "profile.json"
            if not profile_path.exists():
                print("No profile found, skipping documentation")
                return True

            profile = PersonProfile.load(str(profile_path))

            # Generate documentation
            print("Generating documentation...")
            generator = DocumentGenerator()

            # Profile doc
            generator.generate_profile(
                profile,
                str(self.subject_dir / "docs" / "profile.md"),
            )

            # Facettes doc (if any)
            if profile.facettes:
                generator.generate_facettes(
                    profile.name,
                    profile.facettes,
                    str(self.subject_dir / "docs" / "facettes.md"),
                )

            print(f"Documentation generated in: {self.subject_dir / 'docs'}")
            self.result.steps_completed.append("documentation")
            return True

        except Exception as e:
            print(f"Documentation failed: {e}")
            self.result.error = str(e)
            return False

    def run_persona(self) -> bool:
        """
        Run the persona generation phase.

        Returns:
            True if successful
        """
        print(f"\n{'='*60}")
        print(f"PHASE 4: PERSONA GENERATION")
        print(f"{'='*60}\n")

        try:
            from ..research.extractor import PersonProfile
            from ..avatar.prompt_generator import PromptGenerator

            # Load profile
            profile_path = self.subject_dir / "research" / "profile.json"
            if not profile_path.exists():
                print("No profile found, skipping persona generation")
                return True

            profile = PersonProfile.load(str(profile_path))

            # Generate persona prompt
            print("Generating persona...")
            generator = PromptGenerator()
            persona = generator.generate(profile)

            # Save persona
            persona_path = self.subject_dir / "persona.json"
            persona.save(str(persona_path))

            # Also save system prompt as text
            prompt_path = self.subject_dir / "system_prompt.txt"
            prompt_path.write_text(persona.system_prompt, encoding="utf-8")

            print(f"Persona saved: {persona_path}")
            self.result.persona_path = persona_path
            self.result.steps_completed.append("persona")
            return True

        except Exception as e:
            print(f"Persona generation failed: {e}")
            self.result.error = str(e)
            return False

    def run(self, skip_research: bool = False) -> PipelineResult:
        """
        Run the full pipeline.

        Args:
            skip_research: Skip research phase (use existing profile)

        Returns:
            PipelineResult with all paths and status
        """
        print(f"\n{'#'*60}")
        print(f"# WHYTCARD-HUMAN PIPELINE: {self.config.subject_name}")
        print(f"{'#'*60}")
        print(f"Subject directory: {self.subject_dir}")

        try:
            # Phase 1: Research
            if not skip_research:
                if not self.run_research():
                    return self.result
            else:
                print("\nSkipping research phase (using existing profile)")
                self.result.profile_path = self.subject_dir / "research" / "profile.json"

            # Phase 2: Audio
            if self.config.audio_url or self.config.audio_file:
                if not self.run_audio():
                    return self.result
            else:
                print("\nNo audio source provided, skipping audio phase")

            # Phase 3: Documentation
            if not self.run_documentation():
                return self.result

            # Phase 4: Persona
            if not self.run_persona():
                return self.result

            # Success!
            self.result.success = True

            print(f"\n{'#'*60}")
            print(f"# PIPELINE COMPLETE!")
            print(f"{'#'*60}")
            print(f"Subject directory: {self.subject_dir}")
            print(f"Steps completed: {', '.join(self.result.steps_completed)}")

            if self.result.persona_path:
                print(f"\nNext step: Create avatar with:")
                print(f"  from src.avatar import AvatarEngine")
                print(f"  from src.avatar.prompt_generator import PersonaPrompt")
                print(f"  persona = PersonaPrompt.load('{self.result.persona_path}')")
                if self.result.vocals_path:
                    print(f"  avatar = AvatarEngine(")
                    print(f"      model_path='{self.config.gguf_model}',")
                    print(f"      persona=persona,")
                    print(f"      speaker_wav='{self.result.vocals_path}',")
                    print(f"  )")

            return self.result

        except Exception as e:
            self.result.error = str(e)
            return self.result


def create_human(
    name: str,
    context: Optional[str] = None,
    audio_url: Optional[str] = None,
    audio_file: Optional[str] = None,
    skip_research: bool = False,
) -> PipelineResult:
    """
    Convenience function to create a human avatar.

    Args:
        name: Person's name
        context: Additional context (profession, etc.)
        audio_url: YouTube or web URL for voice audio
        audio_file: Local audio file path
        skip_research: Skip research (use existing data)

    Returns:
        PipelineResult
    """
    config = PipelineConfig(
        subject_name=name,
        subject_context=context,
        audio_url=audio_url,
        audio_file=audio_file,
    )

    pipeline = HumanPipeline(config)
    return pipeline.run(skip_research=skip_research)


# CLI interface
if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="WhytCard-Human Pipeline - Create AI avatar from a person"
    )
    parser.add_argument("name", help="Person's name")
    parser.add_argument("--context", "-c", help="Additional context (profession, etc.)")
    parser.add_argument("--audio-url", "-u", help="YouTube or web URL for voice")
    parser.add_argument("--audio-file", "-f", help="Local audio file path")
    parser.add_argument("--skip-research", "-s", action="store_true",
                       help="Skip research phase (use existing profile)")

    args = parser.parse_args()

    result = create_human(
        name=args.name,
        context=args.context,
        audio_url=args.audio_url,
        audio_file=args.audio_file,
        skip_research=args.skip_research,
    )

    if result.success:
        print("\nPipeline completed successfully!")
    else:
        print(f"\nPipeline failed: {result.error}")
        exit(1)
