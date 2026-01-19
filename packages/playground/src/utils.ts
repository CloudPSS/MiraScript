import type { VmAny } from '@mirascript/mirascript';
import { monaco, mirascript, ready, mirascriptMonaco } from './loader.js';

/** HTML escape */
export function escapeHtml(value: string): string {
    return value
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
}

/** 将值转为语法高亮的显示 */
export async function print(value: VmAny | Error): Promise<string> {
    if (value === undefined) return `<span style="color: #888">&lt;uninitialized&gt;</span>`;
    if (value === null) return syntaxHighlight('nil', 'mirascript');
    if (value instanceof Error) return escapeHtml(value.toString());
    await ready;
    const { isVmExtern, isVmModule, serialize, isVmFunction, getVmFunctionInfo } = mirascript;
    if (isVmFunction(value)) {
        const info = getVmFunctionInfo(value)!;
        if (info.isLib) {
            return await syntaxHighlight(
                `fn ${value.name}(${Object.entries(info.params ?? {})
                    .map(([pn]) => {
                        const type = info.paramsType?.[pn];
                        return type ? `${pn}: ${type}` : pn;
                    })
                    .join(', ')})${info.returnsType ? ` -> ${info.returnsType}` : ''}`,
                'mirascript-doc',
            );
        } else {
            return syntaxHighlight(String(value), 'javascript');
        }
    }
    if (isVmExtern(value) || isVmModule(value)) {
        return await syntaxHighlight(`/* <${value.type} ${value.tag}> */`, 'mirascript-doc');
    }
    const valueStr = serialize(value);
    const model = monaco.editor.createModel(valueStr, 'mirascript');
    const { FormatterProvider } = mirascriptMonaco;
    try {
        const formatted = await FormatterProvider.format(model);
        if (formatted) {
            return syntaxHighlight(formatted, 'mirascript');
        }
    } finally {
        model.dispose();
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
