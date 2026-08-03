from __future__ import annotations
from pathlib import Path
import sys
import argparse
import traceback

from ._main.unparse import unparse
from ._main.argparse import create_parser
from . import VmValue, compile, InputMode, VmScript, Diagnostic, VmContext, display


def _compile(
    code: str, mode: InputMode, filename: str | None = None
) -> tuple[VmScript | None, list[Diagnostic]]:
    try:
        script, diagnostics = compile(code, input_mode=mode, filename=filename)
        return script, diagnostics
    except Exception as e:
        traceback.print_exc(file=sys.stderr)
        print(f"Error during compilation: {e}", file=sys.stderr)
        sys.exit(2)


def _parse_variable_definition(var: str) -> tuple[str, VmValue] | None:
    if "=" not in var:
        print(
            f"Error: Invalid variable definition '{var}'. Expected format NAME=VALUE.",
            file=sys.stderr,
        )
        return None
    name, value = var.split("=", 1)
    try:
        script, diagnostics = _compile(
            f"return ({value});", "script", f"<variable:{name}>"
        )
        if script is None:
            print(
                f"Error: Failed to compile variable '{name}={value}'. Diagnostics:",
                *[diag for diag in diagnostics if diag.level == "Error"],
                file=sys.stderr,
            )
            return None
        return name, script()
    except Exception as e:
        print(f"Error evaluating variable '{name}={value}': {e}", file=sys.stderr)
        return None


def _parse_variables(variable_list: list[str]) -> dict[str, VmValue] | None:
    variables: dict[str, VmValue] = {}
    has_error = False
    for var in variable_list:
        result = _parse_variable_definition(var)
        if result is None:
            has_error = True
            continue
        name, value = result
        variables[name] = value
    if has_error:
        return None
    return variables


def main() -> int:
    parser = create_parser()
    if (
        parser.prog == "__main__.py" or parser.prog.endswith(" -m mirascript")
    ) and __name__ == "__main__":
        parser.prog = "python -m mirascript"
    else:
        parser.prog = "mirascript"
    args = parser.parse_args()

    variables = _parse_variables(args.variable) if args.variable else {}
    if variables is None:
        return 1

    if args.eval:
        script = args.eval
        mode = "template" if args.template else "script"
        script_file = "<eval>"
        if args.script_file is not None:
            print(
                "Error: --eval option cannot be used with a script file argument.",
                file=sys.stderr,
            )
            return 1
    elif args.script_file is None:
        parser.print_help()
        return 1
    elif args.script_file == "-":
        script = sys.stdin.read()
        mode = "template" if args.template else "script"
        script_file = "<stdin>"
    else:
        script_file = Path(args.script_file).resolve()
        if not script_file.is_file():
            print(f"Error: File '{script_file}' does not exist.", file=sys.stderr)
            return 1
        script = script_file.read_text(encoding="utf-8")
        mode = (
            "template" if args.template or script_file.suffix != ".mira" else "script"
        )

    # Compile and execute the script
    result, diagnostics = _compile(script, mode, str(script_file))

    for diag in diagnostics:
        print(diag, file=sys.stderr)

    if result is None:
        return 1

    if result and args.generate:
        unparse(result, args.generate, variables)

    try:
        print("[OK]", display(result(VmContext(variables))))
    except Exception:
        traceback.print_exc()
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
