# InvalidOrdinalLiteral

等级：**错误**

无效的序数字面量；请移除前导零和下划线，或改用 `[$0]`

## 如何修复

序数字段不能带前导零或下划线；也可以改用方括号索引。

### 修改前

```mira
let second = point.01;
```

### 修复后

```mira
let second = point[1];
```
