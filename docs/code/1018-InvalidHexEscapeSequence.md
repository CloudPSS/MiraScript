# InvalidHexEscapeSequence

等级：**错误**

十六进制转义序列的值不是有效的 ASCII 字符

## 如何修复

十六进制转义必须表示有效的 ASCII 字符。

### 修改前

```mira
let letter = "\xFF";
```

### 修复后

```mira
let letter = "\x41";
```
