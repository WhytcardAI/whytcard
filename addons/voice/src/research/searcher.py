"""
Web search for information about a person using Tavily API.
"""

import os
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime

try:
    from tavily import TavilyClient
    HAS_TAVILY = True
except ImportError:
    HAS_TAVILY = False


@dataclass
class SearchResult:
    """A single search result."""
    title: str
    url: str
    content: str
    score: float = 0.0
    published_date: Optional[str] = None


@dataclass
class PersonSearchResults:
    """All search results for a person."""
    name: str
    query: str
    timestamp: str
    results: list[SearchResult] = field(default_factory=list)
    raw_response: Optional[dict] = None

    @property
    def urls(self) -> list[str]:
        return [r.url for r in self.results]

    @property
    def summary(self) -> str:
        return "\n\n".join([
            f"**{r.title}**\n{r.url}\n{r.content[:500]}..."
            for r in self.results[:5]
        ])


class PersonSearcher:
    """
    Search the web for information about a person.
    Uses Tavily API for high-quality search results.
    """

    def __init__(self, api_key: Optional[str] = None):
        """
        Initialize the searcher.

        Args:
            api_key: Tavily API key. If not provided, reads from TAVILY_API_KEY env var.
        """
        if not HAS_TAVILY:
            raise ImportError("tavily-python not installed. Run: pip install tavily-python")

        self.api_key = api_key or os.getenv("TAVILY_API_KEY")
        if not self.api_key:
            raise ValueError("Tavily API key required. Set TAVILY_API_KEY environment variable.")

        self.client = TavilyClient(api_key=self.api_key)

    def search(
        self,
        name: str,
        context: Optional[str] = None,
        max_results: int = 10,
        search_depth: str = "advanced",
        include_domains: Optional[list[str]] = None,
        exclude_domains: Optional[list[str]] = None,
    ) -> PersonSearchResults:
        """
        Search for information about a person.

        Args:
            name: Person's name
            context: Additional context (profession, location, etc.)
            max_results: Maximum number of results
            search_depth: "basic" or "advanced"
            include_domains: Only search these domains
            exclude_domains: Exclude these domains

        Returns:
            PersonSearchResults with all found information
        """
        # Build query
        query = f"{name}"
        if context:
            query += f" {context}"

        # Execute search
        response = self.client.search(
            query=query,
            search_depth=search_depth,
            max_results=max_results,
            include_domains=include_domains or [],
            exclude_domains=exclude_domains or [],
        )

        # Parse results
        results = []
        for item in response.get("results", []):
            results.append(SearchResult(
                title=item.get("title", ""),
                url=item.get("url", ""),
                content=item.get("content", ""),
                score=item.get("score", 0.0),
                published_date=item.get("published_date"),
            ))

        return PersonSearchResults(
            name=name,
            query=query,
            timestamp=datetime.now().isoformat(),
            results=results,
            raw_response=response,
        )

    def search_social(
        self,
        name: str,
        platforms: list[str] = None,
    ) -> PersonSearchResults:
        """
        Search specifically for social media profiles.

        Args:
            name: Person's name
            platforms: List of platforms to search (instagram, linkedin, youtube, etc.)

        Returns:
            PersonSearchResults focused on social media
        """
        platforms = platforms or ["instagram", "linkedin", "youtube", "twitter", "tiktok"]

        # Build platform-specific query
        platform_str = " OR ".join(platforms)
        query = f"{name} ({platform_str}) profile"

        return self.search(
            name=name,
            context=f"site:({' OR site:'.join([f'{p}.com' for p in platforms])})",
            max_results=20,
        )

    def search_professional(self, name: str, profession: Optional[str] = None) -> PersonSearchResults:
        """
        Search for professional information.

        Args:
            name: Person's name
            profession: Known profession to narrow search

        Returns:
            PersonSearchResults focused on professional info
        """
        context = profession or ""
        context += " biography career portfolio work"

        return self.search(name=name, context=context)


# CLI interface
if __name__ == "__main__":
    import sys
    import json

    if len(sys.argv) < 2:
        print("Usage: python -m src.research.searcher <name> [context]")
        sys.exit(1)

    name = sys.argv[1]
    context = sys.argv[2] if len(sys.argv) > 2 else None

    searcher = PersonSearcher()
    results = searcher.search(name, context)

    print(f"\n{'='*60}")
    print(f"Search Results for: {results.name}")
    print(f"Query: {results.query}")
    print(f"Found: {len(results.results)} results")
    print(f"{'='*60}\n")

    for i, r in enumerate(results.results, 1):
        print(f"{i}. {r.title}")
        print(f"   URL: {r.url}")
        print(f"   {r.content[:200]}...")
        print()
