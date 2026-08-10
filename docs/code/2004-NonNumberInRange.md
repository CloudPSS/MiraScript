# NonNumberInRange

等级：**警告**

范围中不能使用非数字字面量

## 如何修复

范围模式的边界应为数字；字符串区间请改成明确的比较条件。

### 修改前

```mira
value is "a".."z"
```

### 修复后

```mira
value is 1..10
```
