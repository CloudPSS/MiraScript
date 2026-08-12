import type { editor, languages, CancellationToken } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/constants';

const REG_COLOR_STR = /^(@*)(['"`])(#(?:[0-9a-f]{6}|[0-9a-f]{3}|[0-9a-f]{8}|[0-9a-f]{4}))\2\1$/iu;
const { parseInt } = Number;

/** 解析颜色 */
function parseColorPart(text: string): number {
    if (text.length === 1) {
        // 处理单字符的情况，例如 #f00 -> #ff0000
        text = text + text;
    }
    return parseInt(text, 16) / 255;
}

/** 解析颜色 */
function parseColorString(text: string):
    | {
          ats: string;
          quote: string;
          colorString: string;
          color: languages.IColor;
      }
    | undefined {
    const colorMatch = REG_COLOR_STR.exec(text);
    if (!colorMatch) {
        return undefined;
    }
    const ats = colorMatch[1] || '';
    const quote = colorMatch[2] || '';
    const colorString = colorMatch[3]!;
    let color: languages.IColor;
    if (colorString.startsWith('#')) {
        const colorCode = colorString.slice(1);

        if (colorCode.length === 3 || colorCode.length === 4) {
            // 处理短格式的颜色代码，例如 #f00 或 #f00f
            color = {
                red: parseColorPart(colorCode[0]!),
                green: parseColorPart(colorCode[1]!),
                blue: parseColorPart(colorCode[2]!),
                alpha: colorCode[3] ? parseColorPart(colorCode[3]) : 1, // 如果没有 alpha，则假设不透明
            };
        } else if (colorCode.length === 6 || colorCode.length === 8) {
            // 处理长格式的颜色代码，例如 #ff0000 或 #ff0000ff
            color = {
                red: parseColorPart(colorCode.slice(0, 2)),
                green: parseColorPart(colorCode.slice(2, 4)),
                blue: parseColorPart(colorCode.slice(4, 6)),
                alpha: colorCode.length === 8 ? parseColorPart(colorCode.slice(6, 8)) : 1, // 如果没有 alpha，则假设不透明
            };
        } else {
            return undefined; // 不支持的颜色格式
        }
    } else {
        return undefined; // 不是以 # 开头的颜色字符串
    }

    return {
        ats,
        quote,
        colorString,
        color,
    };
}

/** 生成颜色 */
function serializeColorPart(part: number): string {
    const intPart = Math.round(part * 255);
    const hex = intPart.toString(16).padStart(2, '0');
    return hex;
}

/** 生成颜色 */
function serializeColor(color: languages.IColor): string {
    const r = serializeColorPart(color.red);
    const g = serializeColorPart(color.green);
    const b = serializeColorPart(color.blue);
    if (color.alpha >= 1) {
        return `#${r}${g}${b}`;
    } else {
        const a = serializeColorPart(color.alpha);
        return `#${r}${g}${b}${a}`;
    }
}

/** @inheritdoc */
export class ColorProvider extends Provider implements languages.DocumentColorProvider {
    /** @inheritdoc */
    async provideDocumentColors(
        model: editor.ITextModel,
        token: CancellationToken,
    ): Promise<languages.IColorInformation[] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const info: languages.IColorInformation[] = [];
        for (const { range, code } of compiled.groupedTags(model).ranges) {
            if (code !== DiagnosticCode.String) continue;
            if (range.startLineNumber !== range.endLineNumber) {
                // 只处理单行字符串
                continue;
            }
            const text = model.getValueInRange(range);
            const parsed = parseColorString(text);
            if (!parsed) continue;
            info.push({
                range: {
                    startLineNumber: range.startLineNumber,
                    startColumn: range.startColumn + parsed.ats.length + parsed.quote.length,
                    endLineNumber: range.endLineNumber,
                    endColumn: range.endColumn - parsed.ats.length - parsed.quote.length,
                },
                color: parsed.color,
            });
        }
        return info;
    }
    /** @inheritdoc */
    provideColorPresentations(
        model: editor.ITextModel,
        colorInfo: languages.IColorInformation,
        token: CancellationToken,
    ): languages.ProviderResult<languages.IColorPresentation[]> {
        const { color } = colorInfo;
        return [{ label: serializeColor(color) }];
    }
}
