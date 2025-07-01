const timeFormatter = new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3,
});

/** 管理控制台输出的类 */
export class ConsoleManager {
    private entries: Array<{
        type: 'log' | 'error' | 'warn' | 'info';
        message: Promise<string> | string;
        timestamp: Date;
    }> = [];
    private readonly outputElement: HTMLElement;

    constructor(outputElement: HTMLElement) {
        this.outputElement = outputElement;
    }

    /** 添加日志消息 */
    log(message: Promise<string> | string): void {
        this.addEntry('log', message);
    }

    /** 添加错误消息 */
    error(message: Promise<string> | string): void {
        this.addEntry('error', message);
    }

    /** 添加警告消息 */
    warn(message: Promise<string> | string): void {
        this.addEntry('warn', message);
    }

    /** 添加信息消息 */
    info(message: Promise<string> | string): void {
        this.addEntry('info', message);
    }

    /** 添加条目到控制台 */
    private addEntry(type: 'log' | 'error' | 'warn' | 'info', message: Promise<string> | string): void {
        const entry = {
            type,
            message,
            timestamp: new Date(),
        };
        this.entries.push(entry);
    }

    /** 清空控制台 */
    clear(): void {
        this.entries = [];
    }
    /** 渲染控制台内容 */
    async render(): Promise<void> {
        const htmlArray = this.entries.map(async (entry) => {
            const time = timeFormatter.format(entry.timestamp);
            return /* html */ `<div class="console-entry ${entry.type}">
                <time class="console-time" datetime=${entry.timestamp.toISOString()}>[${time}]</time>
                <span class="console-message">${await entry.message}</span>
            </div>`;
        });

        const html = await Promise.all(htmlArray).then((lines) => lines.join(''));
        this.outputElement.innerHTML = html;
        this.outputElement.scrollTop = this.outputElement.scrollHeight;
    }
}
