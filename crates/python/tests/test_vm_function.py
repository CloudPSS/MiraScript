from __future__ import annotations
import pytest
from typing_extensions import Callable
from mirascript import compile, vm_function


def _check(script: str, add: Callable[..., str]):
    compiled, _ = compile(script)
    assert compiled is not None
    assert compiled({"add": add}) == "OK"


def test_call_with_optional_args():
    @vm_function
    def add(a, b=2):
        assert isinstance(a, float) and a == 1
        assert isinstance(b, int) and b == 2
        return "OK"

    _check("add(1)", add)


def test_call_with_optional_args_set():
    @vm_function
    def add(a, b=2):
        assert isinstance(a, float) and a == 1
        assert isinstance(b, float) and b == 2
        return "OK"

    _check("add(1, 2)", add)


def test_call_with_less_args():
    @vm_function
    def add(a, b):
        assert isinstance(a, float) and a == 1
        assert b is None
        return "OK"

    _check("add(1)", add)


def test_call_with_more_args():
    @vm_function
    def add(a, b):
        assert isinstance(a, float) and a == 1
        assert isinstance(b, float) and b == 2
        return "OK"

    _check("add(1, 2, 3)", add)


def test_call_with_kwargs():
    @vm_function
    def add(*, a, b):
        assert isinstance(a, float) and a == 1
        assert isinstance(b, float) and b == 2
        return "OK"

    with pytest.raises(
        TypeError, match="missing 2 required keyword-only arguments: 'a' and 'b'"
    ):
        _check("add(1, 2)", add)


def test_call_with_kargs():
    @vm_function
    def add(a, *kargs):
        assert isinstance(a, float) and a == 1
        assert len(kargs) == 0
        return "OK"

    _check("add(1)", add)


def test_call_with_kargs_set():
    @vm_function
    def add(a, *kargs):
        assert isinstance(a, float) and a == 1
        assert isinstance(kargs[0], float) and kargs[0] == 2
        return "OK"

    _check("add(1, 2)", add)
