import ast
import linecache
from typing_extensions import TYPE_CHECKING


from .script import VmScriptLike, wrap_vm_script, VmScript
from .emitter import Emitter
from .ast_helper import ASTHelper

if TYPE_CHECKING:
    from . import InputMode
    from .diagnostics import SourceMapEntry


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
    input_mode: "InputMode",
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
        fence = "`" * 5
        ext = "miratpl" if input_mode == "template" else "mira"
        filename = _filename(filename)
        hint = (
            "\nGenerated from "
            + filename.replace("\\", "/")
            + ":\n\n"
            + fence
            + ext
            + "\n"
            + source.rstrip("\r\n")
            + "\n"
            + fence
            + "\n"
        )
        module = ast.Module(
            body=[
                ast_helper.vm_hint(hint),
                ast_helper.set_position(
                    ast.ImportFrom(
                        module="mirascript.vm.operations",
                        names=[
                            ast.alias(name="*", asname=None, lineno=0, col_offset=0)
                        ],
                        level=0,
                    )
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
        return wrap_vm_script(result, filename=filename, ast=module, source=source)

    except Exception as e:
        return wrap_vm_script(e, filename=filename, ast=module, source=source)
