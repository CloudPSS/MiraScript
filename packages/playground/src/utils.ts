import { type VmAny, isVmExtern, isVmModule, serialize } from '@mirascript/mirascript';
import { createConfig, wasm } from '@mirascript/wasm';
import { editor } from '@private/monaco-editor';

/** HTML escape */
export function escapeHtml(value: string): string {
    return value
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
}

const formatConfig = createConfig({
    input_mode: 'Script',
    trivia: true,
    track_references: false,
    diagnostic_reference: false,
    diagnostic_position_encoding: 'None',
});
/** 将值转为语法高亮的显示 */
export async function print(value: VmAny | Error): Promise<string> {
    if (value === undefined) return escapeHtml('<uninitialized>');
    if (value === null) return syntaxHighlight('nil', 'mirascript');
    if (value instanceof Error) return escapeHtml(value.toString());
    if (typeof value == 'function') {
        return syntaxHighlight(String(value), 'javascript');
    }
    if (isVmExtern(value) || isVmModule(value)) {
        const colorized = await syntaxHighlight('\0/* ' + value.toString() + ' */', 'mirascript');
        return colorized.replace('>&#00;<', '><');
    }
    const valueStr = serialize(value);
    const formatter = new wasm.MonacoCompiler(valueStr, formatConfig);
    try {
        const formatted = formatter.parse() && formatter.format();
        if (formatted) {
            return syntaxHighlight(formatted, 'mirascript');
        }
    } finally {
        formatter.free();
    }
    return syntaxHighlight(serialize(valueStr), 'mirascript');
}

// 加载 javascript 模型以便语法高亮
editor.createModel('', 'javascript').dispose();
/** 语法高亮 */
export async function syntaxHighlight(value: string, languageId: string): Promise<string> {
    let highlighted = await editor.colorize(value, languageId, {});
    if (highlighted.endsWith('<br/>') && !value.endsWith('\n')) {
        // 如果高亮结果以 <br/> 结尾且原始值没有换行符，则去掉 <br/>
        highlighted = highlighted.slice(0, -'<br/>'.length);
    }
    highlighted = highlighted.replaceAll(/(\u00A0)+/g, '$&<wbr>');
    return highlighted;
}
