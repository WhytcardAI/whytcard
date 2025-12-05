"""
FastAPI server for WhytCard Transcript addon
"""

import asyncio
import tempfile
from pathlib import Path
from typing import Optional
import httpx
import aiofiles
from fastapi import FastAPI, File, UploadFile, HTTPException, BackgroundTasks
from fastapi.responses import FileResponse, JSONResponse
from pydantic import BaseModel

from .config import settings
from .engine import TranscriptionEngine, TranscriptionResult, get_engine
from .formatters import get_formatter


app = FastAPI(
    title="WhytCard Transcript",
    description="Audio transcription addon for WhytCard Hub",
    version="0.1.0"
)


# ============== Models ==============

class TranscribeURLRequest(BaseModel):
    """Request to transcribe audio from URL"""
    url: str
    language: Optional[str] = None
    output_format: Optional[str] = "markdown"
    title: Optional[str] = None


class TranscriptionResponse(BaseModel):
    """Response containing transcription result"""
    raw_text: str
    corrected_text: str
    summary: str
    language: str
    duration: float
    model_used: str


class HealthResponse(BaseModel):
    """Health check response"""
    status: str
    version: str
    model_loaded: bool
    hub_connected: bool


class LanguagesResponse(BaseModel):
    """Available languages response"""
    languages: list[dict]


class ModelsResponse(BaseModel):
    """Available models response"""
    models: list[dict]


# ============== Hub Integration ==============

async def register_with_hub():
    """Register addon capabilities with WhytCard Hub"""
    if not settings.hub.auto_register:
        return

    try:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{settings.hub.url}/api/clients/register",
                json={
                    "client_type": "addon",
                    "name": "WhytCard Transcript",
                    "version": "0.1.0",
                    "capabilities": [
                        {
                            "name": cap.name,
                            "description": cap.description,
                            "timeout_ms": cap.timeout_ms
                        }
                        for cap in settings.capabilities
                    ]
                },
                headers={
                    "Authorization": f"Bearer {settings.hub.token}" if settings.hub.token else ""
                },
                timeout=10.0
            )

            if response.status_code == 200:
                print(f"Registered with Hub: {settings.hub.url}")
            else:
                print(f"Failed to register with Hub: {response.status_code}")

    except Exception as e:
        print(f"Could not connect to Hub: {e}")


@app.on_event("startup")
async def startup_event():
    """Run on server startup"""
    # Register with hub in background
    asyncio.create_task(register_with_hub())


# ============== Endpoints ==============

@app.get("/api/health", response_model=HealthResponse)
async def health_check():
    """Check service health"""
    engine = get_engine()
    model_loaded = engine._model is not None

    # Check hub connection
    hub_connected = False
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{settings.hub.url}/api/ping",
                timeout=5.0
            )
            hub_connected = response.status_code == 200
    except Exception:
        pass

    return HealthResponse(
        status="ok",
        version="0.1.0",
        model_loaded=model_loaded,
        hub_connected=hub_connected
    )


@app.get("/api/languages", response_model=LanguagesResponse)
async def get_languages():
    """Get supported languages"""
    languages = [
        {"code": "fr", "name": "Francais"},
        {"code": "en", "name": "English"},
        {"code": "de", "name": "Deutsch"},
        {"code": "es", "name": "Espanol"},
        {"code": "it", "name": "Italiano"},
        {"code": "pt", "name": "Portugues"},
        {"code": "nl", "name": "Nederlands"},
    ]
    return LanguagesResponse(languages=languages)


@app.get("/api/models", response_model=ModelsResponse)
async def get_models():
    """Get available Whisper models"""
    models = [
        {"name": "tiny", "size": "39 MB", "speed": "fastest", "quality": "lowest"},
        {"name": "base", "size": "139 MB", "speed": "fast", "quality": "good"},
        {"name": "small", "size": "466 MB", "speed": "medium", "quality": "better"},
        {"name": "medium", "size": "1.5 GB", "speed": "slow", "quality": "great"},
        {"name": "large", "size": "2.9 GB", "speed": "slowest", "quality": "best"},
    ]
    return ModelsResponse(models=models)


@app.post("/api/transcribe", response_model=TranscriptionResponse)
async def transcribe_file(
    file: UploadFile = File(...),
    language: Optional[str] = None,
):
    """Transcribe uploaded audio file"""
    # Validate file type
    allowed_extensions = {".mp3", ".wav", ".ogg", ".m4a", ".flac", ".webm"}
    file_ext = Path(file.filename).suffix.lower()

    if file_ext not in allowed_extensions:
        raise HTTPException(
            status_code=400,
            detail=f"Unsupported file type: {file_ext}. Allowed: {allowed_extensions}"
        )

    # Save to temp file
    with tempfile.NamedTemporaryFile(delete=False, suffix=file_ext) as tmp:
        content = await file.read()
        tmp.write(content)
        tmp_path = tmp.name

    try:
        # Transcribe
        engine = get_engine()
        result = engine.transcribe(tmp_path, language=language)

        return TranscriptionResponse(
            raw_text=result.raw_text,
            corrected_text=result.corrected_text,
            summary=result.summary,
            language=result.language,
            duration=result.duration,
            model_used=result.model_used
        )

    finally:
        # Cleanup temp file
        Path(tmp_path).unlink(missing_ok=True)


@app.post("/api/transcribe/url")
async def transcribe_from_url(request: TranscribeURLRequest):
    """Transcribe audio from URL"""
    # Download file
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(request.url, timeout=60.0)
            response.raise_for_status()
            content = response.content
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Failed to download: {e}")

    # Determine extension from URL or content-type
    url_path = Path(request.url.split("?")[0])
    file_ext = url_path.suffix.lower() or ".ogg"

    # Save to temp file
    with tempfile.NamedTemporaryFile(delete=False, suffix=file_ext) as tmp:
        tmp.write(content)
        tmp_path = tmp.name

    try:
        # Transcribe
        engine = get_engine()
        result = engine.transcribe(tmp_path, language=request.language)

        # Format output
        formatter = get_formatter(request.output_format)

        if request.output_format == "json":
            return JSONResponse(content={
                "raw_text": result.raw_text,
                "corrected_text": result.corrected_text,
                "summary": result.summary,
                "language": result.language,
                "duration": result.duration,
                "model_used": result.model_used
            })
        else:
            formatted = formatter.format(result, title=request.title)
            return {"content": formatted, "format": request.output_format}

    finally:
        Path(tmp_path).unlink(missing_ok=True)


@app.post("/api/transcribe/export")
async def transcribe_and_export(
    file: UploadFile = File(...),
    language: Optional[str] = None,
    output_format: str = "markdown",
    title: Optional[str] = None,
):
    """Transcribe and export to file"""
    # Validate file type
    allowed_extensions = {".mp3", ".wav", ".ogg", ".m4a", ".flac", ".webm"}
    file_ext = Path(file.filename).suffix.lower()

    if file_ext not in allowed_extensions:
        raise HTTPException(
            status_code=400,
            detail=f"Unsupported file type: {file_ext}"
        )

    # Save input to temp file
    with tempfile.NamedTemporaryFile(delete=False, suffix=file_ext) as tmp:
        content = await file.read()
        tmp.write(content)
        tmp_path = tmp.name

    try:
        # Transcribe
        engine = get_engine()
        result = engine.transcribe(tmp_path, language=language)

        # Determine output extension
        ext_map = {
            "markdown": ".md",
            "md": ".md",
            "json": ".json",
            "pdf": ".pdf",
        }
        out_ext = ext_map.get(output_format.lower(), ".md")

        # Save to temp output file
        with tempfile.NamedTemporaryFile(delete=False, suffix=out_ext) as out_tmp:
            out_path = out_tmp.name

        formatter = get_formatter(output_format)
        formatter.save(result, out_path, title=title)

        # Return file
        return FileResponse(
            out_path,
            media_type="application/octet-stream",
            filename=f"transcription{out_ext}"
        )

    finally:
        Path(tmp_path).unlink(missing_ok=True)


def create_app() -> FastAPI:
    """Create FastAPI application"""
    return app
