# MisuseOfGlobalKeyword

等级：**错误**

`global` 关键字只能用作 `global.<name>`、`global[<name>]` 或 `in` 运算符右侧

## 如何修复

通过 `global.<name>` 或 `global[<name>]` 访问全局变量；不要切片或单独使用 `global`。

### 修改前

```mira
let values = global[1..2];
```

### 修复后

```mira
let value = global["value"];
```
