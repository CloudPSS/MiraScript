import { wrapScript, type VmScript } from './create-script.js';
import type { TranspileOptions } from './types.js';
import { $GlobalFallback } from '../vm/operations/index.js';
import type { VmContext, VmValue } from '../vm/index.js';
import { defineProperty, isFinite } from '../helpers/utils.js';
import { keywords } from './keywords.js';

const REG_NUMBER_FULL = /^(?:[+-])?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?$/;
// 只识别 ASCII 范围内的标识符，以提升性能
const REG_IDENTIFIER_FAST = /^(?:\$+|@+|[A-Za-z])[a-zA-Z0-9_]*$/;

// 为避免结果不一致，只对常量进行处理

// 根据目前的 benchmark 结果：
//   对长度满足 FAST_SCRIPT_MAX_LEN 的代码：
//       未命中时：产生大约 5% 的损失
//       命中时：提升性能 12 倍
//   对长度超过 FAST_SCRIPT_MAX_LEN 的代码（未调用此函数）：未产生可观测的影响

const FAST_SCRIPT_MAX_LEN = 32;

/** 构造返回常量的函数 */
function constantFiniteNumber(value: number): () => number {
    if (value === 0) {
        if (Object.is(value, -0)) {
            return () => -0;
        } else {
            return () => 0;
        }
    }
    const f = () => value;
    defineProperty(f, 'toString', {
        value: () => {
            return `() => ${value};`;
        },
        writable: false,
        enumerable: false,
        configurable: false,
    });
    return f;
}
/** 构造返回常量的函数 */
function constantString(value: string): () => string {
    const f = () => value;
    defineProperty(f, 'toString', {
        value: () => {
            return `() => ${JSON.stringify(value)};`;
        },
        writable: false,
        enumerable: false,
        configurable: false,
    });
    return f;
}
/** 构造返回常量的函数 */
function nan(): () => number {
    return () => 0 / 0;
}

/** 构造返回常量的函数 */
function posInf(): () => number {
    return () => +1 / 0;
}

/** 构造返回常量的函数 */
function negInf(): () => number {
    return () => -1 / 0;
}

/** 构造返回常量的函数 */
function constantBoolean(value: boolean): () => boolean {
    if (value) {
        return () => true;
    } else {
        return () => false;
    }
}

/** 构造返回常量的函数 */
function nil(): () => null {
    return () => null;
}

/** 构造返回全局变量的函数 */
function globalVariable(id: string): (global: VmContext | undefined) => VmValue {
    const f = (global = $GlobalFallback()): VmValue => global.get(id);
    defineProperty(f, 'toString', {
        value: () => {
            return `(global = $GlobalFallback()) => global.get(${JSON.stringify(id)});`;
        },
        writable: false,
        enumerable: false,
        configurable: false,
    });
    return f;
}

let kw: Set<string> | undefined;

/**
 * 对短代码进行编译
 */
function compileScriptFast(code: string, options: TranspileOptions): VmScript | undefined {
    if (code.length > FAST_SCRIPT_MAX_LEN) return undefined; // 超过长度限制，直接返回 undefined
    const mode = options.input_mode ?? 'Script';
    const trimmedCode = code.trim();
    if (!trimmedCode) {
        return wrapScript(code, mode, nil());
    }
    switch (trimmedCode) {
        case 'nil':
            return wrapScript(code, mode, nil());
        case 'true':
            return wrapScript(code, mode, constantBoolean(true));
        case 'false':
            return wrapScript(code, mode, constantBoolean(false));
        case 'nan':
            return wrapScript(code, mode, nan());
        case 'inf':
        case '+inf':
            return wrapScript(code, mode, posInf());
        case '-inf':
            return wrapScript(code, mode, negInf());
    }
    if (REG_IDENTIFIER_FAST.test(trimmedCode)) {
        // 直接返回标识符
        const id = trimmedCode;
        kw ??= new Set(keywords());
        if (kw.has(id)) {
            return undefined; // 关键字不处理
        }
        return wrapScript(code, mode, globalVariable(id));
    }
    if (REG_NUMBER_FULL.test(trimmedCode)) {
        const num = Number(trimmedCode);
        if (!isFinite(num)) return undefined;
        // 直接返回数字
        return wrapScript(code, mode, constantFiniteNumber(num));
    }
    return undefined;
}

const FAST_TEMPLATE_MAX_LEN = 1024;

/**
 * 对短代码进行编译
 */
function compileTemplateFast(code: string, options: TranspileOptions): VmScript | undefined {
    if (code.length > FAST_TEMPLATE_MAX_LEN) return undefined; // 超过长度限制，直接返回 undefined

    if (!code.includes('$')) {
        const mode = options.input_mode ?? 'Template';
        // 不包含插值的模板
        return wrapScript(code, mode, constantString(code));
    }
    return undefined;
}

/**
 * 对短代码进行编译
 */
export function compileFast(code: string, options: TranspileOptions): VmScript | undefined {
    if (options.sourceMap) return undefined; // 不支持源映射
    return (options.input_mode === 'Template' ? compileTemplateFast : compileScriptFast)(code, options);
}
