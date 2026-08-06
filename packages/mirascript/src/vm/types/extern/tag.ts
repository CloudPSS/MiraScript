import { getPrototypeOf } from '../../../helpers/utils.js';

const ObjectPrototype = Object.prototype;

/** get js tag of value */
function getJsObjectTag(value: object): string {
    return ObjectPrototype.toString.call(value).slice(8, -1);
}

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

/** Get the tag of the extern object */
export function getTag(value: object): string {
    const tag = getJsObjectTag(value);
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
