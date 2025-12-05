#!/usr/bin/env python
"""
Interactive chat with Vincent's AI avatar.

Requires either:
1. A GGUF model in models/gguf/ (local inference)
2. OpenAI API key (cloud inference)
3. Ollama running locally

Usage:
    python scripts/chat_vincent.py [--backend local|openai|ollama]
"""

import sys
import os
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))


def find_gguf_model() -> str | None:
    """Find a GGUF model in models/gguf/."""
    models_dir = Path("models/gguf")
    if models_dir.exists():
        for model_file in models_dir.glob("*.gguf"):
            return str(model_file)
    return None


def chat_with_local(persona_path: str, vocals_path: str | None):
    """Chat using local llama.cpp model."""
    from src.avatar.prompt_generator import PersonaPrompt
    from src.avatar.avatar import AvatarEngine

    model_path = find_gguf_model()
    if not model_path:
        print("No GGUF model found in models/gguf/")
        print("Please download a model first:")
        print("  pip install huggingface-hub")
        print("  huggingface-cli download TheBloke/Mistral-7B-Instruct-v0.2-GGUF mistral-7b-instruct-v0.2.Q4_K_M.gguf --local-dir models/gguf/")
        return 1

    persona = PersonaPrompt.load(persona_path)
    avatar = AvatarEngine(
        model_path=model_path,
        persona=persona,
        speaker_wav=vocals_path,
        enable_voice=vocals_path is not None,
    )

    return run_chat_loop(avatar, persona.name)


def chat_with_openai(persona_path: str):
    """Chat using OpenAI API."""
    import openai
    from src.avatar.prompt_generator import PersonaPrompt

    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print("OPENAI_API_KEY environment variable not set")
        return 1

    persona = PersonaPrompt.load(persona_path)
    client = openai.OpenAI(api_key=api_key)

    messages = [{"role": "system", "content": persona.system_prompt}]

    print(f"\nChat with {persona.name} (OpenAI)")
    print("Type 'quit' to exit, 'reset' to clear history")
    print("-" * 60)

    while True:
        try:
            user_input = input("\nYou: ").strip()
            if not user_input:
                continue
            if user_input.lower() == "quit":
                break
            if user_input.lower() == "reset":
                messages = [{"role": "system", "content": persona.system_prompt}]
                print("Conversation reset.")
                continue

            messages.append({"role": "user", "content": user_input})

            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=messages,
                temperature=0.7,
                max_tokens=500,
            )

            assistant_message = response.choices[0].message.content
            messages.append({"role": "assistant", "content": assistant_message})

            print(f"\n{persona.name}: {assistant_message}")

        except KeyboardInterrupt:
            print("\n\nGoodbye!")
            break
        except Exception as e:
            print(f"Error: {e}")

    return 0


def chat_with_ollama(persona_path: str, model_name: str = "mistral"):
    """Chat using Ollama API."""
    import requests
    from src.avatar.prompt_generator import PersonaPrompt

    persona = PersonaPrompt.load(persona_path)

    # Test connection
    try:
        r = requests.get("http://localhost:11434/api/tags")
        r.raise_for_status()
    except Exception as e:
        print(f"Cannot connect to Ollama: {e}")
        print("Make sure Ollama is running: ollama serve")
        return 1

    messages = [{"role": "system", "content": persona.system_prompt}]

    print(f"\nChat with {persona.name} (Ollama - {model_name})")
    print("Type 'quit' to exit, 'reset' to clear history")
    print("-" * 60)

    while True:
        try:
            user_input = input("\nYou: ").strip()
            if not user_input:
                continue
            if user_input.lower() == "quit":
                break
            if user_input.lower() == "reset":
                messages = [{"role": "system", "content": persona.system_prompt}]
                print("Conversation reset.")
                continue

            messages.append({"role": "user", "content": user_input})

            response = requests.post(
                "http://localhost:11434/api/chat",
                json={
                    "model": model_name,
                    "messages": messages,
                    "stream": False,
                },
            )
            response.raise_for_status()

            data = response.json()
            assistant_message = data["message"]["content"]
            messages.append({"role": "assistant", "content": assistant_message})

            print(f"\n{persona.name}: {assistant_message}")

        except KeyboardInterrupt:
            print("\n\nGoodbye!")
            break
        except Exception as e:
            print(f"Error: {e}")

    return 0


def chat_simple(persona_path: str):
    """Simple chat mode - just display the prompt, no actual LLM."""
    from src.avatar.prompt_generator import PersonaPrompt

    persona = PersonaPrompt.load(persona_path)

    print("=" * 60)
    print(f"Vincent's Persona Loaded: {persona.name}")
    print("=" * 60)
    print("\nSystem Prompt:")
    print("-" * 60)
    print(persona.system_prompt)
    print("-" * 60)

    print("\nThis persona can be used with:")
    print("  - Local LLM: Download a GGUF model to models/gguf/")
    print("  - OpenAI: Set OPENAI_API_KEY and use --backend openai")
    print("  - Ollama: Run 'ollama serve' and use --backend ollama")

    return 0


def run_chat_loop(avatar, name: str) -> int:
    """Run the interactive chat loop."""
    print(f"\nChat with {name}")
    print("Type 'quit' to exit, 'reset' to clear history")
    print("-" * 60)

    while True:
        try:
            user_input = input("\nYou: ").strip()
            if not user_input:
                continue
            if user_input.lower() == "quit":
                break
            if user_input.lower() == "reset":
                avatar.reset_conversation()
                print("Conversation reset.")
                continue

            response = avatar.chat(user_input)
            print(f"\n{name}: {response.text}")

            if response.audio_path:
                print(f"[Audio saved: {response.audio_path}]")

        except KeyboardInterrupt:
            print("\n\nGoodbye!")
            break
        except Exception as e:
            print(f"Error: {e}")

    return 0


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Chat with Vincent's AI avatar")
    parser.add_argument(
        "--backend",
        choices=["local", "openai", "ollama", "simple"],
        default="simple",
        help="Backend to use for chat (default: simple - just show persona)"
    )
    parser.add_argument(
        "--model",
        default="mistral",
        help="Model name for Ollama (default: mistral)"
    )
    parser.add_argument(
        "--voice",
        action="store_true",
        help="Enable voice synthesis (local backend only)"
    )
    args = parser.parse_args()

    # Paths
    persona_path = "data/subjects/vincent/persona.json"
    vocals_path = "data/subjects/vincent/audio/source_frankenstein_vocals.wav"

    if not Path(persona_path).exists():
        print(f"Persona not found: {persona_path}")
        print("Run 'python scripts/generate_persona.py' first")
        return 1

    vocals = vocals_path if args.voice and Path(vocals_path).exists() else None

    if args.backend == "simple":
        return chat_simple(persona_path)
    elif args.backend == "local":
        return chat_with_local(persona_path, vocals)
    elif args.backend == "openai":
        return chat_with_openai(persona_path)
    elif args.backend == "ollama":
        return chat_with_ollama(persona_path, args.model)

    return 0


if __name__ == "__main__":
    sys.exit(main())
