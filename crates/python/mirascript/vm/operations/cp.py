import time

MAX_DEPTH = 128
cp_depth = 0
cp = float("nan")
cp_timeout = 500  # 默认超时时间，单位毫秒


def Cp():
    """检查点"""
    pass
    global cp
    current_time = int(time.time() * 1000)
    if not cp or (isinstance(cp, float) and cp != cp):  # NaN 检查
        cp = current_time
    elif current_time - cp > cp_timeout:
        raise RuntimeError(
            f"Execution timed out, exceeded {cp_timeout} ms , last checkpoint at {cp} ms , current time {current_time} ms"
        )


def CpEnter():
    """进入检查点"""
    global cp_depth, cp
    cp_depth += 1
    if cp_depth <= 1:
        cp = int(time.time() * 1000)
        cp_depth = 1
    elif cp_depth > MAX_DEPTH:
        raise RuntimeError("Maximum call depth exceeded")
    else:
        Cp()


def CpExit():
    """退出检查点"""
    global cp_depth, cp
    cp_depth -= 1
    if cp_depth < 1:
        cp = float("nan")
        cp_depth = 0
    else:
        Cp()


def config_checkpoint(timeout=100):
    """设置检查点超时时间"""
    global cp_timeout
    if (
        not isinstance(timeout, (int, float))
        or timeout is None
        or timeout <= 0
        or (isinstance(timeout, float) and timeout != timeout)
    ):
        raise ValueError("Invalid timeout value")
    cp_timeout = timeout
