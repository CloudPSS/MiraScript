# MissingCloseParen

等级：**错误**

缺少 `)` 以关闭括号

## 如何修复

为参数列表、调用、记录或分组表达式补上右括号。

### 修改前

```mira
fn add(a, b { a + b }
```

### 修复后

```mira
fn add(a, b) { a + b }
```
