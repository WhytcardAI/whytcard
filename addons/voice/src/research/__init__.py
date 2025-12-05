"""
Research module: Web search and information extraction about a person.
"""

from .searcher import PersonSearcher
from .extractor import InfoExtractor

__all__ = ["PersonSearcher", "InfoExtractor"]
