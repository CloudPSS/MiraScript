---
token: 'global'
---

`global` 表示全局对象（全局命名空间）。通常用于读取全局变量或判断某个全局键是否存在。

```mira
// 访问全局变量
global.some_name;

// 动态访问全局变量
global['some_name'];

// 判断键是否存在
'some_name' in global;
```
