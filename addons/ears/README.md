# WhytCard Addon - Ears (Transcription)

Service de transcription audio (Speech-to-Text).

## Fonctionnalite

- Transcription audio vers texte
- Support multi-langues
- Export Markdown/PDF

## Stack

| Composant | Technologie    |
| --------- | -------------- |
| Moteur    | OpenAI Whisper |
| Runtime   | Python 3.11+   |
| API       | FastAPI        |

## Langues

FR, EN, DE, ES, IT, PT, NL

## API

| Endpoint           | Description        |
| ------------------ | ------------------ |
| `POST /transcribe` | Transcrire fichier |
| `GET /languages`   | Langues supportees |
| `GET /health`      | Status             |

## Enregistrement Hub

```
POST /api/addons/register
{
  "type": "service",
  "name": "WhytCard Ears",
  "capabilities": ["transcribe_audio"],
  "endpoint": "http://localhost:3001"
}
```

## Source

Migration depuis: `old/Persona/Ears/`
