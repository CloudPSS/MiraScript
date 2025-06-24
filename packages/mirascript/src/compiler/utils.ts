import { DiagnosticCode, wasm } from '@mirascript/wasm';

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
