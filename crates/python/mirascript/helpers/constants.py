import re

REG_IDENTIFIER = re.compile(
    r"(?:_+|@+|\$+|\\p{XID_Start})\\p{XID_Continue}*", re.UNICODE
)

# 第二行：序数正则表达式
REG_ORDINAL = re.compile(
    r"(?:214748364[0-7]|21474836[0-3]\d|2147483[0-5]\d{2}|214748[0-2]\d{3}|21474[0-7]\d{4}|2147[0-3]\d{5}|214[0-6]\d{6}|21[0-3]\d{7}|20\d{8}|1\d{9}|[1-9]\d{0,8}|0)"
)
kVmScript = "mirascript.vm.script"
kVmFunction = "mirascript.vm.function"
kVmFunctionProxy = "mirascript.vm.function.proxy"
kVmContext = "mirascript.vm.context"
kVmExtern = "mirascript.vm.extern"
kVmModule = "mirascript.vm.module"
kVmWrapper = "mirascript.vm.wrapper"


VM_SCRIPT_NAME = "<script_root>"
VM_FUNCTION_ANONYMOUS_NAME = "<anonymous>"
