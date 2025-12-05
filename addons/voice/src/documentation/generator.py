"""
Generate documentation from research data using Jinja2 templates.
"""

import os
from pathlib import Path
from datetime import datetime
from typing import Optional

try:
    from jinja2 import Environment, FileSystemLoader, select_autoescape
    HAS_JINJA = True
except ImportError:
    HAS_JINJA = False

from ..research.extractor import PersonProfile, Facette


class DocumentGenerator:
    """
    Generate documentation from research data using templates.
    """

    def __init__(self, templates_dir: str = "docs/templates"):
        """
        Initialize the generator.

        Args:
            templates_dir: Directory containing Jinja2 templates
        """
        if not HAS_JINJA:
            raise ImportError("jinja2 not installed. Run: pip install jinja2")

        self.templates_dir = Path(templates_dir)

        # Create templates dir if it doesn't exist
        self.templates_dir.mkdir(parents=True, exist_ok=True)

        # Create default templates if they don't exist
        self._ensure_default_templates()

        # Initialize Jinja environment
        self.env = Environment(
            loader=FileSystemLoader(str(self.templates_dir)),
            autoescape=select_autoescape(["html", "xml"]),
        )

    def _ensure_default_templates(self) -> None:
        """Create default templates if they don't exist."""
        # Profile template
        profile_template = self.templates_dir / "profile.md.j2"
        if not profile_template.exists():
            profile_template.write_text(DEFAULT_PROFILE_TEMPLATE, encoding="utf-8")

        # Facettes template
        facettes_template = self.templates_dir / "facettes.md.j2"
        if not facettes_template.exists():
            facettes_template.write_text(DEFAULT_FACETTES_TEMPLATE, encoding="utf-8")

        # System prompt template
        prompt_template = self.templates_dir / "system_prompt.md.j2"
        if not prompt_template.exists():
            prompt_template.write_text(DEFAULT_PROMPT_TEMPLATE, encoding="utf-8")

    def generate_profile(
        self,
        profile: PersonProfile,
        output_path: Optional[str] = None,
    ) -> str:
        """
        Generate profile documentation.

        Args:
            profile: PersonProfile to document
            output_path: Path to save output. If None, returns string only.

        Returns:
            Generated markdown content
        """
        template = self.env.get_template("profile.md.j2")

        content = template.render(
            profile=profile,
            name=profile.name,
            aliases=profile.aliases,
            professions=profile.professions,
            bio=profile.bio,
            location=profile.location,
            social_profiles=profile.social_profiles,
            facettes=profile.facettes,
            keywords=profile.keywords,
            sources=profile.sources,
            generated_at=datetime.now().isoformat(),
        )

        if output_path:
            path = Path(output_path)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

        return content

    def generate_facettes(
        self,
        name: str,
        facettes: list[Facette],
        output_path: Optional[str] = None,
    ) -> str:
        """
        Generate facettes documentation.

        Args:
            name: Person's name
            facettes: List of Facette objects
            output_path: Path to save output

        Returns:
            Generated markdown content
        """
        template = self.env.get_template("facettes.md.j2")

        content = template.render(
            name=name,
            facettes=facettes,
            generated_at=datetime.now().isoformat(),
        )

        if output_path:
            path = Path(output_path)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

        return content

    def generate_custom(
        self,
        template_name: str,
        output_path: Optional[str] = None,
        **context
    ) -> str:
        """
        Generate documentation using a custom template.

        Args:
            template_name: Template file name
            output_path: Path to save output
            **context: Template context variables

        Returns:
            Generated content
        """
        template = self.env.get_template(template_name)

        # Add common context
        context["generated_at"] = datetime.now().isoformat()

        content = template.render(**context)

        if output_path:
            path = Path(output_path)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

        return content


# Default templates
DEFAULT_PROFILE_TEMPLATE = """# {{ name }}

> Generated: {{ generated_at }}

## Overview

{% if bio %}
{{ bio }}
{% else %}
Profile of {{ name }}.
{% endif %}

{% if aliases %}
## Also Known As
{% for alias in aliases %}
- {{ alias }}
{% endfor %}
{% endif %}

{% if professions %}
## Professions
{% for prof in professions %}
- {{ prof }}
{% endfor %}
{% endif %}

{% if location %}
## Location
{{ location }}
{% endif %}

{% if social_profiles %}
## Social Profiles
| Platform | URL |
|----------|-----|
{% for sp in social_profiles %}
| {{ sp.platform }} | [{{ sp.username or 'Profile' }}]({{ sp.url }}) |
{% endfor %}
{% endif %}

{% if facettes %}
## Facettes
{% for f in facettes %}
### {{ f.name }}{% if f.color %} ({{ f.color }}){% endif %}

{{ f.description }}

{% if f.keywords %}
**Keywords:** {{ f.keywords | join(', ') }}
{% endif %}

{% endfor %}
{% endif %}

{% if keywords %}
## Keywords
{{ keywords | join(', ') }}
{% endif %}

{% if sources %}
## Sources
{% for src in sources[:10] %}
- {{ src }}
{% endfor %}
{% if sources | length > 10 %}
... and {{ sources | length - 10 }} more sources
{% endif %}
{% endif %}
"""

DEFAULT_FACETTES_TEMPLATE = """# Les Facettes de {{ name }}

> Generated: {{ generated_at }}

{% for f in facettes %}
---

## {{ f.name }}

{% if f.color %}**Couleur:** {{ f.color }}{% endif %}
{% if f.symbol %}**Symbole:** {{ f.symbol }}{% endif %}

{{ f.description }}

{% if f.keywords %}
### Mots-cles
{% for kw in f.keywords %}
- {{ kw }}
{% endfor %}
{% endif %}

{% endfor %}
"""

DEFAULT_PROMPT_TEMPLATE = """# System Prompt: {{ name }}

Tu es {{ name }}, une vraie personne avec une personnalite unique.

## Identite
{% if bio %}
{{ bio }}
{% else %}
{{ name }} est une personne authentique avec de nombreuses facettes.
{% endif %}

## Traits de personnalite
{% for trait in personality_traits %}
- {{ trait }}
{% endfor %}

## Style de communication
{{ speaking_style | default('Naturel et expressif.') }}

## Domaines d'expertise
{% for domain in knowledge_domains %}
- {{ domain }}
{% endfor %}

## Facettes
{% for f in facettes %}
### {{ f.name }}
{{ f.description }}
{% endfor %}

## Instructions
- Reponds toujours comme si tu etais vraiment {{ name }}
- Utilise ton style de communication naturel
- Sois authentique et coherent avec ton identite
- Exprime ta personnalite a travers tes differentes facettes

{{ additional_instructions | default('') }}
"""


# CLI interface
if __name__ == "__main__":
    print("DocumentGenerator - Generate documentation from research data")
    print("")
    print("Example usage:")
    print("  from src.research.extractor import PersonProfile")
    print("  from src.documentation.generator import DocumentGenerator")
    print("")
    print("  profile = PersonProfile.load('data/subjects/john/profile.json')")
    print("  generator = DocumentGenerator()")
    print("  generator.generate_profile(profile, 'data/subjects/john/docs/profile.md')")
