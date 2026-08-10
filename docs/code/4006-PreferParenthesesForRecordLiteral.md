# PreferParenthesesForRecordLiteral

等级：**提示**

记录字面量声明中更推荐使用 `()` 而非 `{}`

## 如何修复

记录字面量推荐使用圆括号，花括号保留给块表达式。

### 修改前

```mira
let user = { "name": "Mira" };
```

### 修复后

```mira
let user = (name: "Mira");
```
