# WhytCard Transcript

Addon de transcription audio pour WhytCard Hub.

## Fonctionnalites

- Transcription audio vers texte via OpenAI Whisper
- Support multi-langues (FR, EN, DE, ES, IT, PT, NL)
- Generation de fichiers Markdown structures
- Correction automatique et resume
- Integration avec WhytCard Hub via API

## Stack Technique

| Composant      | Role                    |
| -------------- | ----------------------- |
| OpenAI Whisper | Moteur de transcription |
| Python 3.11+   | Runtime                 |
| FastAPI        | API REST                |
| ReportLab      | Export PDF              |

## Installation

```bash
cd WhytCard-Transcript
pip install -r requirements.txt
```

## Usage

### CLI

```bash
python -m whytcard_transcript transcribe audio.ogg --lang fr --output transcription.md
```

### API

```bash
python -m whytcard_transcript serve --port 3001
```

### Integration Hub

L'addon s'enregistre automatiquement aupres du Hub WhytCard sur le port 3000.

## Endpoints API

| Endpoint              | Methode | Description                 |
| --------------------- | ------- | --------------------------- |
| `/api/transcribe`     | POST    | Transcrire un fichier audio |
| `/api/transcribe/url` | POST    | Transcrire depuis URL       |
| `/api/languages`      | GET     | Langues supportees          |
| `/api/models`         | GET     | Modeles Whisper disponibles |
| `/api/health`         | GET     | Status du service           |

## Configuration

```yaml
# config.yaml
whisper:
  model: base # tiny, base, small, medium, large
  device: cpu # cpu, cuda

hub:
  url: http://localhost:3000
  token: ${WHYTCARD_TOKEN}

output:
  format: markdown # markdown, json, txt, pdf
  include_timestamps: false
  include_confidence: false
```

## Capabilities Hub

L'addon expose les capabilities suivantes au Hub:

- `transcribe_audio` - Transcrire un fichier audio
- `transcribe_url` - Transcrire depuis une URL
- `export_pdf` - Exporter en PDF

## License

MIT - WhytCard Project
