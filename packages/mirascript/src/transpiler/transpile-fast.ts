import { wrapScript } from './create-script.js';
import { REG_NUMBER } from '../helpers/constants.js';
import type { TranspileOptions } from './types.js';
import type { VmScript } from '../vm/types/index.js';

const REG_NUMBER_FULL = new RegExp(`^${REG_NUMBER.source}$`, 'ug');

// 为避免结果不一致，只对常量进行处理

// 根据目前的 benchmark 结果：
//   对长度满足 FAST_MAX_LEN 的代码：
//       未命中时：产生大约 5% 的损失
//       命中时：提升性能 12 倍
//   对长度超过 FAST_MAX_LEN 的代码（未调用此函数）：未产生可观测的影响

/**
 * 对短代码进行编译
 */
export function transpileFast(code: string, options: TranspileOptions): VmScript | undefined {
    const trimmedCode = code.trim();
    if (!code) return wrapScript(code, () => null);
    switch (code) {
        case '':
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
    if (REG_NUMBER_FULL.test(trimmedCode)) {
        const num = Number(trimmedCode);
        if (Number.isNaN(num)) return undefined;
        // 直接返回数字
        return wrapScript(code, () => num);
    }
    return undefined;
}
