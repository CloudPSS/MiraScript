# MissingOpenBrace

等级：**错误**

缺少 `{` 以打开花括号

## 如何修复

在条件、循环、函数或模块主体前添加左花括号。

### 修改前

```mira
if true 1 }
```

### 修复后

```mira
if true { 1 }
```
