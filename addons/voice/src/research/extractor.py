"""
Extract and structure information from search results.
"""

import re
import json
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime

from .searcher import PersonSearchResults, SearchResult


@dataclass
class SocialProfile:
    """A social media profile."""
    platform: str
    url: str
    username: Optional[str] = None
    verified: bool = False


@dataclass
class Facette:
    """A facette/aspect of a person's identity."""
    name: str
    description: str
    color: Optional[str] = None
    symbol: Optional[str] = None
    keywords: list[str] = field(default_factory=list)
    sources: list[str] = field(default_factory=list)


@dataclass
class PersonProfile:
    """Structured profile of a person."""
    name: str
    aliases: list[str] = field(default_factory=list)
    professions: list[str] = field(default_factory=list)
    bio: Optional[str] = None
    location: Optional[str] = None
    social_profiles: list[SocialProfile] = field(default_factory=list)
    facettes: list[Facette] = field(default_factory=list)
    keywords: list[str] = field(default_factory=list)
    sources: list[str] = field(default_factory=list)
    raw_info: dict = field(default_factory=dict)
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "name": self.name,
            "aliases": self.aliases,
            "professions": self.professions,
            "bio": self.bio,
            "location": self.location,
            "social_profiles": [
                {"platform": sp.platform, "url": sp.url, "username": sp.username}
                for sp in self.social_profiles
            ],
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
            "keywords": self.keywords,
            "sources": self.sources,
            "created_at": self.created_at,
        }

    def to_json(self, indent: int = 2) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict(), indent=indent, ensure_ascii=False)

    def save(self, path: str) -> None:
        """Save profile to JSON file."""
        with open(path, "w", encoding="utf-8") as f:
            f.write(self.to_json())

    @classmethod
    def load(cls, path: str) -> "PersonProfile":
        """Load profile from JSON file (supports extended format)."""
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)

        profile = cls(name=data["name"])

        # Handle aliases or identity variations
        profile.aliases = data.get("aliases", [])

        # Handle professions (can be 'professions' or 'occupations')
        profile.professions = data.get("professions", data.get("occupations", []))

        # Handle bio (can be 'bio' or 'biography')
        profile.bio = data.get("bio", data.get("biography"))

        profile.location = data.get("location")
        profile.keywords = data.get("keywords", data.get("personality_traits", []))
        profile.sources = data.get("sources", [])
        profile.created_at = data.get("created_at", datetime.now().isoformat())

        # Store extended data
        profile.raw_info = {
            k: v for k, v in data.items()
            if k not in ["name", "aliases", "professions", "occupations", "bio",
                        "biography", "location", "social_profiles", "facettes",
                        "keywords", "sources", "created_at", "personality_traits"]
        }

        # Reconstruct social profiles (support multiple formats)
        for sp in data.get("social_profiles", []):
            profile.social_profiles.append(SocialProfile(
                platform=sp.get("platform", "unknown"),
                url=sp.get("url", ""),
                username=sp.get("username", sp.get("handle")),
            ))

        # Reconstruct facettes
        for f in data.get("facettes", []):
            profile.facettes.append(Facette(
                name=f.get("name", ""),
                description=f.get("description", ""),
                color=f.get("color"),
                symbol=f.get("symbol", f.get("element")),
                keywords=f.get("keywords", []),
            ))

        return profile


class InfoExtractor:
    """
    Extract structured information from search results.
    """

    # Platform patterns for social profile detection
    PLATFORM_PATTERNS = {
        "instagram": r"instagram\.com/([a-zA-Z0-9_.]+)",
        "linkedin": r"linkedin\.com/in/([a-zA-Z0-9_-]+)",
        "youtube": r"youtube\.com/(?:@|channel/|c/)([a-zA-Z0-9_-]+)",
        "twitter": r"(?:twitter|x)\.com/([a-zA-Z0-9_]+)",
        "tiktok": r"tiktok\.com/@([a-zA-Z0-9_.]+)",
        "facebook": r"facebook\.com/([a-zA-Z0-9.]+)",
    }

    def __init__(self):
        """Initialize the extractor."""
        pass

    def extract_profile(self, search_results: PersonSearchResults) -> PersonProfile:
        """
        Extract a structured profile from search results.

        Args:
            search_results: Results from PersonSearcher

        Returns:
            PersonProfile with structured information
        """
        profile = PersonProfile(name=search_results.name)

        # Collect all sources
        profile.sources = search_results.urls

        # Extract social profiles
        for result in search_results.results:
            social = self._extract_social_profile(result)
            if social and social.platform not in [sp.platform for sp in profile.social_profiles]:
                profile.social_profiles.append(social)

        # Extract keywords from all content
        all_content = " ".join([r.content for r in search_results.results])
        profile.keywords = self._extract_keywords(all_content, search_results.name)

        # Store raw info for later processing
        profile.raw_info = {
            "search_query": search_results.query,
            "search_timestamp": search_results.timestamp,
            "result_count": len(search_results.results),
            "contents": [
                {"title": r.title, "url": r.url, "content": r.content}
                for r in search_results.results
            ],
        }

        return profile

    def _extract_social_profile(self, result: SearchResult) -> Optional[SocialProfile]:
        """Extract social media profile from a search result."""
        url = result.url.lower()

        for platform, pattern in self.PLATFORM_PATTERNS.items():
            match = re.search(pattern, result.url, re.IGNORECASE)
            if match:
                return SocialProfile(
                    platform=platform,
                    url=result.url,
                    username=match.group(1) if match.groups() else None,
                )

        return None

    def _extract_keywords(self, text: str, name: str, max_keywords: int = 20) -> list[str]:
        """Extract relevant keywords from text."""
        # Simple keyword extraction - can be enhanced with NLP
        # Remove the person's name parts
        name_parts = name.lower().split()

        # Common words to ignore
        stopwords = {
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would",
            "could", "should", "may", "might", "must", "shall", "can", "this",
            "that", "these", "those", "it", "its", "they", "them", "their", "he",
            "him", "his", "she", "her", "hers", "we", "us", "our", "you", "your",
            "qui", "que", "est", "et", "le", "la", "les", "un", "une", "des",
            "de", "du", "en", "dans", "sur", "pour", "par", "avec", "son", "sa",
        }

        # Extract words
        words = re.findall(r"\b[a-zA-Z\u00C0-\u017F]{4,}\b", text.lower())

        # Filter and count
        word_counts = {}
        for word in words:
            if word not in stopwords and word not in name_parts:
                word_counts[word] = word_counts.get(word, 0) + 1

        # Sort by frequency
        sorted_words = sorted(word_counts.items(), key=lambda x: x[1], reverse=True)

        return [word for word, count in sorted_words[:max_keywords]]

    def merge_profiles(self, profiles: list[PersonProfile]) -> PersonProfile:
        """
        Merge multiple profiles into one.

        Args:
            profiles: List of profiles to merge

        Returns:
            Single merged profile
        """
        if not profiles:
            raise ValueError("No profiles to merge")

        merged = PersonProfile(name=profiles[0].name)

        for profile in profiles:
            # Merge aliases
            for alias in profile.aliases:
                if alias not in merged.aliases:
                    merged.aliases.append(alias)

            # Merge professions
            for prof in profile.professions:
                if prof not in merged.professions:
                    merged.professions.append(prof)

            # Merge social profiles
            existing_platforms = {sp.platform for sp in merged.social_profiles}
            for sp in profile.social_profiles:
                if sp.platform not in existing_platforms:
                    merged.social_profiles.append(sp)
                    existing_platforms.add(sp.platform)

            # Merge facettes
            existing_facettes = {f.name for f in merged.facettes}
            for f in profile.facettes:
                if f.name not in existing_facettes:
                    merged.facettes.append(f)
                    existing_facettes.add(f.name)

            # Merge keywords
            for kw in profile.keywords:
                if kw not in merged.keywords:
                    merged.keywords.append(kw)

            # Merge sources
            for src in profile.sources:
                if src not in merged.sources:
                    merged.sources.append(src)

            # Use first non-empty bio
            if not merged.bio and profile.bio:
                merged.bio = profile.bio

            # Use first non-empty location
            if not merged.location and profile.location:
                merged.location = profile.location

        return merged


# CLI interface
if __name__ == "__main__":
    import sys

    print("InfoExtractor - Use with PersonSearcher results")
    print("Example usage:")
    print("  from src.research.searcher import PersonSearcher")
    print("  from src.research.extractor import InfoExtractor")
    print("")
    print("  searcher = PersonSearcher()")
    print("  results = searcher.search('John Doe', 'software engineer')")
    print("  extractor = InfoExtractor()")
    print("  profile = extractor.extract_profile(results)")
    print("  profile.save('data/subjects/john_doe/profile.json')")
