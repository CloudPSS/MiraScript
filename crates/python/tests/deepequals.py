"""
deepequals.py

Utilities for deep equality comparisons of MiraScript values.
"""

from __future__ import annotations
from typing_extensions import Any, Mapping, Sequence, Set, Callable, TypeVar
from math import isnan, copysign

T = TypeVar("T")


def _compare(
    t: tuple[type[T], ...], a: Any, b: Any, comparator: Callable[[T, T], bool]
) -> bool | None:
    """
    Compare two values a and b of type t using the provided comparator function.
    Returns True if equal, False if not equal, or None if types do not match.
    """
    a_valid = isinstance(a, t)
    b_valid = isinstance(b, t)
    if a_valid and b_valid:
        return comparator(a, b)
    if a_valid or b_valid:
        return False  # One is of type t, the other is not
    return None  # Neither is of type t


def _number_equal(a: int | float, b: int | float) -> bool:
    """
    Compare two numbers a and b for equality with special rules:
    - NaN is equal to NaN
    - +0.0 is not equal to -0.0
    """
    a = float(a)
    b = float(b)
    if isnan(a) and isnan(b):
        return True
    if a != 0.0 or b != 0.0:
        return a == b
    # Both are zero, check signed zero
    return copysign(1.0, a) == copysign(1.0, b)


def _mapping_equal(a: Mapping, b: Mapping) -> bool:
    """
    Compare two mappings (dict-like) a and b for deep equality.
    Keys and values are compared recursively.
    """
    if len(a) != len(b):
        return False

    b_keys_unmatched = list(b.keys())

    for ka, va in a.items():
        # Try to find a matching key in b_keys_unmatched using deep equality
        match_index = None
        for i, kb in enumerate(b_keys_unmatched):
            if deep_equal(ka, kb):
                match_index = i
                break
        if match_index is None:
            return False
        kb = b_keys_unmatched.pop(match_index)
        vb = b[kb]
        if not deep_equal(va, vb):
            return False
    return True


def _sequence_equal(a: Sequence, b: Sequence) -> bool:
    """
    Compare two sequences (list/tuple) a and b for deep equality.
    Order matters, and elements are compared recursively.
    """
    if len(a) != len(b):
        return False
    for xa, xb in zip(a, b):
        if not deep_equal(xa, xb):
            return False
    return True


def _set_equal(a: Set, b: Set) -> bool:
    """
    Compare two sets a and b for deep equality.
    Order does not matter, but each element must match one in the other set.
    """
    if len(a) != len(b):
        return False
    # Convert to lists to allow popping matched items
    b_items = list(b)
    for xa in a:
        matched = False
        for i, xb in enumerate(b_items):
            if deep_equal(xa, xb):
                matched = True
                b_items.pop(i)
                break
        if not matched:
            return False
    return True


def deep_equal(a: Any, b: Any) -> bool:
    """
    Recursively compare a and b for deep equality with special float rules.

    Parameters:
    - a, b: objects to compare

    Returns:
    - True if considered equal under the given rules, else False.
    """
    # Boolean fast-path: True/False are distinct from 1/0
    if isinstance(a, bool) or isinstance(b, bool):
        return a is b

    # Numeric fast-path: if both are numbers (int/float), compare with special rules
    number_equal = _compare((int, float), a, b, _number_equal)
    if number_equal is not None:
        return number_equal

    # Identity fast-path
    if a is b:
        return True

    # Mapping (dict-like) — match keys and values structurally
    mapping_equal = _compare((Mapping,), a, b, _mapping_equal)
    if mapping_equal is not None:
        return mapping_equal

    # Special sequences
    str_like_equal = _compare((str, bytes, bytearray), a, b, lambda x, y: x == y)
    if str_like_equal is not None:
        return str_like_equal

    # Sequence (list/tuple) — ordered comparison
    sequence_equal = _compare((Sequence,), a, b, _sequence_equal)
    if sequence_equal is not None:
        return sequence_equal

    # Set-like (unordered) — attempt to match each element in a to one in b
    set_equal = _compare((Set,), a, b, _set_equal)
    if set_equal is not None:
        return set_equal

    # Fallback: use standard equality
    return a == b


def assert_deep_equal(a: Any, b: Any, *, message: str | None = None) -> None:
    """
    Assert that deep_equal(a, b) is True. Raises AssertionError otherwise.
    The raised message includes a brief description of the mismatch.
    """
    if deep_equal(a, b):
        return
    base_msg = "Objects are not equal"
    if message:
        raise AssertionError(f"{base_msg}: {message}")
    else:
        raise AssertionError(base_msg)


def assert_not_deep_equal(a: Any, b: Any, *, message: str | None = None) -> None:
    """
    Assert that deep_equal(a, b) is False. Raises AssertionError otherwise.
    """
    if not deep_equal(a, b):
        return
    base_msg = "Objects are unexpectedly equal"
    if message:
        raise AssertionError(f"{base_msg}: {message}")
    else:
        raise AssertionError(base_msg)
