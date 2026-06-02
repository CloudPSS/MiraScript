from pathlib import Path
import sys
import argparse
import traceback

from .vm.types.types import VmValue
from .compiler import compile, InputMode, VmScript, Diagnostic


def _compile(
    code: str, mode: InputMode, filename: "str | None" = None
) -> "tuple[VmScript | None, list[Diagnostic]]":
    try:
        script, diagnostics = compile(code, input_mode=mode, filename=filename)
        return script, diagnostics
    except Exception as e:
        traceback.print_exc(file=sys.stderr)
        print(f"Error during compilation: {e}", file=sys.stderr)
        sys.exit(2)


def _print_debug(script: VmScript, output_file: str, variables: dict):
    import ast

    unparse = getattr(ast, "unparse", None)
    if not unparse:
        try:
            import astunparse

            unparse = astunparse.unparse
        except ImportError:
            unparse = None
            raise ImportError(
                "Neither 'ast.unparse' nor 'astunparse' is available. Please install 'astunparse' with `pip install astunparse` to enable debug output generation."
            )

    with open(output_file, "w", encoding="utf-8") as f:
        f.write(
            f"{unparse(script.ast)}"
            "\n\n"
            "if __name__ == '__main__':\n"
            f"    result = script({variables})\n"
            "    print('[OK]', result)"
        )


def main(prog: "str | None" = "mirascript") -> int:
    parser = argparse.ArgumentParser(
        prog=prog, description="Compile and execute a MiraScript file"
    )
    if parser.prog == "__main__.py" and __name__ == "__main__":
        parser.prog = "python -m mirascript"
    parser.add_argument(
        "-t",
        "--template",
        action="store_true",
        help="Indicates the input is a template file",
    )
    parser.add_argument(
        "-g",
        "--generate",
        metavar="output.py",
        action="store",
        help="Output generated code to the specified file",
    )
    parser.add_argument(
        "-e",
        "--eval",
        action="store",
        metavar="SCRIPT",
        help="Evaluate a MiraScript code snippet directly from the command line",
    )
    parser.add_argument(
        "script_file",
        nargs="?",
        help="Path to the MiraScript file to compile (use '-' for stdin)",
    )
    parser.add_argument(
        "-v",
        "--variable",
        action="append",
        metavar="NAME=VALUE",
        help="Define a variable for evaluation (can be used multiple times)",
    )
    args = parser.parse_args()

    variables: "dict[str, VmValue]" = {}
    if args.variable:
        has_error = False
        for var in args.variable:
            if "=" not in var:
                print(
                    f"Error: Invalid variable definition '{var}'. Expected format NAME=VALUE.",
                    file=sys.stderr,
                )
                return 1
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
                    has_error = True
                    continue
                variables[name] = script()
            except Exception as e:
                print(
                    f"Error evaluating variable '{name}={value}': {e}", file=sys.stderr
                )
                has_error = True
        if has_error:
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

    try:
        print("[OK]", result(variables))
    except Exception as e:
        traceback.print_exc()
        return 1

    if result and args.generate:
        _print_debug(result, args.generate, variables)

    return 0


if __name__ == "__main__":
    sys.exit(main(None))
