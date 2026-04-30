from typing_extensions import Literal, TypeAlias

from ..mirascript import Config as _CompileConfig, compile as _compile
from .emit import emit
from .script import VmScript, VmScriptLike
from .diagnostics import Diagnostic, decode_diagnostics

__all__ = ["Diagnostic", "compile", "VmScript", "VmScriptLike"]

InputMode: TypeAlias = Literal["script", "template"]


def compile(
    script: str,
    *,
    input_mode: InputMode = "script",
    filename: "str | None" = None,
) -> "tuple[VmScript | None, list[Diagnostic]]":
    """
    编译 MiraScript 代码，生成 Python 函数

    Args:
        script (str): 要编译的 MiraScript 代码
        input_mode (InputMode, optional): 输入模式，支持 'script' 和 'template'，默认为 'script'
        filename (str | None, optional): 代码来源的文件名，用于诊断信息，默认为 None

    Returns:
        (VmScript | None): 编译后的 Python 函数
        list[Diagnostic]: 编译过程中产生的诊断信息
    """
    config = _CompileConfig(input_mode=input_mode)
    bytecode, diagnostics = _compile(script, config)
    decoded_diagnostics, source_map = decode_diagnostics(diagnostics)
    func = (
        emit(bytecode, filename=filename, source=script, source_map=source_map)
        if bytecode
        else None
    )
    return func, decoded_diagnostics
