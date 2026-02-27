# MiraScript

[![Visual Studio Marketplace](https://img.shields.io/visual-studio-marketplace/v/CloudPSS.mirascript?label=VS%20Code%20Marketplace&cacheSeconds=7200)](https://marketplace.visualstudio.com/items?itemName=CloudPSS.mirascript)
[![Open VSX Registry](https://img.shields.io/open-vsx/v/CloudPSS/mirascript?label=Open%20VSX%20Registry)](https://open-vsx.org/extension/CloudPSS/mirascript)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

MiraScript 语言的 Visual Studio Code 扩展，提供全面的语言支持与智能编辑体验。

MiraScript 是一门**表达式优先、不可变数据为核心**的脚本语言，具有简洁的语法和强大的模式匹配能力。

## 功能特性

### 语法高亮

为 MiraScript（`.mira`）和 MiraScript 模板（`.miratpl`）文件提供完整的语法高亮支持，包括：

- 关键字、运算符、字面量的着色
- 字符串插值语法高亮
- 语义标记（Semantic Tokens）：基于编译分析提供更精确的变量、函数、参数、模块等着色

### 智能补全

- **自动补全**：输入时自动提供上下文相关的补全建议，包括变量、函数、关键字和属性
- **签名帮助**：调用函数时显示参数签名信息

### 代码导航

- **转到定义**：跳转到变量或函数的定义位置
- **查找所有引用**：查看符号在项目中的所有使用位置
- **文档大纲**：在大纲视图中快速浏览文件结构
- **符号高亮**：高亮显示当前光标所在符号的所有出现位置

### 代码诊断

实时检测并报告代码中的错误和警告，包括词法错误、语法错误和语义错误。

### 代码操作与重构

- **重命名符号**：安全地重命名变量和函数，自动更新所有引用
- **代码操作**：快速修复建议和代码重构操作
- **CodeLens**：在函数声明上方显示内联操作信息

### 代码格式化

支持文档格式化，自动调整代码缩进和排版风格。

### 内嵌提示

通过 Inlay Hints 在编辑器中显示推断的参数名等辅助信息。

### 智能选择

支持基于语法结构的智能选择范围扩展，快速选中表达式、语句块等。

### Markdown 集成

- 在 Markdown 文件中使用 ` ```mira ` 和 ` ```miratpl ` 代码块时自动启用语法高亮
- Markdown 预览中支持 MiraScript 代码块的渲染

## 支持的文件类型

| 文件扩展名 | 语言 ID               | 说明                |
| ---------- | --------------------- | ------------------- |
| `.mira`    | `mirascript`          | MiraScript 脚本文件 |
| `.miratpl` | `mirascript-template` | MiraScript 模板文件 |

## 项目配置

本扩展支持配置文件为

- Yaml/JSON: `.mirarc`、`.mirarc.yml`、`.mirarc.yaml`、`.mirarc.json`
- JS/TS: `.mirarc.js`、`.mirarc.cjs`、`.mirarc.mjs`、`.mirarc.ts`、`.mirarc.cts`、`.mirarc.mts`

### 配置选项

```js
// mira.config.js
export default {
  // 全局变量
  globals: {
    PI: 3.14159,
    appName: 'MyApp',
  },
  // 全局函数定义
  functions: {
    myFunc: { name: 'myFunc', description: '自定义函数', parameters: [] },
  },
  // 全局模块
  modules: {
    utils: {
      helper: { name: 'helper', description: '辅助函数' },
    },
  },
  // 全局外部对象
  externs: {
    context: { value: 42 },
  },
};
```

配置文件中声明的全局变量、函数和模块会被语言服务识别，提供补全和诊断支持。修改配置文件后保存即可生效。

## MiraScript 语言速览

```mira
// 变量声明
let x = 42;
let mut count = 0;

// 字符串插值
let name = "World";
let greeting = "Hello, $name!";

// 函数声明
fn add(a, b) { a + b }
fn square { it ^ 2 }

// 扩展调用（链式风格）
[1, 2, 3, 4, 5]
  ::filter(fn { it > 2 })
  ::map(fn { it * 10 });

// 模式匹配
match value {
  case 0 { "零" }
  case 1..10 { "小数" }
  case > 100 { "大数" }
  case _ { "其他" }
}

// 记录（对象）
let point = (x: 1, y: 2);
let (x: p_x, y: p_y) = point;

// 控制流
if condition {
  "yes"
} else {
  "no"
}

// 空安全
let result = obj.prop ?? "default";
```

更多语法参考请查看 [MiraScript 文档](https://mira.cloudpss.net/)。

## 系统要求

- Visual Studio Code `≥ 1.90.0`

## 许可证

[MIT](https://opensource.org/licenses/MIT) © [CloudPSS](https://github.com/CloudPSS)
