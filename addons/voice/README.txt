# WhytCard-Human

**Pipeline complet pour creer des avatars IA personnalises a partir de vraies personnes.**

Local-first, privacy-friendly, open-source.

## Vision

Transformer n'importe quelle personne en un avatar IA interactif qui :

- Parle avec sa vraie voix (clonee)
- Repond avec sa personnalite et son histoire
- Fonctionne 100% en local (pas de cloud)

## Pipeline

```
RECHERCHE          DOCUMENTATION         AUDIO              VOIX              AVATAR
    |                   |                  |                  |                  |
    v                   v                  v                  v                  v
+--------+        +-----------+      +----------+       +----------+      +----------+
| Web    |   ->   | Profil    |  ->  | Download |  ->   | Clone    |  ->  | LLM +    |
| Search |        | Facettes  |      | Separate |       | XTTS     |      | Voice    |
| Tavily |        | Style     |      | Demucs   |       | Train    |      | Prompt   |
+--------+        +-----------+      +----------+       +----------+      +----------+
```

## Structure

```
WhytCard-Human/
|-- src/
|   |-- research/       # Module recherche web
|   |-- documentation/  # Generation de profils
|   |-- audio/          # Traitement audio
|   |-- voice/          # Clonage vocal
|   |-- avatar/         # Creation avatar IA
|   |-- pipeline/       # Orchestration complete
|
|-- data/
|   |-- subjects/       # Donnees par personne (Vincent/, Alice/, ...)
|   |-- audio/
|   |   |-- raw/        # Audio brut telecharge
|   |   |-- processed/  # Audio separe/nettoye
|   |-- voices/         # Modeles de voix clonees
|
|-- models/
|   |-- gguf/           # Modeles LLM locaux
|   |-- xtts/           # Modeles XTTS
|
|-- docs/
|   |-- guides/         # Guides d'utilisation
|   |-- templates/      # Templates de documentation
|
|-- scripts/            # Scripts utilitaires
|-- config/             # Configuration
```

## Installation

```bash
# Cloner le repo
git clone https://github.com/WhytcardAI/WhytCard-Human.git
cd WhytCard-Human

# Installer les dependances Python
pip install -r requirements.txt

# Telecharger les modeles (optionnel, fait automatiquement)
python scripts/download_models.py
```

## Usage Rapide

```bash
# Pipeline complet pour une nouvelle personne
python -m src.pipeline.create_human "Vincent Veve" \
    --instagram "@vincentveve" \
    --linkedin "vincent-veve" \
    --youtube "https://youtube.com/watch?v=..." \
    --output data/subjects/vincent

# Ou etape par etape
python -m src.research.search "Vincent Veve"
python -m src.audio.download "https://youtube.com/..."
python -m src.audio.separate data/audio/raw/video.wav
python -m src.voice.clone data/audio/processed/vocals.wav
python -m src.avatar.create data/subjects/vincent
```

## Modules

### Research

Recherche automatisee d'informations sur une personne via Tavily et extraction structuree.

### Documentation

Generation de profils structures avec facettes, style graphique, liens.

### Audio

- Telechargement YouTube/web
- Conversion formats
- Separation vocale (Demucs)
- Nettoyage audio

### Voice

- Clonage vocal avec Coqui XTTS
- Fine-tuning optionnel
- Inference temps reel

### Avatar

- Generation de system prompts personnalises
- Integration LLM local (llama.cpp)
- Pipeline voix complete (text -> audio)

## Technologies

| Composant         | Technologie         |
| ----------------- | ------------------- |
| Recherche Web     | Tavily API          |
| Audio Download    | yt-dlp              |
| Audio Conversion  | ffmpeg              |
| Separation Vocale | Demucs (Meta)       |
| Clonage Voix      | Coqui XTTS          |
| LLM Local         | llama.cpp / Qwen2.5 |
| Embeddings        | FastEmbed           |

## License

MIT - WhytCard Project 2024
