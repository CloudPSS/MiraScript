from typing_extensions import Union, List, Optional, Sequence
import ast

from .._vm.operations import ToString
from .opcode import OpCode, get_opcode_name
from .consts import (
    read_constants,
    split_chunk,
    read_index as _read_index,
    read_param as _read_param,
)
from .ast_helper import ASTHelper
from .diagnostics import SourceMapEntry


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
        self.closures: List[ast.FunctionDef] = []

    @property
    def closure_counter(self) -> int:
        return len(self.closures)

    @property
    def current_closure(self) -> Optional[ast.FunctionDef]:
        return self.closures[-1] if self.closures else None

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
            (self.wv(i + start_index if i else 0, -1) for i in range(nreg + 1)), "Store"
        )
        values = helper.binary(
            helper.tuple([helper.uninitialized()]), "Mult", helper.const_int(nreg + 1)
        )
        return helper.assign(target, values)

    def create_loop(
        self, helper: ASTHelper, nreg, code, increment: Optional[ast.AugAssign] = None
    ):
        closure_name = f"closure_{self.source_map_index}"
        block = helper.if_expr(test=helper.const(True))
        closure = helper.func_def(
            closure_name,
            helper.args(kwarg=None, vararg=None),
            [
                self.create_regs_array(helper, nreg - 1, 2),
                block,
                helper.ret("LoopContinue"),
            ],
            ["Closure"],
        )

        hint = helper.vm_hint()
        if hint:
            closure.body.insert(0, hint)
        code.body.append(closure)

        result_name = f"{closure_name}_result"
        code.body.append(helper.assign_call(result_name, closure_name, []))
        code.body.append(increment) if increment else None
        code.body.append(helper.vm_loop_control(result_name))
        return block, closure

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
                source_lines=self.source_lines,
            )
        )
        if not peek:
            self.code_offset += 1
            self.source_map_index += 1
        return opcode, wide, helper

    def skip_opcode(self) -> None:
        self.code_offset += 1
        self.source_map_index += 1

    def mark_nonlocal(self, name: str) -> None:
        """标记非局部变量"""
        current_closure = self.current_closure
        assert current_closure is not None, "No closure to mark nonlocal variable"
        body = current_closure.body
        if body and isinstance(body[0], ast.Nonlocal):
            nc = body[0].names
        else:
            nc = []
            body.insert(0, ast.Nonlocal(names=nc, lineno=0, col_offset=0))
        if name not in nc:
            nc.append(name)

    def read_closure(self, closure: ast.FunctionDef) -> None:
        """读取闭包"""

        self.closures.append(closure)  # 进入闭包
        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode(peek=True)

            if opcode != OpCode.FuncEnd:
                self.read(closure.body)
                continue
            self.skip_opcode()
            self.closures.pop()  # 退出当前闭包
            break

    def read_block_end(
        self,
        end_opcode: int,
        current_blocks_body: List[ast.stmt],
        closure: Optional[ast.FunctionDef] = None,
    ) -> None:
        """读取块结束"""
        if closure is not None:
            self.closures.append(closure)  # 进入闭包
        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode(peek=True)

            if opcode != end_opcode:
                self.read(current_blocks_body)
                continue

            self.skip_opcode()
            if not current_blocks_body:
                current_blocks_body.append(helper.pass_stmt())

            if closure is not None:
                self.closures.pop()  # 退出当前闭包
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

            def el_opt(name: "str | ast.expr", value: "str | ast.expr"):
                block.keys.append(None)
                block.values.append(helper.call("ElementOpt", [name, value]))

            def el(name: "str | ast.expr", value: "str | ast.expr"):
                block.keys.append(helper.load(name))
                block.values.append(helper.call("Element", [value]))

            if opcode in (OpCode.FieldOpt, OpCode.Field):
                field = read()
                field_name = self.constants[field]
                if field_name is None:
                    raise ValueError(f"Unknown field {field},{self.constants}")
                value = read()
                opt = opcode == OpCode.FieldOpt
                if opt:
                    el_opt(helper.const(field_name), self.rv(value))

                else:
                    el(helper.const(field_name), self.rv(value))
            elif opcode in (OpCode.FieldOptDyn, OpCode.FieldDyn):
                field = read()
                value = read()
                opt = opcode == OpCode.FieldOptDyn
                if opt:
                    el_opt(self.rv(field), self.rv(value))
                else:
                    el(self.rv(field), self.rv(value))
            elif opcode in (OpCode.FieldOptIndex, OpCode.FieldIndex):
                field = self.read_index(wide)
                value = read()
                opt = opcode == OpCode.FieldOptIndex
                if opt:
                    el_opt(helper.const(ToString(field)), self.rv(value))
                else:
                    el(helper.const(ToString(field)), self.rv(value))
            elif opcode == OpCode.Spread:
                value = read()
                block.keys.append(None)
                block.values.append(helper.call("RecordSpread", [self.rv(value)]))
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
                block.elts.append(
                    helper.vm_element(
                        self.rv(value), helper_name="ArraySpread", spread=True
                    )
                )
            elif opcode == OpCode.Freeze:
                return
            else:
                self.unknown_opcode(opcode)

    def read_module(self, block: ast.ClassDef) -> None:
        """读取 module"""

        while self.code_offset < len(self.code_data):
            opcode, wide, helper = self.read_opcode()

            if opcode == OpCode.Field:
                field = self.read_index(wide)
                field_name = self.constants[field]
                if not isinstance(field_name, str):
                    raise ValueError(f"Unknown field {field},{self.constants}")
                value = self.read_param(wide)
                args = helper.args(kwarg=None, vararg=None)
                getter = helper.func_def(
                    f"pub_{self.code_offset}",
                    args,
                    [helper.ret(self.rv(value))],
                    [helper.call("Pub", [helper.const(field_name)])],
                )
                block.body.append(getter)
            elif opcode == OpCode.Freeze:
                if block.body == []:
                    block.body.append(helper.pass_stmt())
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

        code = None
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
            args = helper.args(vararg="vargs")
            for i in range(argn):
                wv = self.wv(i + 1, -1)
                if varg and i == argn - 1:
                    # 最后一个参数为可变参数，已在 helper.args() 初始化为 vargs
                    pass
                else:
                    args.args.append(helper.arg(wv))
                    args.defaults.append(helper.const(None))

            regs = self.create_regs_array(helper, regn - argn, argn)

            if script:
                args.args.insert(0, helper.arg("context"))
                args.defaults.insert(0, helper.const(None))
                code = helper.func_def("script", args, [regs])
                self.func_script = code
                code.decorator_list.append(helper.load("Script"))
                code.body.append(helper.assign_call("context", "Context", ["context"]))

            else:
                func_name = self.wv(reg)
                code = helper.func_def(func_name, args, [regs])
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
            code = helper.assign(self.wv(reg), helper.const(c))

        elif opcode == OpCode.Uninit:
            reg = read()
            code = helper.assign(self.wv(reg), helper.uninitialized())

        elif opcode == OpCode.Return:
            reg = read()
            code = helper.ret(self.rv(reg))

        elif opcode == OpCode.Format:
            reg = read()
            leftValue = read()
            fmtValue = read_index()

            code = helper.assign_call(
                self.wv(reg),
                "Format",
                [self.rv(leftValue), helper.const(self.constants[fmtValue])],
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
            code = helper.assign_call(
                self.wv(reg), opcode_name, [self.rv(left), self.rv(right)]
            )

        elif opcode == OpCode.InGlobal:
            reg = read()
            left = read()
            code = helper.assign(
                self.wv(reg),
                helper.compare(
                    helper.call("ToString", [self.rv(left)]),
                    "In",
                    helper.load("context"),
                ),
            )

        elif opcode == OpCode.Concat:
            reg = read()
            n = read()
            parts = [self.rv(read()) for _ in range(n)]
            code = helper.assign_call(self.wv(reg), "Concat", parts)

        elif opcode in (OpCode.Omit, OpCode.Pick):
            reg = read()
            value = read()
            n = read()
            args = [self.constants[read()] for _ in range(n)]
            opcode_name = get_opcode_name(opcode)

            call_args = helper.list(helper.const(value=a) for a in args)
            code = helper.assign_call(
                self.wv(reg),
                opcode_name,
                [self.rv(value), call_args],
            )

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
                call_target = helper.load(self.rv(func))

            fun_args = [
                call_target,
                *(
                    (
                        helper.vm_element(
                            self.rv(a), helper_name="ArraySpread", spread=True
                        )
                        if i in spreads
                        else self.rv(a)
                    )
                    for i, a in enumerate(args)
                ),
            ]

            code = helper.assign_call(self.wv(reg), "Call", fun_args)

        elif opcode == OpCode.Assign:
            reg = read()
            value = read()
            val = helper.load(self.rv(value))
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
            code = helper.expr(helper.call(opcode_name, [self.rv(reg)]))

        # 处理属性访问相关操作
        elif opcode == OpCode.Get:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = helper.assign_call(
                self.wv(reg),
                "Get",
                [self.rv(obj), helper.const(prop)],
            )

        elif opcode == OpCode.GetIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = helper.assign_call(
                self.wv(reg),
                "Get",
                [self.rv(obj), helper.const(index)],
            )

        elif opcode == OpCode.GetDyn:
            reg = read()
            obj = read()
            index = read()
            code = helper.assign_call(
                self.wv(reg),
                "Get",
                [self.rv(obj), self.rv(index)],
            )

        elif opcode == OpCode.Has:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = helper.assign_call(
                self.wv(reg),
                "Has",
                [self.rv(obj), helper.const(prop)],
            )

        elif opcode == OpCode.HasIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = helper.assign_call(
                self.wv(reg),
                "Has",
                [self.rv(obj), helper.const(index)],
            )

        elif opcode == OpCode.HasDyn:
            reg = read()
            obj = read()
            index = read()
            code = helper.assign_call(
                self.wv(reg),
                "Has",
                [self.rv(obj), self.rv(index)],
            )

        elif opcode == OpCode.Set:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = helper.expr(
                helper.call(
                    "Set",
                    [self.rv(obj), helper.const(prop), self.rv(reg)],
                )
            )

        elif opcode == OpCode.SetIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = helper.expr(
                helper.call(
                    "Set",
                    [self.rv(obj), helper.const(index), self.rv(reg)],
                )
            )

        elif opcode == OpCode.SetDyn:
            reg = read()
            obj = read()
            index = read()
            code = helper.expr(
                helper.call(
                    "Set",
                    [self.rv(obj), self.rv(index), self.rv(reg)],
                )
            )

        # 处理全局变量访问
        elif opcode == OpCode.GetGlobal:
            reg = read()
            i = read()
            c = helper.const(self.constants[i])
            code = helper.assign(self.wv(reg), helper.subscript("context", c))

        elif opcode == OpCode.GetGlobalDyn:
            reg = read()
            name = read()
            slice = helper.call("ToString", [self.rv(name)])
            code = helper.assign(self.wv(reg), helper.subscript("context", slice))

        # 处理闭包变量
        elif opcode == OpCode.GetUpvalue:
            reg = read()
            level = read()
            up = read()

            local = self.wv(reg)
            upvalue_name = self.rv(up, level)
            self.mark_nonlocal(upvalue_name)
            code = helper.assign_call(self.wv(reg), "Upvalue", [upvalue_name])

        elif opcode == OpCode.SetUpvalue:
            reg = read()
            level = read()
            up = read()

            local = self.rv(reg)
            upvalue_name = self.wv(up, level)
            self.mark_nonlocal(upvalue_name)
            code = helper.assign(upvalue_name, helper.load(local))

        # 处理数组切片
        elif opcode == OpCode.Slice:
            reg = read()
            obj = read()
            start = read_index()
            end = read_index()
            code = helper.assign_call(
                self.wv(reg),
                "Slice",
                [self.rv(obj), helper.const(start), helper.const(end)],
            )

        elif opcode == OpCode.SliceStart:
            reg = read()
            obj = read()
            end = read_index()
            code = helper.assign_call(
                self.wv(reg),
                "Slice",
                [self.rv(obj), helper.const(None), helper.const(end)],
            )

        elif opcode == OpCode.SliceEnd:
            reg = read()
            obj = read()
            start = read_index()
            code = helper.assign_call(
                self.wv(reg),
                "Slice",
                [self.rv(obj), helper.const(start), helper.const(None)],
            )

        elif opcode == OpCode.SliceDyn:
            reg = read()
            obj = read()
            start = read()
            end = read()
            code = helper.assign_call(
                self.wv(reg),
                "Slice",
                [self.rv(obj), self.rv(start), self.rv(end)],
            )

        elif opcode == OpCode.SliceExclusiveDyn:
            reg = read()
            obj = read()
            start = read()
            end = read()
            code = helper.assign_call(
                self.wv(reg),
                "SliceExclusive",
                [self.rv(obj), self.rv(start), self.rv(end)],
            )

        # 处理数据结构初始化
        elif opcode == OpCode.Module:
            reg = read()
            nameIdx = read_index()
            name = self.constants[nameIdx]
            code = helper.class_def(
                self.wv(reg),
                [],
                decorator_list=[helper.call("Module", [helper.const(name)])],
            )
        elif opcode == OpCode.Record:
            reg = read()
            code = helper.assign(self.wv(reg), helper.dict([]))

        elif opcode == OpCode.Array:
            reg = read()
            code = helper.assign(self.wv(reg), helper.list([]))

        # # 处理条件语句
        elif opcode == OpCode.If:
            cond = read()
            code = helper.vm_if(self.rv(cond), False)

        elif opcode == OpCode.IfNot:
            cond = read()
            code = helper.vm_if(self.rv(cond), True)

        elif opcode == OpCode.IfInit:
            cond = read()
            code = helper.if_expr(
                helper.compare(self.rv(cond), "IsNot", helper.uninitialized())
            )

        elif opcode == OpCode.IfNotInit:
            cond = read()
            code = helper.if_expr(
                helper.compare(self.rv(cond), "Is", helper.uninitialized())
            )

        elif opcode == OpCode.IfNil:
            cond = read()
            code = helper.if_expr(
                helper.compare(self.rv(cond), "Is", helper.const(None))
            )

        elif opcode == OpCode.IfNotNil:
            cond = read()
            code = helper.if_expr(
                helper.compare(self.rv(cond), "IsNot", helper.const(None))
            )

        elif opcode == OpCode.LoopFor:
            nreg = read()
            iterable = read()
            code = helper.for_expr(
                self.wv(1, -1), helper.call("Iterable", [self.rv(iterable)])
            )

            loop_node = self.create_loop(helper, nreg, code)

        elif opcode in (OpCode.LoopRange, OpCode.LoopRangeExclusive):
            assert current_blocks_body is not None

            nreg = read()
            start = read()
            end = read()

            init, code = helper.vm_range_loop(
                self.wv(1, -1),
                self.rv(start),
                self.rv(end),
                opcode == OpCode.LoopRangeExclusive,
            )
            current_blocks_body.extend(init)
            loop_node = self.create_loop(
                helper,
                nreg,
                code,
                helper.aug_assign(self.wv(1, -1), helper.op("Add"), helper.const(1)),
            )
        elif opcode == OpCode.Loop:
            nreg = read()

            code = helper.while_expr(helper.const(True))

            loop_node = self.create_loop(helper, nreg, code)

        elif opcode == OpCode.Break:
            code = helper.ret("LoopBreak")
        elif opcode == OpCode.Continue:
            code = helper.ret("LoopContinue")
        elif opcode == OpCode.Noop:
            self.source_map_index -= 1  # Noop 不消耗 source map
        else:
            self.unknown_opcode(opcode)

        if current_blocks_body is None:
            assert isinstance(code, ast.FunctionDef)
            current_blocks_body = code.body
        elif code is not None:
            assert current_blocks_body is not None
            current_blocks_body.append(code)

        # 处理特殊的 opcode 后续逻辑
        if opcode in (OpCode.FuncVarg, OpCode.Func):
            assert isinstance(code, ast.FunctionDef)
            self.read_closure(code)
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
                code.body.append(helper.pass_stmt())

        elif opcode in (
            OpCode.Loop,
            OpCode.LoopFor,
            OpCode.LoopRange,
            OpCode.LoopRangeExclusive,
        ):
            assert loop_node is not None
            self.read_block_end(OpCode.LoopEnd, loop_node[0].body, loop_node[1])
        elif opcode == OpCode.Record:
            assert isinstance(code, ast.Assign) and isinstance(code.value, ast.Dict)
            self.read_record(reg, code.value)
        elif opcode == OpCode.Array:
            assert isinstance(code, ast.Assign) and isinstance(code.value, ast.List)
            self.read_array(reg, code.value)
        elif opcode == OpCode.Module:
            assert isinstance(code, ast.ClassDef)
            self.read_module(code)
