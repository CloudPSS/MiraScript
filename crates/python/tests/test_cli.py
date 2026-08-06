"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations

from .cli import run_main
from .conftest import EXAMPLES_DIR

# ---------------------------------------------------------------------------
# 无参数 / 帮助
# ---------------------------------------------------------------------------


def test_no_args_shows_help():
    """不带任何参数时应打印帮助并返回 1。"""
    exit_code, stdout, stderr = run_main([])
    assert exit_code == 1
    assert "usage:" in stdout or "usage:" in stderr


# ---------------------------------------------------------------------------
# 脚本文件执行
# ---------------------------------------------------------------------------


def test_run_mira_file():
    """执行一个 .mira 脚本文件。"""
    hello_file = EXAMPLES_DIR / "01_hello_world.mira"
    exit_code, stdout, _ = run_main([str(hello_file)])
    # hello_world.mira 没有 return 语句，结果是 nil
    assert stdout.endswith("[OK] nil\n")
    assert exit_code == 0


def test_run_mira_file_41_fib():
    """执行 fib 示例确认返回值。"""
    fib_file = EXAMPLES_DIR / "41_fib.mira"
    exit_code, _, _ = run_main([str(fib_file)])
    assert exit_code == 0


def test_nonexistent_file():
    """执行不存在的文件应报错。"""
    exit_code, _, stderr = run_main(["nonexistent_file.mira"])
    assert exit_code == 1
    assert "does not exist" in stderr or "Error" in stderr


# ---------------------------------------------------------------------------
# stdin 输入
# ---------------------------------------------------------------------------


def test_stdin_input():
    """通过 stdin 传入代码。"""
    exit_code, stdout, _ = run_main(["-"], stdin="return 123;")
    assert exit_code == 0
    assert stdout == "[OK] 123\n"


# ---------------------------------------------------------------------------
# 边界 / 错误场景
# ---------------------------------------------------------------------------


def test_eval_and_file_together():
    """同时使用 --eval 和文件参数应报错。"""
    exit_code, _, stderr = run_main(["-e", "return 1;", "some_file.mira"])
    assert exit_code == 1
    assert "cannot be used with" in stderr.lower() or "Error" in stderr


def test_eval_syntax_error():
    """语法错误的代码应返回非 0 退出码。"""
    exit_code, _, _ = run_main(["-e", "this is not valid code !!!"])
    assert exit_code == 1


def test_eval_with_no_return():
    """没有 return 语句的 eval（模板模式测试）。"""
    exit_code, _, _ = run_main(["-t", "-e", "just a string"])
    assert exit_code == 0
