export const VM_ARRAY_MAX_LENGTH = 0x100_0000; // 16 M

export const kVmScript = Symbol.for('mirascript.vm.script');
export const kVmFunction = Symbol.for('mirascript.vm.function');
export const kVmFunctionProxy = Symbol.for('mirascript.vm.function.proxy');
export const kVmContext = Symbol.for('mirascript.vm.context');
export const kVmExtern = Symbol.for('mirascript.vm.extern');
export const kVmModule = Symbol.for('mirascript.vm.module');
export const kVmWrapper = Symbol.for('mirascript.vm.wrapper');

export const VM_SCRIPT_NAME = `<script_root>`;
export const VM_FUNCTION_ANONYMOUS_NAME = `<anonymous>`;
