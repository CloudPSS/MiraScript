from mirascript import VmContext, is_vm_context, VmError
import pytest


def test_is_vm_context():
    ctx = VmContext()
    assert is_vm_context(ctx)
    assert not is_vm_context(None)
    assert not is_vm_context({})
    assert not is_vm_context([])
    assert not is_vm_context(123)
    assert not is_vm_context("abc")


def test_vm_context():
    ctx = VmContext()
    assert isinstance(ctx, VmContext)
    assert len(ctx) == 0
    assert repr(ctx) == "VmContext({})"
    assert "sin" in ctx
    assert "nonexistent" not in ctx
    with pytest.raises(VmError, match="Global variable 'nonexistent' is not defined."):
        _ = ctx["nonexistent"]

    ctx = VmContext(no_defaults=True)
    assert len(ctx) == 0
    assert repr(ctx) == "VmContext({})"
    assert "sin" not in ctx
    assert "nonexistent" not in ctx
    with pytest.raises(VmError, match="Global variable 'nonexistent' is not defined."):
        _ = ctx["nonexistent"]


def test_vm_context_creation():
    ctx = VmContext(a=1, b=2, c="XX")
    assert "a" in ctx
    assert ctx["a"] == 1.0
    assert type(ctx["a"]) is float
    assert "b" in ctx
    assert ctx["b"] == 2.0
    assert type(ctx["b"]) is float
    assert "c" in ctx
    assert ctx["c"] == "XX"
    assert len(ctx) == 3
    assert repr(ctx) == "VmContext({'a': 1.0, 'b': 2.0, 'c': 'XX'})"

    with pytest.raises(TypeError, match="Invalid value for global variable 'a'"):
        VmContext(a=object())  # type: ignore


def test_vm_context_wrapper():
    d = {"x": 1, "y": 2, 1: 3}
    ctx = VmContext(d)
    assert "x" in ctx
    assert ctx["x"] == 1.0
    assert "y" in ctx
    assert ctx["y"] == 2.0
    assert 1 not in ctx
    assert len(ctx) == 3

    assert "d" not in ctx
    d["d"] = 4
    assert "d" in ctx
    assert ctx["d"] == 4.0
    assert type(ctx["d"]) is float
    assert len(ctx) == 4
