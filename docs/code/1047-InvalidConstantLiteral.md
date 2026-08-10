# InvalidConstantLiteral

等级：**错误**

此处只允许常量或字面量

## 如何修复

关系和范围模式的边界必须是字面量或 `@` 常量，不能使用普通变量。

### 修改前

```mira
let limit = 10;
20 is > limit
```

### 修复后

```mira
const @LIMIT = 10;
20 is > @LIMIT
```
