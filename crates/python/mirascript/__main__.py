from __future__ import annotations
from pathlib import Path
import sys
import traceback

from ._main.unparse import unparse
from ._main.argparse import create_parser
from ._main.compile import compile_code
from ._main.variables import parse_variables
from . import VmValue, VmContext, display


def main() -> int:
    parser = create_parser()
    if (
        parser.prog == "__main__.py" or parser.prog.endswith(" -m mirascript")
    ) and __name__ == "__main__":
        parser.prog = "python -m mirascript"
    else:
        parser.prog = "mirascript"
    args = parser.parse_args()

    variables = parse_variables(args.variable)
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
    result, diagnostics = compile_code(script, mode, str(script_file))

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
