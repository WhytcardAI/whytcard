"""
Separate vocals from audio using Demucs.

Note: Uses soundfile for saving to work around torchcodec issues on Windows with FFmpeg 8.
"""

import os
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

import torch
import soundfile as sf

try:
    from demucs.pretrained import get_model
    from demucs.apply import apply_model
    HAS_DEMUCS = True
except ImportError:
    HAS_DEMUCS = False


@dataclass
class SeparationResult:
    """Result of audio separation."""
    success: bool
    input_path: Path
    vocals_path: Optional[Path] = None
    accompaniment_path: Optional[Path] = None
    error: Optional[str] = None


class VocalSeparator:
    """
    Separate vocals from audio using Demucs.

    Uses the htdemucs model for best quality vocal separation.
    Saves using soundfile to work around Windows torchcodec issues.
    """

    def __init__(
        self,
        model_name: str = "htdemucs",
        output_dir: str = "data/audio/processed",
        device: Optional[str] = None,
    ):
        """
        Initialize the separator.

        Args:
            model_name: Demucs model to use (htdemucs, htdemucs_ft, etc.)
            output_dir: Directory to save separated audio
            device: Device to use (cuda, cpu). Auto-detected if None.
        """
        if not HAS_DEMUCS:
            raise ImportError("demucs not installed. Run: pip install demucs")

        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Auto-detect device
        if device is None:
            self.device = "cuda" if torch.cuda.is_available() else "cpu"
        else:
            self.device = device

        print(f"Loading Demucs model '{model_name}' on {self.device}...")
        self.model = get_model(model_name)
        self.model.to(self.device)
        self.model.eval()

        self.sample_rate = self.model.samplerate
        print(f"Model loaded. Sample rate: {self.sample_rate} Hz")

    def separate(
        self,
        input_path: str,
        output_name: Optional[str] = None,
        save_all_stems: bool = False,
    ) -> SeparationResult:
        """
        Separate vocals from audio file.

        Args:
            input_path: Path to input audio file
            output_name: Output filename prefix. If None, uses input filename.
            save_all_stems: If True, save all stems (drums, bass, other, vocals)

        Returns:
            SeparationResult with paths to separated audio
        """
        input_path = Path(input_path)

        if not input_path.exists():
            return SeparationResult(
                success=False,
                input_path=input_path,
                error=f"Input file not found: {input_path}"
            )

        if output_name is None:
            output_name = input_path.stem

        try:
            # Load audio using soundfile
            print(f"Loading audio: {input_path}")
            audio_data, sr = sf.read(str(input_path))

            # Convert to tensor
            # soundfile returns (samples, channels) or (samples,) for mono
            if audio_data.ndim == 1:
                audio_data = audio_data.reshape(-1, 1)

            # Transpose to (channels, samples) as expected by demucs
            audio_tensor = torch.tensor(audio_data.T, dtype=torch.float32)

            # Ensure stereo
            if audio_tensor.shape[0] == 1:
                audio_tensor = audio_tensor.repeat(2, 1)

            # Resample if needed
            if sr != self.sample_rate:
                print(f"Resampling from {sr} to {self.sample_rate} Hz...")
                import torchaudio.functional as F
                audio_tensor = F.resample(audio_tensor, sr, self.sample_rate)

            # Add batch dimension: (batch, channels, samples)
            audio_tensor = audio_tensor.unsqueeze(0).to(self.device)

            # Apply model
            print("Separating audio...")
            with torch.no_grad():
                sources = apply_model(self.model, audio_tensor, progress=True)

            # sources shape: (batch, sources, channels, samples)
            # Sources order for htdemucs: drums, bass, other, vocals
            source_names = self.model.sources
            vocals_idx = source_names.index("vocals")

            print("Saving separated audio...")

            # Save vocals
            vocals = sources[0, vocals_idx].cpu().numpy()
            vocals_path = self.output_dir / f"{output_name}_vocals.wav"

            # Transpose back to (samples, channels) for soundfile
            sf.write(str(vocals_path), vocals.T, self.sample_rate)
            print(f"Saved vocals: {vocals_path}")

            # Optionally save accompaniment (all non-vocal sources mixed)
            accompaniment_path = None
            if save_all_stems:
                # Save each stem
                for i, name in enumerate(source_names):
                    stem = sources[0, i].cpu().numpy()
                    stem_path = self.output_dir / f"{output_name}_{name}.wav"
                    sf.write(str(stem_path), stem.T, self.sample_rate)
                    print(f"Saved {name}: {stem_path}")

                # Also save accompaniment (everything except vocals)
                accompaniment = sources[0].sum(dim=0) - sources[0, vocals_idx]
                accompaniment = accompaniment.cpu().numpy()
                accompaniment_path = self.output_dir / f"{output_name}_accompaniment.wav"
                sf.write(str(accompaniment_path), accompaniment.T, self.sample_rate)
                print(f"Saved accompaniment: {accompaniment_path}")

            return SeparationResult(
                success=True,
                input_path=input_path,
                vocals_path=vocals_path,
                accompaniment_path=accompaniment_path,
            )

        except Exception as e:
            import traceback
            return SeparationResult(
                success=False,
                input_path=input_path,
                error=f"{str(e)}\n{traceback.format_exc()}"
            )


# CLI interface
if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python -m src.audio.separator <input_audio> [output_name]")
        sys.exit(1)

    input_path = sys.argv[1]
    output_name = sys.argv[2] if len(sys.argv) > 2 else None

    separator = VocalSeparator()
    result = separator.separate(input_path, output_name)

    if result.success:
        print(f"\nSuccess!")
        print(f"Vocals: {result.vocals_path}")
        if result.accompaniment_path:
            print(f"Accompaniment: {result.accompaniment_path}")
    else:
        print(f"\nFailed: {result.error}")
