from .mirascript import Config, compile as _compile
from .emit import emit
from .diagnostics import decode_diagnostics, Diagnostic

DefaultConfig = Config(input_mode="script")


def compile(script: str, config: Config = DefaultConfig):
    """
    编译 MiraScript 代码，生成 Python 函数

    Args:
        script (str): 要编译的 MiraScript 代码
        config (Config): 编译配置

    Returns:
        (Script | None): 编译后的 Python 函数
        list[Diagnostic]: 编译过程中产生的诊断信息
    """
    bytecode, diagnostics = _compile(script, config)
    decoded_diagnostics = decode_diagnostics(diagnostics)
    func = emit(bytecode) if bytecode else None
    return func, decoded_diagnostics
