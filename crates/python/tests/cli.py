"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations

from io import StringIO
from unittest.mock import patch


import mirascript.__main__ as cli


def run_main(args: list[str], *, stdin: str | None = None) -> tuple[int, str, str]:
    """运行 main 函数，返回 (exit_code, stdout, stderr)。"""

    stdout = StringIO()
    stderr = StringIO()

    @patch("sys.argv", ["mirascript", *args])
    @patch("sys.stdout", stdout)
    @patch("sys.stderr", stderr)
    @patch("sys.stdin", StringIO(stdin) if stdin is not None else StringIO())
    def run_main():
        return cli.main()

    exit_code = run_main()
    return exit_code, stdout.getvalue(), stderr.getvalue()
