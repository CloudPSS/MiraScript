# NonCompoundIn

等级：**警告**

`in` 运算符右侧必须为复合类型

## 如何修复

`in` 右侧应为数组、记录或其他复合值。

### 修改前

```mira
let found = 1 in 2;
```

### 修复后

```mira
let found = 1 in [1, 2];
```
