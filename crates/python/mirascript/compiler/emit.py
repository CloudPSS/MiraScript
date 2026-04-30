import ast

from .script import VmScriptLike, wrap_vm_script, VmScript
from .deep_nonlocal_fix import deep_nonlocal_fix
from .emitter import Emitter


def _set_ast_positions(node: ast.AST, lineno=1, col_offset=0):
    """为AST节点设置行号和列偏移量"""
    for field, value in ast.iter_fields(node):
        if isinstance(value, list):
            for item in value:
                if isinstance(item, ast.AST):
                    _set_ast_positions(item, lineno, col_offset)
        elif isinstance(value, ast.AST):
            _set_ast_positions(value, lineno, col_offset)

    # 设置当前节点的位置信息
    if not hasattr(node, "lineno"):
        setattr(node, "lineno", lineno)
    if not hasattr(node, "col_offset"):
        setattr(node, "col_offset", col_offset)


def emit(
    chunk: bytes, *, filename: "str | None" = None, source: str = ""
) -> "VmScript | None":
    """生成代码"""
    module = None
    try:
        gen = Emitter(chunk)
        gen.read()
        if gen.func_script is None:
            return None

        script = deep_nonlocal_fix(gen.func_script)
        _set_ast_positions(script)
        module = ast.Module(
            body=[
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
        code = compile(module, filename or "<mira_script>", "exec")
        exec_globals = {}
        exec(code, exec_globals)
        result: VmScriptLike = exec_globals.get("script", None)  # type: ignore
        return wrap_vm_script(result, filename=filename, ast=module, source=source)

    except Exception as e:
        return wrap_vm_script(e, filename=filename, ast=module, source=source)
