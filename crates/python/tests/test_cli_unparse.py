"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations

import ast
import sys
import types
from pathlib import Path

import pytest

import mirascript
import mirascript._main.unparse as unparse

if sys.version_info >= (3, 9):

    def test_get_unparse_prefers_stdlib():
        unparse_func = unparse._get_unparse()
        assert unparse_func is ast.unparse

    def test_get_unparse_raises_if_all_missing(monkeypatch: pytest.MonkeyPatch):
        monkeypatch.delattr(ast, "unparse", raising=False)
        monkeypatch.delitem(sys.modules, "astunparse", raising=False)

        with pytest.raises(ImportError):
            unparse._get_unparse()


def test_get_unparse_fallback_to_astunparse(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.delattr(ast, "unparse", raising=False)

    fake = types.ModuleType("astunparse")

    def fake_unparse(node):
        return "fake_unparse"

    fake.unparse = fake_unparse  # pyright: ignore[reportAttributeAccessIssue]
    monkeypatch.setitem(sys.modules, "astunparse", fake)

    unparse_func = unparse._get_unparse()
    assert unparse_func is fake_unparse


def test_unparse_writes_python_file(tmp_path: Path):
    script, diagnostics = mirascript.compile(
        "return 42;", input_mode="script", filename="<test>"
    )
    assert diagnostics is not None
    assert script is not None

    out = tmp_path / "debug_out.py"
    unparse.unparse(script, str(out), {"x": 1})

    content = out.read_text(encoding="utf-8")
    assert "if __name__ == '__main__'" in content
    assert "result = script({'x': 1})" in content
