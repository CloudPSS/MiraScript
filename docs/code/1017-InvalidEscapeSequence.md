# InvalidEscapeSequence

等级：**错误**

字符串中的转义序列无效

## 如何修复

使用受支持的转义序列；若需要原样保留反斜杠，可使用逐字字符串。

### 修改前

```mira
let path = "C:\q";
```

### 修复后

```mira
let path = @"C:\q"@;
```
