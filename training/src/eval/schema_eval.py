"""Schema evaluation harness for LDrago brain model outputs."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

try:
    import jsonschema
    from jsonschema import Draft7Validator

    _HAS_JSONSCHEMA = True
except ImportError:
    _HAS_JSONSCHEMA = False


@dataclass
class EvaluationResult:
    """Result of validating a dictionary against a JSON schema."""

    is_valid: bool
    errors: List[str] = field(default_factory=list)

    def __bool__(self) -> bool:
        return self.is_valid


def resolve_schema_path(schema_relative_path: Union[str, Path]) -> Path:
    """Locate schema file by relative or filename path.

    Searches in:
    1. Direct filesystem path (if exists or absolute)
    2. brains/coding/schemas/
    3. brains/design/schemas/
    4. Repo root relative paths
    """
    p = Path(schema_relative_path)
    if p.is_file():
        return p.resolve()

    # Search upwards from current file to locate workspace root
    curr = Path(__file__).resolve()
    repo_roots: List[Path] = []
    for parent in [curr] + list(curr.parents):
        if (parent / "brains").is_dir():
            repo_roots.append(parent)
        if (parent / "pyproject.toml").is_file():
            repo_roots.append(parent.parent)

    cwd = Path.cwd()
    search_bases: List[Path] = []
    for root in repo_roots:
        search_bases.extend([
            root,
            root / "brains",
            root / "brains" / "coding" / "schemas",
            root / "brains" / "design" / "schemas",
        ])
    search_bases.extend([
        cwd,
        cwd / "brains",
        cwd / "brains" / "coding" / "schemas",
        cwd / "brains" / "design" / "schemas",
        cwd.parent / "brains" / "coding" / "schemas",
        cwd.parent / "brains" / "design" / "schemas",
    ])

    for base in search_bases:
        cand = base / schema_relative_path
        if cand.is_file():
            return cand.resolve()
        cand_name = base / p.name
        if cand_name.is_file():
            return cand_name.resolve()

    raise FileNotFoundError(
        f"Schema file '{schema_relative_path}' could not be located in brains/ or workspace."
    )


def _validate_draft7_fallback(instance: Any, schema: Dict[str, Any], path: str = "$") -> List[str]:
    """Custom Draft-07 JSON schema validator fallback when jsonschema is not installed."""
    errors: List[str] = []

    # Type check
    expected_type = schema.get("type")
    if expected_type:
        type_map = {
            "object": dict,
            "array": list,
            "string": str,
            "integer": int,
            "number": (int, float),
            "boolean": bool,
            "null": type(None),
        }
        if isinstance(expected_type, str):
            expected_types = [expected_type]
        else:
            expected_types = list(expected_type)

        matched = False
        for t_name in expected_types:
            py_type = type_map.get(t_name)
            if py_type is not None:
                if t_name == "integer" and isinstance(instance, bool):
                    continue
                if t_name == "number" and isinstance(instance, bool):
                    continue
                if isinstance(instance, py_type):
                    matched = True
                    break
        if not matched:
            errors.append(f"{path}: expected type '{expected_type}', got '{type(instance).__name__}'")
            return errors

    # Enum check
    if "enum" in schema:
        if instance not in schema["enum"]:
            errors.append(f"{path}: value '{instance}' not in allowed enum {schema['enum']}")

    # String constraints
    if isinstance(instance, str):
        if "minLength" in schema and len(instance) < schema["minLength"]:
            errors.append(f"{path}: string length {len(instance)} is shorter than minLength {schema['minLength']}")
        if "maxLength" in schema and len(instance) > schema["maxLength"]:
            errors.append(f"{path}: string length {len(instance)} is longer than maxLength {schema['maxLength']}")

    # Numeric constraints
    if isinstance(instance, (int, float)) and not isinstance(instance, bool):
        if "minimum" in schema and instance < schema["minimum"]:
            errors.append(f"{path}: value {instance} is less than minimum {schema['minimum']}")
        if "maximum" in schema and instance > schema["maximum"]:
            errors.append(f"{path}: value {instance} is greater than maximum {schema['maximum']}")

    # Object constraints
    if isinstance(instance, dict):
        required = schema.get("required", [])
        for req_prop in required:
            if req_prop not in instance:
                errors.append(f"{path}: required property '{req_prop}' is missing")

        properties = schema.get("properties", {})
        for prop_name, prop_val in instance.items():
            if prop_name in properties:
                errors.extend(_validate_draft7_fallback(prop_val, properties[prop_name], f"{path}.{prop_name}"))
            elif schema.get("additionalProperties") is False:
                errors.append(f"{path}: additional property '{prop_name}' not allowed")

        # allOf conditional handling (e.g., if / then)
        all_of = schema.get("allOf", [])
        for sub in all_of:
            if "if" in sub and "then" in sub:
                if_errors = _validate_draft7_fallback(instance, sub["if"], path)
                if not if_errors:
                    errors.extend(_validate_draft7_fallback(instance, sub["then"], path))

    # Array constraints
    if isinstance(instance, list):
        item_schema = schema.get("items")
        if item_schema and isinstance(item_schema, dict):
            for idx, item in enumerate(instance):
                errors.extend(_validate_draft7_fallback(item, item_schema, f"{path}[{idx}]"))

    return errors


def validate_output_against_schema(
    output_dict: Dict[str, Any], schema_relative_path: Union[str, Path]
) -> EvaluationResult:
    """Validate a dictionary output against a specified LDrago brain JSON schema.

    Args:
        output_dict: Dictionary to validate.
        schema_relative_path: Relative path or filename of schema (e.g., 'answer.json', 'coding/schemas/plan.json').

    Returns:
        EvaluationResult with is_valid boolean flag and list of validation error strings.
    """
    if not isinstance(output_dict, dict):
        return EvaluationResult(
            is_valid=False,
            errors=[f"Expected dictionary output, got {type(output_dict).__name__}"],
        )

    try:
        resolved_path = resolve_schema_path(schema_relative_path)
    except FileNotFoundError as err:
        return EvaluationResult(is_valid=False, errors=[str(err)])

    with open(resolved_path, "r", encoding="utf-8") as f:
        schema = json.load(f)

    if _HAS_JSONSCHEMA:
        validator = Draft7Validator(schema)
        errors = [
            f"{err.json_path}: {err.message}" if err.json_path else err.message
            for err in validator.iter_errors(output_dict)
        ]
        return EvaluationResult(is_valid=len(errors) == 0, errors=errors)

    # Use custom Draft-07 validator fallback
    fallback_errors = _validate_draft7_fallback(output_dict, schema)
    return EvaluationResult(is_valid=len(fallback_errors) == 0, errors=fallback_errors)


def evaluate_schema_batch(outputs: List[Dict[str, Any]], schema_path: Union[str, Path]) -> float:
    """Evaluate compliance rate of a batch of outputs against a JSON schema.

    Args:
        outputs: List of dictionaries.
        schema_path: Path or filename of the target schema.

    Returns:
        Compliance rate float between 0.0 and 1.0 (1.0 if empty list).
    """
    if not outputs:
        return 1.0

    valid_count = sum(
        1 for output in outputs
        if validate_output_against_schema(output, schema_path).is_valid
    )
    return float(valid_count / len(outputs))
