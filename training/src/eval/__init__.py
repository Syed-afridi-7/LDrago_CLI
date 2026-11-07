"""Evaluation harness for LDrago CLI Phase 1.

Includes:
- Schema validation against LDrago brain JSON schemas (Draft-07)
- Safety and adversarial prompt detection
- Model response sensitive token leakage verification
"""

from .schema_eval import (
    EvaluationResult,
    evaluate_schema_batch,
    validate_output_against_schema,
)
from .safety_eval import (
    SafetyResult,
    evaluate_model_response_safety,
    evaluate_safety_prompt,
)

__all__ = [
    "EvaluationResult",
    "validate_output_against_schema",
    "evaluate_schema_batch",
    "SafetyResult",
    "evaluate_safety_prompt",
    "evaluate_model_response_safety",
]
