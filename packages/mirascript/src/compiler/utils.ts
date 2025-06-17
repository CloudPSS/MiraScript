import { DiagnosticCode, wasm } from '@mirascript/wasm';

/** 所有 MiraScript 关键字 */
export let keywords = (): readonly string[] => {
    const kw = wasm.keywords();
    Object.freeze(kw);
    keywords = () => kw;
    return kw;
};

/** MiraScript 控制流关键字 */
export let controlKeywords = (): readonly string[] => {
    const kw = wasm.control_keywords();
    Object.freeze(kw);
    controlKeywords = () => kw;
    return kw;
};

/** MiraScript 数值字面量关键字 */
export let numericKeywords = (): readonly string[] => {
    const kw = wasm.numeric_keywords();
    Object.freeze(kw);
    numericKeywords = () => kw;
    return kw;
};

/** MiraScript 字面量关键字 */
export let constantKeywords = (): readonly string[] => {
    const kw = wasm.constant_keywords();
    Object.freeze(kw);
    constantKeywords = () => kw;
    return kw;
};

/** MiraScript 保留字关键字 */
export let reservedKeywords = (): readonly string[] => {
    const kw = wasm.reserved_keywords();
    Object.freeze(kw);
    reservedKeywords = () => kw;
    return kw;
};

export { DiagnosticCode };
const diagnosticMessages = new Map<DiagnosticCode, string | undefined>();
/** 获取 {@link DiagnosticCode} 对应的消息 */
export function getDiagnosticMessage(code: DiagnosticCode): string | undefined {
    if (!Number.isSafeInteger(code) || code < 0 || code >= 0xffff) {
        throw new RangeError(`Invalid DiagnosticCode: ${code}`);
    }
    if (diagnosticMessages.has(code)) {
        return diagnosticMessages.get(code);
    }
    const msg = wasm.get_diagnostic_message(code);
    diagnosticMessages.set(code, msg);
    return msg;
}
