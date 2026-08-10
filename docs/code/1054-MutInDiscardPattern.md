# MutInDiscardPattern

等级：**错误**

弃元模式中不能使用 `mut`

## 如何修复

弃元不会创建可修改的绑定，因此删除 `_` 前的 `mut`。

### 修改前

```mira
let mut _ = compute();
```

### 修复后

```mira
let _ = compute();
```
