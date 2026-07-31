from __future__ import annotations
from typing_extensions import TYPE_CHECKING
import ast
import linecache


from .script import VmScriptLike, wrap_vm_script, VmScript
from .emitter import Emitter

if TYPE_CHECKING:
    from . import InputMode
    from .diagnostics import SourceMapEntry


_filename_counter = 0


def _filename(filename: str | None) -> str:
    """获取文件名"""
    if filename is None:
        global _filename_counter
        _filename_counter += 1
        filename = f"<mirascript_{_filename_counter}>"
    return filename


def emit(
    chunk: bytes,
    *,
    filename: str | None = None,
    source: str = "",
    source_map: list[SourceMapEntry] | None = None,
    input_mode: InputMode,
) -> VmScript | None:
    """生成代码"""
    module = None
    filename = _filename(filename)
    try:
        source_lines = source.splitlines(True)
        gen = Emitter(chunk, source_lines, source_map or [])
        gen.read()
        if gen.func_script is None:
            return None

        script = gen.func_script
        module = ast.Module(
            body=[
                ast.ImportFrom(
                    module="mirascript._vm.operations",
                    names=[ast.alias(name="*", asname=None, lineno=0, col_offset=0)],
                    level=0,
                    lineno=0,
                    col_offset=0,
                ),
                script,
            ],
            type_ignores=[],
        )
        code = compile(module, filename, "exec")
        exec_globals = {}
        exec(code, exec_globals)
        result: VmScriptLike = exec_globals.get("script", None)  # type: ignore
        linecache.cache[filename] = (
            len(source),
            None,
            source_lines,
            filename,
        )
        return wrap_vm_script(
            result, filename=filename, ast=module, source=source, input_mode=input_mode
        )

    except Exception as e:
        return wrap_vm_script(
            e, filename=filename, ast=module, source=source, input_mode=input_mode
        )
