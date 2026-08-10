# MissingBindOperator

等级：**错误**

在 bind 或 const 语句中需要 `=` 运算符

## 如何修复

在 `let` 或 `const` 的名称与初始值之间添加 `=`。

### 修改前

```mira
let answer 42;
```

### 修复后

```mira
let answer = 42;
```
