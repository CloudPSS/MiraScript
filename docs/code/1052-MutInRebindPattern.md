# MutInRebindPattern

等级：**错误**

重绑定时不允许使用 `mut`

## 如何修复

`mut` 只写在首次声明处；后续赋值直接使用变量名。

### 修改前

```mira
let mut count = 0;
mut count = 1;
```

### 修复后

```mira
let mut count = 0;
count = 1;
```
