# BadOmitKeyRecordPattern

等级：**错误**

省略记录字段名时需要绑定模式

## 如何修复

字段名简写只能配合同名绑定模式，例如 `:name`。

### 修改前

```mira
let (:1) = record;
```

### 修复后

```mira
let (:name) = record;
```
