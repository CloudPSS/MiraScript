import type { CSPair } from 'ansi-styles';
import supportsColor from 'supports-color';
export const noColor = !supportsColor.stdout;

/** 格式化 */
export function format(text: string, style?: CSPair): string {
    if (style && !noColor) {
        return style.open + text + style.close;
    } else {
        return text;
    }
}
/** 写入 */
export function write(text: string, style?: CSPair): void {
    process.stdout.write(format(text, style));
}
