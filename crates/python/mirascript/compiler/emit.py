import ast
import linecache

from .diagnostics import SourceMapEntry
from .script import VmScriptLike, wrap_vm_script, VmScript
from .emitter import Emitter
from .ast_helper import ASTHelper

_filename_counter = 1


def _filename(filename: "str | None") -> str:
    """获取文件名"""
    global _filename_counter
    if filename is None:
        filename = f"<mirascript_{_filename_counter}>"
        _filename_counter += 1
    return filename


def emit(
    chunk: bytes,
    *,
    filename: "str | None" = None,
    source: str = "",
    source_map: "list[SourceMapEntry] | None" = None,
) -> "VmScript | None":
    """生成代码"""
    module = None
    try:
        source_lines = source.splitlines(True)
        gen = Emitter(chunk, source_lines, source_map or [])
        gen.read()
        if gen.func_script is None:
            return None

        script = gen.func_script
        ast_helper = ASTHelper()
        ast_helper.set_position(script, deep=True)
        module = ast.Module(
            body=[
                ast.Expr(
                    value=ast.Constant(
                        value=f"\nGenerated from {filename or '<unknown>'}:\n\n```mira\n{source}\n```\n",
                        lineno=0,
                        col_offset=0,
                    ),
                    lineno=0,
                    col_offset=0,
                ),
                ast.ImportFrom(
                    module="mirascript.vm.env",
                    names=[
                        ast.alias(
                            name="*",
                            asname=None,
                            lineno=0,
                            col_offset=0,
                        )
                    ],
                    level=0,
                    lineno=0,
                    col_offset=0,
                ),
                script,
            ],
            type_ignores=[],
        )
        filename = _filename(filename)
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
        return wrap_vm_script(result, filename=filename, ast=module, source=source)

    except Exception as e:
        return wrap_vm_script(e, filename=filename, ast=module, source=source)
