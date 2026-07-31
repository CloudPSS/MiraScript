from mirascript import VmContext, is_vm_context
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
    assert len(ctx) != 0
    ctx = VmContext(no_defaults=True)
    assert len(ctx) == 0
    assert repr(ctx) == "VmContext({})"


def test_vm_context_creation():
    with pytest.raises(TypeError, match="Global variable name must be a string"):
        VmContext(values={"a": 1, 2: "b"})  # type: ignore
    with pytest.raises(TypeError, match="Invalid value for global variable 'a'"):
        VmContext(values={"a": object()})  # type: ignore
