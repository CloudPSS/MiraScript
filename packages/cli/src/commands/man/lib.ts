import styles from 'ansi-styles';
import { format, write } from '../../utils/color.js';
import { print } from '../../utils/print.js';
import { lib } from '@mirascript/mirascript/subtle';
import { printTable } from './table.js';

/** 分析路径 */
function analyzePath(path: string): readonly string[] {
    const segments = path
        .split('.')
        .map((s) => s.trim())
        .filter((s) => s.length > 0);
    if (segments.length === 0) return [];
    if (segments[0] === 'lib') {
        return segments.slice(1);
    }
    return segments;
}

/** 获取库值 */
function getLibValue(path: readonly string[]): (typeof lib)[keyof typeof lib] | undefined {
    let value: object = lib;
    for (const seg of path) {
        if (value === undefined) return undefined;
        value = value[seg.trim() as keyof typeof value];
    }
    return value as (typeof lib)[keyof typeof lib] | undefined;
}

/** 打印关键字 */
function printKw(kw: string): void {
    write(kw, styles.blueBright);
}

/** 打印注释 */
function printComment(comment: string): void {
    const lines = comment.split('\n');
    for (const line of lines) {
        write('// ', styles.dim);
        write(line, styles.gray);
        write('\n');
    }
}

/** 打印库 */
export function printLib(name: string): boolean {
    const path = analyzePath(name);
    const value = getLibValue(path);
    if (value === undefined) return false;
    if ('value' in value) {
        if (value.summary) {
            printComment(value.summary);
        }
        printKw('let');
        write(' ');
        write(path.join('.'), styles.cyanBright);
        write(' = ');
        write(print(value.value));
        write(';\n');
    } else if ('summary' in value) {
        if (value.summary) {
            printComment(value.summary);
        }
        const params = value.params ? Object.entries(value.params) : null;
        for (const [name, param] of params ?? []) {
            if (!param.description) continue;
            const comment = `\`${name}\` - ${param.description}`;
            printComment(comment);
        }
        if (value.returns?.description) {
            printComment(`返回值 - ${value.returns.description}`);
        }
        printKw('fn');
        write(' ');
        write(path.join('.'), styles.yellowBright);
        write('(');
        if (!params) {
            write('..');
        } else {
            for (let i = 0; i < params.length; i++) {
                const [name, param] = params[i]!;
                if (i > 0) {
                    write(', ');
                }
                write(format(name, styles.italic), styles.cyanBright);
                write(': ');
                write(param.type, styles.greenBright);
            }
        }
        write(')');
        if (value.returns) {
            write(' -> ');
            write(value.returns.type, styles.greenBright);
        }
        write(';\n');
        if (value.examples?.length) {
            for (const example of value.examples) {
                printComment(example);
            }
        }
    } else {
        printKw('mod');
        write(' ');
        write(path.join('.'), styles.cyanBright);
        write(' {\n');
        printTable(value as Record<string, { summary: string }>, '', (v) => v.summary);
        write('}\n');
    }
    return true;
}
