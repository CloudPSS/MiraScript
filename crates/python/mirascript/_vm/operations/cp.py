import math
import time
from threading import local

from ..._helpers.checker import is_number

MAX_DEPTH = 128
TIMEOUT = 500  # 默认超时时间，单位毫秒


class ThreadLocalData(local):
    cp_depth: int = 0
    cp: float = float("nan")


thread_local = ThreadLocalData()


def Cp():
    """检查点"""
    current_time = int(time.time() * 1000)
    if not thread_local.cp or (
        isinstance(thread_local.cp, float) and thread_local.cp != thread_local.cp
    ):  # NaN 检查
        thread_local.cp = current_time
    elif current_time - thread_local.cp > TIMEOUT:
        raise RuntimeError(
            f"Execution timed out, exceeded {TIMEOUT} ms , last checkpoint at {thread_local.cp} ms , current time {current_time} ms"
        )


def CpEnter():
    """进入检查点"""
    thread_local.cp_depth += 1
    if thread_local.cp_depth <= 1:
        thread_local.cp = int(time.time() * 1000)
        thread_local.cp_depth = 1
    elif thread_local.cp_depth > MAX_DEPTH:
        raise RuntimeError("Maximum call depth exceeded")
    else:
        Cp()


def CpExit():
    """退出检查点"""
    thread_local.cp_depth -= 1
    if thread_local.cp_depth < 1:
        thread_local.cp = float("nan")
        thread_local.cp_depth = 0
    else:
        Cp()


def config_checkpoint(timeout: "float | int" = 100):
    """设置检查点超时时间"""
    global TIMEOUT
    if not is_number(timeout) or timeout <= 0 or math.isnan(timeout):
        raise ValueError("Invalid timeout value")
    TIMEOUT = int(timeout) if math.isfinite(timeout) else 1000000000000000
