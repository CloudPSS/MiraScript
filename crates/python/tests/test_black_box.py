"""MiraScript 黑盒测试

遍历 tests/ 下所有 .mira 文件，注入辅助函数后在 VM 中执行。
每个 .mira 文件对应一个参数化测试用例。
"""

from __future__ import annotations
from typing_extensions import TYPE_CHECKING
from concurrent.futures import ThreadPoolExecutor
from os import environ
from pathlib import Path

import pytest

from mirascript import (
    compile as mira_compile,
    config_checkpoint,
    vm_function,
    VmError,
    VmModule,
    VmValue,
    VmFunction,
)
from .deepequals import assert_deep_equal, assert_not_deep_equal

# 并行 workers 数
MAX_WORKERS = 2

# 跳过大型文件
SKIP_HUGE = environ.get("SKIP_HUGE", "0") != "0"

if TYPE_CHECKING:
    from typing_extensions import Callable, TypeAlias

    TimeoutFns: TypeAlias = list[tuple[Callable, str]]
    VmTestHelpers: TypeAlias = tuple[TimeoutFns, dict[str, VmValue]]


def _make_vm_helpers() -> VmTestHelpers:
    """创建注入 Mira 脚本的全局辅助函数与变量。"""

    timeout_fns: TimeoutFns = []

    @vm_function
    def t_eq(a: VmValue, b: VmValue, message: str | None = None):
        assert_deep_equal(a, b, message=message)

    @vm_function
    def t_ne(a: VmValue, b: VmValue, message: str | None = None):
        assert_not_deep_equal(a, b, message=message)

    @vm_function
    def t_true(v: VmValue, message: str | None = None):
        assert v is True, message

    @vm_function
    def t_false(v: VmValue, message: str | None = None):
        assert v is False, message

    @vm_function
    def t_throws(fn: VmFunction, message: str | None = None):
        try:
            fn()
        except VmError:
            return
        msg = message or "Expected VmError but none was raised"
        raise AssertionError(msg)

    @vm_function
    def t_timeout(fn: VmFunction, message: str = "Execution timed out"):
        timeout_fns.append((fn, message))

    @vm_function
    def t_never(message: str = "This should never be called"):
        raise AssertionError(message)

    context = {
        "t_eq": t_eq,
        "t_ne": t_ne,
        "t_true": t_true,
        "t_false": t_false,
        "t_throws": t_throws,
        "t_timeout": t_timeout,
        "t_never": t_never,
        "v_array": [],
        "v_record": {},
        "v_nil": None,
        "v_true": True,
        "v_false": False,
        "v_number": 42,
        "v_string": "Hello, Mira!",
        "v_fn": vm_function(lambda: "I am a function"),
        "v_fn_another": vm_function(lambda: "I am another function"),
        "has_extern": False,
        "v_module": VmModule("v_module", {}),
        "v_module_another": VmModule("v_module_another", {}),
    }

    return timeout_fns, context


def _run_mira_file(
    mira_path: Path,
) -> TimeoutFns:
    """编译并执行单个 .mira 测试文件。"""
    code = mira_path.read_text(encoding="utf-8")

    script, _ = mira_compile(code, filename=mira_path)
    if script is None:
        raise AssertionError("Compilation failed, no script generated")

    timeout_fns, context = _make_vm_helpers()
    script(context)
    return timeout_fns


def _run_timeout_fns(timeout_fns: TimeoutFns) -> None:
    """执行超时回调函数，确保它们抛出 RuntimeError。"""
    for fn, message in timeout_fns:
        with pytest.raises(RuntimeError, match="Execution timed out"):
            fn()


def test_mira_file(mira_file: Path) -> None:
    """执行单个 Mira 测试文件。"""
    is_huge = mira_file.stem.endswith("_huge")
    if is_huge and SKIP_HUGE:
        pytest.skip("Skipping huge test file")

    pool = ThreadPoolExecutor(max_workers=MAX_WORKERS) if MAX_WORKERS > 0 else None

    try:
        timeout_fns = None
        config_checkpoint(120 if is_huge else 10)
        if pool is not None:
            futures = [
                pool.submit(_run_mira_file, mira_file) for _ in range(MAX_WORKERS)
            ]
            timeout_fns = _run_mira_file(mira_file)
            for future in futures:
                future.result()
        else:
            timeout_fns = _run_mira_file(mira_file)
        config_checkpoint()  # 重置检查点配置
        _run_timeout_fns(timeout_fns)
    finally:
        if pool is not None:
            pool.shutdown(wait=False)
