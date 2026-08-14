/* eslint-disable no-console */
import { program } from '@commander-js/extra-typings';
import { lib } from '@mirascript/mirascript/subtle';
import { KEYWORDS, OPERATORS } from '@mirascript/help';
import styles from 'ansi-styles';
import { noColor } from '../utils/color.js';
import { print } from '../utils/print.js';

/** 打印表格 */
function printTable<T>(
    map: Record<string, T>,
    title: string,
    formatter: (value: T, key: string) => string = String,
): void {
    console.log(title);
    const keys = Object.keys(map).sort();
    const maxKeyLength = Math.max(...keys.map((k) => k.length));
    for (const key of keys) {
        const value = map[key];
        if (value === undefined) continue;
        const firstLine = formatter(value, key).split('\n')[0];
        let subject = key.padEnd(maxKeyLength);
        if (!noColor) {
            subject = styles.bold.open + subject + styles.bold.close;
        }
        console.log(`  ${subject}  ${firstLine}`);
    }
}

/** 打印库 */
function printLib(key: string): void {
    const value = lib[key as keyof typeof lib];
    if (value === undefined) return;
    if ('summary' in value) {
        if (typeof value == 'function') {
            console.log(`fn ${key}()`);
            console.log(value.summary ?? '');
        } else {
            console.log(`${key} = ${print(value.value)}`);
            console.log(value.summary ?? '');
        }
    } else {
        console.log(`模块 ${key}`);
    }
}

const command = program.command('man');
command
    .description('显示 MiraScript 的手册')
    .argument('<topic>', '要显示的主题')
    .action((topic) => {
        if (topic in KEYWORDS) {
            const desc = KEYWORDS[topic as keyof typeof KEYWORDS];
            console.log(desc);
        } else if (topic in OPERATORS) {
            const desc = OPERATORS[topic as keyof typeof OPERATORS];
            console.log(desc);
        } else if (topic in lib) {
            printLib(topic);
        } else if (topic === 'keywords') {
            printTable(KEYWORDS, 'MiraScript 关键字：');
        } else if (topic === 'operators') {
            printTable(OPERATORS, 'MiraScript 操作符：');
        } else if (topic === 'libraries') {
            printTable(lib, 'MiraScript 标准库：', (v, k) => {
                if ('summary' in v) {
                    return v.summary ?? k;
                }
                return `模块 ${k}`;
            });
        } else {
            command.help({ error: true });
        }
    });
