kVmScript = "mirascript.vm.script"
kVmFunction = "mirascript.vm.function"
kVmFunctionProxy = "mirascript.vm.function.proxy"
kVmContext = "mirascript.vm.context"
kVmExtern = "mirascript.vm.extern"
kVmModule = "mirascript.vm.module"
kVmWrapper = "mirascript.vm.wrapper"


VM_ARRAY_MAX_LENGTH = 0x100_0000
# 16 M

VmUninitialized = type("VmUninitialized", (), {})
"""Mirascript 虚拟机内的未初始化变量"""
Uninitialized = VmUninitialized()
"""Mirascript 虚拟机内的未初始化变量"""
