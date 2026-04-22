---
name: mirascript-cli
description: Guide AI to use MiraScript CLI to run and format MiraScript code
license: MIT
---

# MiraScript CLI 技能

MiraScript CLI 是验证、运行和格式化 MiraScript 代码的官方命令行工具。本技能指导 AI **在生成代码后主动使用 CLI 验证代码正确性并根据错误迭代修复**。

## When to use

当需要执行、验证或格式化 MiraScript 代码时使用本技能，尤其是在生成代码后应立即用 CLI 进行验证。

---

## 安装与调用方式

无需全局安装，可直接通过包管理器调用最新版本：

```sh
# 推荐方式（三选一）
npx @mirascript/cli <command>
pnpm dlx @mirascript/cli <command>
yarn dlx @mirascript/cli <command>
```

若已在本地项目安装（`devDependencies`），则直接使用：

```sh
npx mirascript <command>
```

---

## 命令：`run`（默认命令）

执行 MiraScript 脚本，是 CLI 的默认命令（可省略 `run`）。

### 语法

```sh
mirascript run [options] [script]
# 或省略 run：
mirascript [options] [script]
```

### 选项

| 选项                       | 说明                                             |
| -------------------------- | ------------------------------------------------ |
| `-e, --eval <script>`      | 直接执行字符串代码（不传文件路径时使用）         |
| `-v, --variable <key=val>` | 设置全局变量，值支持 MiraScript 表达式；可多次用 |
| `-t, --template`           | 以模板模式运行（`.miratpl` 文件或模板字符串）    |
| `--no-template`            | 强制以脚本模式运行                               |
| `--timeout <ms>`           | 执行超时时间（毫秒），`0` 表示不超时，默认 3000  |

### 示例

```sh
# 执行脚本文件
mirascript script.mira

# 执行内联代码片段（AI 验证代码的主要方式）
mirascript -e "let x = 1 + 2; debug_print(x)"

# 使用标准输入传递多行代码（script 为 -）
mirascript - <<EOF
fn factorial(n) {
  if n <= 1 { return 1; }
  n * factorial(n - 1)
}
factorial(10)
EOF

# 带变量执行
mirascript -e "debug_print(name)" -v "name='Alice'"

# 变量值为 MiraScript 表达式
mirascript -e "debug_print(data::map(fn { it * 2 }))" -v "data=[1,2,3]"

# 模板模式执行文件
mirascript --template template.miratpl

# 不超时执行（长计算）
mirascript --timeout 0 heavy.mira
```

### 输出行为

- **脚本模式**：打印最终表达式的值（格式化后）
- **模板模式**：打印渲染结果字符串
- **错误**：错误信息输出到 stderr，退出码 `2`；文件不存在退出码 `2`，权限不足退出码 `3`

---

## 命令：`format`

格式化 MiraScript 脚本文件。

### 语法

```sh
mirascript format [options] <script...>
```

`<script...>` 支持多个文件路径、glob 模式，或 `-` 表示从标准输入读取。

### 选项

| 选项             | 说明                                          |
| ---------------- | --------------------------------------------- |
| `-w, --write`    | 直接将格式化结果写回文件（否则输出到 stdout） |
| `-t, --template` | 对无法通过扩展名推断类型的文件使用模板模式    |

### 示例

```sh
# 检查格式化结果（输出到 stdout，不修改文件）
mirascript format script.mira

# 直接格式化并写回文件
mirascript format -w script.mira

# 格式化目录下所有 .mira 文件
mirascript format -w "src/**/*.mira"

# 从 stdin 格式化（管道）
echo 'let x=1+2' | mirascript format -
```

---

## AI 工作流：生成→验证→迭代

**生成 MiraScript 代码后，必须遵循以下验证流程：**

### 1. 语法与运行时验证

使用 `run -e` / `run -` 验证代码片段：

```sh
npx mirascript -e "<生成的代码>"
```

**注意**：`-e` 传入的代码会被当作完整脚本执行，最后一个表达式的值会被打印。

### 2. 解读错误并修复

根据错误信息定位问题，修改代码后重新验证，直到运行成功。

### 3. 典型验证场景

```sh
# 验证函数定义和调用
mirascript - <<EOF
fn factorial(n) {
  if n <= 1 { return 1; }
  n * factorial(n - 1)
}
factorial(10)
EOF

# 验证带变量的逻辑
mirascript - <<EOF
let items = [1, 2, 3, 4, 5];
items::filter(fn { it > 2 })::map(fn { it * 2 })
EOF

# 验证模板
mirascript --template -e "Hello, \${name}!" -v "name='World'"
```

### 4. 常见错误代码

| 退出码 | 含义                        |
| ------ | --------------------------- |
| `0`    | 成功                        |
| `1`    | 内部错误                    |
| `2`    | 语法/运行时错误或文件不存在 |
| `3`    | 权限不足                    |

退出码非 `0` 时必须检查 stderr 输出并修复。

---

## 注意事项

- **多行代码**：在 shell 中传递多行代码时，使用引号包裹或 heredoc；在 PowerShell 中使用 `@'...'@` 多行字符串
- **引号转义**：`-v` 中的字符串字面量须注意 shell 引号转义，优先使用逐字字符串 `@"..."@` 避免转义冲突
- **超时限制**：默认超时 3000ms，复杂算法或循环测试时加 `--timeout 0`
