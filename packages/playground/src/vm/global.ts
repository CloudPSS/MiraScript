import { $ToNumber } from './operations';
import {
    VmFunction,
    type VmExtern,
    type VmModule,
    type VmAny,
    type VmImmutable,
    type VmPrimitive,
    type VmValue,
    wrapToVmValue,
} from './types';

const global = Object.create(null) as Record<string, VmImmutable>;

global['max'] = VmFunction(
    (...args) => {
        const numbers = args.map($ToNumber);
        return Math.max(...numbers);
    },
    {
        isLib: true,
        fullName: 'global.max',
    },
);

/** 定义全局变量 */
export function defineGlobalFunction(name: string, fn: (...args: VmAny[]) => VmAny, override = false): void {
    if (!override && name in global) throw new Error(`Global variable '${name}' is already defined.`);
    global[name] = VmFunction(fn, {
        isLib: true,
        fullName: `global.${name}`,
    });
}
/** 定义全局变量 */
export function defineGlobalValue(name: string, value: VmImmutable, override = false): void {
    if (!override && name in global) throw new Error(`Global variable '${name}' is already defined.`);
    global[name] = value ?? null;
}

/** 创建全局环境 */
export function createGlobal(
    override?: Record<string, VmFunction | VmExtern | VmPrimitive | VmModule | object>,
): Record<string, VmValue> {
    const env = Object.create(global) as Record<string, VmValue>;
    if (override) {
        for (const key of Object.keys(override)) {
            const value = override[key];
            env[key] = wrapToVmValue(value, null);
        }
    }
    return env;
}
