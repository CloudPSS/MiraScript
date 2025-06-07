import { type editor, languages, type CancellationToken } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';

const REG_COLOR_STR = /^(@*)(['"`])(#(?:[0-9a-f]{6}|[0-9a-f]{3}|[0-9a-f]{8}|[0-9a-f]{4}))\2\1$/iu;

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

        if (colorCode.length === 3) {
            // 处理 #RGB 格式
            color = {
                red: Number.parseInt(colorCode[0]! + colorCode[0]!, 16) / 255,
                green: Number.parseInt(colorCode[1]! + colorCode[1]!, 16) / 255,
                blue: Number.parseInt(colorCode[2]! + colorCode[2]!, 16) / 255,
                alpha: 1,
            };
        } else if (colorCode.length === 6) {
            color = {
                red: Number.parseInt(colorCode.slice(0, 2), 16) / 255,
                green: Number.parseInt(colorCode.slice(2, 4), 16) / 255,
                blue: Number.parseInt(colorCode.slice(4, 6), 16) / 255,
                alpha: 1, // 假设不透明
            };
        } else if (colorCode.length === 8) {
            // 处理 #RRGGBBAA 格式
            color = {
                red: Number.parseInt(colorCode.slice(0, 2), 16) / 255,
                green: Number.parseInt(colorCode.slice(2, 4), 16) / 255,
                blue: Number.parseInt(colorCode.slice(4, 6), 16) / 255,
                alpha: Number.parseInt(colorCode.slice(6, 8), 16) / 255,
            };
        } else if (colorCode.length === 4) {
            // 处理 #RGBA 格式
            color = {
                red: Number.parseInt(colorCode[0]! + colorCode[0]!, 16) / 255,
                green: Number.parseInt(colorCode[1]! + colorCode[1]!, 16) / 255,
                blue: Number.parseInt(colorCode[2]! + colorCode[2]!, 16) / 255,
                alpha: Number.parseInt(colorCode[3]! + colorCode[3]!, 16) / 255,
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

/** @inheritdoc */
class ColorProvider extends Provider implements languages.DocumentColorProvider {
    /** @inheritdoc */
    async provideDocumentColors(
        model: editor.ITextModel,
        token: CancellationToken,
    ): Promise<languages.IColorInformation[] | undefined> {
        const compiled = await Provider.getCompileResult(model);
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
        return [
            {
                label: `#${Math.round(color.red * 255)
                    .toString(16)
                    .padStart(2, '0')}${Math.round(color.green * 255)
                    .toString(16)
                    .padStart(2, '0')}${Math.round(color.blue * 255)
                    .toString(16)
                    .padStart(2, '0')}${
                    color.alpha >= 1
                        ? ''
                        : Math.round(color.alpha * 255)
                              .toString(16)
                              .padStart(2, '0')
                }`,
            },
        ];
    }
}

languages.registerColorProvider('mirascript', new ColorProvider());
