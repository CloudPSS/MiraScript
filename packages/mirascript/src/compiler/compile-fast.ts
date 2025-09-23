import { wrapScript } from './create-script.js';
import type { TranspileOptions } from './types.js';
import type { VmScript } from '../vm/types/index.js';
import { GlobalFallback } from '../vm/helpers.js';
import { isFinite } from '../helpers/utils.js';

const REG_NUMBER_FULL = /^\d+(?:\.\d+)?(?:[eE][+-]?\d+)?$/;
// 只识别特殊变量名，其他标识符可能有与关键字冲突等情况，需要编译器处理
const REG_IDENTIFIER_FAST = /^(?:\$+|@+)[a-zA-Z0-9_]*$/;

// 为避免结果不一致，只对常量进行处理

// 根据目前的 benchmark 结果：
//   对长度满足 FAST_SCRIPT_MAX_LEN 的代码：
//       未命中时：产生大约 5% 的损失
//       命中时：提升性能 12 倍
//   对长度超过 FAST_SCRIPT_MAX_LEN 的代码（未调用此函数）：未产生可观测的影响

const FAST_SCRIPT_MAX_LEN = 32;

/**
 * 对短代码进行编译
 */
function compileScriptFast(code: string, options: TranspileOptions): VmScript | undefined {
    if (code.length > FAST_SCRIPT_MAX_LEN) return undefined; // 超过长度限制，直接返回 undefined

    const trimmedCode = code.trim();
    if (!trimmedCode) {
        return wrapScript(code, () => null);
    }
    switch (trimmedCode) {
        case 'nil':
            return wrapScript(code, () => null);
        case 'true':
            return wrapScript(code, () => true);
        case 'false':
            return wrapScript(code, () => false);
        case 'nan':
            return wrapScript(code, () => 0 / 0);
        case 'inf':
        case '+inf':
            return wrapScript(code, () => 1 / 0);
        case '-inf':
            return wrapScript(code, () => -1 / 0);
    }
    if (REG_IDENTIFIER_FAST.test(trimmedCode)) {
        // 直接返回标识符
        const id = trimmedCode;
        return wrapScript(code, (global = GlobalFallback()) => global.get(id) ?? null);
    }
    if (REG_NUMBER_FULL.test(trimmedCode)) {
        const num = Number(trimmedCode);
        if (!isFinite(num)) return undefined;
        // 直接返回数字
        return wrapScript(code, () => num);
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
        // 不包含插值的模板
        return wrapScript(code, () => code);
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
