# NonNumberInArithmetic

等级：**警告**

算术表达式中不能使用非数字字面量

## 如何修复

算术运算符两侧应为数字；其他类型请使用对应操作。

### 修改前

```mira
let value = true + 1;
```

### 修复后

```mira
let value = 1 + 1;
```
