import struct
import json
import base64
import traceback
from typing import Any, Union, List, Tuple, Optional, Dict
import ast

from mirascript.vm.operations import ToString_, is_decimal_number
from .vm.helpers import GlobalFallback
from .vm.env import vm_globals
from .deep_nonlocal_fix import deep_nonlocal_fix
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

def to_python(value: VmConst) -> Any:
    """将值转为 Python"""
    
    if value is None:
        return None
    if isinstance(value, (dict, list,str,bool)):
        return value
    if isinstance(value, (int, float)):
        return float(value)
    return value

def get_opcode_name(opcode: int) -> str:
    """获取操作码名称"""
    for attr_name in dir(OpCode):
        if not attr_name.startswith('_') and getattr(OpCode, attr_name, None) == opcode:
            return attr_name
    return str(opcode)

def ast_call(func_id: str, call_args: List[Any]) -> ast.Call:
    """生成 call"""
    return ast.Call(
            func=ast.Name(id=func_id, ctx=ast.Load()),
            args=call_args,
            keywords=[],lineno=0)  

def assign_call(target_id: str, func_id: str, call_args: List[Any]) -> ast.Assign:
    """生成 assign call"""
    return ast.Assign(
        targets=[
            ast.Name(id=target_id, ctx=ast.Store())],
        value=ast_call(func_id=func_id, call_args=call_args), lineno=0)

def create_Element(argsValue:List[str],fun_id='Element') -> ast.Call:
    return ast.Call(func=ast.Name(id=fun_id, ctx=ast.Load()),
                args=[ast.Name(id=a, ctx=ast.Load()) for a in argsValue],
                keywords=[])

def cp(id):
    """生成 Cp 调用"""
    return ast.Expr(
            value=ast.Call(
                func=ast.Name(id=id, ctx=ast.Load()),
                args=[],
                keywords=[]))

def create_parameter(name):
    """生成参数"""
    if name =='None':
        return ast.Constant(value=None, lineno=0)
    return ast.Name(id=name, ctx=ast.Load(), lineno=0)

def create_if(name: str, negate) -> ast.If:
    return ast.If(test=
                            ast.Compare(
                                left=ast.Call(
                                    func=ast.Name(id='ToBoolean_', ctx=ast.Load()),
                                    args=[
                                        ast.Name(id=name, ctx=ast.Load())],
                                    keywords=[]),
                                ops=[
                                    ast.NotEq()],
                                comparators=[
                                    ast.Constant(value=negate)]),
                        body=[],
                        orelse=[])
    
    
def create_range_loop(index,start,end,exclusive=False):
    
    s=ast.Assign(
            targets=[
                ast.Name(id='start', ctx=ast.Store())],
            value=ast.Call(
                func=ast.Name(id='ToNumber_', ctx=ast.Load()),
                args=[
                    ast.Name(id=start, ctx=ast.Load())],
                keywords=[]))
    e= ast.Assign(
            targets=[
                ast.Name(id='end', ctx=ast.Store())],
            value=ast.Call(
                func=ast.Name(id='ToNumber_', ctx=ast.Load()),
                args=[
                    ast.Name(id=end, ctx=ast.Load())],
                keywords=[]))
    i =ast.Assign(
                    targets=[
                        ast.Name(id=index, ctx=ast.Store())],
                    value=ast.Name(id='start', ctx=ast.Load()))
    w=ast.While(
                    test=ast.Compare(
                        left=ast.Name(id=index, ctx=ast.Load()),
                        ops=[
                            ast.LtE() if not exclusive else ast.Lt()],
                        comparators=[
                            ast.Name(id='end', ctx=ast.Load())]),
                    body=[],
                    orelse=[])
    return s,e,i,w
    
# def RangeExclusive_(start,end):
#     pass
#     s = ToNumber_(start)
#     e = ToNumber_(end)
#     if math.isnan(s) or math.isnan(e):
#         return []
#     # return list(range(int(s),int(e)))
#     result=[]
#     while s<e:
#         result.append(s)
#         s+=1
#     return result
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
        self.constants: List[Any] = []
        self.code_lines: List[str] = []
        
        self.func_script=None
        self.code_offset = 0
        self.closure_counter = 0
        self.ident_counter = 0
        self.current_blocks_body=None
        self.pre_blocks=None
        self.fun_name_counter=0
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
            return 'None'
        c = self.closure_counter - level
        return f'var_{c}_{i}'
    
    def wv(self, i: int, level: int = 0) -> str:
        """Write variable"""
        if not i:
            return '_'
        return self.rv(i, level)
    
    def fun_name(self) -> str:
        """函数名称"""
        self.fun_name_counter += 1
        return f'fun_{self.fun_name_counter}'
    
    def create_regs_array(self,nreg,start_index=0):
        target= ast.Tuple(
                        elts=[
                        ],
                        ctx=ast.Store())
        regsValue = ast.Tuple(elts=[], ctx=ast.Load())
        regs = ast.Assign(
                targets=[target],
                value=regsValue, lineno=0)

        for i in range(nreg + 1):
            if i:
                target.elts.append(ast.Name(id=self.wv(i+start_index , -1), ctx=ast.Store()))
                regsValue.elts.append(ast.Name(id='Uninitialized', ctx=ast.Load()))
            else:
                target.elts.append(ast.Name(id=self.wv(0 ,-1), ctx=ast.Store()))
                regsValue.elts.append(ast.Name(id='Uninitialized', ctx=ast.Load()))
                
        return regs
    
    def create_function(self,func_name,regs):
    
        args = ast.arguments(
            posonlyargs=[],
            args=[],
            vararg=None,
            kwonlyargs=[],
            kw_defaults=[],
            kwarg=None,
            defaults=[]
        )
                
        code = ast.FunctionDef(func_name,args=args,body=[],decorator_list=[],lineno=0)
        
        try_code=ast.Try(body=[cp('CpEnter'),regs],handlers=[],finalbody=[ast.Expr(
                    value=ast.Call(
                        func=ast.Name(id='CpExit', ctx=ast.Load()),
                        args=[],
                        keywords=[]))],orelse=[])
        code.body.append(try_code)
                
        return [code,try_code] 

    def create_loop(self,nreg,code,increment:Optional[ast.AugAssign]=None):
        
        
        func_name = self.fun_name()
        [loop_func_code,try_code] =self.create_function(func_name, self.create_regs_array(nreg-1,2))
        code.body.append(loop_func_code) 
        loop_func_code.body.append(ast.Return(value=ast.Name(id='LoopContinue', ctx=ast.Load()))) 
        
        
        code.body.append(
            ast.Assign(
                            targets=[
                                ast.Name(id=f'inner{func_name}Result', ctx=ast.Store())],
                            value=ast_call('Call_', [ast.Name(id=func_name, ctx=ast.Load())]), lineno=0))
        code.body.append(increment) if increment else None
        code.body.append(
                        ast.If(
                            test=ast.Compare(
                                left=ast.Name(id=f'inner{func_name}Result', ctx=ast.Load()),
                                ops=[
                                    ast.Is()],
                                comparators=[
                                    ast.Name(id='LoopContinue', ctx=ast.Load())]),
                            body=[
                                ast.Continue()],
                            orelse=[
                                ast.If(
                                    test=ast.Compare(
                                        left=ast.Name(id=f'inner{func_name}Result', ctx=ast.Load()),
                                        ops=[
                                            ast.Is()],
                                        comparators=[
                                            ast.Name(id='LoopBreak', ctx=ast.Load())]),
                                    body=[
                                        ast.Break()],
                                    orelse=[
                                        ast.Return(
                                            value=ast.Name(id=f'inner{func_name}Result', ctx=ast.Load()))])])
        )
        
        return try_code
        
        
        
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
    
    def read_closure(self,current_blocks_body:List[ast.stmt]) -> None:
        """读取闭包"""
        self.closure_counter += 1
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            opcode = opcode_raw & 0x7f
            
            if opcode != OpCode.FuncEnd:
                self.read_code(current_blocks_body)
                continue
            self.code_offset += 1
            self.closure_counter -= 1
            self.ident_counter -= 1
            break
    
    def read_block_end(self, end_opcode: int,current_blocks_body:List[ast.stmt]) -> None:
        """读取块结束"""
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            opcode = opcode_raw & 0x7f
            
            if opcode != end_opcode:
                self.read_code(current_blocks_body)
                continue
                
            self.code_offset += 1
            self.ident_counter -= 1
            
            if end_opcode == OpCode.LoopEnd:
                self.closure_counter -= 1
                
            # body = self.ident() + "};"
            # self.code_lines.append(body)
            break
    
    def read_if_else(self,block:ast.If) -> None:
        """读取 if else 或 if 结束"""
        self.ident_counter += 1
        body = block.body
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            opcode = opcode_raw & 0x7f
            
            if opcode == OpCode.IfEnd:
                return self.read_block_end(OpCode.IfEnd,body)
            elif opcode == OpCode.Else:
                self.code_offset += 1
                body=block.orelse
                break
                
            elif opcode == OpCode.ElIf:
                self.code_offset += 1
                # body = self.ident(-1) + "} else "
                # self.code_lines.append(body)
                raise ValueError("ElIf not supported in Python emitter")
                
                
                # return self.read_code()
            
            self.read_code(block.body)
            
        return self.read_block_end(OpCode.IfEnd,body)
    
    def read_record(self,obj, block: ast.Dict) -> None:
        """读取 record"""
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            self.code_offset += 1
            opcode = opcode_raw & 0x7f
            wide = opcode_raw >= 0x80
            
            def read():
                return self.read_param(wide)
               
            def add_Element(argsValue:list,fun_id='ElementOpt',key=None) -> None:
                block.keys.append(key)
                block.values.append(ast.Call(
                            func=ast.Name(id=fun_id, ctx=ast.Load()),
                            args=[ast.Name(id=a, ctx=ast.Load()) if not isinstance(a,ast.expr) else a  for a in argsValue],
                            keywords=[]))
            if opcode in (OpCode.FieldOpt, OpCode.Field):
                field = read()
                field_name = self.constants[field]
                if  field_name is None:
                    raise ValueError(f"Unknown field {field},{self.constants}")
                value = read()
                opt = opcode == OpCode.FieldOpt
                if opt:
                    
                    add_Element([ast.Constant(value=f"{field_name}"), self.rv(value)])
                    
                else:
                    add_Element([self.rv(value)],fun_id='Element',key=ast.Constant(value=field_name))
            elif opcode in (OpCode.FieldOptDyn, OpCode.FieldDyn):
                field = read()
                value = read()
                opt = opcode == OpCode.FieldOptDyn
                if opt:
                    add_Element([self.rv(field), self.rv(value)])
                else:
                    add_Element([self.rv(value)],fun_id='Element',key=ast.Name(id=self.rv(field), ctx=ast.Load()))
            elif opcode in (OpCode.FieldOptIndex, OpCode.FieldIndex):
                field = self.read_index(wide)
                value = read()
                opt = opcode == OpCode.FieldOptIndex
                if opt:
                    add_Element([ast.Constant(value=ToString_(field)), self.rv(value)])
                else:
                    add_Element([self.rv(value)],fun_id='Element',key=ast.Constant(value=str(field)))
            elif opcode == OpCode.Spread:
                value = read()
                add_Element([ self.rv(value)],"RecordSpread_")
                
                
           
            
            ident = self.ident()
            # self.code_lines.append(ident + code)
            
            if opcode == OpCode.Freeze:
                return
    
    def read_array(self, arr: int,block:ast.List) -> None:
        """读取 array"""
        self.ident_counter += 1
        
        while self.code_offset < self.code_size:
            opcode_raw = self.code_data[self.code_offset]
            self.code_offset += 1
            opcode = opcode_raw & 0x7f
            wide = opcode_raw >= 0x80
            
            def read():
                return self.read_param(wide)
        
            
            
            if opcode == OpCode.Item:
                value = read()
                block.elts.append(create_Element([self.rv(value)]))
                
            elif opcode == OpCode.ItemRange:
                start = read()
                end = read()
                # code = f"...ArrayRange({start}, {end}),"
                block.elts.append(ast.Starred(value=create_Element([self.rv(start), self.rv(end)],fun_id="ArrayRange"), ctx=ast.Load()))
            elif opcode == OpCode.ItemRangeDyn:
                start = read()
                end = read()
                # code = f"...ArrayRange({self.rv(start)}, {self.rv(end)}),"
                block.elts.append(ast.Starred(value=create_Element([self.rv(start), self.rv(end)],fun_id="ArrayRange"), ctx=ast.Load()))
            elif opcode == OpCode.ItemRangeExclusiveDyn:
                start = read()
                end = read()
                # code = f"...ArrayRangeExclusive({self.rv(start)}, {self.rv(end)}),"
                block.elts.append(ast.Starred(value=create_Element([self.rv(start), self.rv(end)],fun_id="ArrayRangeExclusive"), ctx=ast.Load()))
            elif opcode == OpCode.Spread:
                value = read()
                # code = f"...$ArraySpread({self.rv(value)}),"
                block.elts.append(ast.Starred(value=create_Element([self.rv(value)],fun_id="ArraySpread_"), ctx=ast.Load()))
            elif opcode == OpCode.Freeze:
                pass
                # self.ident_counter -= 1
                # code = "]);"
            else:
                opcode_name = get_opcode_name(opcode)
                code = f"// ?{opcode_name}"
            
            ident = self.ident()
            # self.code_lines.append(ident + code)
            
            if opcode == OpCode.Freeze:
                return
    
    def read_code(self,current_blocks_body:Union[List[ast.stmt],None]=None) -> None:
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
        code=ast.Pass()
        reg = 0
        func_name=None
        loop_node=None
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
                vararg=ast.arg(arg='args'),
                kwonlyargs=[],
                kw_defaults=[],
                kwarg=ast.arg(arg='kwargs'),
                defaults=[]
            )
            for i in range(argn):
                wv = self.wv(i + 1, -1)
                if varg and i == argn - 1:
                    # 最后一个参数为可变参数
                    args.vararg=ast.arg(arg='vargs')
                else:
                    args.args.append(ast.arg(f"{wv}"))
                    args.defaults.append(ast.Constant(value=None, lineno=0))
            
            target= ast.Tuple(
                            elts=[
                            ],
                            ctx=ast.Store())
            regsValue = ast.Tuple(elts=[], ctx=ast.Load())
            regs = ast.Assign(
                    targets=[target],
                    value=regsValue, lineno=0)
            for i in range(regn - argn + 1):
                if i:
                    target.elts.append(ast.Name(id=self.wv(i + argn, -1), ctx=ast.Store()))
                    regsValue.elts.append(ast.Name(id='Uninitialized',ctx=ast.Load()))
                else:
                    target.elts.append(ast.Name(id=self.wv(0 ,-1), ctx=ast.Store()))
                    regsValue.elts.append(ast.Name(id='Uninitialized',ctx=ast.Load()))
            
            if script:
                args.args.insert(0, ast.arg(arg='context'))
                args.defaults.insert(0, ast.Call(
                        func=ast.Name(id='GlobalFallback', ctx=ast.Load()),
                        args=[],
                        keywords=[]))
                code = ast.FunctionDef("script",args=args,body=[],decorator_list=[],lineno=0)
                
                self.func_script=code
                
                try_code=ast.Try(body=[cp('CpEnter'),regs],handlers=[],finalbody=[ast.Expr(
                            value=ast.Call(
                                func=ast.Name(id='CpExit', ctx=ast.Load()),
                                args=[],
                                keywords=[]))],orelse=[])
                code.body.append(try_code)
                
            else:
                func_name = self.wv(reg)
                code = ast.FunctionDef(func_name,args=args,body=[],decorator_list=[],lineno=0)
                try_code=ast.Try(body=[cp('CpEnter'),regs],handlers=[],finalbody=[cp('CpExit')],orelse=[])
                
                code.body.append(try_code)
            
            if varg:
                try_code.body.append(ast.Assign(
                        targets=[ast.Name(id=self.wv(argn,-1), ctx=ast.Store())],
                        value=ast.Call(
                        func=ast.Name(id='Vargs', ctx=ast.Load()),
                        args=[
                            ast.Name(id='vargs', ctx=ast.Load())],
                        keywords=[]), lineno=0))
        
        elif opcode == OpCode.Constant:
            reg = read()
            i = read()
            c = self.constants[i]
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
            code =ast.Return(value=create_parameter(self.rv(reg)))
        
        elif opcode in (OpCode.Add, OpCode.Sub, OpCode.Mul, OpCode.Div, OpCode.Mod, OpCode.Pow,
                       OpCode.Gt, OpCode.Gte, OpCode.Lt, OpCode.Lte, OpCode.Eq, OpCode.Neq,
                       OpCode.Aeq, OpCode.Naeq, OpCode.Same, OpCode.Nsame, OpCode.In, OpCode.And, OpCode.Or,OpCode.Format):
            reg = read()
            left = read()
            right = read()
            opcode_name = get_opcode_name(opcode)
            opArgs=[]
            leftValue=self.rv(left)
            rightValue=self.rv(right)
            if leftValue=='None':
                opArgs.append(ast.Constant(value=None))
            else:
                opArgs.append(ast.Name(id=leftValue, ctx=ast.Load()))
            if rightValue=='None':
                opArgs.append(ast.Constant(value=None))
            else:
                opArgs.append(ast.Name(id=rightValue, ctx=ast.Load()))

            code =ast.Assign(
                    targets=[
                        ast.Name(id=self.wv(reg), ctx=ast.Store())],
                    value=ast.Call(
                        func=ast.Name(id=f"{opcode_name}_", ctx=ast.Load()),
                        args=opArgs,
                        keywords=[]),lineno=0)
        
        elif opcode == OpCode.InGlobal:
            reg = read()
            left = read()
            code = ast.Assign(
                    targets=[
                        ast.Name(id=self.wv(reg), ctx=ast.Store())],
                    value=ast.Call(
                        func=ast.Attribute(
                            value=ast.Name(id='context', ctx=ast.Load()),
                            attr='has',
                            ctx=ast.Load()),
                        args=[
                            ast.Call(
                                func=ast.Name(id='ToString_', ctx=ast.Load()),
                                args=[
                                     ast.Name(id=self.rv(left), ctx=ast.Load())],
                                keywords=[])
                           ],
                        keywords=[]),lineno=0)
            
        
        elif opcode == OpCode.Concat:
            reg = read()
            n = read()
            args = [self.rv(read()) for _ in range(n)]
            opcode_name = get_opcode_name(opcode)
            code  = assign_call(self.wv(reg), f"{opcode_name}_", [ast.Name(id=a, ctx=ast.Load()) for a in args])
        
        elif opcode in (OpCode.Omit, OpCode.Pick):
            reg = read()
            value = read()
            n = read()
            args = [self.constants[read()] for _ in range(n)]
            opcode_name = get_opcode_name(opcode)
            
            call_args = ast.List(elts=[ast.Constant(value=a) for a in args], ctx=ast.Load())
            code = assign_call(self.wv(reg), f"{opcode_name}_", [ ast.Name(id=self.rv(value), ctx=ast.Load()), call_args])
            
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
                call_target=ast.Subscript(
                                value=ast.Name(id='context', ctx=ast.Load()),
                                slice=ast.Constant(value=self.constants[func]),
                                ctx=ast.Load())
            else:
                call_target = ast.Name(id=self.rv(func), ctx=ast.Load())
                
            
            call_args = ast.Tuple(elts=[], ctx=ast.Load())
            for i, a in enumerate(args):
                if i in spreads:
                    # call_args.append(f"...$ArraySpread({self.rv(a)})")
                    call_args.elts.append(ast.Starred(create_Element([self.rv(a)],fun_id="ArraySpread_"), ctx=ast.Load()))
                else:
                    # call_args.append(self.rv(a))
                    call_args.elts.append(ast.Name(id=self.rv(a), ctx=ast.Load(),lineno=0))


            fun_args:list=[call_target]
            
            if len(call_args.elts)>0:
                fun_args.append(ast.Starred(value=call_args, ctx=ast.Load()))
            code = assign_call(self.wv(reg), 'Call_', fun_args)

            
        
        elif opcode == OpCode.Assign:
            reg = read()
            value = read()
            if not value:
                val = ast.Constant(value=None)
            else:
                val = ast.Name(id=self.rv(value), ctx=ast.Load())
            code =ast.Assign(
                    targets=[
                        ast.Name(id=self.wv(reg), ctx=ast.Store())],
                    value=val,lineno=0)
        
        elif opcode in (OpCode.Pos, OpCode.Neg, OpCode.Not, OpCode.Type, OpCode.ToBoolean,
                       OpCode.ToNumber, OpCode.ToString, OpCode.IsBoolean, OpCode.IsNumber,
                       OpCode.IsString, OpCode.IsRecord, OpCode.IsArray, OpCode.Length):
            reg = read()
            value = read()
            opcode_name = get_opcode_name(opcode)
            code = assign_call(self.wv(reg), f"{opcode_name}_", [ast.Name(id=self.rv(value), ctx=ast.Load())])
        #     code = f"{self.wv(reg)} = ${opcode_name}({self.rv(value)});"
        
        elif opcode in (OpCode.AssertInit, OpCode.AssertNonNil):
            reg = read()
            opcode_name = get_opcode_name(opcode)
            # code =ast.Expr( value=ast_call(f"{opcode_name}_", [ast.Constant(value=None)]))
            if reg==0:
                code =ast.Expr( value=ast_call(f"{opcode_name}_", [ast.Constant(value=None)]))
            else:
                code = assign_call('', f"{opcode_name}_", [ast.Name(id=self.rv(reg), ctx=ast.Load())])
        #     code = f"${opcode_name}({self.rv(reg)})"
        
        # # 处理属性访问相关操作
        elif opcode == OpCode.Get:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code  = assign_call(self.wv(reg), 'Get_', [ast.Name(id=self.rv(obj), ctx=ast.Load()),ast.Constant(value=prop)])
        
        elif opcode == OpCode.GetIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = assign_call(self.wv(reg), 'Get_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=index)])
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {index});"
        
        elif opcode == OpCode.GetDyn:
            reg = read()
            obj = read()
            index = read()
            code = assign_call(self.wv(reg), 'Get_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Name(id=self.rv(index), ctx=ast.Load())])
        #     code = f"{self.wv(reg)} = $Get({self.rv(obj)}, {self.rv(index)});"
        
        elif opcode == OpCode.Has:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = assign_call(self.wv(reg), 'Has_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=prop)])
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {prop});"
        
        elif opcode == OpCode.HasIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = assign_call(self.wv(reg), 'Has_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=index)])
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {index});"
        
        elif opcode == OpCode.HasDyn:
            reg = read()
            obj = read()
            index = read()
            code = assign_call(self.wv(reg), 'Has_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Name(id=self.rv(index), ctx=ast.Load())])
        #     code = f"{self.wv(reg)} = $Has({self.rv(obj)}, {self.rv(index)});"
        
        elif opcode == OpCode.Set:
            reg = read()
            obj = read()
            prop = self.constants[read()]
            code = assign_call(self.wv(reg), 'Set_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=prop), ast.Name(id=self.rv(reg), ctx=ast.Load())])
        #     code = f"$Set({self.rv(obj)}, {prop}, {self.rv(reg)});"
        
        elif opcode == OpCode.SetIndex:
            reg = read()
            obj = read()
            index = read_index()
            code = assign_call(self.wv(reg), 'Set_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=index), ast.Name(id=self.rv(reg), ctx=ast.Load())])
        #     code = f"$Set({self.rv(obj)}, {index}, {self.rv(reg)});"
        
        elif opcode == OpCode.SetDyn:
            reg = read()
            obj = read()
            index = read()
            code = assign_call(self.wv(reg), 'Set_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Name(id=self.rv(index), ctx=ast.Load()), ast.Name(id=self.rv(reg), ctx=ast.Load())])
        #     code = f"$Set({self.rv(obj)}, {self.rv(index)}, {self.rv(reg)});"
        
        # 处理全局变量访问
        elif opcode == OpCode.GetGlobal:
            reg = read()
            i = read()
            c = self.constants[i]
            code = assign_call(self.wv(reg), 'GetGlobal_', [ast.Name(id='context', ctx=ast.Load()), ast.Constant(value=c)])
        
        elif opcode == OpCode.GetGlobalDyn:
            reg = read()
            name = read()
            code = assign_call(self.wv(reg), 'GetGlobal_', [ast.Name(id='context', ctx=ast.Load()), ast.Name(id=self.rv(name), ctx=ast.Load())])
        #     code = f"{self.wv(reg)} = global[{self.rv(name)}] ?? null;"
        
        # # 处理闭包变量
        elif opcode == OpCode.GetUpvalue:
            reg = read()
            level = read()
            up = read()
            code = assign_call(self.wv(reg), 'Upvalue', [ast.Name(id=self.rv(up, level), ctx=ast.Load())])

        
        elif opcode == OpCode.SetUpvalue:
            reg = read()
            level = read()
            up = read()
            if not current_blocks_body:
                raise ValueError("No current block to set upvalue")
            
            current_blocks_body.insert(0,ast.Nonlocal(
                            names=[self.rv(up, level)]))
            code = ast.Assign(
                            targets=[
                                ast.Name(id=self.rv(up, level), ctx=ast.Store())],
                            value=ast.Name(id=self.rv(reg), ctx=ast.Load()),lineno=0)
        
        # # 处理数组切片
        elif opcode == OpCode.Slice:
            reg = read()
            obj = read()
            start = read_index()
            end = read_index()
            code = assign_call(self.wv(reg), 'Slice_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=start), ast.Constant(value=end)])
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {start}, {end});"
        
        elif opcode == OpCode.SliceStart:
            reg = read()
            obj = read()
            end = read_index()
            code = assign_call(self.wv(reg), 'Slice_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=None), ast.Constant(value=end)])
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, null, {end});"
        
        elif opcode == OpCode.SliceEnd:
            reg = read()
            obj = read()
            start = read_index()
            code = assign_call(self.wv(reg), 'Slice_', [ast.Name(id=self.rv(obj), ctx=ast.Load()), ast.Constant(value=start), ast.Constant(value=None)])
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {start}, null);"
        
        elif opcode == OpCode.SliceDyn:
            reg = read()
            obj = read()
            start = read()
            end = read()
            code = assign_call(self.wv(reg), 'Slice_', [create_parameter(self.rv(obj)), create_parameter(self.rv(start)), create_parameter(self.rv(end))])
        #     code = f"{self.wv(reg)} = $Slice({self.rv(obj)}, {self.rv(start)}, {self.rv(end)});"
        
        elif opcode == OpCode.SliceExclusiveDyn:
            reg = read()
            obj = read()
            start = read()
            end = read()


            code = assign_call(self.wv(reg), 'SliceExclusive_', [create_parameter(self.rv(obj)), create_parameter(self.rv(start)), create_parameter(self.rv(end))])
        #     code = f"{self.wv(reg)} = $SliceExclusive({self.rv(obj)}, {self.rv(start)}, {self.rv(end)});"
        
        # # 处理数据结构初始化
        elif opcode == OpCode.Record:
            reg = read()
            # code = f"{self.wv(reg)} = ({{"
            code = ast.Assign(targets=[
                        ast.Name(id=self.wv(reg), ctx=ast.Store())],
                    value=ast.Dict(
                        keys=[],
                        values=[]),lineno=0)
        
        elif opcode == OpCode.Array:
            reg = read()
            code =ast.Assign(targets=[
                        ast.Name(id=self.wv(reg), ctx=ast.Store())],
                    value=ast.List(
                    elts=[],
                    ctx=ast.Load()),lineno=0) 
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
            code = ast.If(test=ast.Compare(
                        left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                        ops=[
                            ast.IsNot()],
                        comparators=[
                           ast.Name(id='Uninitialized', ctx=ast.Load())]),
                        body=[],
                        orelse=[])
            
        #     code = f"if ({self.rv(cond)} !== undefined) {{"
        
        elif opcode == OpCode.IfNotInit:
            cond = read()
            code = ast.If(test=ast.Compare(
                        left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                        ops=[
                            ast.Is()],
                        comparators=[
                           ast. Name(id='Uninitialized', ctx=ast.Load())]),
                        body=[],
                        orelse=[])
        #     code = f"if ({self.rv(cond)} === undefined) {{"
        
        elif opcode == OpCode.IfNil:
            cond = read()
            code = ast.If(test=ast.Compare(
                        left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                        ops=[
                            ast.Is()],
                        comparators=[
                            ast.Constant(value=None)]),
                        body=[],
                        orelse=[])
        #     code = f"if ({self.rv(cond)} === null) {{"
        
        elif opcode == OpCode.IfNotNil:
            cond = read()
            code = ast.If(test=ast.Compare(
                        left=ast.Name(id=self.rv(cond), ctx=ast.Load()),
                        ops=[
                            ast.IsNot()],
                        comparators=[
                            ast.Constant(value=None)]),
                        body=[],
                        orelse=[])
            
        #     code = f"if ({self.rv(cond)} !== null) {{"
        
        # # 处理循环语句
        elif opcode == OpCode.LoopFor:
            nreg = read()
            iterable = read()
            regs = [self.wv(i + 2, -1) for i in range(nreg - 1)]
            # code = f"for (let {self.wv(1, -1)} of $Iterable({self.rv(iterable)})) {{ Cp(); let _, {', '.join(regs)};"
            
            code = ast.For(
                            target=ast.Name(id=self.wv(1, -1), ctx=ast.Store()),
                            iter=ast_call('Iterable_', [ast.Name(id=self.rv(iterable), ctx=ast.Load())]),
                            body=[],
                            orelse=[],lineno=0)
            
            loop_node=self.create_loop(nreg,code)
        
        elif opcode in (OpCode.LoopRange, OpCode.LoopRangeExclusive):
            nreg = read()
            start = read()
            end = read()
            exclusive = opcode == OpCode.LoopRangeExclusive
            
            s,e,i,code= create_range_loop(self.wv(1, -1), self.rv(start), self.rv(end), exclusive)
            # code = ast.For(
            #                 target=ast.Name(id=self.wv(1, -1), ctx=ast.Store()),
            #                 iter=ast_call('Range_' if not exclusive else 'RangeExclusive_', [ast.Name(id=self.rv(start), ctx=ast.Load()), ast.Name(id=self.rv(end), ctx=ast.Load())]),
            #                 body=[],
            #                 orelse=[],lineno=0)
            
            current_blocks_body.append(s)
            current_blocks_body.append(e)
            current_blocks_body.append(i)
            loop_node=self.create_loop(nreg,code,ast.AugAssign(
                            target=ast.Name(id=self.wv(1, -1), ctx=ast.Store()),
                            op=ast.Add(),
                            value=ast.Constant(value=1)))
        elif opcode == OpCode.Loop:
            nreg = read()
            
            
            code = ast.While(test=ast.Constant(value=True), body=[ast.Expr(
                    value=ast.Call(
                        func=ast.Name(id='Cp', ctx=ast.Load()),
                        args=[],
                        keywords=[]))], orelse=[])
            
            loop_node=self.create_loop(nreg,code)
        
        elif opcode == OpCode.Break:
            # code = ast.Break()
            code = ast.Return(value=ast.Name(id='LoopBreak', ctx=ast.Load()))
        
        elif opcode == OpCode.Continue:
        #     code = "continue;"
            # code = ast.Continue()
            code = ast.Return(value=ast.Name(id='LoopContinue', ctx=ast.Load()))
        
        else:
            # 默认处理未知 opcode
            opcode_name = get_opcode_name(opcode)
            print(f"Unknown opcode: {opcode_name} ({opcode}) at offset {self.code_offset - 1}")
        #     code = f"; // {opcode_name}"
        
        # self.code_lines.append(ident + code)
        
        if current_blocks_body is None :
            if not isinstance(code,ast.FunctionDef):
                raise ValueError("current_blocks_body is None, please set it before calling read_code")
            try_code= code.body[0]
            if not isinstance(try_code,ast.Try):
                raise ValueError(f"Expected Try node, got {type(try_code)}")
            current_blocks_body= try_code.body
                
        else:
            current_blocks_body.append(code)
            
        # if code is not None:
        #     self.current_blocks.body.append(code)
        # elif opcode not in (OpCode.FuncVarg, OpCode.Func):
        #     print('opcode:',opcode)
            
        
        
        # 处理特殊的 opcode 后续逻辑
        if opcode in (OpCode.FuncVarg, OpCode.Func):
            if not isinstance(code,ast.FunctionDef):
                raise ValueError("current_blocks_body is None, please set it before calling read_code")
            try_code= code.body[0]
            if not isinstance(try_code,ast.Try):
                raise ValueError(f"Expected Try node, got {type(try_code)}")
            self.read_closure(try_code.body)
            self.ident_counter += 2
            pass
        elif opcode in (OpCode.If, OpCode.IfNot, OpCode.IfNil, OpCode.IfNotNil, OpCode.IfInit, OpCode.IfNotInit):
            if not isinstance(code, ast.If):
                raise ValueError(f"Expected If node, got {type(code)}")
            self.read_if_else(code)
            if len(code.body)<1:
                code.body.append(ast.Pass())
                
        elif opcode in (OpCode.Loop, OpCode.LoopFor, OpCode.LoopRange, OpCode.LoopRangeExclusive):
            self.ident_counter += 1
            self.closure_counter += 1
            if loop_node is None:
                raise ValueError("while_node is None, please set it before calling read_code")
            self.read_block_end(OpCode.LoopEnd,loop_node.body)
        elif opcode == OpCode.Record:
            if not isinstance(code, ast.Assign):
                raise ValueError(f"Expected Assign node for Record, got {type(code)}")
            if not isinstance(code.value, ast.Dict):
                raise ValueError(f"Expected Dict value for Record, got {type(code.value)}")
            self.read_record(reg,code.value)
        elif opcode == OpCode.Array:
            if not isinstance(code, ast.Assign):
                raise ValueError(f"Expected Assign node for Record, got {type(code)}")
            if not isinstance(code.value, ast.List):
                raise ValueError(f"Expected Dict value for Record, got {type(code.value)}")
            self.read_array(reg,code.value)
    
    def read(self) -> None:
        """读取 chunk"""
        self.read_consts()
        self.read_code()

def set_ast_positions(node, lineno=1, col_offset=0):
    """为AST节点设置行号和列偏移量"""
    for field, value in ast.iter_fields(node):
        if isinstance(value, list):
            for item in value:
                if isinstance(item, ast.AST):
                    set_ast_positions(item, lineno, col_offset)
        elif isinstance(value, ast.AST):
            set_ast_positions(value, lineno, col_offset)
    
    # 设置当前节点的位置信息
    if not hasattr(node, 'lineno'):
        node.lineno = lineno
    if not hasattr(node, 'col_offset'):
        node.col_offset = col_offset

## 将所有 nonlocal 声明提升到函数体顶部,有些情况下nonlocal声明可能在条件语句或循环内
# ...existing code...


def emit( chunk: bytes) :
    """生成代码"""
    try:
        
        gen = Emitter( chunk)
        gen.read()
        code=None
        if gen.func_script is not None:
            script =deep_nonlocal_fix(gen.func_script)
            set_ast_positions(script)
            module = ast.Module(body=[script], type_ignores=[])
            code_script=ast.unparse(gen.func_script)
            f =open('out_script.py', 'w', encoding='utf-8')
            f.write("""from mirascript.vm.operations import *
from mirascript.vm.types.checker import is_vm_const
from mirascript.vm.types.extern import VmExtern
from mirascript.vm.helpers import GlobalFallback,Element,ArrayRange,ArrayRangeExclusive,ElementOpt,Upvalue
from mirascript.vm.helpers import CpEnter,CpExit
Uninitialized = type("Uninitialized", (), {})()\n""")
            f.write(code_script)
            f.write("\nscript()")
            f.close()
            f2 =open('out_script2.py', 'w', encoding='utf-8')
            f2.write(ast.dump(module,indent=4))
            f2.close()
            code=compile(module, "<string>", "exec")
        if code is not None:
            exec(code, vm_globals,)
        return vm_globals.get('script', None)

    except Exception as e:
        traceback.print_exc()
        print(f"Error during code emission: {e}")
        
        return None


