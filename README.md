<div align="center">
	<img src="packages/website/static/favicon.svg" width="128" height="128" alt="MiraScript Logo" />

  <h1>MiraScript</h1>

  <p>
    <a href="https://www.npmjs.com/package/@mirascript/mirascript"><img alt="npm @mirascript/mirascript" src="https://img.shields.io/npm/v/%40mirascript%2Fmirascript?style=for-the-badge&logo=npm&label=%40mirascript%2Fmirascript" /></a>
    <a href="https://pypi.org/project/mirascript/"><img alt="PyPI mirascript" src="https://img.shields.io/pypi/v/mirascript?style=for-the-badge&logo=pypi&label=mirascript" /></a>
    <a href="https://open-vsx.org/extension/CloudPSS/mirascript"><img alt="Open VSX MiraScript" src="https://img.shields.io/open-vsx/v/CloudPSS/mirascript?style=for-the-badge&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHZpZXdCb3g9JzAgMCAxMzYgMTM2Jz48cGF0aCBkPSdNMzAgNDQuMkw1Mi42IDVINy4zek00LjYgODguNWg0NS4zTDI3LjIgNDkuNHptNTEgMGwyMi42IDM5LjIgMjIuNi0zOS4yeicgZmlsbD0nI2MxNjBlZic%2BPC9wYXRoPjxwYXRoIGQ9J001Mi42IDVMMzAgNDQuMmg0NS4yek0yNy4yIDQ5LjRsMjIuNyAzOS4xIDIyLjYtMzkuMXptNTEgMEw1NS42IDg4LjVoNDUuMnonIGZpbGw9JyNhNjBlZTUnPjwvcGF0aD48L3N2Zz4%3D&label=MiraScript" /></a>
    <a href="LICENSE"><img alt="License MIT" src="https://img.shields.io/badge/License-MIT-2563eb?style=for-the-badge" /></a>
  </p>

  <p>
    <b>
      表达式优先、不可变数据为核心的现代脚本语言<br>
      简洁、安全、易于嵌入
    </b>
  </p>

<hr>

</div>

MiraScript 是 CloudPSS 维护的表达式优先脚本语言项目。当前仓库包含语言核心、编译器绑定、命令行工具、编辑器集成、在线 Playground、文档站点与测试集。

## 目录结构

- `crates/`：Rust 侧实现，包括编译器核心、常量、NAPI/WASM/Python 绑定等。
- `packages/`：JavaScript/TypeScript 生态包，包括核心 API、CLI、编辑器集成与站点。
- `docs/`：官方文档源文件。
- `examples/`：可直接运行的示例脚本。
- `tests/`：语言特性、逻辑与端到端测试数据。

## crates 一览

| Crate 名称       | 目录               | 说明                                  |
| ---------------- | ------------------ | ------------------------------------- |
| `mira-core`      | `crates/core`      | 语言核心实现，包含词法/语法/执行能力  |
| `mira-constants` | `crates/constants` | Rust 侧常量导出，供上层包生成常量数据 |
| `mira-napi`      | `crates/napi`      | Node.js 原生绑定（N-API）             |
| `mira-wasm`      | `crates/wasm`      | WebAssembly 绑定                      |
| `mira-python`    | `crates/python`    | Python 绑定                           |

## packages 一览

| 包名                     | 目录                  | 说明                                        |
| ------------------------ | --------------------- | ------------------------------------------- |
| `@mirascript/bindings`   | `packages/bindings`   | 统一封装 Node 原生与 WebAssembly 编译器绑定 |
| `@mirascript/cli`        | `packages/cli`        | 命令行工具，提供运行与格式化能力            |
| `@mirascript/constants`  | `packages/constants`  | 编译器常量、关键字与诊断码导出              |
| `@mirascript/help`       | `packages/help`       | 从文档生成关键字与运算符帮助数据            |
| `@mirascript/mirascript` | `packages/mirascript` | 语言核心 TypeScript API                     |
| `@mirascript/monaco`     | `packages/monaco`     | Monaco Editor 集成                          |
| `@mirascript/napi`       | `packages/napi`       | Node.js 原生编译器包                        |
| `@mirascript/playground` | `packages/playground` | 在线 Playground 前端                        |
| `@mirascript/typed`      | `packages/typed`      | 类型定义解析与 JSON Schema 生成             |
| `@mirascript/vscode`     | `packages/vscode`     | Visual Studio Code 扩展                     |
| `@mirascript/wasm`       | `packages/wasm`       | WebAssembly 编译器包                        |
| `@mirascript/website`    | `packages/website`    | Docusaurus 文档站点                         |

## 开发环境

建议使用以下版本：

- Node.js 20 及以上
- pnpm 11
- Rust stable toolchain
- `wasm-pack`（构建 `@mirascript/constants` 与 `@mirascript/wasm` 时需要）

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

常见子包场景：

- 构建 CLI：`pnpm --filter @mirascript/cli build`
- 构建语言核心：`pnpm --filter @mirascript/mirascript build`
- 启动 Playground：`pnpm --filter @mirascript/playground start`
- 启动文档站点：`pnpm --filter @mirascript/website start`

## 文档与示例

- 语言文档位于 `docs/`
- 示例脚本位于 `examples/`
- VS Code 扩展说明见 `packages/vscode/README.md`
- Python 绑定说明见 `crates/python/README.md`

## 许可证

本项目使用 MIT 许可证，详见 `LICENSE`。
