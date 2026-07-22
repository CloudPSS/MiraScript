from __future__ import annotations
from time import time
from math import isfinite, isnan
from threading import local
from typing_extensions import Optional

from ..._helpers.checker import is_number

__all__ = [
    "Cp",
    "CpEnter",
    "CpExit",
    "config_checkpoint",
]

TIME_ORIGIN: float = time()


def _timestamp() -> float:
    """返回自程序启动以来的时间戳，单位为秒"""
    return time() - TIME_ORIGIN


CP_DEFAULT_INTERVAL: int = 100
MAX_DEPTH: int = 128
CP_DEFAULT_TIMEOUT: float = 0.5  # 默认超时时间，单位秒

cp_timeout: float = CP_DEFAULT_TIMEOUT
cp_interval: int = CP_DEFAULT_INTERVAL


class _ThreadLocalData(local):
    cp_depth: int = 0
    cp_counter: int = 0
    cp: float = float("nan")


thread_local = _ThreadLocalData()


def Cp() -> None:
    """检查点"""
    thread_local.cp_counter += 1
    if thread_local.cp_counter % cp_interval != 0:
        return
    thread_local.cp_counter = 0
    if isnan(thread_local.cp):
        thread_local.cp = _timestamp()
    elif _timestamp() - thread_local.cp > cp_timeout:
        raise RuntimeError("Execution timed out")


def CpEnter() -> None:
    """进入检查点"""
    thread_local.cp_depth += 1
    if thread_local.cp_depth <= 1:
        thread_local.cp = float("nan")
        thread_local.cp_counter = 0
        thread_local.cp_depth = 1
    elif thread_local.cp_depth > MAX_DEPTH:
        raise RuntimeError("Maximum call depth exceeded")
    else:
        Cp()


def CpExit() -> None:
    """退出检查点"""
    thread_local.cp_depth -= 1
    if thread_local.cp_depth < 1:
        thread_local.cp = float("nan")
        thread_local.cp_counter = 0
        thread_local.cp_depth = 0
    else:
        Cp()


def config_checkpoint(
    timeout: float | int = CP_DEFAULT_TIMEOUT,
    check_interval: int = CP_DEFAULT_INTERVAL,
) -> None:
    """
    设置检查点超时时间和检查间隔

    :param timeout: 超时时间，单位秒
    :param check_interval: 检查间隔，单位为检查点调用次数
    """
    global cp_timeout, cp_interval

    if not is_number(timeout) or timeout <= 0 or not isfinite(timeout):
        raise ValueError("Invalid timeout value")
    cp_timeout = float(timeout)

    if not isinstance(check_interval, int) or check_interval <= 0:
        raise ValueError("Invalid check interval value")
    cp_interval = int(check_interval)
