---
title: Rust VM 设计
description: MiraScript Rust 虚拟机、宿主值桥接、派生宏与内存管理设计。
toc_max_heading_level: 4
---

# Rust VM 设计

## 状态

本文描述 MiraScript Rust VM 的实施方案和后续优化路线。Rust VM 直接复用 `mira-core` 产生的字节码，在 Rust 进程内完成编译、执行、标准库调用和宿主数据访问。

设计目标如下：

- 在 stable Rust 中编译并运行完整的 MiraScript，而不是维护第二套解析器或编译器。
- 与现有 TypeScript、Python 运行时保持语言语义和标准库行为一致。
- 通过 `#[derive(MiraRecord)]`、`#[derive(MiraArray)]` 和 `#[derive(MiraExtern)]` 将 Rust 数据结构作为活引用暴露给 MiraScript。
- MiraScript 自身产生的 array 和 record 使用确定的 Rust 容器表示。
- 不实现运行期垃圾回收；脚本结束时统一释放执行期对象。

首版为单线程 VM。宿主值默认不要求 `Send` 或 `Sync`，线程间共享不属于本设计范围。

### 当前进度

截至 2026-08-15，首个可用版本已经完成，并已进入解释器结构优化阶段：

- `mira-vm` 和 `mira-vm-derive` 已接入 workspace，公开 API、值模型、严格字节码解码、全部现有 opcode、标准库和 matrix module 均已实现。
- Rust record、array 和 extern 活引用以及三个 derive 宏已经实现，并覆盖 rename、skip、readonly、tag、泛型和错误形状。
- 执行 arena、闭包/module 逃逸检测、超时、调用深度、array 长度限制、provider 注入和错误上下文已经实现。
- Rust VM 可以运行全部现有 `tests/**/*.mira`，包括 huge fixture；TypeScript 698 项测试和 Python 128 项黑盒测试同时通过。
- 已增加 compile+run、run-only、宿主值示例、trybuild、资源释放测试和使用文档。

crate 当前保持 `publish = false`。在性能优化阶段结束、公开 API 再次审查并完成发布元数据前，不改变这一状态。

### 性能基线

以下数据在同一台 Windows 主机的 release benchmark 中测得。简单脚本与 `packages/mirascript/bench/index.ts` 保持一致：

```mira
sin(x) + cos(y + PI / 2) + 0
```

其中 `x = 1`、`y = 2`，脚本和 context 均在计时前创建：

| 项目                | 平均延迟 |       中位延迟 |
| ------------------- | -------: | -------------: |
| TypeScript VM `run` |   167 ns |         200 ns |
| Rust VM `run_with`  |   672 ns |         650 ns |
| Rust VM `run`       |   802 ns |         769 ns |
| native JavaScript   | 29.24 ns | 计时器精度不足 |
| native Rust         |  6.86 ns |        6.64 ns |

当前 Rust VM 在简单脚本上的主要差距来自单次执行初始化、寄存器动态借用、静态全局查找、调用参数分配和原生函数分派，不是数值运算本身。Tinybench 与 Divan 使用不同 harness，因此绝对倍率用于判断方向；每项改动必须使用相同命令在当前主机重新测量。

### 第一轮优化结果

阶段 0 的细分 benchmark 和阶段 1 的首批优化已经实施。最近一次完整 VM benchmark 使用 `--min-time 2 --sample-count 100`，结果如下：

| 项目                | 优化前平均延迟 | 优化后平均延迟 |   变化 |
| ------------------- | -------------: | -------------: | -----: |
| 简单脚本 `run_with` |         672 ns |       389.5 ns | -42.0% |
| 简单脚本 `run`      |         802 ns |       396.0 ns | -50.6% |
| 空脚本 `run_with`   |       105.8 ns |       84.16 ns | -20.5% |
| 一次 native call    |       313.6 ns |       163.4 ns | -47.9% |
| 标量循环            |       15.38 µs |       14.45 µs |  -6.0% |
| 容器链              |       70.67 µs |       65.28 µs |  -7.6% |
| 标准库调用          |       4.601 µs |       4.121 µs | -10.4% |

已经保留的改动包括：

- 静态 global key 借用常量字符串，非字符串常量仍保持原有转换语义。
- 零至四参数且无 spread 的调用使用栈上参数数组。
- 调用栈使用共享函数名和八层 inline storage，只在错误路径物化字符串。
- checkpoint 使用倒计数代替每次取模。
- frame 只保留一层动态借用，并将每次执行必需的 root frame 内联到 arena。
- 默认 provider 按线程复用，`RunOptions` 仍是每次执行独立且可覆盖的配置。

使用 2 秒窗口复测时，TypeScript VM 的简单脚本平均延迟为 178.6 ns、中位延迟为 200 ns；Rust VM 当前差距约为平均 2.2 倍、中位 1.9–2.0 倍，已从初始约 4.0–4.8 倍明显收窄。下一轮继续以多类 benchmark 的共同收益为准，不为简单表达式引入专用语义。

### 第二轮优化结果

阶段 2 的第一项结构优化已经完成：`Runtime` 现在由执行入口独占可变访问，frame/register arena、调用栈、checkpoint 倒计时和调用深度不再使用 `RefCell`/`Cell`。`MiraCallContext` 持有可变 runtime 接口，因此原生函数和 extern 仍能回调脚本函数；现有回调重入测试和完整黑盒 fixture 均通过。

同机连续两次使用 `--min-time 2 --sample-count 100`，与第一轮最终结果直接比较：

| 项目                | 第一轮平均延迟 | 第二轮两次平均延迟 |            变化 |
| ------------------- | -------------: | -----------------: | --------------: |
| 简单脚本 `run_with` |       389.5 ns |     372.1–378.7 ns |   改善 2.8–4.5% |
| 简单脚本 `run`      |       396.0 ns |     373.0–378.7 ns |   改善 4.4–5.8% |
| 空脚本 `run_with`   |       84.16 ns |     78.76–79.15 ns |   改善 6.0–6.4% |
| global 算术         |       141.1 ns |     128.1–129.1 ns |   改善 8.5–9.2% |
| 一次 native call    |       163.4 ns |     162.9–164.1 ns |        基本持平 |
| 标量循环 `run_with` |       14.45 µs |     12.51–13.16 µs |  改善 8.9–13.4% |
| 标量循环 `run`      |       14.32 µs |     12.29–12.42 µs | 改善 13.3–14.2% |
| 容器链              |       65.28 µs |     62.23–64.17 µs |   改善 1.7–4.7% |
| 闭包                |       376.8 ns |     351.6–364.2 ns |   改善 3.3–6.7% |
| 标准库调用          |       4.121 µs |     4.236–4.264 µs |   回退 2.8–3.5% |

纯 native 基准保持不变；解释器密集的标量循环、global 算术、容器和闭包均有可重复收益，因此保留该结构调整。标准库单项的 2.8–3.5% 小幅回退在两次测量中重复出现，后续条目继续跟踪该项目。

固定字段内部指令也已完成。解码后的 IR 不再为普通指令保存 `OpCode + Vec<i64>`；固定参数使用具名 `usize`/`i64` 字段，只有 concat、pick/omit 和 call 等可变参数指令保留 boxed slice。两次同参数 benchmark 中，compile+run 简单脚本由上一项的 5.43–5.52 µs 降至 5.03–5.04 µs，标准库 run-only 由 4.24–4.26 µs 降至 3.53–3.88 µs；标量循环、容器和简单脚本 run-only 均与上一项区间重叠，没有发现稳定的共同回退。

静态 global slot 已完成。解码时按首次出现顺序去重静态名称，每次 run 从当前 `MiraContext` 解析为借用 slot；零、一、二和八个以内的常见规模不分配堆容器，更多名称才使用 `Vec`。缺失名称仍只在对应指令实际执行时报告，复用脚本、更换或修改 context、用户覆盖标准库值的语义均有测试覆盖。最终复测中，空脚本为 80.09 ns、单 global 为 91.84 ns、简单脚本 `run_with` 为 342.1 ns，均未引入固定成本回退；新增八次重复 global 基准为 331.9 ns。

安全循环 frame 复用已完成。解码器递归检查循环体；只要任意嵌套位置创建 script function 或 module，就继续为每次迭代建立独立词法 frame，否则复用一个 frame 并在下一次迭代前清空寄存器。IR 判定测试和逐迭代闭包捕获测试同时覆盖正反路径。标量循环 `run_with` 从上一项的 12.17 µs 降至 7.88 µs，改善约 35%；compile+run 从 17.82 µs 降至 12.45 µs，改善约 30%。

#### 阶段 2 最终汇总

全部四项完成后连续两次运行完整 Rust benchmark，以下数据与阶段 1 最终结果比较：

| 项目                | 阶段 1 平均延迟 | 阶段 2 最终平均延迟 |                 变化 |
| ------------------- | --------------: | ------------------: | -------------------: |
| 简单脚本 `run_with` |        389.5 ns |      358.2–379.2 ns |        改善 2.6–8.0% |
| 简单脚本 `run`      |        396.0 ns |      359.6–360.0 ns |        改善 9.1–9.2% |
| 空脚本 `run_with`   |        84.16 ns |      81.75–84.25 ns |  基本持平至改善 2.9% |
| 常量脚本            |        84.86 ns |      83.08–86.54 ns |       ±2.1% 测量波动 |
| 单 global           |        95.69 ns |      93.18–95.91 ns |  基本持平至改善 2.6% |
| global 算术         |        141.1 ns |      133.6–141.5 ns |  基本持平至改善 5.3% |
| 一次 native call    |        163.4 ns |      152.1–157.1 ns |        改善 3.9–6.9% |
| 标量循环 `run_with` |        14.45 µs |      7.883–8.018 µs |      改善 44.5–45.4% |
| 标量循环 `run`      |        14.32 µs |      7.888–7.990 µs |      改善 44.2–44.9% |
| 容器链              |        65.28 µs |      64.18–65.30 µs |  基本持平至改善 1.7% |
| 闭包                |        376.8 ns |      332.9–365.2 ns |       改善 3.1–11.7% |
| 标准库调用          |        4.121 µs |      3.413–4.167 µs | 波动较大，无稳定回退 |

新增的八次重复 global 基准最终为 330.4–377.3 ns。相对最初未优化版本，Rust `run_with` 简单脚本累计改善 43.6–46.7%，默认 `run` 改善约 55.1%，标量循环改善 47.9–48.7%。同次 2 秒 TypeScript 参考 benchmark 的简单脚本 `run` 平均为 163.11 ns；Rust `run_with` 的绝对差距约为 2.20–2.33 倍。TypeScript 中位数受 100 ns 计时器粒度影响，不用于精确倍率判断。

阶段 2 已全部完成。当前没有证据要求直接进入 JIT；下一步按阶段 3 评估 quickening、稳定静态调用缓存和少量跨脚本有效的专用指令。

### 解释器优化路线

优化遵循“先测量、保持语义、逐项提交”的原则。每个阶段都必须通过完整 Rust 黑盒测试，并至少复测简单脚本、标量循环、容器、闭包和标准库调用。

#### 阶段 0：细分基准

- 增加 `nil`、常量、单个 global、global 算术、一次原生调用、两次原生调用和长循环基准，分离 run 固定成本、global 读取、call 边界与 opcode dispatch。
- 同时保留 `MiraScript::run` 和预先创建 `RunOptions` 的 `run_with`，不得用更窄的内部入口替代公开 API 数据。
- 基准使用 `black_box`，记录平均值与中位数；亚微秒结果至少运行 2 秒、100 个 sample。TypeScript 参考 bench 通过 `MIRASCRIPT_BENCH_TIME=2000` 使用相同测量窗口，默认仍保持 100 ms。

#### 阶段 1：低风险热路径优化

- 静态 global 名直接借用已验证的 string 常量，避免 `String` 克隆；继续保留动态 global 的完整转换规则。
- 小参数 call 使用栈内存，避免常见零至四参数调用创建堆 `Vec`；spread 调用仍走可扩展容器。
- 调用栈保存共享函数名或函数标识，仅在错误输出需要时物化字符串。
- checkpoint 改为低成本倒计数，只在命中间隔时读取时钟；调用和从原生函数返回的检查点语义不变。
- 降低 `RunOptions::default()` 和默认 provider 的重复分配，但不引入进程级可变配置。

#### 阶段 2：解释器结构优化

- 已将 runtime 改为独占可变访问，并移除 frame/register 热路径上的 `RefCell`/`Cell`；原生函数回调脚本的能力和测试保持不变。
- 已将通用 `OpCode + Vec<i64>` 指令替换为固定字段的内部指令，普通指令不再分配参数 `Vec`，解释器不再索引通用参数或转换寄存器整数。
- 已为静态 global 建立每次执行的解析 slot；slot 借用当前 context 的值，用户覆盖同名标准库值、跨 context 复用脚本和缺失 global 的按需报错规则保持不变。
- 已对不会产生 frame 捕获的循环复用 frame/register 存储；含 script function 或 module 的循环保持每次迭代独立词法环境，并由 IR 与运行时捕获测试双重覆盖。

#### 阶段 3：quickening 与内建特化

- 对稳定的静态调用缓存已解析目标，context 变化或用户覆盖 builtin 时必须安全失效。
- 评估 `CallGlobal0..4`、数值算术和常见数学 builtin 的专用指令，但不得绕过统一参数验证、超时和错误模型。
- 只有跨多类脚本均有稳定收益时保留 superinstruction，避免为单个 microbenchmark 特化。

#### JIT 决策门槛

JIT 不作为当前阶段的默认路线。完成前三个阶段后，仅在长时间运行的纯计算脚本仍主要受 opcode dispatch 限制，并且实际使用场景需要进一步数量级提升时，再评估 Cranelift 等后端。JIT 方案还必须覆盖平台支持、可执行内存策略、编译延迟、宿主/extern 回调、超时检查、错误栈和解释器回退路径；简单脚本的单次启动成本不能作为单独采用 JIT 的理由。

## Crate 结构

新增两个 workspace crate：

- `mira-vm`：值模型、上下文、字节码解码器、解释器、标准库和公开执行 API。
- `mira-vm-derive`：三个派生宏的 proc-macro 实现。

`mira-vm` 依赖 `mira-core`，并重新导出 `mira-vm-derive` 中的宏。宏展开统一引用 `::mira_vm`，同时支持 `#[mira(crate = "...")]` 处理依赖重命名。

## 公开 API

### 编译与执行

基础接口为可复用的已编译脚本：

```rust
use mira_vm::{compile, MiraAny, MiraContext, MiraRecord};

#[derive(MiraRecord)]
struct Foo {
    pub bar: u8,
}

fn run() -> mira_vm::Result<()> {
    let mut global = MiraContext::new();
    global.insert("foo", MiraAny::from(Foo { bar: 42 }));

    let script = compile("foo.bar")?;
    assert_eq!(script.run(&global)?, MiraAny::from(42));
    Ok(())
}
```

公开入口包括：

```rust
pub fn compile(source: &str) -> Result<MiraScript>;
pub fn compile_with(source: &str, config: &mira_core::Config) -> Result<MiraScript>;
pub fn eval(source: &str, context: &MiraContext) -> Result<MiraAny>;

impl MiraScript {
    pub fn run(&self, context: &MiraContext) -> Result<MiraAny>;
    pub fn run_with(&self, context: &MiraContext, options: &RunOptions) -> Result<MiraAny>;
}
```

stable Rust 不能为普通库类型实现自定义 `Fn` 调用，因此不提供 `compile("...")(&global)` 形式；`MiraScript::run` 是正式的可复用接口，`eval` 是一次性便捷接口。

`MiraContext::new()` 自动挂载标准库。`insert` 接受 `&mut self`，用户值优先于同名标准库值：

```rust
impl MiraContext {
    pub fn new() -> Self;
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<MiraAny>) -> Option<MiraAny>;
    pub fn insert_fn(&mut self, name: impl Into<String>, function: impl Into<MiraNativeFn>);
    pub fn get(&self, name: &str) -> Option<MiraAny>;
    pub fn contains(&self, name: &str) -> bool;
}
```

### 执行限制

`RunOptions` 按每次执行配置，而不是使用进程级全局状态：

```rust
pub struct RunOptions {
    pub timeout: Duration,          // 默认 100 ms
    pub checkpoint_interval: u32,  // 默认 100
    pub max_call_depth: u32,       // 默认 128
    pub max_array_len: usize,      // 默认 0x100_0000
}
```

循环回边、MiraScript 函数调用和从原生函数返回 VM 时执行检查点。无法抢占一个正在阻塞的宿主函数；宿主函数自身负责遵守时间限制。

## 值模型

`MiraAny` 表示 VM 寄存器和公开返回值。它覆盖：

- `Uninitialized`：仅供解释器内部使用，读取时产生运行时错误。
- `Nil`
- `Boolean(bool)`
- `Number(f64)`
- `String(String)`
- `Array(Vec<MiraAny>)`
- `Record(IndexMap<String, MiraAny>)`
- `Function(MiraFunction)`
- `Module(MiraModule)`
- `Extern(MiraExternValue)`
- 内部的 Rust record/array 活引用包装。

MiraScript 字面量、切片、展开和标准库新建的数据只产生 `Vec<MiraAny>` 与 `IndexMap<String, MiraAny>`。`IndexMap` 用于保持确定的字段遍历和序列化顺序。

array 和 record 遵循不可变值语义。解释器在构造其元素时沿用现有运行时规则：不能作为值类型元素保存的 function、module 或 extern 转为 `nil`。function、module 和 extern 使用对象身份比较。

实现以下常用转换：

- 所有 Rust 整数和浮点数转换为 MiraScript `number`；反向转换检查有限性、整数性和目标范围。
- `String`、`&str`、`bool`、`Option<T>`。
- `Vec<T>`、数组、`IndexMap<String, T>` 和受支持的标准 map。
- `TryFrom<MiraAny>` 返回结构化转换错误，不进行静默截断。

`PartialEq` 按 MiraScript 语义实现，包括 `NaN`、`-0`、array/record 递归值比较以及引用类型的身份比较。

## Rust 活引用

派生的 Rust record 和 array 在 MiraScript 中仍分别报告为 `record` 和 `array`，但每次读取都访问当前 Rust 对象，不在插入 context 时生成快照。

直接转换会把对象所有权交给 VM 包装：

```rust
let value = MiraAny::from(Foo { bar: 42 });
```

需要宿主继续持有并修改同一对象时，使用 `MiraShared<T>`：

```rust
let foo = MiraShared::new(Foo { bar: 42 });
global.insert("foo", MiraAny::from(foo.clone()));

foo.borrow_mut().bar = 7;
assert_eq!(script.run(&global)?, MiraAny::from(7));
```

`MiraShared<T>` 基于 `Rc<RefCell<T>>`。动态借用冲突转换为 `MiraError::BorrowConflict`，不得传播为 `RefCell` panic。

读取普通 Rust 字段需要把该字段转换为 `MiraAny`。因此后续读取能观察字段替换，但返回的标量和普通容器是当次读取的值。需要保持嵌套对象身份时，字段本身应使用 `MiraShared<T>` 或 `MiraAny`。

## 派生宏

trait 与同名 derive 宏位于不同命名空间，均由 `mira-vm` 导出。

### `#[derive(MiraRecord)]`

`MiraRecord` 支持具名结构体和 unit struct。它生成实时的字段枚举与读取实现，以及 `From<T> for MiraAny`。

```rust
#[derive(MiraRecord)]
struct User {
    id: u64,
    #[mira(rename = "display_name")]
    name: String,
    #[mira(skip)]
    token: String,
}
```

规则如下：

- 默认导出所有字段，包括私有字段；添加 derive 即视为显式授权。
- 支持 `#[mira(rename = "...")]` 和 `#[mira(skip)]`。
- 重复导出名在编译期报错。
- record 对 MiraScript 只读；脚本赋值仍按语言规则报告“expected extern”。
- 字段必须支持到 `MiraAny` 的转换，宏为泛型字段补齐所需约束。
- enum 和 union 不受支持并给出明确的编译期错误。

### `#[derive(MiraArray)]`

`MiraArray` 支持 tuple struct 和 unit struct。tuple 字段按声明顺序形成固定长度 array，并生成 `From<T> for MiraAny`。

```rust
#[derive(MiraArray)]
struct Point(f64, f64, #[mira(skip)] Metadata);
```

规则如下：

- 支持字段级 `#[mira(skip)]`。
- 长度和元素每次从当前 Rust 对象读取。
- 不支持具名结构体、enum 和 union，避免隐式决定字段顺序。
- `MiraScript` 不能修改 array 元素。

### `#[derive(MiraExtern)]`

`MiraExtern` 支持具名结构体，保留 Rust 对象身份，并允许 MiraScript 读写未标记为只读的字段。

```rust
#[derive(MiraExtern)]
#[mira(tag = "Counter")]
struct Counter {
    value: i64,
    #[mira(readonly)]
    limit: i64,
    #[mira(skip)]
    internal_id: u64,
}
```

规则如下：

- 支持 `rename`、`skip`、`readonly`、结构体级 `tag` 和 `crate`。
- 读取通过到 `MiraAny` 的转换完成。
- 写入通过 `TryFrom<MiraAny>` 完成；失败时对象保持不变并返回字段级转换错误。
- `has`、`keys`、`get` 和 `set` 由 derive 生成。
- derive 不扫描 `impl` block，也不自动导出方法。可调用字段使用 `MiraNativeFn`；更复杂的动态属性、调用或迭代能力通过手写 `MiraExtern` trait 实现。
- enum 和 union 不受支持。

同一类型不能同时派生多个值语义；多个宏生成的 `From<T> for MiraAny` 冲突会在编译期阻止含糊的类型映射。

## 原生函数接口

原生函数需要能够回调 MiraScript 闭包，以支持 `map`、`filter`、`fold` 等标准库函数：

```rust
pub type MiraNativeFn = Rc<dyn for<'a> Fn(
    &mut MiraCallContext<'a>,
    &[MiraAny],
) -> Result<MiraAny>>;

impl MiraCallContext<'_> {
    pub fn call(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny>;
    pub fn options(&self) -> &RunOptions;
    pub fn checkpoint(&mut self) -> Result<()>;
}
```

标准库与用户原生函数使用同一调用通道，保证参数检查、调用深度、超时和错误包装一致。

## 字节码加载

Rust VM 直接消费 `mira-core::Compiler::compile` 的输出。加载过程执行完整的边界验证：

- chunk、code 和 constants 长度。
- 常量标签、数字宽度、字符串长度和 UTF-8。
- 普通与 wide 参数宽度。
- opcode 合法性。
- 常量、寄存器和跳转引用范围。
- `Func`、`If`、`Loop`、`Record`、`Array`、`Module` 与对应结束 opcode 的嵌套关系。

验证后的字节码解码为内部结构化指令树。解释器不在热路径中重复寻找块边界，也不依赖不安全的字节读取。保留原始字节码偏移，用于运行时错误定位。

## 执行模型

### Frame 与闭包

一次执行拥有独立的 arena：

- function frame 保存寄存器和父级捕获环境。
- loop 每次迭代建立独立的词法 frame，使循环中创建的闭包捕获对应迭代值。
- script function 保存函数体编号和捕获 frame 的整数句柄。
- `GetUpvalue`、`SetUpvalue` 按字节码的 lexical level 沿父链访问。

句柄只在当前 run 内有效。脚本对象之间不使用拥有关系的 `Rc`，因而递归函数、自引用绑定和互相引用的闭包不会形成 Rust 引用计数环。

### 控制流

解释器内部使用显式控制结果：

```rust
enum Flow {
    Continue,
    Break,
    LoopContinue,
    Return(MiraAny),
}
```

`if`、loop、range loop、for loop 和 function body 消费各自允许的控制结果；非法控制流在字节码验证阶段拒绝。

### Module

脚本 module 的导出保存为“frame 句柄 + 寄存器”访问器，因此 `pub let mut` 能反映模块内部函数的后续修改。标准库 module 则由不依赖执行 arena 的原生 module 实现。

## 生命周期与清理

不实现运行期 GC。执行完成时按以下顺序收尾：

1. 从返回值出发检查是否包含 script function 或引用 script frame 的 module。
2. 将运行期生成的 array/record 物化为独立的 `Vec<MiraAny>` 和 `IndexMap<String, MiraAny>`。
3. 保留返回值中的 Rust 活引用包装和原生函数/module。
4. 整体释放 frame、闭包和临时对象 arena。

本设计不支持 MiraScript 闭包逃逸出 `run`。直接返回闭包、返回脚本 module，或以其他方式让返回值引用执行 frame 时，返回 `MiraError::EscapingClosure`。宿主不得在 extern 或原生函数中缓存脚本闭包；若违反约定，句柄仍是安全的无效句柄，后续调用返回“execution has ended”，不会形成悬垂引用或未定义行为。

成功、编译后运行错误、extern 错误、超时和最大调用深度错误都通过同一个 arena guard 完成清理。

## 运算语义

Rust VM 以 `packages/mirascript/src/vm/operations` 和 Python 对应实现为兼容基准，覆盖全部现有 opcode：

- 算术、逻辑、比较、近似比较与 SameValueZero。
- number、string、boolean 转换和格式化。
- record/array 构造、可选元素、range、spread、pick、omit、slice。
- `get`、`has`、`length`、动态 key 和负数 array index。
- extern 访问、写入、调用、array-like 和 iterable 行为。
- 全局变量、闭包、可变 upvalue、递归、可变参数和 spread 参数。
- branch、loop、range loop、for、break、continue。
- module 与可变导出。

无法在 Rust 中逐字复现的宿主细节，以 MiraScript 规范定义的结果为准，并通过跨后端测试记录差异；不得直接照搬 JavaScript 原型链等宿主特有行为。

## 标准库

首版完整移植当前 TypeScript 标准库，不发布只有核心 opcode 的不完整运行时。范围包括：

- 数学函数、常量、位运算、整数转换、gamma 和 factorial。
- string 的字符拆分、trim、大小写、搜索、replace、split 和 join。
- sequence 的 with、entries、len、map、filter、find、flatten、fold、group、reverse、sort、unique、repeat、new、zip、all 和 any。
- JSON、基础类型转换、debug 和 time。
- `matrix` module 的逐项运算、fill、invert、size 和 transpose。

标准库模块按现有导出名注册到 `MiraContext::new()`。函数参数验证、恢复值和错误消息尽量与 TypeScript 运行时一致。

随机数、当前时间和调试输出通过 provider 注入；默认实现使用 Rust 系统能力，测试使用确定性实现。JSON 使用 `serde_json`，并明确拒绝或规范化不可序列化的 function、module 和 extern。

## 错误模型

统一错误枚举至少区分：

- `Compile`：保留 `mira-core` 的序列化诊断。
- `InvalidBytecode`：包含字节偏移和原因。
- `Runtime`：类型、未初始化变量、不可调用值等脚本错误。
- `Conversion`：Rust/MiraScript 值转换失败，包含类型和字段或参数位置。
- `Extern`：宿主对象或原生函数返回的错误。
- `BorrowConflict`：活引用发生动态借用冲突。
- `Timeout`。
- `MaxCallDepth`。
- `EscapingClosure`。

运行时错误记录当前函数、opcode 字节偏移和调用栈。首版不改变现有字节码格式来嵌入 source map；源码位置增强作为兼容的后续扩展处理。

## 实施顺序

1. 建立两个 crate、公开类型、基础转换、context 和 derive 的编译期骨架。
2. 实现严格字节码解码与结构化指令表示。
3. 实现寄存器、frame、闭包、控制流及全部 opcode。
4. 实现 Rust 活引用、extern 写入和原生回调接口。
5. 完整移植标准库和 matrix module。
6. 完成 arena 收尾、逃逸检测、错误上下文和执行限制。
7. 接入跨后端测试、基准、文档和示例。

每一步完成时保持 workspace 可编译；标准库未完整通过兼容测试前，不把 crate 标记为可发布状态。

## 测试与验收

### 字节码与解释器

- 覆盖普通和 wide 参数、所有常量类型、截断输入、未知 opcode、非法嵌套和越界引用。
- 将仓库现有 `tests/**/*.mira` 接入 Rust 黑盒运行器，复用既有断言函数。
- 覆盖递归、闭包捕获、可变 upvalue、循环逐次捕获、module 可变导出和动态全局变量。

### Derive 与活引用

- 使用 `trybuild` 验证泛型、rename、skip、readonly、重复名称和不支持的数据类型。
- 运行测试覆盖本文 `Foo` 示例、宿主修改实时可见、extern 写回 Rust、共享对象身份和转换失败不修改原值。
- 验证动态借用冲突返回错误而不是 panic。

### 标准库兼容

- 对确定性函数在 Rust、TypeScript 和 Python 后端比较返回值、序列化结果及错误类别。
- random、time 和 debug 使用注入 provider 测试。
- 复用现有标准库和 matrix 测试样例，不另造弱化的 Rust 专用语义。

### 内存与限制

- 使用 drop 计数器验证成功、错误、超时、递归和循环闭包场景都释放执行 arena。
- 验证闭包和脚本 module 返回被拒绝，原生值和 Rust 活引用可以安全返回。
- 重复执行同一已编译脚本，确认 frame 和临时对象不会跨 run 累积。

### 性能与仓库验证

- 增加 compile+run 与 run-only 基准，覆盖标量计算、容器、闭包和标准库调用。
- Rust VM 接入不得改变 `mira-core` 原有编译热路径；继续运行现有 compiler benchmark 作为对照。
- 最终执行 `cargo fmt --all -- --check`、`cargo check --workspace`、`cargo test --workspace` 和 `git diff --check`，并运行相关跨后端测试。
