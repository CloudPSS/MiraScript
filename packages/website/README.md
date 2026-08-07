# @mirascript/website

`@mirascript/website` 是 MiraScript 官方文档站点，基于 Docusaurus 构建，文档内容来自仓库中的 `docs/`、`blog/` 与 `examples/`。

## 能力概览

- 构建中文文档站点
- 构建前自动抽取示例代码与参考文档
- 集成本地搜索
- 提供在线编辑体验组件

## 本地开发

在线编辑器的 Python 源代码展示需要 Pyodide wheel。首次构建前，在 `crates/python` 中执行 `pyodide build`，并将
`typing_extensions` 的纯 Python wheel 下载到 `crates/python/dist`。也可以通过
`MIRASCRIPT_PYODIDE_WHEELS` 指向同时包含这两个 wheel 的目录。

```bash
pnpm --filter @mirascript/website start
```

## 构建与预览

```bash
pnpm --filter @mirascript/website build
pnpm --filter @mirascript/website serve
```

`start` 和 `build` 会先执行 `generate`，该步骤包含：

- `extract-code`
- `extract-doc`

## 目录说明

- `docusaurus.config.ts`：站点配置
- `src/`：页面与主题逻辑
- `static/`：静态资源
- `scripts/`：文档与示例抽取脚本
