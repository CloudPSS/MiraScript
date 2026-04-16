# @mirascript/website

`@mirascript/website` 是 MiraScript 官方文档站点，基于 Docusaurus 构建，内容来自仓库根目录下的 `docs/`，并集成了在线代码编辑能力。

## 特性

- 使用 Docusaurus 构建中文文档站点
- 构建前自动提取示例代码与参考文档
- 集成本地搜索
- 支持在文档页面中嵌入 MiraScript 编辑器体验

## 本地开发

```bash
pnpm --filter @mirascript/website start
```

## 构建

```bash
pnpm --filter @mirascript/website build
pnpm --filter @mirascript/website serve
```

`build` 前会自动执行：

- `extract-code`
- `extract-doc`

用于从仓库文档和示例中生成网站所需数据。

## 目录说明

- `docusaurus.config.ts`：站点配置
- `src/`：主题、页面与前端逻辑
- `static/`：静态资源
- `scripts/`：文档与示例抽取脚本

如果你修改了 `docs/`、`examples/` 或网站主题代码，建议重新执行一次完整构建确认结果。
