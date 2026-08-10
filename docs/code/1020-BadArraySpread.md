# BadArraySpread

等级：**错误**

在 `..` 之后需要表达式

## 如何修复

在 `..` 后提供要展开的数组表达式；若没有内容可展开，直接删除 `..`。

### 修改前

```mira
let values = [1, ..];
```

### 修复后

```mira
let more = [2, 3];
let values = [1, ..more];
```
