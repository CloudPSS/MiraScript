from mirascript.compiler.diagnostics import SourceMapEntry
from typing_extensions import Any, Union, List, Tuple, Optional, TypeAlias, Sequence
import ast
import sys

from ..vm.operations import ToString
from .opcode import OpCode, get_opcode_name
from .consts import (
    read_constants,
    split_chunk,
    read_index as _read_index,
    read_param as _read_param,
)
from .ast_helper import ASTHelper


def ast_call(func_id: str, call_args: List[Any]) -> ast.Call:
    """生成 call"""
    return ast.Call(
        func=ast.Name(id=func_id, ctx=ast.Load()), args=call_args, keywords=[], lineno=0
    )


def subscript(value: ast.expr, slice: ast.expr, ctx=ast.Load()) -> ast.Subscript:
    # Compatibility: ast.Index was removed in Python 3.9.
    # Use ast.Index for older Pythons, and plain expr (ast.Constant) for 3.9+.
    if sys.version_info < (3, 9):
        slice = ast.Index(value=slice)

    return ast.Subscript(
        value=value,
        slice=slice,
        ctx=ctx,
    )


def assign_call(target_id: str, func_id: str, call_args: List[Any]) -> ast.Assign:
    """生成 assign call"""
    return assign(target_id, ast_call(func_id, call_args))


def assign(target_id: str, value: ast.expr) -> ast.Assign:
    return ast.Assign(
        targets=[ast.Name(id=target_id, ctx=ast.Store())],
        value=value,
        lineno=0,
    )


def create_parameter(name):
    """生成参数"""
    if name == "None":
        return ast.Constant(value=None, lineno=0)
    return ast.Name(id=name, ctx=ast.Load(), lineno=0)


def create_if(name: str, negate) -> ast.If:
    return ast.If(
        test=ast.Compare(
            left=ast.Call(
                func=ast.Name(id="ToBoolean", ctx=ast.Load()),
                args=[ast.Name(id=name, ctx=ast.Load())],
                keywords=[],
            ),
            ops=[ast.NotEq()],
            comparators=[ast.Constant(value=negate)],
        ),
        body=[],
        orelse=[],
    )


def create_range_loop(
    helper: ASTHelper, index: str, start: str, end: str, exclusive=False
):
    start_name = f"start_{helper.lineno}"
    end_name = f"end_{helper.lineno}"
    s = helper.assign_call(start_name, "ToNumber", [helper.load_var(start)])
    e = helper.assign_call(end_name, "ToNumber", [helper.load_var(end)])
    i = helper.assign(index, helper.load_var(start_name))
    w = helper.set_position(
        ast.While(
            test=ast.Compare(
                left=helper.load_var(index),
                ops=[ast.LtE() if not exclusive else ast.Lt()],
                comparators=[helper.load_var(end_name)],
            ),
            body=[],
        )
    )
    return s, e, i, w


class Emitter:
    """代码生成"""

    def __init__(
        self,
        chunk: bytes,
        source_lines: "Sequence[str]",
        source_map: "Sequence[SourceMapEntry]",
    ):
        self.const_data, self.code_data = split_chunk(chunk)
        self.constants = read_constants(self.const_data)
        self.source_map = source_map
        self.source_map_index = 0
        self.source_lines = source_lines

        self.func_script = None
        self.code_offset = 0
        self.closure_counter = 0
        self.fun_name_counter = 0

    def unknown_opcode(self, opcode: int) -> None:
        if not opcode:
            return
        name = get_opcode_name(opcode)
        raise ValueError(
            f"Unknown opcode: {name} ({opcode}) at offset {self.code_offset - 1}"
        )

    def rv(self, i: int, level: int = 0) -> str:
        """Read variable"""
        if not i:
            return "None"
        c = self.closure_counter - level
        return f"var_{c}_{i}"

    def wv(self, i: int, level: int = 0) -> str:
        """Write variable"""
        if not i:
            return "_"
        return self.rv(i, level)

    def create_regs_array(self, helper: ASTHelper, nreg: int, start_index: int = 0):
        target = helper.tuple(
            (self.wv(i + start_index if i else 0, -1) for i in range(nreg + 1)),
            ctx=ast.Store(),
        )
        values = helper.tuple(["Uninitialized"] * (nreg + 1))
        return helper.assign(target, values)

    def create_loop(
        self, helper: ASTHelper, nreg, code, increment: Optional[ast.AugAssign] = None
    ):
        closure_name = f"closure_{self.source_map_index}"
        source = (
            self.source_lines[helper.lineno - 1]
            if helper.lineno - 1 < len(self.source_lines)
            else ""
        )
        block = helper.set_position(ast.If(test=helper.const(True), body=[]))
        closure = helper.set_position(
            ast.FunctionDef(
                name=closure_name,
                args=ast.arguments(),
                body=[
                    self.create_regs_array(helper, nreg - 1, 2),
                    block,
                    helper.ret("LoopContinue"),
                ],
                decorator_list=[helper.load_var("Closure")],
            )
        )
        if source:
            closure.body.insert(
                0, helper.expr(helper.const(f"{helper.lineno}: {source.strip()} ..."))
            )
        code.body.append(closure)

        result_name = f"{closure_name}_result"
        code.body.append(helper.assign_call(result_name, closure_name, []))
        code.body.append(increment) if increment else None
        code.body.append(
            helper.set_position(
                ast.If(
                    test=ast.Compare(
                        left=helper.load_var(result_name),
                        ops=[ast.Is()],
                        comparators=[helper.load_var("LoopContinue")],
                    ),
                    body=[ast.Continue()],
                    orelse=[
                        ast.If(
                            test=ast.Compare(
                                left=helper.load_var(result_name),
                                ops=[ast.Is()],
                                comparators=[helper.load_var("LoopBreak")],
                            ),
                            body=[ast.Break()],
                            orelse=[helper.ret(result_name)],
                        )
                    ],
                ),
                deep=True,
            )
        )

        return block

    def read_param(self, wide: bool) -> int:
        value, size = _read_param(self.code_data, self.code_offset, wide)
        self.code_offset += size
        return value

    def read_index(self, wide: bool) -> int:
        value, size = _read_index(self.code_data, self.code_offset, wide)
        self.code_offset += size
        return value

    def read_opcode(self, peek=False) -> "tuple[int, bool, ASTHelper]":
        opcode_raw = self.code_data[self.code_offset]
        opcode = opcode_raw & 0x7F
        wide = opcode_raw >= 0x80
        source_map_entry = (
            self.source_map[self.source_map_index]
            if self.source_map_index < len(self.source_map)
            else None
        )
        helper = (
            ASTHelper()
            if source_map_entry is None
            else ASTHelper(
                lineno=source_map_entry.start_line,
                col_offset=source_map_entry.start_column - 1,
                end_lineno=source_map_entry.end_line,
                end_col_offset=source_map_entry.end_column - 1,
            )
        )
        if not peek:
            self.code_offset += 1
            self.source_map_index += 1
        return opcode, wide, helper

    def skip_opcode(self) -> None:
        self.code_offset += 1
        self.source_map_index += 1

    def read_closure(self, current_blocks_body: List[ast.stmt]) -> None:
        """读取闭包"""
        self.closure_counter += 1

        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode(peek=True)

            if opcode != OpCode.FuncEnd:
                self.read(current_blocks_body)
                continue
            self.skip_opcode()
            self.closure_counter -= 1
            break

    def read_block_end(
        self, end_opcode: int, current_blocks_body: List[ast.stmt]
    ) -> None:
        """读取块结束"""
        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode(peek=True)

            if opcode != end_opcode:
                self.read(current_blocks_body)
                continue

            self.skip_opcode()
            if not current_blocks_body:
                current_blocks_body.append(ast.Pass(lineno=0))

            if end_opcode == OpCode.LoopEnd:
                self.closure_counter -= 1
            break

    def read_if_else(self, block: ast.If) -> None:
        """读取 if else 或 if 结束"""
        body = block.body
        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode(peek=True)

            if opcode == OpCode.IfEnd:
                return self.read_block_end(OpCode.IfEnd, body)
            elif opcode == OpCode.Else:
                self.skip_opcode()
                body = block.orelse
                break
            elif opcode == OpCode.ElIf:
                self.skip_opcode()
                raise ValueError("ElIf not supported in Python emitter")

            self.read(block.body)

        return self.read_block_end(OpCode.IfEnd, body)

    def read_record(self, obj, block: ast.Dict) -> None:
        """读取 record"""

        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode()

            def read():
                return self.read_param(wide)

            def add_Element(argsValue: list, fun_id="ElementOpt", key=None) -> None:
                block.keys.append(key)
                block.values.append(
                    ast.Call(
                        func=ast.Name(id=fun_id, ctx=ast.Load()),
                        args=[
                            (
                                ast.Name(id=a, ctx=ast.Load())
                                if not isinstance(a, ast.expr)
                                else a
                            )
                            for a in argsValue
                        ],
                        keywords=[],
                    )
                )

            if opcode in (OpCode.FieldOpt, OpCode.Field):
                field = read()
                field_name = self.constants[field]
                if field_name is None:
                    raise ValueError(f"Unknown field {field},{self.constants}")
                value = read()
                opt = opcode == OpCode.FieldOpt
                if opt:
                    add_Element([ast.Constant(value=f"{field_name}"), self.rv(value)])

                else:
                    add_Element(
                        [self.rv(value)],
                        fun_id="Element",
                        key=ast.Constant(value=field_name),
                    )
            elif opcode in (OpCode.FieldOptDyn, OpCode.FieldDyn):
                field = read()
                value = read()
                opt = opcode == OpCode.FieldOptDyn
                if opt:
                    add_Element([self.rv(field), self.rv(value)])
                else:
                    add_Element(
                        [self.rv(value)],
                        fun_id="Element",
                        key=ast.Name(id=self.rv(field), ctx=ast.Load()),
                    )
            elif opcode in (OpCode.FieldOptIndex, OpCode.FieldIndex):
                field = self.read_index(wide)
                value = read()
                opt = opcode == OpCode.FieldOptIndex
                if opt:
                    add_Element([ast.Constant(value=ToString(field)), self.rv(value)])
                else:
                    add_Element(
                        [self.rv(value)],
                        fun_id="Element",
                        key=ast.Constant(value=str(field)),
                    )
            elif opcode == OpCode.Spread:
                value = read()
                add_Element([self.rv(value)], "RecordSpread")
            elif opcode == OpCode.Freeze:
                return
            else:
                self.unknown_opcode(opcode)

    def read_array(self, arr: int, block: ast.List) -> None:
        """读取 array"""

        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode()

            def read():
                return self.read_param(wide)

            if opcode == OpCode.Item:
                value = read()
                block.elts.append(helper.vm_element(self.rv(value)))

            elif opcode == OpCode.ItemRange:
                start = self.read_index(wide)
                end = self.read_index(wide)
                block.elts.append(
                    helper.vm_element(
                        [helper.const(start), helper.const(end)],
                        helper_name="ArrayRange",
                        spread=True,
                    )
                )
            elif opcode == OpCode.ItemRangeDyn:
                start = read()
                end = read()
                block.elts.append(
                    helper.vm_element(
                        [self.rv(start), self.rv(end)],
                        helper_name="ArrayRange",
                        spread=True,
                    )
                )
            elif opcode == OpCode.ItemRangeExclusiveDyn:
                start = read()
                end = read()
                block.elts.append(
                    helper.vm_element(
                        [self.rv(start), self.rv(end)],
                        helper_name="ArrayRangeExclusive",
                        spread=True,
                    )
                )
            elif opcode == OpCode.Spread:
                value = read()
                # code = f"...$ArraySpread({self.rv(value)}),"
                block.elts.append(
                    helper.vm_element(
                        self.rv(value), helper_name="ArraySpread", spread=True
                    )
                )
            elif opcode == OpCode.Freeze:
                return
            else:
                self.unknown_opcode(opcode)

    def read(self, current_blocks_body: Union[List[ast.stmt], None] = None) -> None:
        """读取代码"""
        opcode, wide, helper = self.read_opcode()

        def read():
            return self.read_param(wide)

        def read_index():
            return self.read_index(wide)

        code = ast.Pass()
        reg = 0
        func_name = None
        loop_node = None
        # 处理各种操作码
        if opcode in (OpCode.FuncVarg, OpCode.Func):
            script = self.code_offset == 1
            reg = read()
            varg = opcode == OpCode.FuncVarg
            argn = read()
            regn = read()
            args = ast.arguments(
                posonlyargs=[],
                args=[],
                vararg=ast.arg(arg="args"),
                kwonlyargs=[],
                kw_defaults=[],
                kwarg=ast.arg(arg="kwargs"),
                defaults=[],
            )
            for i in range(argn):
                wv = self.wv(i + 1, -1)
                if varg and i == argn - 1:
                    # 最后一个参数为可变参数
                    args.vararg = ast.arg(arg="vargs")
                else:
                    args.args.append(ast.arg(f"{wv}"))
                    args.defaults.append(ast.Constant(value=None, lineno=0))

            regs = self.create_regs_array(helper, regn - argn, argn)

            if script:
                args.args.insert(0, ast.arg(arg="context"))
                args.defaults.insert(
                    0,
                    ast.Call(
                        func=ast.Name(id="GlobalFallback", ctx=ast.Load()),
                        args=[],
                        keywords=[],
                    ),
                )
                code = ast.FunctionDef(
                    "script", args=args, body=[regs], decorator_list=[], lineno=0
                )
                self.func_script = code
                code.decorator_list.append(helper.load_var("Script"))

            else:
                func_name = self.wv(reg)
                code = ast.FunctionDef(
                    func_name, args=args, body=[regs], decorator_list=[], lineno=0
                )
                source_line = (
                    self.source_lines[helper.lineno - 1]
                    if helper.lineno - 1 < len(self.source_lines)
                    else ""
                )
                fn_decl_name = (
                    source_line[helper.col_offset : helper.end_col_offset].strip()
                    if helper.col_offset is not None
                    and helper.end_col_offset is not None
                    and helper.col_offset < len(source_line)
                    and helper.end_col_offset <= len(source_line)
                    else ""
                )
                code.decorator_list.append(
                    helper.call("Fn", [helper.const(fn_decl_name)])
                )

            if varg:
                code.body.append(
                    helper.assign_call(self.wv(argn, -1), "Vargs", ["vargs"])
                )

        elif opcode == OpCode.Constant:
            reg = read()
            i = read()
            c = self.constants[i]
            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.Constant(value=c),
                lineno=0,
            )

        elif opcode == OpCode.Uninit:
            reg = read()
            # code = f"{self.wv(reg)} = undefined;"
            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.Name(id="Uninitialized", ctx=ast.Load()),
                lineno=0,
            )

        elif opcode == OpCode.Return:
            reg = read()
            # code = f"return {self.rv(reg)};"
            code = ast.Return(value=create_parameter(self.rv(reg)))

        elif opcode == OpCode.Format:
            reg = read()
            leftValue = read()
            fmtValue = read_index()

            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.Call(
                    func=ast.Name(id="Format", ctx=ast.Load()),
                    args=[
                        ast.Name(id=self.rv(leftValue), ctx=ast.Load()),
                        ast.Constant(value=self.constants[fmtValue]),
                    ],
                    keywords=[],
                ),
                lineno=0,
            )

        elif opcode in (
            OpCode.Add,
            OpCode.Sub,
            OpCode.Mul,
            OpCode.Div,
            OpCode.Mod,
            OpCode.Pow,
            OpCode.Gt,
            OpCode.Gte,
            OpCode.Lt,
            OpCode.Lte,
            OpCode.Eq,
            OpCode.Neq,
            OpCode.Aeq,
            OpCode.Naeq,
            OpCode.Same,
            OpCode.Nsame,
            OpCode.In,
            OpCode.And,
            OpCode.Or,
        ):
            reg = read()
            left = read()
            right = read()
            opcode_name = get_opcode_name(opcode)
            opArgs = []
            leftValue = self.rv(left)
            rightValue = self.rv(right)

            if leftValue == "None":
                opArgs.append(ast.Constant(value=None))
            else:
                opArgs.append(ast.Name(id=leftValue, ctx=ast.Load()))
            if rightValue == "None":
                opArgs.append(ast.Constant(value=None))
            else:
                opArgs.append(ast.Name(id=rightValue, ctx=ast.Load()))
            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.Call(
                    func=ast.Name(id=f"{opcode_name}", ctx=ast.Load()),
                    args=opArgs,
                    keywords=[],
                ),
                lineno=0,
            )

        elif opcode == OpCode.InGlobal:
            reg = read()
            left = read()
            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.Compare(
                    left=ast_call(
                        "ToString",
                        [ast.Name(id=self.rv(left), ctx=ast.Load())],
                    ),
                    ops=[ast.In()],
                    comparators=[ast.Name(id="context", ctx=ast.Load())],
                ),
                lineno=0,
            )

        elif opcode == OpCode.Concat:
            reg = read()
            n = read()
            args = [self.rv(read()) for _ in range(n)]
            opcode_name = get_opcode_name(opcode)
            code = assign_call(
                self.wv(reg),
                f"{opcode_name}",
                [ast.Name(id=a, ctx=ast.Load()) for a in args],
            )

        elif opcode in (OpCode.Omit, OpCode.Pick):
            reg = read()
            value = read()
            n = read()
            args = [self.constants[read()] for _ in range(n)]
            opcode_name = get_opcode_name(opcode)

            call_args = ast.List(
                elts=[ast.Constant(value=a) for a in args], ctx=ast.Load()
            )
            code = assign_call(
                self.wv(reg),
                f"{opcode_name}",
                [ast.Name(id=self.rv(value), ctx=ast.Load()), call_args],
            )

        #
        #     code = f"{self.wv(reg)} = ${opcode_name}({self.rv(value)}, [{', '.join(args)}]);"

        elif opcode in (OpCode.Call, OpCode.CallDyn):
            reg = read()
            func = read()
            n = read()
            args = [read() for _ in range(n)]
            ns = read()
            spreads = [read() for _ in range(ns)]

            if opcode == OpCode.Call:
                call_target = helper.subscript(
                    "context", helper.const(self.constants[func])
                )
            else:
                call_target = helper.load_var(self.rv(func))

            call_args = ast.Tuple(elts=[], ctx=ast.Load())
            for i, a in enumerate(args):
                if i in spreads:
                    call_args.elts.append(
                        helper.vm_element(
                            self.rv(a), helper_name="ArraySpread", spread=True
                        )
                    )
                else:
                    call_args.elts.append(helper.load_var(self.rv(a)))

            fun_args: list = [call_target]

            if len(call_args.elts) > 0:
                fun_args.append(helper.starred(call_args))
            code = helper.assign_call(self.wv(reg), "Call", fun_args)

        elif opcode == OpCode.Assign:
            reg = read()
            value = read()
            val = helper.load_var(self.rv(value))
            code = helper.assign(self.wv(reg), val)

        elif opcode in (
            OpCode.Pos,
            OpCode.Neg,
            OpCode.Not,
            OpCode.Type,
            OpCode.ToBoolean,
            OpCode.ToNumber,
            OpCode.ToString,
            OpCode.IsBoolean,
            OpCode.IsNumber,
            OpCode.IsString,
            OpCode.IsRecord,
            OpCode.IsArray,
            OpCode.Length,
        ):
            reg = read()
            value = read()
            opcode_name = get_opcode_name(opcode)
            code = helper.assign_call(self.wv(reg), opcode_name, [self.rv(value)])

        elif opcode in (OpCode.AssertInit, OpCode.AssertNonNil):
            reg = read()
            opcode_name = get_opcode_name(opcode)
            code = helper.expr(
                helper.call(opcode_name, [helper.load_var(self.rv(reg))])
            )

        # # 处理属性访问相关操作
        elif opcode == OpCode.Get:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = assign_call(
                self.wv(reg),
                "Get",
                [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=prop)],
            )

        elif opcode == OpCode.GetIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = assign_call(
                self.wv(reg),
                "Get",
                [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=index)],
            )
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {index});"

        elif opcode == OpCode.GetDyn:
            reg = read()
            obj = read()
            index = read()
            code = assign_call(
                self.wv(reg),
                "Get",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Name(id=self.rv(index), ctx=ast.Load()),
                ],
            )
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {self.rv(index)});"

        elif opcode == OpCode.Has:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = assign_call(
                self.wv(reg),
                "Has",
                [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=prop)],
            )
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {prop});"

        elif opcode == OpCode.HasIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = assign_call(
                self.wv(reg),
                "Has",
                [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=index)],
            )
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {index});"

        elif opcode == OpCode.HasDyn:
            reg = read()
            obj = read()
            index = read()
            code = assign_call(
                self.wv(reg),
                "Has",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Name(id=self.rv(index), ctx=ast.Load()),
                ],
            )
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {self.rv(index)});"

        elif opcode == OpCode.Set:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = assign_call(
                self.wv(reg),
                "Set",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Constant(value=prop),
                    ast.Name(id=self.rv(reg), ctx=ast.Load()),
                ],
            )
        #     code = f"$Set({self.rv(obj)}, {prop}, {self.rv(reg)});"

        elif opcode == OpCode.SetIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = assign_call(
                self.wv(reg),
                "Set",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Constant(value=index),
                    ast.Name(id=self.rv(reg), ctx=ast.Load()),
                ],
            )
        #     code = f"$Set({self.rv(obj)}, {index}, {self.rv(reg)});"

        elif opcode == OpCode.SetDyn:
            reg = read()
            obj = read()
            index = read()
            code = assign_call(
                self.wv(reg),
                "Set",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Name(id=self.rv(index), ctx=ast.Load()),
                    ast.Name(id=self.rv(reg), ctx=ast.Load()),
                ],
            )
        #     code = f"$Set({self.rv(obj)}, {self.rv(index)}, {self.rv(reg)});"

        # 处理全局变量访问
        elif opcode == OpCode.GetGlobal:
            reg = read()
            i = read()
            c = self.constants[i]
            code = assign(
                self.wv(reg),
                subscript(
                    ast.Name(id="context", ctx=ast.Load()),
                    ast.Constant(value=c),
                ),
            )

        elif opcode == OpCode.GetGlobalDyn:
            reg = read()
            name = read()
            slice = ast_call("ToString", [ast.Name(id=self.rv(name), ctx=ast.Load())])
            code = assign(
                self.wv(reg),
                subscript(
                    ast.Name(id="context", ctx=ast.Load()),
                    slice,
                ),
            )
        #     code = f"{self.wv(reg)} = global[{self.rv(name)}] ?? null;"

        # # 处理闭包变量
        elif opcode == OpCode.GetUpvalue:
            reg = read()
            level = read()
            up = read()
            code = assign_call(
                self.wv(reg),
                "Upvalue",
                [ast.Name(id=self.rv(up, level), ctx=ast.Load())],
            )

        elif opcode == OpCode.SetUpvalue:
            reg = read()
            level = read()
            up = read()
            if not current_blocks_body:
                raise ValueError("No current block to set upvalue")
            if not reg:
                val = ast.Constant(value=None)
            else:
                val = ast.Name(id=self.rv(reg), ctx=ast.Load())

            upvalue_name = self.rv(up, level)
            current_blocks_body.insert(0, ast.Nonlocal(names=[upvalue_name]))
            code = ast.Assign(
                targets=[ast.Name(id=upvalue_name, ctx=ast.Store())],
                value=val,
                lineno=0,
            )

        # # 处理数组切片
        elif opcode == OpCode.Slice:
            reg = read()
            obj = read()
            start = read_index()
            end = read_index()
            code = assign_call(
                self.wv(reg),
                "Slice",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Constant(value=start),
                    ast.Constant(value=end),
                ],
            )
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {start}, {end});"

        elif opcode == OpCode.SliceStart:
            reg = read()
            obj = read()
            end = read_index()
            code = assign_call(
                self.wv(reg),
                "Slice",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Constant(value=None),
                    ast.Constant(value=end),
                ],
            )
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, null, {end});"

        elif opcode == OpCode.SliceEnd:
            reg = read()
            obj = read()
            start = read_index()
            code = assign_call(
                self.wv(reg),
                "Slice",
                [
                    ast.Name(id=self.rv(obj), ctx=ast.Load()),
                    ast.Constant(value=start),
                    ast.Constant(value=None),
                ],
            )
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {start}, null);"

        elif opcode == OpCode.SliceDyn:
            reg = read()
            obj = read()
            start = read()
            end = read()
            code = assign_call(
                self.wv(reg),
                "Slice",
                [
                    create_parameter(self.rv(obj)),
                    create_parameter(self.rv(start)),
                    create_parameter(self.rv(end)),
                ],
            )
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {self.rv(start)}, {self.rv(end)});"

        elif opcode == OpCode.SliceExclusiveDyn:
            reg = read()
            obj = read()
            start = read()
            end = read()

            code = assign_call(
                self.wv(reg),
                "SliceExclusive",
                [
                    create_parameter(self.rv(obj)),
                    create_parameter(self.rv(start)),
                    create_parameter(self.rv(end)),
                ],
            )
        #     code = f"{self.wv(reg)} = $SliceExclusive({self.rv(obj)}, {self.rv(start)}, {self.rv(end)});"

        # # 处理数据结构初始化
        elif opcode == OpCode.Record:
            reg = read()
            # code = f"{self.wv(reg)} = ({{"
            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.Dict(keys=[], values=[]),
                lineno=0,
            )

        elif opcode == OpCode.Array:
            reg = read()
            code = ast.Assign(
                targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                value=ast.List(elts=[], ctx=ast.Load()),
                lineno=0,
            )
        #     code = f"{self.wv(reg)} = (["

        # # 处理条件语句
        elif opcode == OpCode.If:
            cond = read()
            code = create_if(self.rv(cond), False)
        #     code = f"if ($ToBoolean({self.rv(cond)})) {{"

        elif opcode == OpCode.IfNot:
            cond = read()
            code = create_if(self.rv(cond), True)
        #     code = f"if (!$ToBoolean({self.rv(cond)})) {{"

        elif opcode == OpCode.IfInit:
            cond = read()
            code = ast.If(
                test=ast.Compare(
                    left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                    ops=[ast.IsNot()],
                    comparators=[ast.Name(id="Uninitialized", ctx=ast.Load())],
                ),
                body=[],
                orelse=[],
            )

        #     code = f"if ({self.rv(cond)} !== undefined) {{"

        elif opcode == OpCode.IfNotInit:
            cond = read()
            code = ast.If(
                test=ast.Compare(
                    left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                    ops=[ast.Is()],
                    comparators=[ast.Name(id="Uninitialized", ctx=ast.Load())],
                ),
                body=[],
                orelse=[],
            )
        #     code = f"if ({self.rv(cond)} === undefined) {{"

        elif opcode == OpCode.IfNil:
            cond = read()
            code = ast.If(
                test=ast.Compare(
                    left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                    ops=[ast.Is()],
                    comparators=[ast.Constant(value=None)],
                ),
                body=[],
                orelse=[],
            )
        #     code = f"if ({self.rv(cond)} === null) {{"

        elif opcode == OpCode.IfNotNil:
            cond = read()
            code = ast.If(
                test=ast.Compare(
                    left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                    ops=[ast.IsNot()],
                    comparators=[ast.Constant(value=None)],
                ),
                body=[],
                orelse=[],
            )

        elif opcode == OpCode.LoopFor:
            nreg = read()
            iterable = read()
            code = helper.set_position(
                ast.For(
                    target=helper.store_var(self.wv(1, -1)),
                    iter=helper.call("Iterable", [self.rv(iterable)]),
                    body=[],
                )
            )

            loop_node = self.create_loop(helper, nreg, code)

        elif opcode in (OpCode.LoopRange, OpCode.LoopRangeExclusive):
            assert current_blocks_body is not None

            nreg = read()
            start = read()
            end = read()

            s, e, i, code = create_range_loop(
                helper,
                self.wv(1, -1),
                self.rv(start),
                self.rv(end),
                opcode == OpCode.LoopRangeExclusive,
            )
            current_blocks_body.append(s)
            current_blocks_body.append(e)
            current_blocks_body.append(i)
            loop_node = self.create_loop(
                helper,
                nreg,
                code,
                ast.AugAssign(
                    target=ast.Name(id=self.wv(1, -1), ctx=ast.Store()),
                    op=ast.Add(),
                    value=ast.Constant(value=1),
                ),
            )
        elif opcode == OpCode.Loop:
            nreg = read()

            code = helper.set_position(
                ast.While(
                    test=helper.const(True),
                    body=[],
                )
            )

            loop_node = self.create_loop(helper, nreg, code)

        elif opcode == OpCode.Break:
            # code = ast.Break()
            code = ast.Return(value=ast.Name(id="LoopBreak", ctx=ast.Load()))

        elif opcode == OpCode.Continue:
            #     code = "continue;"
            # code = ast.Continue()
            code = ast.Return(value=ast.Name(id="LoopContinue", ctx=ast.Load()))
        elif opcode == OpCode.Noop:
            pass
        else:
            self.unknown_opcode(opcode)

        if current_blocks_body is None:
            assert isinstance(code, ast.FunctionDef)
            current_blocks_body = code.body
        else:
            current_blocks_body.append(code)

        # 处理特殊的 opcode 后续逻辑
        if opcode in (OpCode.FuncVarg, OpCode.Func):
            assert isinstance(code, ast.FunctionDef)
            self.read_closure(code.body)
            pass
        elif opcode in (
            OpCode.If,
            OpCode.IfNot,
            OpCode.IfNil,
            OpCode.IfNotNil,
            OpCode.IfInit,
            OpCode.IfNotInit,
        ):
            assert isinstance(code, ast.If)
            self.read_if_else(code)
            if len(code.body) < 1:
                code.body.append(ast.Pass())

        elif opcode in (
            OpCode.Loop,
            OpCode.LoopFor,
            OpCode.LoopRange,
            OpCode.LoopRangeExclusive,
        ):
            self.closure_counter += 1
            assert loop_node is not None
            self.read_block_end(OpCode.LoopEnd, loop_node.body)
        elif opcode == OpCode.Record:
            assert isinstance(code, ast.Assign) and isinstance(code.value, ast.Dict)
            self.read_record(reg, code.value)
        elif opcode == OpCode.Array:
            assert isinstance(code, ast.Assign) and isinstance(code.value, ast.List)
            self.read_array(reg, code.value)
