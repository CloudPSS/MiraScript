# MispositionedRestParameter

等级：**错误**

函数声明中剩余参数应为最后一个参数

## 如何修复

剩余参数必须是函数参数列表中的最后一个参数。

### 修改前

```mira
fn collect(..rest, last) { rest }
```

### 修复后

```mira
fn collect(first, ..rest) { rest }
```
