# WhytCard Addon - Voice (TTS + Clonage)

Service de synthese vocale et clonage.

## Fonctionnalite

- Text-to-Speech
- Clonage vocal
- Separation audio (Demucs)

## Stack

| Composant  | Technologie  |
| ---------- | ------------ |
| TTS        | Coqui XTTS   |
| Separation | Demucs       |
| Download   | yt-dlp       |
| Runtime    | Python 3.11+ |

## API

| Endpoint           | Description          |
| ------------------ | -------------------- |
| `POST /synthesize` | Texte vers audio     |
| `POST /clone`      | Cloner une voix      |
| `POST /separate`   | Separer vocals/music |
| `GET /voices`      | Voix disponibles     |

## Enregistrement Hub

```
POST /api/addons/register
{
  "type": "service",
  "name": "WhytCard Voice",
  "capabilities": ["synthesize_speech", "clone_voice"],
  "endpoint": "http://localhost:3002"
}
```

## Source

Migration depuis: `old/Persona/Voice/`
