import { Disposable } from '#loader';

/** Disposable 基类 */
export class DisposableManager extends Disposable {
    constructor() {
        super(() => {
            for (const disposable of this.#disposables) {
                disposable.dispose();
            }
        });
    }
    readonly #disposables = new Set<Disposable>();
    /** 添加 Disposable */
    protected addDisposables(...disposables: Disposable[]): void {
        for (const disposable of disposables) {
            this.#disposables.add(disposable);
        }
    }
}
