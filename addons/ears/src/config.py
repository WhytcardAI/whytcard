"""
Configuration management for WhytCard Transcript
"""

from pathlib import Path
from typing import Optional
import yaml
from pydantic import BaseModel
from pydantic_settings import BaseSettings


class WhisperConfig(BaseModel):
    """Whisper model configuration"""
    model: str = "base"
    device: str = "cpu"
    language: str = "fr"


class HubConfig(BaseModel):
    """WhytCard Hub connection configuration"""
    url: str = "http://localhost:3000"
    token: Optional[str] = None
    auto_register: bool = True


class OutputConfig(BaseModel):
    """Output format configuration"""
    format: str = "markdown"
    include_timestamps: bool = False
    include_confidence: bool = False
    include_raw: bool = True
    include_corrected: bool = True
    include_summary: bool = True


class ServerConfig(BaseModel):
    """Server configuration"""
    host: str = "0.0.0.0"
    port: int = 3001


class CapabilityConfig(BaseModel):
    """Hub capability configuration"""
    name: str
    description: str
    timeout_ms: int = 300000


class Settings(BaseSettings):
    """Main settings class"""
    whisper: WhisperConfig = WhisperConfig()
    hub: HubConfig = HubConfig()
    output: OutputConfig = OutputConfig()
    server: ServerConfig = ServerConfig()
    capabilities: list[CapabilityConfig] = []

    class Config:
        env_prefix = "WHYTCARD_TRANSCRIPT_"
        env_nested_delimiter = "__"


def load_config(config_path: Optional[Path] = None) -> Settings:
    """Load configuration from YAML file and environment variables"""
    if config_path is None:
        config_path = Path(__file__).parent.parent / "config.yaml"

    config_data = {}
    if config_path.exists():
        with open(config_path, "r", encoding="utf-8") as f:
            config_data = yaml.safe_load(f) or {}

    return Settings(**config_data)


# Global settings instance
settings = load_config()
