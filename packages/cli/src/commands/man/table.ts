import styles from 'ansi-styles';
import { write } from '../../utils/color.js';

/** 打印表格 */
export function printTable<T>(
    map: Record<string, T>,
    title: string,
    formatter: (value: T, key: string) => string = String,
): void {
    if (title) {
        write(title + '\n', styles.bold);
    }

    const keys = Object.keys(map).sort();
    const maxKeyLength = Math.max(...keys.map((k) => k.length));

    for (const key of keys) {
        const value = map[key];
        if (value === undefined) continue;

        const firstLine = formatter(value, key).split('\n')[0];
        write('  ', styles.dim);
        write(key, styles.greenBright);
        write(' '.repeat(maxKeyLength - key.length), styles.dim);
        write('  ' + firstLine + '\n');
    }
}
