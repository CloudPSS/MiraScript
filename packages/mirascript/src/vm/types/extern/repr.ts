import { getPrototypeOf } from '../../../helpers/utils.js';

const ObjectPrototype = Object.prototype;
// eslint-disable-next-line @typescript-eslint/unbound-method
const ObjectToString = Object.prototype.toString;
// eslint-disable-next-line @typescript-eslint/unbound-method
const FunctionToString = Function.prototype.toString;
const ArrayToString = Array.prototype.toString;
// eslint-disable-next-line @typescript-eslint/unbound-method
const TypedArrayToString = Uint8Array.prototype.toString;

/** 获取类的名称，如果无法确定则返回 null */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
function classNameOf(kls: Function): string | null {
    const { displayName } = kls as typeof kls & { displayName?: string };
    if (typeof displayName === 'string' && displayName.trim()) {
        return displayName.trim();
    }
    const { name } = kls;
    if (typeof name == 'string' && name.length > 2) {
        // Looks like a non-minified name
        return name;
    }
    return null;
}

/** Check toString method of the extern object */
export function hasCustomToString(value: object): boolean {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const { toString } = value;
    if (typeof toString != 'function') return false;
    if (
        toString === ObjectToString ||
        toString === FunctionToString ||
        toString === ArrayToString ||
        toString === TypedArrayToString
    )
        return false;
    return true;
}

/** Get the tag of the extern object */
export function getTag(value: object): string {
    const tag = ObjectToString.call(value).slice(8, -1);
    if (tag === 'Function') {
        if ('prototype' in value && typeof value.prototype == 'object') {
            const className = classNameOf(value as unknown as new () => unknown);
            if (!className) return `class`;
            return `class ${className}`;
        }
        return `function`;
    }
    if (tag === 'AsyncFunction') {
        return `async function`;
    }
    if (tag === 'GeneratorFunction') {
        return `function*`;
    }
    if (tag === 'AsyncGeneratorFunction') {
        return `async function*`;
    }
    if (tag === 'Object') {
        const proto = getPrototypeOf(value);
        if (proto === ObjectPrototype) {
            return 'Object';
        }
        if (proto == null) {
            return 'Object: null prototype';
        }
        if (typeof proto.constructor === 'function') {
            return classNameOf(proto.constructor) ?? 'Object';
        }
    }
    return tag;
}
