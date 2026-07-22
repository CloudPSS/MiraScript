"""MiraScript 黑盒测试

遍历 tests/ 下所有 .mira 文件，注入辅助函数后在 VM 中执行。
每个 .mira 文件对应一个参数化测试用例。
"""

from __future__ import annotations
from concurrent.futures import ThreadPoolExecutor
from os import environ
from pathlib import Path

import pytest

from mirascript import VmContext, compile as mira_compile, config_checkpoint

# 并行 workers 数
MAX_WORKERS = 2

# 跳过大型文件
SKIP_HUGE = environ.get("SKIP_HUGE", "0") != "0"


def _run_mira_file(mira_path: Path, globals_: dict) -> None:
    """编译并执行单个 .mira 测试文件。"""
    code = mira_path.read_text(encoding="utf-8")

    script, _ = mira_compile(code, filename=mira_path)
    if script is None:
        raise AssertionError("Compilation failed, no script generated")

    ctx = VmContext(globals_)
    script(ctx)

    # 执行超时回调
    timeout_fns: list = globals_.pop("_timeout_fns", [])
    for fn, message in timeout_fns:
        with pytest.raises(RuntimeError, match="Execution timed out"):
            fn()


def test_mira_file(mira_file: Path, vm_helpers: dict) -> None:
    """执行单个 Mira 测试文件。"""
    is_huge = mira_file.stem.endswith("_huge")
    if is_huge and SKIP_HUGE:
        pytest.skip("Skipping huge test file")

    pool = ThreadPoolExecutor(max_workers=MAX_WORKERS) if MAX_WORKERS > 0 else None

    try:
        config_checkpoint(60 if is_huge else 1)
        if pool is not None:
            futures = [
                pool.submit(_run_mira_file, mira_file, dict(vm_helpers))
                for _ in range(MAX_WORKERS)
            ]
            _run_mira_file(mira_file, vm_helpers)
            for future in futures:
                future.result()
        else:
            _run_mira_file(mira_file, vm_helpers)
    finally:
        if pool is not None:
            pool.shutdown(wait=False)
        config_checkpoint()
