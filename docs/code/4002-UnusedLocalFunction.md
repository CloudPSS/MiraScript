# UnusedLocalFunction

等级：**提示**

局部函数未使用；请考虑删除它

## 如何修复

删除未调用的局部函数，或在需要的位置调用它。

### 修改前

```mira
fn double(x) { x * 2 }
42
```

### 修复后

```mira
fn double(x) { x * 2 }
double(21)
```
