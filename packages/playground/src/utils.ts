import { type VmAny, isVmExtern, isVmModule, serialize } from 'mirascript';
import { editor } from '@private/monaco-editor';

/** 将值转为语法高亮的显示 */
export async function print(value: VmAny | Error): Promise<string> {
    if (value === undefined) return '<uninitialized>';
    if (value === null) return syntaxHighlight('nil', 'mirascript');
    if (value instanceof Error) return value.toString();
    if (typeof value == 'function') {
        return syntaxHighlight(String(value), 'javascript');
    }
    if (isVmExtern(value) || isVmModule(value) || typeof value == 'function') {
        return String(value);
    }
    return syntaxHighlight(serialize(value), 'mirascript');
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
