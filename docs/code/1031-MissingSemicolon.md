# MissingSemicolon

等级：**错误**

语句末尾缺少 `;`

## 如何修复

语句需要以分号结尾；只有块末尾作为返回值的表达式可省略。

### 修改前

```mira
let answer = 42
answer
```

### 修复后

```mira
let answer = 42;
answer
```
