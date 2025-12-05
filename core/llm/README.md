# WhytCard Core - LLM

Service d'inference LLM locale.

## Responsabilite

- Inference via llama.cpp
- Gestion des modeles GGUF
- Streaming des tokens

## Stack

| Composant | Technologie        |
| --------- | ------------------ |
| Runtime   | llama-server (C++) |
| Modele    | Qwen2.5-Coder-7B   |
| Format    | GGUF Q4_K_M        |

## API Interne

Communique avec le Hub via:

- HTTP local (llama-server sur port 8080)
- Ou integration directe Rust (llama-cpp-rs)

## Modeles

Stockes dans: `data/models/llm/`

| Modele                | Taille  | Usage       |
| --------------------- | ------- | ----------- |
| qwen2.5-coder-7b-q4   | ~4.7 GB | Code + Chat |
| qwen2.5-coder-1.5b-q8 | ~1.6 GB | Rapide      |

## Source

Migration depuis: `old/Persona/Body/apps/core/src/actors/llm_actor.rs`
