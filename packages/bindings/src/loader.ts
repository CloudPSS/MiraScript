/** 模块加载结果 */
class ModuleResult<T> {
    constructor(
        readonly module: T | undefined,
        readonly error: Error | undefined,
    ) {}
    /** 获取模块，如果模块未加载则抛出错误 */
    get(): T {
        if (this.module != null) return this.module;
        throw this.error ?? new Error('Module is not loaded.');
    }
}

/** 运行模块加载器 */
async function runLoader<T>(loader: () => Promise<T>): Promise<ModuleResult<T>> {
    try {
        const module = await loader();
        return new ModuleResult(module, undefined);
    } catch (error) {
        return new ModuleResult<T>(undefined, error as Error);
    }
}
/** 模块加载器 */
export class ModuleLoader<T> {
    constructor(private readonly loader: () => Promise<T>) {}
    private loading: Promise<ModuleResult<T>> | undefined;
    private result: ModuleResult<T> | undefined;

    /** 加载模块 */
    readonly load = async (): Promise<T> => {
        const { result, loading, loader } = this;
        if (result?.module != null) return result.module;
        if (loading != null) return loading.then((r) => r.get());

        const l = runLoader(loader).then((r) => {
            if (this.loading === l) {
                this.loading = undefined;
                this.result = r;
            }
            return r;
        });
        this.loading = l;
        return l.then((r) => r.get());
    };
    /**
     * 获取已加载的模块
     * @throws {Error} 模块尚未加载
     */
    readonly get = (): T => {
        const { result } = this;
        if (result == null) {
            throw new Error('MiraScript compiler module is not loaded.');
        }
        return result.get();
    };
}
