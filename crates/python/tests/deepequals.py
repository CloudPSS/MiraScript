"""
deepequals.py

Utilities for deep equality comparisons of Python objects with adjustable
semantics for floating-point NaN and signed zeros.

Features:
- deep_equal(a, b, *, nan_equal=True, signed_zero_distinct=True)
    Deeply compare dicts, lists, tuples, sets, frozensets and basic scalars.
    By default:
      - nan_equal=True: treat NaN == NaN
      - signed_zero_distinct=True: treat +0.0 != -0.0

- assert_deep_equal(a, b, **kwargs)
    Raise AssertionError with a small message when deep_equal returns False.

Notes:
- Comparison is structural. For dict keys, if keys are non-hashable or
  require special equivalence (e.g., NaN keys), keys are matched by structural
  equality rules.
- This implementation aims for clarity and correctness over extreme performance.
"""

from typing import Any
import math
from collections.abc import Mapping, Sequence, Set

def _is_nan(x: Any) -> bool:
    try:
        # math.isnan raises for non-floats, so guard safely
        return isinstance(x, float) and math.isnan(x)
    except Exception:
        return False

def _is_signed_zero(x: Any) -> bool:
    # Only floats have signed zero behavior in Python
    if not isinstance(x, float):
        return False
    # x == 0.0 will be True for both +0.0 and -0.0
    # copysign(1.0, x) returns -1.0 for negative zero
    return x == 0.0 and math.copysign(1.0, x) < 0

def deep_equal(a: Any, b: Any, *, nan_equal: bool = True, signed_zero_distinct: bool = True) -> bool:
    """
    Recursively compare a and b for deep equality with special float rules.

    Parameters:
    - a, b: objects to compare
    - nan_equal: if True, treat float('nan') == float('nan')
    - signed_zero_distinct: if True, treat +0.0 != -0.0 (only for floats)

    Returns:
    - True if considered equal under the given rules, else False.
    """
    # Identity fast-path
    
    if a is b:
        # But if both are floats and signed_zero_distinct is True we still must
        # ensure +0.0 vs -0.0 are distinguished even if identical object (rare).
        if signed_zero_distinct and isinstance(a, float) and isinstance(b, float):
            if a == 0.0 and b == 0.0:
                if _is_signed_zero(a) != _is_signed_zero(b):
                    return False
        return True
    # Handle NaN behavior for floats
    if nan_equal:
        if _is_nan(a) and _is_nan(b):
            return True
    
    # Both floats: handle signed zero and normal equality
    if isinstance(a, float) and isinstance(b, float):
        # If values equal (this includes +0.0 == -0.0)
        if a == b:
            if signed_zero_distinct and a == 0.0:
                # Distinguish +0.0 vs -0.0
                return _is_signed_zero(a) == _is_signed_zero(b)
            return True
        else:
            # Not equal (covers non-NaN differences)
            return False

    # Mapping (dict-like) — match keys and values structurally
    if isinstance(a, Mapping) and isinstance(b, Mapping):
        # Quick length check
        if len(a) != len(b):
            return False

        b_keys_unmatched = list(b.keys())

        for ka, va in a.items():
            # Try to find a matching key in b_keys_unmatched using deep equality
            match_index = None
            for i, kb in enumerate(b_keys_unmatched):
                if deep_equal(ka, kb, nan_equal=nan_equal, signed_zero_distinct=signed_zero_distinct):
                    match_index = i
                    break
            if match_index is None:
                return False
            kb = b_keys_unmatched.pop(match_index)
            vb = b[kb]
            if not deep_equal(va, vb, nan_equal=nan_equal, signed_zero_distinct=signed_zero_distinct):
                return False
        return True

    # Sequence (list/tuple) — ordered comparison; but strings are sequences too
    if isinstance(a, Sequence) and isinstance(b, Sequence) and not isinstance(a, (str, bytes, bytearray)) and not isinstance(b, (str, bytes, bytearray)):
        if len(a) != len(b):
            return False
        for xa, xb in zip(a, b):
            if not deep_equal(xa, xb, nan_equal=nan_equal, signed_zero_distinct=signed_zero_distinct):
                return False
        return True

    # Set-like (unordered) — attempt to match each element in a to one in b
    if isinstance(a, Set) and isinstance(b, Set):
        if len(a) != len(b):
            return False
        # Convert to lists to allow popping matched items
        b_items = list(b)
        for xa in a:
            matched = False
            for i, xb in enumerate(b_items):
                if deep_equal(xa, xb, nan_equal=nan_equal, signed_zero_distinct=signed_zero_distinct):
                    matched = True
                    b_items.pop(i)
                    break
            if not matched:
                return False
        return True
    # Fallback to regular equality
    try:
        return a == b
    except Exception:
        # If equality throws, fallback to identity (already checked) -> not equal
        return False

def assert_deep_equal(a: Any, b: Any, *, nan_equal: bool = True, signed_zero_distinct: bool = True, message: str = None) -> None:
    """
    Assert that deep_equal(a, b) is True. Raises AssertionError otherwise.
    The raised message includes a brief description of the mismatch.
    """
    if deep_equal(a, b, nan_equal=nan_equal, signed_zero_distinct=signed_zero_distinct):
        return
    base_msg = f"Objects are not equal (nan_equal={nan_equal}, signed_zero_distinct={signed_zero_distinct})"
    if message:
        raise AssertionError(f"{base_msg}: {message}")
    else:
        raise AssertionError(base_msg)
    
def assert_not_deep_equal(a: Any, b: Any, *, nan_equal: bool = True, signed_zero_distinct: bool = True, message: str = None) -> None:
    """
    Assert that deep_equal(a, b) is False. Raises AssertionError otherwise.
    """
    if not deep_equal(a, b, nan_equal=nan_equal, signed_zero_distinct=signed_zero_distinct):
        return
    base_msg = f"Objects are unexpectedly equal (nan_equal={nan_equal}, signed_zero_distinct={signed_zero_distinct})"
    if message:
        raise AssertionError(f"{base_msg}: {message}")
    else:
        raise AssertionError(base_msg)

if __name__ == "__main__":
    # Quick demonstration / smoke tests
    examples = [
        (float("nan"), float("nan"), True),
        (+0.0, -0.0, False),
        ([1, float("nan"), +0.0], [1, float("nan"), -0.0], False),
        ({"a": [1, 2], "b": float("nan")}, {"b": float("nan"), "a": [1, 2]}, True),
        ({"k1": +0.0}, {"k1": -0.0}, False),
        ({1, 2, float("nan")}, {2, float("nan"), 1}, True),
    ]

    for a, b, expected in examples:
        result = deep_equal(a, b)
        print(f"deep_equal({a!r}, {b!r}) -> {result}  (expected {expected})")