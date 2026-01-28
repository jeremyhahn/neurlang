"""
Neurlang Training Data Generators
=================================

This module provides specialized generators for different categories of training data:
- extension_patterns: Extension composition patterns (crypto, JSON, HTTP, DB, etc.)
- http_patterns: HTTP protocol patterns (headers, status codes, content-type)
- rest_patterns: REST API patterns (CRUD, pagination, etc.)
"""

from .extension_patterns import ExtensionPatternGenerator
from .http_patterns import HTTPPatternGenerator

__all__ = ['ExtensionPatternGenerator', 'HTTPPatternGenerator']
