"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations

from pathlib import Path

import mirascript
import mirascript._main.unparse as unparse


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
