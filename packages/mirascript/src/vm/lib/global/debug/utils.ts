import type { VmAny } from '../../../types/index.js';
import { wrapEffect } from '../../../effects/wrap.js';
import { rethrowControl } from '../../../effects/state.js';

/** 序列化格式 */
type SerializeFormat =
    /** 表示未指定格式 */
    | ''
    /** 以 `%` 开头的格式占位符 */
    | `%${string}`;

/** 默认的序列化函数 */
function defaultSerializer(arg: VmAny, format: SerializeFormat): string | null {
    return null;
}

/** 格式化结果 */
interface FormatResult {
    /** 模板字符串数组 */
    templates: readonly string[];
    /** 值数组，长度为 {@link templates}.length - 1 */
    values: readonly VmAny[];
    /** 格式化字符串数组，与 {@link values} 一一对应 */
    formats: readonly SerializeFormat[];
}

/** 序列化值 */
function serializeValue(
    options: PrintOptions,
    arg: VmAny,
    format: SerializeFormat,
): string | null | PromiseLike<string> {
    const { serializer } = options;
    if (serializer === defaultSerializer || serializer === null) {
        return defaultSerializer(arg, format);
    }
    try {
        return serializer.call(options, arg, format);
    } catch (error) {
        rethrowControl(error);
        return defaultSerializer(arg, format);
    }
}

/** 构造格式化字符串 */
function buildFormatString(
    options: PrintOptions,
    args: readonly VmAny[],
): readonly [format: string, values: readonly VmAny[]] {
    const [prefix, ...additional] = options.prefix;
    if (typeof args[0] != 'string' || !args[0].includes('%')) {
        return [prefix || '', [...additional, ...args]];
    } else {
        const [arg0, ...argsRest] = args;
        const values = [...additional, ...argsRest];
        if (!prefix) {
            return [arg0, argsRest];
        } else {
            return [`${prefix} ${arg0}`, values];
        }
    }
}

/** 默认解析函数 */
const printParser: PrintOptions['parser'] = function (args) {
    const [format, values] = buildFormatString(this, args);
    const templates: string[] = [];
    const formats: SerializeFormat[] = [];
    let valIndex = 0;
    if (format.includes('%')) {
        const parts = format.split(/(%[%\w])/g);
        let prevIsTemplate = true;
        for (let i = 0; i < parts.length; i++) {
            if (i % 2 === 0) {
                // Regular string part
                if (prevIsTemplate && templates.length) {
                    templates[templates.length - 1] += parts[i]!;
                } else {
                    templates.push(parts[i]!);
                }
                prevIsTemplate = true;
                continue;
            }
            // Specifier part
            let specifier = parts[i]!;
            if (specifier === '%%' || valIndex >= values.length) {
                if (specifier === '%%') specifier = '%';
                if (!templates.length) {
                    templates.push(specifier);
                } else {
                    templates[templates.length - 1] += specifier;
                }
                prevIsTemplate = true;
                continue;
            }
            prevIsTemplate = false;
            formats.push(specifier as SerializeFormat);
            valIndex++;
        }
    } else if (format) {
        templates.push(format);
    }
    // Append any remaining arguments separated by spaces
    if (valIndex < values.length) {
        if (templates.length) {
            templates[templates.length - 1] += ' ';
        } else {
            templates.push('');
        }
        for (let i = valIndex; i < values.length; i++) {
            formats.push('');
            templates.push(i < values.length - 1 ? ' ' : '');
        }
    }
    return { templates, values, formats };
};

/** 默认格式化函数 */
const printFormatter: PrintOptions['formatter'] = function ({
    templates,
    formats,
    values,
}): unknown[] | PromiseLike<unknown[]> {
    let format = '';
    const formattedValues: unknown[] = [];
    let needAwait = false;
    for (let i = 0; i < templates.length; i++) {
        format += templates[i]!.replaceAll('%', '%%');
        if (i < values.length) {
            const f = formats[i] ?? '';
            const v = values[i]!;
            const serialized = serializeValue(this, v, f);
            if (serialized == null) {
                format += f || (typeof v == 'string' ? '%s' : '%o');
                formattedValues.push(v);
            } else if (typeof serialized == 'string') {
                format += '%s';
                formattedValues.push(serialized);
            } else {
                needAwait = true;
                format += '%s';
                formattedValues.push(serialized);
            }
        }
    }
    if (!needAwait) {
        return [format, ...formattedValues];
    } else {
        return Promise.all(formattedValues).then((resolvedValues) => {
            return [format, ...resolvedValues];
        });
    }
};

/** 默认的输出选项 */
export const createPrintOptions = (consoleMethod: (...args: unknown[]) => void): PrintOptions => {
    return {
        prefix: ['MiraScript'],
        serializer: defaultSerializer,
        parser: printParser,
        formatter: printFormatter,
        logger: consoleMethod,
    };
};

/** 打印输出选项 */
export type PrintOptions = {
    /** 输出时的固定前缀 */
    prefix: readonly [prefix?: string, ...additional: readonly string[]];
    /**
     * 序列化函数
     * @param arg 要序列化的值
     * @param format 序列化格式
     * @returns 序列化后的字符串，或 null 表示直接将原值传递给 {@link printer}
     */
    serializer: (this: PrintOptions, arg: VmAny, format: SerializeFormat) => string | null | PromiseLike<string>;
    /** 解析函数，解析输入并使用 {@link serializer} 序列化值，供 {@link formatter} 使用 */
    parser: (this: PrintOptions, args: readonly VmAny[]) => FormatResult;
    /** 格式化函数，进一步处理解析结果，供 {@link logger} 使用 */
    formatter: (this: PrintOptions, format: FormatResult) => unknown[] | PromiseLike<unknown[]>;
    /** 打印函数，打印格式化结果 */
    logger: (this: PrintOptions, ...args: unknown[]) => void;
};

/** 进行输出 */
export const doPrint = wrapEffect('once', (opt: PrintOptions, args: readonly VmAny[]): void => {
    const formatResult = opt.parser(args);
    void Promise.resolve(opt.formatter(formatResult)).then((printed) => opt.logger(...printed));
});
