"""Data sanitization, secret redaction, license validation, and deduplication filters."""

from __future__ import annotations

import hashlib
import re
from typing import List, Sequence

REDACTED_TAG = "[REDACTED_SECRET]"

# Regex for private keys (RSA, DSA, EC, OPENSSH, etc.)
PRIVATE_KEY_PATTERN = re.compile(
    r"-----BEGIN [A-Z\s]+PRIVATE KEY-----[\s\S]*?-----END [A-Z\s]+PRIVATE KEY-----",
    re.MULTILINE,
)

# Common API key and token signatures
SECRET_PATTERNS = [
    # OpenAI and OpenAI-like keys
    re.compile(r"\bsk-[a-zA-Z0-9_\-]{20,}\b"),
    # GitHub personal access tokens, fine-grained, OAuth, etc.
    re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{36,}\b"),
    re.compile(r"\bgithub_pat_[A-Za-z0-9_]{30,}\b"),
    # AWS Access Key ID (AKIA... / ASIA...)
    re.compile(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"),
    # Slack tokens
    re.compile(r"\bxox[baprs]-[0-9a-zA-Z-]{10,}\b"),
    # Stripe secret/publishable tokens
    re.compile(r"\b[rs]k_(?:live|test)_[0-9a-zA-Z]{24,}\b"),
    # Generic hex tokens 32+ characters (MD5/SHA/Hex API tokens)
    re.compile(r"\b[0-9a-fA-F]{32,}\b"),
]

# Passwords and secrets in key-value assignments
KEY_VALUE_QUOTED_PATTERN = re.compile(
    r"""(?i)\b(?P<prefix>[a-zA-Z0-9_]*(?:password|passwd|pwd|secret|api_key|apikey|auth_token|access_token|client_secret)\s*[:=]\s*)(?P<quote>["'])(?P<val>[^"'\r\n]+)(?P=quote)"""
)

KEY_VALUE_UNQUOTED_PATTERN = re.compile(
    r"""(?i)\b(?P<prefix>[a-zA-Z0-9_]*(?:password|passwd|pwd|secret|api_key|apikey|auth_token|access_token|client_secret)\s*[:=]\s*)(?P<val>[^\s,;]+)"""
)

COMMON_TYPES_OR_KEYWORDS = {
    "str",
    "string",
    "int",
    "bool",
    "boolean",
    "float",
    "bytes",
    "dict",
    "list",
    "any",
    "optional",
    "none",
    "true",
    "false",
}

# Permissible open source license patterns (MIT, Apache-2.0, BSD)
PERMISSIBLE_LICENSE_PATTERNS = [
    # SPDX license identifier tags
    re.compile(r"spdx-license-identifier:\s*(?:mit|apache-2\.0|bsd-2-clause|bsd-3-clause|bsd)", re.IGNORECASE),
    # MIT variations and standard phrase
    re.compile(r"\bmit\s+license\b", re.IGNORECASE),
    re.compile(r"\bpermission\s+is\s+hereby\s+granted,\s+free\s+of\s+charge\b", re.IGNORECASE),
    # Apache 2.0 variations
    re.compile(r"\bapache\s+license(?:,\s+version\s+2\.0)?\b", re.IGNORECASE),
    re.compile(r"\blicensed\s+under\s+the\s+apache\s+license\b", re.IGNORECASE),
    re.compile(r"\bhttp://www\.apache\.org/licenses/license-2\.0\b", re.IGNORECASE),
    re.compile(r"\bapache-2\.0\b", re.IGNORECASE),
    # BSD variations
    re.compile(r"\bbsd(?:\s+license|-2-clause|-3-clause|\s+2-clause|\s+3-clause)\b", re.IGNORECASE),
    re.compile(r"\bredistribution\s+and\s+use\s+in\s+source\s+and\s+binary\s+forms\b", re.IGNORECASE),
]

SHORT_PERMISSIBLE_NAMES = {
    "MIT",
    "APACHE-2.0",
    "APACHE 2.0",
    "APACHE2",
    "APACHE-2",
    "BSD",
    "BSD-2-CLAUSE",
    "BSD-3-CLAUSE",
    "BSD 2-CLAUSE",
    "BSD 3-CLAUSE",
}

RESTRICTIVE_PATTERNS = [
    re.compile(r"\b(?:gpl(?:-[0-9.]+)?|agpl(?:-[0-9.]+)?|lgpl(?:-[0-9.]+)?)\b", re.IGNORECASE),
    re.compile(r"\bgnu\s+(?:general|affine|lesser)\s+public\s+license\b", re.IGNORECASE),
    re.compile(r"\bproprietary\b", re.IGNORECASE),
    re.compile(r"\bcommercial\s+license\b", re.IGNORECASE),
]


def redact_secrets(text: str) -> str:
    """Redact sensitive credentials, API keys, private keys, and passwords.

    Replaces matched secrets with `[REDACTED_SECRET]`.
    """
    if not text:
        return text

    result = text

    # 1. Redact multi-line private keys
    result = PRIVATE_KEY_PATTERN.sub(REDACTED_TAG, result)

    # 2. Redact passwords and secrets in key-value assignments
    def _replace_quoted(m: re.Match) -> str:
        val = m.group("val")
        if val == REDACTED_TAG:
            return m.group(0)
        return f'{m.group("prefix")}{m.group("quote")}{REDACTED_TAG}{m.group("quote")}'

    def _replace_unquoted(m: re.Match) -> str:
        val = m.group("val")
        if val == REDACTED_TAG or val.lower() in COMMON_TYPES_OR_KEYWORDS:
            return m.group(0)
        return f'{m.group("prefix")}{REDACTED_TAG}'

    result = KEY_VALUE_QUOTED_PATTERN.sub(_replace_quoted, result)
    result = KEY_VALUE_UNQUOTED_PATTERN.sub(_replace_unquoted, result)

    # 3. Redact specific API key patterns (OpenAI, GitHub, AWS, Stripe, Hex)
    for pattern in SECRET_PATTERNS:
        result = pattern.sub(REDACTED_TAG, result)

    return result


def is_valid_license(text: str) -> bool:
    """Check if the provided license text or identifier belongs to a permissible open source license.

    Permissible: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause.
    Non-permissible: GPL, AGPL, Proprietary, Commercial, or unstated licenses.
    """
    if not text or not isinstance(text, str):
        return False

    stripped = text.strip()
    if not stripped:
        return False

    # Short exact name check
    if stripped.upper() in SHORT_PERMISSIBLE_NAMES:
        return True

    # Check for permissible license markers
    has_permissible = any(p.search(stripped) for p in PERMISSIBLE_LICENSE_PATTERNS)
    if not has_permissible:
        return False

    # Check if a restrictive license (e.g. GPL, proprietary) overrides or excludes permissive use
    has_restrictive = any(p.search(stripped) for p in RESTRICTIVE_PATTERNS)
    if has_restrictive:
        # Allow dual-licensing only if explicitly stated (e.g. MIT OR GPL)
        dual_license = re.search(r"(?:mit|apache|bsd)\s+(?:or|and)\s+(?:gpl|agpl)", stripped, re.IGNORECASE)
        if not dual_license:
            return False

    return True


def deduplicate_texts(texts: Sequence[str]) -> List[str]:
    """Deduplicate a sequence of text strings based on SHA-256 hash.

    Preserves the initial encounter order.
    """
    seen: set[str] = set()
    unique: List[str] = []
    for item in texts:
        digest = hashlib.sha256(item.encode("utf-8")).hexdigest()
        if digest not in seen:
            seen.add(digest)
            unique.append(item)
    return unique
