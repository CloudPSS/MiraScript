import struct
import json
import base64
from typing import Any, Union, List, Tuple, Optional, Dict
import ast
class OpCodes:
    """MiraScript 操作码"""

    def __init__(self) -> None:
        from .mirascript import op_codes
        self.__dict__.update(op_codes())

    def __getattr__(self, name: str) -> int:
        if name in self.__dict__:
            return self.__dict__[name]
        raise AttributeError(f"No such OpCode: {name}")

    def __setattr__(self, name: str, value) -> None:
        raise AttributeError("OpCodes is immutable, cannot set attributes.")

    def __delattr__(self, name: str) -> None:
        raise AttributeError("OpCodes is immutable, cannot delete attributes.")

OpCode = OpCodes()
"""MiraScript 操作码"""

# 类型定义
VmPrimitive = Union[None, bool, int, float, str]
VmConst = Union[VmPrimitive, dict, list]
ScriptInput = Union[str, bytes]

class TranspileOptions:
    """转译选项"""
    def __init__(self, pretty: bool = False, source_map: bool = False, 
                 file_name: Optional[str] = None, input_mode: str = 'Script'):
        self.pretty = pretty
        self.source_map = source_map
        self.file_name = file_name
        self.input_mode = input_mode

ORIGIN = "mira://MiraScript"
source_id = 1

def read_const(data: bytes, offset: int) -> Tuple[VmPrimitive, int]:
    """解析常量"""
    type_byte = data[offset]
    
    if type_byte == 0:
        return None, 1
    elif type_byte == 1:
        return True, 1
    elif type_byte == 2:
        return False, 1
    elif type_byte == 3:
        ordinal = struct.unpack('<i', data[offset + 1:offset + 5])[0]
        return ordinal, 5
    elif type_byte == 4:
        num = struct.unpack('<d', data[offset + 1:offset + 9])[0]
        return num, 9
    elif type_byte == 5:
        length = struct.unpack('<I', data[offset + 1:offset + 5])[0]
        str_bytes = data[offset + 5:offset + 5 + length]
        text = str_bytes.decode('utf-8')
        return text, 5 + length
    else:
        raise ValueError(f"Unknown constant type: {type_byte}")

def to_python(value: VmConst) -> str:
    """将值转为 Python"""
    if value is None:
        return 'None'
    if isinstance(value, (dict, list)):
        return str(value)
    if isinstance(value, str):
        return json.dumps(value)
    # JSON 无法处理 NaN 等特殊数字
    if isinstance(value, float) and (value != value or value == float('inf') or value == float('-inf')):
        return 'float("nan")'  # 特殊数字处理
    return str(value)

def get_opcode_name(opcode: int) -> str:
    """获取操作码名称"""
    for attr_name in dir(OpCode):
        if not attr_name.startswith('_') and getattr(OpCode, attr_name, None) == opcode:
            return attr_name
    return str(opcode)

class Emitter:
    """代码生成"""
    
    def __init__(self, chunk: bytes):
        self.chunk = chunk
        
        # 解析 chunk 头部
        self.chunk_size = struct.unpack('<I', chunk[0:4])[0]
        self.code_size = struct.unpack('<I', chunk[4:8])[0]
        self.const_size = struct.unpack('<I', chunk[8 + self.code_size:12 + self.code_size])[0]
        
        # 设置数据视图
        self.code_data = chunk[8:8 + self.code_size]
        self.const_data = chunk[12 + self.code_size:12 + self.code_size + self.const_size]
        
        # 初始化状态
        self.constants: List[str] = []
        self.code_lines: List[str] = []
        
        self.func_script=None
        self.code_offset = 0
        self.closure_counter = 0
        self.ident_counter = 0
        self.current_blocks=None
        self.pre_blocks=None
    def read_consts(self) -> None:
        """读取常量表"""
        i = 0
        while i < self.const_size:
            constant, size = read_const(self.const_data, i)
            self.constants.append(to_python(constant))
            i += size
    
    def ident(self, length: int = 0) -> str:
        """制造缩进"""
        return '  ' * (self.ident_counter + length)
    
    def rv(self, i: int, level: int = 0) -> str:
        """Read variable"""
        if not i:
            return 'null'
        c = self.closure_counter - level
        return f'var_{c}_{i}'
    
    def wv(self, i: int, level: int = 0) -> str:
        """Write variable"""
        if not i:
            return '_'
        return self.rv(i, level)
    
    def read_param(self, wide: bool) -> int:
        """读取 code param"""
        if wide:
            value = struct.unpack('<I', self.code_data[self.code_offset:self.code_offset + 4])[0]
            self.code_offset += 4
        else:
            value = self.code_data[self.code_offset]
            self.code_offset += 1
        return value
    
    def read_index(self, wide: bool) -> int:
        """读取 code index"""
        if wide:
            value = struct.unpack('<i', self.code_data[self.code_offset:self.code_offset + 4])[0]
            self.code_offset += 4
        else:
            value = struct.unpack('<b', self.code_data[self.code_offset:self.code_offset + 1])[0]
            self.code_offset += 1
        return value
    
    def read_closure(self) -> None:
        """读取闭包"""
        self.closure_counter += 1
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            opcode = opcode_raw & 0x7f
            
            if opcode != OpCode.FuncEnd:
                self.read_code()
                continue
                
            self.code_offset += 1
            # body = self.ident(-1) + "} finally { CpExit(); } });"
            # self.code_lines.append(body)
            self.closure_counter -= 1
            self.ident_counter -= 1
            break
    
    def read_block_end(self, end_opcode: int) -> None:
        """读取块结束"""
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            opcode = opcode_raw & 0x7f
            
            if opcode != end_opcode:
                self.read_code()
                continue
                
            self.code_offset += 1
            self.ident_counter -= 1
            
            if end_opcode == OpCode.LoopEnd:
                self.closure_counter -= 1
                
            body = self.ident() + "};"
            self.code_lines.append(body)
            break
    
    def read_if_else(self) -> None:
        """读取 if else 或 if 结束"""
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            opcode = opcode_raw & 0x7f
            
            if opcode == OpCode.IfEnd:
                return self.read_block_end(OpCode.IfEnd)
            elif opcode == OpCode.Else:
                self.code_offset += 1
                body = self.ident(-1) + "} else {"
                self.code_lines.append(body)
                break
            elif opcode == OpCode.ElIf:
                self.code_offset += 1
                body = self.ident(-1) + "} else "
                self.code_lines.append(body)
                return self.read_code()
            
            self.read_code()
            
        return self.read_block_end(OpCode.IfEnd)
    
    def read_record(self, obj: int) -> None:
        """读取 record"""
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            self.code_offset += 1
            opcode = opcode_raw & 0x7f
            wide = opcode_raw >= 0x80
            
            def read():
                return self.read_param(wide)
            
            code = ''
            
            if opcode in (OpCode.FieldOpt, OpCode.Field):
                field = read()
                field_name = self.constants[field]
                if not field_name:
                    raise ValueError(f"Unknown field {field}")
                value = read()
                opt = opcode == OpCode.FieldOpt
                # Use computed property names to avoid prototype pollution
                if opt:
                    code = f"...ElementOpt({field_name}, {self.rv(value)}),"
                else:
                    code = f"[{field_name}]: Element({self.rv(value)}),"
            elif opcode in (OpCode.FieldOptDyn, OpCode.FieldDyn):
                field = read()
                value = read()
                opt = opcode == OpCode.FieldOptDyn
                if opt:
                    code = f"...ElementOpt({self.rv(field)}, {self.rv(value)}),"
                else:
                    code = f"[{self.rv(field)}]: Element({self.rv(value)}),"
            elif opcode in (OpCode.FieldOptIndex, OpCode.FieldIndex):
                field = read()
                value = self.read_index(wide)
                opt = opcode == OpCode.FieldOptIndex
                if opt:
                    code = f"...ElementOpt({field}, {self.rv(value)}),"
                else:
                    code = f"[{field}]: Element({self.rv(value)}),"
            elif opcode == OpCode.Spread:
                value = read()
                code = f"...$RecordSpread({self.rv(value)}),"
            elif opcode == OpCode.Freeze:
                self.ident_counter -= 1
                code = "});"
            else:
                opcode_name = get_opcode_name(opcode)
                code = f"// ?{opcode_name}"
            
            ident = self.ident()
            self.code_lines.append(ident + code)
            
            if opcode == OpCode.Freeze:
                return
    
    def read_array(self, arr: int) -> None:
        """读取 array"""
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            self.code_offset += 1
            opcode = opcode_raw & 0x7f
            wide = opcode_raw >= 0x80
            
            def read():
                return self.read_param(wide)
            
            code = ''
            
            if opcode == OpCode.Item:
                value = read()
                code = f"Element({self.rv(value)}),"
            elif opcode == OpCode.ItemRange:
                start = read()
                end = read()
                code = f"...ArrayRange({start}, {end}),"
            elif opcode == OpCode.ItemRangeDyn:
                start = read()
                end = read()
                code = f"...ArrayRange({self.rv(start)}, {self.rv(end)}),"
            elif opcode == OpCode.ItemRangeExclusiveDyn:
                start = read()
                end = read()
                code = f"...ArrayRangeExclusive({self.rv(start)}, {self.rv(end)}),"
            elif opcode == OpCode.Spread:
                value = read()
                code = f"...$ArraySpread({self.rv(value)}),"
            elif opcode == OpCode.Freeze:
                self.ident_counter -= 1
                code = "]);"
            else:
                opcode_name = get_opcode_name(opcode)
                code = f"// ?{opcode_name}"
            
            ident = self.ident()
            self.code_lines.append(ident + code)
            
            if opcode == OpCode.Freeze:
                return
    
    def read_code(self) -> None:
        """读取代码"""
        opcode_raw = self.code_data[self.code_offset]
        self.code_offset += 1
        opcode = opcode_raw & 0x7f
        wide = opcode_raw >= 0x80
        
        def read():
            return self.read_param(wide)
        
        def read_index():
            return self.read_index(wide)
        
        ident = self.ident()
        code = None
        reg = 0
        
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
                vararg=None,
                kwonlyargs=[],
                kw_defaults=[],
                kwarg=None,
                defaults=[]
            )
            for i in range(argn):
                wv = self.wv(i + 1, -1)
                if varg and i == argn - 1:
                    # 最后一个参数为可变参数
                    args.kwarg=ast.arg(arg=f'{wv}')
                else:
                    args.args.append(ast.arg(f"wv"))
            
            target= ast.Tuple(
                            elts=[
                            ],
                            ctx=ast.Store())
            regs = ast.Assign(
                    targets=[target],
                    value=ast.Name(id='Uninitialized',ctx=ast.Load()),lineno=0)
            for i in range(regn - argn + 1):
                if i:
                    # regs.append(self.wv(i + argn, -1))
                    target.elts.append(ast.Name(id=self.wv(i + argn, -1), ctx=ast.Store()))
                else:
                    # regs.append(self.wv(0, -1))
                    target.elts.append(ast.Name(id=self.wv(0 ,-1), ctx=ast.Store()))
            
            if script:
                tree = ast.FunctionDef("script",args=args,body=[regs],decorator_list=[],lineno=0)
                
                self.func_script=tree
                
                blocks=ast.Try(body=[ast.Assign(
                            targets=[
                               ast.Name(id='globalArgs', ctx=ast.Store())],
                            value=ast.Call(
                                func=ast.Name(id='GlobalFallback', ctx=ast.Load()),
                                args=[],
                                keywords=[]),lineno=0)
                            ],handlers=[],finalbody=[ast.Expr(
                            value=ast.Call(
                                func=ast.Name(id='CpExit', ctx=ast.Load()),
                                args=[],
                                keywords=[]))],orelse=[])
                self.func_script.body.append(blocks)
                self.current_blocks=blocks
            else:
                tree = ast.FunctionDef("script",args=args,body=[],decorator_list=[],lineno=0)
                blocks=ast.Try(body=[regs],handlers=[],finalbody=[ast.Expr(
                            value=ast.Call(
                                func=ast.Name(id='CpExit', ctx=ast.Load()),
                                args=[],
                                keywords=[]))],orelse=[])
                tree.body.append(blocks)
                
                self.pre_blocks=self.current_blocks
                self.current_blocks.body.append(tree)
                self.current_blocks=blocks
                
                
                pass
        
        elif opcode == OpCode.Constant:
            reg = read()
            i = read()
            c = self.constants[i]
            # code = f"{self.wv(reg)} = {c};"
            code = ast.Assign(
                            targets=[
                                ast.Name(id=self.wv(reg), ctx=ast.Store())],
                            value=ast.Constant(value=c),lineno=0)
        
        elif opcode == OpCode.Uninit:
            reg = read()
            # code = f"{self.wv(reg)} = undefined;"
            code = ast.Assign(
                    targets=[ast.Name(id=self.wv(reg), ctx=ast.Store())],
                    value=ast.Name(id='Uninitialized',ctx=ast.Load()),lineno=0)
        
        elif opcode == OpCode.Return:
            reg = read()
            # code = f"return {self.rv(reg)};"
            code =ast.Return(value=ast.Name(id=self.rv(reg), ctx=ast.Load()))
        
        elif opcode in (OpCode.Add, OpCode.Sub, OpCode.Mul, OpCode.Div, OpCode.Mod, OpCode.Pow,
                       OpCode.Gt, OpCode.Gte, OpCode.Lt, OpCode.Lte, OpCode.Eq, OpCode.Neq,
                       OpCode.Aeq, OpCode.Naeq, OpCode.Same, OpCode.Nsame, OpCode.In, OpCode.And, OpCode.Or):
            reg = read()
            left = read()
            right = read()
            opcode_name = get_opcode_name(opcode)
            # getattr(ast,"ADD")
            OpCode.Add
            code =ast.Assign(
                            targets=[
                                ast.Name(id=self.wv(reg), ctx=ast.Store())],
                            value=ast.BinOp(
                                left=ast.Name(id=self.rv(left), ctx=ast.Load()),
                                op=ast.Add(),
                                right=ast.Name(id=self.rv(right), ctx=ast.Load())))
        #     code = f"{self.wv(reg)} = ${opcode_name}({self.rv(left)}, {self.rv(right)});"
        
        # elif opcode == OpCode.InGlobal:
        #     reg = read()
        #     left = read()
        #     # code = f"{self.wv(reg)} = global[{self.rv(left)}] !== undefined;"
        #     code =ast.Assign(
        #                     targets=[
        #                        ast. Name(id='i', ctx=ast.Store())],
        #                     value=ast.Call(
        #                         func=ast.Name(id='Concat', ctx=ast.Load()),
        #                         args=[],
        #                         keywords=[]))
        
        # elif opcode == OpCode.Concat:
        #     reg = read()
        #     n = read()
        #     args = [self.rv(read()) for _ in range(n)]
        #     opcode_name = get_opcode_name(opcode)
        #     # code = f"{self.wv(reg)} = ${opcode_name}({', '.join(args)});"
        #     code =ast.Assign(
        #                     targets=[
        #                        ast. Name(id='i', ctx=ast.Store())],
        #                     value=ast.Call(
        #                         func=ast.Name(id='Concat', ctx=ast.Load()),
        #                         args=[ ast.Name(id=a, ctx=ast.Load()) for a in args],
        #                         keywords=[]))
        
        # elif opcode in (OpCode.Omit, OpCode.Pick):
        #     reg = read()
        #     value = read()
        #     n = read()
        #     args = [self.constants[read()] for _ in range(n)]
        #     opcode_name = get_opcode_name(opcode)
        #     code = f"{self.wv(reg)} = ${opcode_name}({self.rv(value)}, [{', '.join(args)}]);"
        
        # elif opcode in (OpCode.Call, OpCode.CallDyn):
        #     reg = read()
        #     func = read()
        #     n = read()
        #     args = [read() for _ in range(n)]
        #     ns = read()
        #     spreads = [read() for _ in range(ns)]
            
        #     if opcode == OpCode.Call:
        #         call_target = f"global[{self.constants[func]}]"
        #     else:
        #         call_target = self.rv(func)
            
        #     call_args = []
        #     for i, a in enumerate(args):
        #         if i in spreads:
        #             call_args.append(f"...$ArraySpread({self.rv(a)})")
        #         else:
        #             call_args.append(self.rv(a))
                    
        #     code = f"{self.wv(reg)} = $Call({call_target}, [{', '.join(call_args)}]);"
        
        # elif opcode == OpCode.Assign:
        #     reg = read()
        #     value = read()
        #     code = f"{self.wv(reg)} = {self.rv(value)};"
        
        # elif opcode in (OpCode.Pos, OpCode.Neg, OpCode.Not, OpCode.Type, OpCode.ToBoolean,
        #                OpCode.ToNumber, OpCode.ToString, OpCode.IsBoolean, OpCode.IsNumber,
        #                OpCode.IsString, OpCode.IsRecord, OpCode.IsArray, OpCode.Length):
        #     reg = read()
        #     value = read()
        #     opcode_name = get_opcode_name(opcode)
        #     code = f"{self.wv(reg)} = ${opcode_name}({self.rv(value)});"
        
        # elif opcode in (OpCode.AssertInit, OpCode.AssertNonNil):
        #     reg = read()
        #     opcode_name = get_opcode_name(opcode)
        #     code = f"${opcode_name}({self.rv(reg)})"
        
        # # 处理属性访问相关操作
        # elif opcode == OpCode.Get:
        #     reg = read()
        #     obj = read()
        #     prop = self.constants[read()]
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {prop});"
        
        # elif opcode == OpCode.GetIndex:
        #     reg = read()
        #     obj = read()
        #     index = read_index()
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {index});"
        
        # elif opcode == OpCode.GetDyn:
        #     reg = read()
        #     obj = read()
        #     index = read()
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {self.rv(index)});"
        
        # elif opcode == OpCode.Has:
        #     reg = read()
        #     obj = read()
        #     prop = self.constants[read()]
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {prop});"
        
        # elif opcode == OpCode.HasIndex:
        #     reg = read()
        #     obj = read()
        #     index = read_index()
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {index});"
        
        # elif opcode == OpCode.HasDyn:
        #     reg = read()
        #     obj = read()
        #     index = read()
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {self.rv(index)});"
        
        # elif opcode == OpCode.Set:
        #     reg = read()
        #     obj = read()
        #     prop = self.constants[read()]
        #     code = f"$Set({self.rv(obj)}, {prop}, {self.rv(reg)});"
        
        # elif opcode == OpCode.SetIndex:
        #     reg = read()
        #     obj = read()
        #     index = read_index()
        #     code = f"$Set({self.rv(obj)}, {index}, {self.rv(reg)});"
        
        # elif opcode == OpCode.SetDyn:
        #     reg = read()
        #     obj = read()
        #     index = read()
        #     code = f"$Set({self.rv(obj)}, {self.rv(index)}, {self.rv(reg)});"
        
        # 处理全局变量访问
        elif opcode == OpCode.GetGlobal:
            reg = read()
            i = read()
            c = self.constants[i]
            # code = f"{self.wv(reg)} = global[{c}] ?? null;"
            code = ast.Assign(
                            targets=[
                                ast.Name(id='h', ctx=ast.Store())],
                            value=ast.Subscript(
                                value=ast.Name(id='globalArgs', ctx=ast.Load()),
                                slice=ast.Constant(value=c),
                                ctx=ast.Load()),lineno=0)
        
        # elif opcode == OpCode.GetGlobalDyn:
        #     reg = read()
        #     name = read()
        #     code = f"{self.wv(reg)} = global[{self.rv(name)}] ?? null;"
        
        # # 处理闭包变量
        # elif opcode == OpCode.GetUpvalue:
        #     reg = read()
        #     level = read()
        #     up = read()
        #     code = f"{self.wv(reg)} = {self.rv(up, level)};"
        
        # elif opcode == OpCode.SetUpvalue:
        #     reg = read()
        #     level = read()
        #     up = read()
        #     code = f"{self.wv(up, level)} = {self.rv(reg)};"
        
        # # 处理数组切片
        # elif opcode == OpCode.Slice:
        #     reg = read()
        #     obj = read()
        #     start = read_index()
        #     end = read_index()
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {start}, {end});"
        
        # elif opcode == OpCode.SliceStart:
        #     reg = read()
        #     obj = read()
        #     end = read_index()
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, null, {end});"
        
        # elif opcode == OpCode.SliceEnd:
        #     reg = read()
        #     obj = read()
        #     start = read_index()
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {start}, null);"
        
        # elif opcode == OpCode.SliceDyn:
        #     reg = read()
        #     obj = read()
        #     start = read()
        #     end = read()
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {self.rv(start)}, {self.rv(end)});"
        
        # elif opcode == OpCode.SliceExclusiveDyn:
        #     reg = read()
        #     obj = read()
        #     start = read()
        #     end = read()
        #     code = f"{self.wv(reg)} = $SliceExclusive({self.rv(obj)}, {self.rv(start)}, {self.rv(end)});"
        
        # # 处理数据结构初始化
        # elif opcode == OpCode.Record:
        #     reg = read()
        #     code = f"{self.wv(reg)} = ({{"
        
        # elif opcode == OpCode.Array:
        #     reg = read()
        #     code = f"{self.wv(reg)} = (["
        
        # # 处理条件语句
        # elif opcode == OpCode.If:
        #     cond = read()
        #     code = f"if ($ToBoolean({self.rv(cond)})) {{"
        
        # elif opcode == OpCode.IfNot:
        #     cond = read()
        #     code = f"if (!$ToBoolean({self.rv(cond)})) {{"
        
        # elif opcode == OpCode.IfInit:
        #     cond = read()
        #     code = f"if ({self.rv(cond)} !== undefined) {{"
        
        # elif opcode == OpCode.IfNotInit:
        #     cond = read()
        #     code = f"if ({self.rv(cond)} === undefined) {{"
        
        # elif opcode == OpCode.IfNil:
        #     cond = read()
        #     code = f"if ({self.rv(cond)} === null) {{"
        
        # elif opcode == OpCode.IfNotNil:
        #     cond = read()
        #     code = f"if ({self.rv(cond)} !== null) {{"
        
        # # 处理循环语句
        # elif opcode == OpCode.LoopFor:
        #     nreg = read()
        #     iterable = read()
        #     regs = [self.wv(i + 2, -1) for i in range(nreg - 1)]
        #     code = f"for (let {self.wv(1, -1)} of $Iterable({self.rv(iterable)})) {{ Cp(); let _, {', '.join(regs)};"
        
        # elif opcode in (OpCode.LoopRange, OpCode.LoopRangeExclusive):
        #     nreg = read()
        #     start = read()
        #     end = read()
        #     exclusive = opcode == OpCode.LoopRangeExclusive
        #     regs = [self.wv(i + 2, -1) for i in range(nreg - 1)]
        #     i = self.wv(1, -1)
        #     op = '<' if exclusive else '<='
        #     code = f"for (let start = $ToNumber({self.rv(start)}), end = $ToNumber({self.rv(end)}), {i} = start; {i} {op} end; {i} += 1) {{ Cp(); let _, {', '.join(regs)};"
        
        # elif opcode == OpCode.Loop:
        #     nreg = read()
        #     regs = [self.wv(i + 1, -1) for i in range(nreg)]
        #     code = f"while (true) {{ Cp(); let _, {', '.join(regs)};"
        
        # elif opcode == OpCode.Break:
        #     code = "break;"
        
        # elif opcode == OpCode.Continue:
        #     code = "continue;"
        
        # else:
        #     # 默认处理未知 opcode
        #     opcode_name = get_opcode_name(opcode)
        #     code = f"; // {opcode_name}"
        
        # self.code_lines.append(ident + code)
        
        if code is not None:
            self.current_blocks.body.append(code)
        else:
            print('opcode:',opcode)
        
        
        # 处理特殊的 opcode 后续逻辑
        if opcode in (OpCode.FuncVarg, OpCode.Func):
            self.read_closure()
            self.ident_counter += 2
            pass
        elif opcode in (OpCode.If, OpCode.IfNot, OpCode.IfNil, OpCode.IfNotNil, OpCode.IfInit, OpCode.IfNotInit):
            self.read_if_else()
        elif opcode in (OpCode.Loop, OpCode.LoopFor, OpCode.LoopRange, OpCode.LoopRangeExclusive):
            self.ident_counter += 1
            self.closure_counter += 1
            self.read_block_end(OpCode.LoopEnd)
        elif opcode == OpCode.Record:
            self.read_record(reg)
        elif opcode == OpCode.Array:
            self.read_array(reg)
    
    def read(self) -> None:
        """读取 chunk"""
        self.read_consts()
        self.read_code()
    
def emit( chunk: bytes) -> str:
    """生成代码"""
    gen = Emitter( chunk)
    gen.read()
    # code = '\n'.join(gen.code_lines)
    # print(ast.unparse(tree))
    code=""
    if gen.func_script is not None:
        code =ast.unparse(gen.func_script)
    print(code)
    return code

# 兼容性部分 - 保留原有的接口
CommonContext = dict()
"""MiraScript 通用执行上下文"""

CommonContext["debug_print"] = lambda *message: print(f"[MiraScript] {message}")

class Context(dict):
    """
    MiraScript 执行上下文
    """

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)

    def __getitem__(self, key: str):
        if key in self:
            return super().__getitem__(key)
        if key in CommonContext:
            return CommonContext[key]
        return None

Env = dict()
"""MiraScript 执行环境"""

Env["Add"] = lambda x, y: x + y
Env["Context"] = Context
Env["Any"] = "Any"
Env["dict"] = dict

Uninitialized = type("Uninitialized", (), {})()
"""用于标记 MiraScript 中未初始化的变量"""
Env["Uninitialized"] = Uninitialized

class Script:
    """
    MiraScript 生成的 Python 函数
    """

    def __call__(self, context=None): ...


