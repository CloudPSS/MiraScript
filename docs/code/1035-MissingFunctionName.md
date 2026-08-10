# MissingFunctionName

等级：**错误**

声明中缺少函数名

## 如何修复

函数声明必须提供名称；若需要匿名函数，请将函数表达式绑定到变量。

### 修改前

```mira
fn (x) { x * 2 }
```

### 修复后

```mira
fn double(x) { x * 2 }
```
