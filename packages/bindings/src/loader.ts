/** 模块加载器 */
export class ModuleLoader<T> {
    constructor(private readonly loader: () => Promise<T>) {}
    private module: T | undefined;
    private loading: Promise<T> | undefined;

    /** 加载模块 */
    readonly load = async (): Promise<T> => {
        const { module, loading, loader } = this;
        if (module != null) return module;
        if (loading != null) return loading;

        const l = (async () => {
            const mod = await loader();
            this.module = mod;
            return mod;
        })();
        void l.finally(() => {
            if (this.loading === l) {
                this.loading = undefined;
            }
        });
        this.loading = l;
        return l;
    };
    /**
     * 获取已加载的模块
     * @throws {Error} 模块尚未加载
     */
    readonly get = (): T => {
        const { module } = this;
        if (module == null) {
            throw new Error('MiraScript compiler module is not loaded.');
        }
        return module;
    };
}
