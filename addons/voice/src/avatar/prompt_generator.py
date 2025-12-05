"""
Generate system prompts for AI personas based on research and facettes.
"""

import json
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional

try:
    from jinja2 import Environment, FileSystemLoader, select_autoescape
    HAS_JINJA = True
except ImportError:
    HAS_JINJA = False

from ..research.extractor import PersonProfile, Facette


@dataclass
class PersonaPrompt:
    """A complete persona prompt for an AI avatar."""
    name: str
    system_prompt: str
    personality_traits: list[str] = field(default_factory=list)
    speaking_style: Optional[str] = None
    knowledge_domains: list[str] = field(default_factory=list)
    example_phrases: list[str] = field(default_factory=list)
    facettes: list[Facette] = field(default_factory=list)
    metadata: dict = field(default_factory=dict)

    def save(self, path: str) -> None:
        """Save prompt to file."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        data = {
            "name": self.name,
            "system_prompt": self.system_prompt,
            "personality_traits": self.personality_traits,
            "speaking_style": self.speaking_style,
            "knowledge_domains": self.knowledge_domains,
            "example_phrases": self.example_phrases,
            "facettes": [
                {
                    "name": f.name,
                    "description": f.description,
                    "color": f.color,
                    "symbol": f.symbol,
                    "keywords": f.keywords,
                }
                for f in self.facettes
            ],
            "metadata": self.metadata,
        }

        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    @classmethod
    def load(cls, path: str) -> "PersonaPrompt":
        """Load prompt from file."""
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)

        prompt = cls(
            name=data["name"],
            system_prompt=data["system_prompt"],
            personality_traits=data.get("personality_traits", []),
            speaking_style=data.get("speaking_style"),
            knowledge_domains=data.get("knowledge_domains", []),
            example_phrases=data.get("example_phrases", []),
            metadata=data.get("metadata", {}),
        )

        # Reconstruct facettes
        for f in data.get("facettes", []):
            prompt.facettes.append(Facette(
                name=f["name"],
                description=f["description"],
                color=f.get("color"),
                symbol=f.get("symbol"),
                keywords=f.get("keywords", []),
            ))

        return prompt


class PromptGenerator:
    """
    Generate AI persona prompts from research data.

    Can use Jinja2 templates for customizable prompt generation.
    """

    DEFAULT_TEMPLATE = """Tu es {name}, une vraie personne avec une personnalite unique.

## Identite
{bio}

## Traits de personnalite
{personality_section}

## Style de communication
{speaking_style}

## Domaines d'expertise
{knowledge_section}

## Facettes de ta personnalite
{facettes_section}

## Instructions
- Reponds toujours comme si tu etais vraiment {name}
- Utilise ton style de communication naturel
- Puise dans tes domaines d'expertise pour enrichir tes reponses
- Exprime ta personnalite a travers tes differentes facettes
- Sois authentique et coherent avec ton identite

{additional_instructions}
"""

    def __init__(self, templates_dir: Optional[str] = None):
        """
        Initialize the generator.

        Args:
            templates_dir: Directory containing Jinja2 templates. If None, uses default template.
        """
        self.templates_dir = Path(templates_dir) if templates_dir else None
        self.env = None

        if self.templates_dir and HAS_JINJA:
            self.env = Environment(
                loader=FileSystemLoader(str(self.templates_dir)),
                autoescape=select_autoescape(["html", "xml"]),
            )

    def generate(
        self,
        profile: PersonProfile,
        template_name: Optional[str] = None,
        additional_instructions: str = "",
        speaking_style: Optional[str] = None,
    ) -> PersonaPrompt:
        """
        Generate a persona prompt from a profile.

        Args:
            profile: PersonProfile with research data
            template_name: Jinja2 template file name (if using templates)
            additional_instructions: Extra instructions to add
            speaking_style: Override speaking style

        Returns:
            PersonaPrompt ready for use
        """
        # Build personality section
        if profile.keywords:
            personality_traits = profile.keywords[:10]
        else:
            personality_traits = ["authentique", "passionne", "expressif"]

        personality_section = "\n".join([f"- {trait}" for trait in personality_traits])

        # Build knowledge section
        knowledge_domains = profile.professions if profile.professions else ["son domaine d'expertise"]
        knowledge_section = "\n".join([f"- {domain}" for domain in knowledge_domains])

        # Build facettes section
        if profile.facettes:
            facettes_lines = []
            for f in profile.facettes:
                line = f"### {f.name}"
                if f.color:
                    line += f" ({f.color})"
                facettes_lines.append(line)
                facettes_lines.append(f.description)
                if f.keywords:
                    facettes_lines.append(f"Mots-cles: {', '.join(f.keywords)}")
                facettes_lines.append("")
            facettes_section = "\n".join(facettes_lines)
        else:
            facettes_section = "Personnalite riche et multifacette."

        # Determine speaking style
        if speaking_style is None:
            speaking_style = "Naturel, expressif, avec une touche de creativite."

        # Bio
        bio = profile.bio if profile.bio else f"{profile.name} est une personne unique avec de nombreuses facettes."

        # Generate prompt
        if self.env and template_name:
            # Use Jinja2 template
            template = self.env.get_template(template_name)
            system_prompt = template.render(
                name=profile.name,
                bio=bio,
                personality_traits=personality_traits,
                personality_section=personality_section,
                speaking_style=speaking_style,
                knowledge_domains=knowledge_domains,
                knowledge_section=knowledge_section,
                facettes=profile.facettes,
                facettes_section=facettes_section,
                additional_instructions=additional_instructions,
                profile=profile,
            )
        else:
            # Use default template
            system_prompt = self.DEFAULT_TEMPLATE.format(
                name=profile.name,
                bio=bio,
                personality_section=personality_section,
                speaking_style=speaking_style,
                knowledge_section=knowledge_section,
                facettes_section=facettes_section,
                additional_instructions=additional_instructions,
            )

        return PersonaPrompt(
            name=profile.name,
            system_prompt=system_prompt,
            personality_traits=personality_traits,
            speaking_style=speaking_style,
            knowledge_domains=knowledge_domains,
            facettes=profile.facettes,
            metadata={
                "sources": profile.sources,
                "created_from": "PersonProfile",
            },
        )

    def generate_from_facettes(
        self,
        name: str,
        facettes: list[Facette],
        bio: Optional[str] = None,
    ) -> PersonaPrompt:
        """
        Generate a prompt directly from facettes.

        Args:
            name: Person's name
            facettes: List of Facette objects
            bio: Optional biography

        Returns:
            PersonaPrompt
        """
        # Create a minimal profile
        profile = PersonProfile(name=name)
        profile.bio = bio
        profile.facettes = facettes

        # Extract keywords from facettes
        for f in facettes:
            profile.keywords.extend(f.keywords)

        return self.generate(profile)


# CLI interface
if __name__ == "__main__":
    import sys

    print("PromptGenerator - Generate AI persona prompts")
    print("")
    print("Example usage:")
    print("  from src.research.extractor import PersonProfile, Facette")
    print("  from src.avatar.prompt_generator import PromptGenerator")
    print("")
    print("  profile = PersonProfile.load('data/subjects/john/profile.json')")
    print("  generator = PromptGenerator()")
    print("  persona = generator.generate(profile)")
    print("  persona.save('data/subjects/john/persona.json')")
    print("  print(persona.system_prompt)")
