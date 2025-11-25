import type { BcModule } from '@mirascript/bindings/wasm';
import type { VmAny } from '@mirascript/mirascript';
import { monaco, mirascript, ready, mirascriptBc } from './loader.js';

/** HTML escape */
export function escapeHtml(value: string): string {
    return value
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
}

let formatConfig: BcModule.WasmConfig;
/** 将值转为语法高亮的显示 */
export async function print(value: VmAny | Error): Promise<string> {
    if (value === undefined) return escapeHtml('<uninitialized>');
    if (value === null) return syntaxHighlight('nil', 'mirascript');
    if (value instanceof Error) return escapeHtml(value.toString());
    if (typeof value == 'function') {
        return syntaxHighlight(String(value), 'javascript');
    }
    await ready;
    const { isVmExtern, isVmModule, serialize } = mirascript;
    if (isVmExtern(value) || isVmModule(value)) {
        return await syntaxHighlight(`/* <${value.type} ${value.describe}> */`, 'mirascript-doc');
    }
    const valueStr = serialize(value);
    const { wasm, createConfig } = mirascriptBc;
    formatConfig ??= createConfig({
        input_mode: 'Script',
        trivia: true,
        diagnostic_reference: false,
        diagnostic_tag: false,
        diagnostic_sourcemap: false,
        diagnostic_position_encoding: 'None',
    });
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

void ready.then(() => {
    // 加载 javascript 模型以便语法高亮
    monaco.editor.createModel('', 'javascript').dispose();
});
/** 语法高亮 */
export async function syntaxHighlight(value: string, languageId: string): Promise<string> {
    await ready;
    let highlighted = await monaco.editor.colorize(value, languageId, { tabSize: 2 });
    if (highlighted.endsWith('<br/>') && !value.endsWith('\n')) {
        // 如果高亮结果以 <br/> 结尾且原始值没有换行符，则去掉 <br/>
        highlighted = highlighted.slice(0, -'<br/>'.length);
    }
    highlighted = highlighted.replaceAll(/(\u00A0)+/g, '$&<wbr>');
    return highlighted;
}
