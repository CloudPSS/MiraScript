# MiraScript

MiraScript 是 CloudPSS 维护的表达式优先脚本语言项目，仓库同时包含语言核心、运行时、编译器绑定、命令行工具、编辑器集成、在线 Playground 和文档站点。

## 仓库内容

仓库主要由以下几部分组成：

- `crates/`：Rust 实现的编译器核心、CLI、常量、Node API、WASM 与 Python 绑定。
- `packages/`：面向 JavaScript/TypeScript 生态的包，包括语言核心、CLI、Monaco/VS Code 集成、文档站点等。
- `docs/`：官方文档源码，供网站与帮助包构建使用。
- `examples/`：MiraScript 示例程序。
- `tests/`：语言特性、逻辑和端到端测试。

## packages 一览

| 包名                     | 目录                  | 说明                                        |
| ------------------------ | --------------------- | ------------------------------------------- |
| `@mirascript/bindings`   | `packages/bindings`   | 统一封装 Node 原生与 WebAssembly 编译器绑定 |
| `@mirascript/cli`        | `packages/cli`        | 命令行工具，提供运行与格式化能力            |
| `@mirascript/constants`  | `packages/constants`  | 编译器常量、关键字与诊断码导出              |
| `@mirascript/help`       | `packages/help`       | 从文档生成的关键字与运算符帮助数据          |
| `@mirascript/mirascript` | `packages/mirascript` | 语言核心 TypeScript API                     |
| `@mirascript/monaco`     | `packages/monaco`     | Monaco Editor 集成                          |
| `@mirascript/napi`       | `packages/napi`       | Node.js 原生编译器包                        |
| `@mirascript/playground` | `packages/playground` | 在线 Playground 前端                        |
| `@mirascript/vscode`     | `packages/vscode`     | Visual Studio Code 扩展                     |
| `@mirascript/wasm`       | `packages/wasm`       | WebAssembly 编译器包                        |
| `@mirascript/website`    | `packages/website`    | Docusaurus 文档站点                         |

## 开发环境

建议使用以下工具版本：

- Node.js 20 及以上
- pnpm 10
- Rust stable toolchain
- `wasm-pack`（构建 `@mirascript/constants` 与 `@mirascript/wasm` 时需要）
- Docker Buildx（构建多平台 `@mirascript/napi` 发布产物时可选）

安装依赖：

```bash
pnpm install
```

## 常用命令

在仓库根目录执行：

```bash
pnpm run lint
pnpm run format
cargo fmt --all
```

各子包的构建命令定义在对应目录下的 `package.json` 中。常见场景：

- 构建 CLI：`pnpm --filter @mirascript/cli build`
- 构建语言核心：`pnpm --filter @mirascript/mirascript build`
- 启动 Playground：`pnpm --filter @mirascript/playground start`
- 启动文档站点：`pnpm --filter @mirascript/website start`

## 文档与示例

- 语言文档源码位于 `docs/`
- 示例程序位于 `examples/`
- VS Code 扩展说明见 `packages/vscode/README.md`
- Python 绑定说明见 `crates/python/README.md`

## 许可证

本项目使用 MIT 许可证。详见仓库根目录 `LICENSE`。
