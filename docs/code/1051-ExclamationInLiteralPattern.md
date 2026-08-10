# ExclamationInLiteralPattern

等级：**错误**

字面量模式中不允许 `!` 运算符

## 如何修复

模式取反使用 `not`，不要在字面量前使用表达式运算符 `!`。

### 修改前

```mira
value is !true
```

### 修复后

```mira
value is not true
```
