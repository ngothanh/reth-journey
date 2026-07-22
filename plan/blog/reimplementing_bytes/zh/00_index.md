# 重新实现 `Bytes`：一个类型，三种拥有内存的方式

这是一个系列，讲的是一小段但出了名难懂的代码：一个**零拷贝字节句柄（zero-copy byte
handle）**。如果你用过 Rust 的 `bytes` crate、Facebook 的 `IOBuf`，或者 Netty 的
`ByteBuf`，那么它们内部就是这个东西——只不过我们要从头把它重建一遍，好搞清楚它为什么要
这样设计。

这个系列不是照着敲的编码教程。它是一次**调查**：我们从一个很日常的需求出发（一个网络程序
把数据读进来再传出去），撞上一个性能问题，试几种显而易见的做法，看着它们失败，而每一次撞
墙，真正设计的一块拼图就露出来。没有哪一块是凭空冒出来的；每一个决定都是被上一个**逼**出来
的。

你不需要事先知道 `Bytes`、`BytesMut` 或 `freeze` 是什么——第 1 部分会从零把一切搭起来。

这个系列分两段：**第 1–5 部分是*设计***（为什么 `Bytes` 长成这样），**第 6–8 部分是*实
现***（动手把每个 vtable 函数、`from_vec`、`slice` 都写对，连同设计阶段有意留下的那些代码
细节：refcount 的 memory ordering 纪律、bit 打包的技巧，以及 promotion 竞争）。

## 设计阶段——五个部分

**[第 1 部分——一个字节从网线到程序的旅程。](01_the_problem.md)**
我们搭好背景：一个网络程序把数据收进来，需要一个可写的缓冲区（`BytesMut`），然后要通过一
个叫 `freeze` 的操作，把它变成一个可共享的只读句柄（`Bytes`）。我们发现 `freeze` 如果去
拷贝就会很慢，于是提出一条要求：`freeze` 不能拷贝。接着我们试两种显而易见的设计
（`Vec<u8>` 和 `Arc<[u8]>`），看它们在哪里坏掉——由此暴露出核心矛盾：*一个类型，三种清理
内存的方式。*

**[第 2 部分——一个类型，多种行为。](02_vtable.md)**
在 Rust 里，"怎么清理内存"通常就长在*类型*里，编译器全包了。但我们只有一个类型，却需要三种
行为。这一部分展示如何把"清理决定"从编译器手里降到*结构体里的数据*——一张手写的分派表
（vtable）。外加一个任何设计这种类型的人都必须答得上来的问题：为什么这张表恰好有*两个*槽。

**[第 3 部分——把"哪些字节"和"谁拥有"分开。](03_split_and_counting.md)**
让这个设计既灵活又*快*的诀窍：把结构体的字段这样排布，使得读字节永远不必去看拥有权的信息。
这一部分还引入一种简单的思维方式，它是后面难点的骨架——每一种拥有内存的方式，归根到底就是一
个问题：*这块内存被释放了恰好几次？*

**[第 4 部分——那堵墙：当 clone 把一切搞砸。](04_promotion.md)**
这是最难的一部分。四种行为里三种都简单，但克隆一个独占句柄会造成 double-free。唯一的出路
——叫作 *promotion（升级）*——逼出一件在 Rust 里非常反常的事：一个值必须*回头改写*另一个已
经存在的值，在它生命周期的中途。

**[第 5 部分——`AtomicPtr`：安全地回写。](05_atomics.md)**
第 4 部分的回写提出了三条互相独立的要求，而这三条恰好由同一个字段类型的选择解决。这一部分
会走过三个很多人觉得最抽象的并发概念——interior mutability、CAS、内存序（memory
ordering）——但这一次每个概念都挂在一个我们真正必须解决的具体问题上，而不是空谈理论。结尾
给出五个问题，你可以把它们带到往后任何系统编程的难题里。

## 实现阶段——三个部分

**[第 6 部分——从模型到代码：`static` 和 `shared`。](06_static_and_shared.md)**
写头四个 vtable 函数。`static` 是热身（一个空的 `drop` 函数正是"free 0 次"）。`shared`
只有一个地方难，但那个地方是设计阶段还没触及的一课重要 ordering：`share_drop` 必须防
*free-while-read*——在别的 thread 还在读时释放 buffer——靠减 counter 时用 `Release` 和释
放前的一个 `fence(Acquire)`。我们把它和第 5 部分的 *publish* ordering 对照，看清两种不同
的危险。

**[第 7 部分——最简单的一版：zero-copy、zero-alloc 的 `freeze`。](07_from_vec_and_bit_tagging.md)**
搭出恰好满足当前需求的*能跑的最小版*。关键：对一个还没 slice 的单主 handle，`self.ptr` *本
就是* buffer 的 base，所以 `ctx` 空出来可以直接把 `cap` pack 进去——**一张 `OWNED_VTABLE`，
没有 EVEN/ODD**。全套：`from_vec`（保留 cap，不 realloc）、`promote_owned`（CAS + 败者分
支）、`slice` *执行* `self.ptr == buf` 这条 invariant。结果：`freeze` zero-copy **且**
zero-alloc，Miri strict 干净。

**[第 8 部分——当需求长出来：advance、lazy-promote、trilemma。](08_promotable_and_slice.md)**
现实会长出更多需求。*一个一个*地加，看什么会崩：**就地 `advance`**（何时需要、为什么
cap-in-ctx 会崩、以及两条修法——EVEN/ODD *正是存指针的代价*，或者 refcount-从头），然后是
把 **lazy-promote** 当作硬约束。把*每一种* `ctx` 编码并排列出，并以 **trilemma** 收尾：
{lazy-promote、`advance`、zero-alloc-freeze}——4 个字里只能得 2 个。"对"的设计 = *你的*需
求。

## 怎么读

按顺序读——每一部分都直接建立在上一部分刚立起来的东西上。每部分约 15 分钟，自成一体，从上一
部分停下的地方开始，以下一部分接手的问题收尾。

## 范围

设计阶段（1–5）讲的是*为什么*，有意略过代码细节，好让模型清楚地显出来。实现阶段（6–8）把
那些细节一一捡回来——函数签名、refcount 的 ordering 纪律、bit 打包的技巧、CAS 竞争——并写
到你能照着敲的程度。如果你只想*理解*设计，读到第 5 部分就完整了；如果想*重写* `Bytes`，就
继续走最后三部分。

## 术语速查（需要时快速查）

我们保留英文术语；这里给出一行中文释义，省得你离开正文去查：

- **`deref`** — 从一个 `Bytes` 取出 `&[u8]` slice（通过 `Deref` trait）。这是*读*数据
  的路径，便宜，不碰拥有权部分。
- **refcount** — 记录有多少 handle 正共享同一个 buffer 的计数器；归 0 就释放。
- **CAS**（*compare-and-swap*）——"如果你还是 X 就换成 Y"的原子操作，没有哪个 thread 能
  插在中间。lock-free 更新的基础。
- **`Release` / `Acquire`** — 一对*内存序（memory ordering）*标签：一边*发布*，一边*订
  阅接收*；它们只有成对作用在同一个变量上才有效。
- **UB**（*undefined behavior*）——未定义行为；一旦沾上，编译器就被允许做*任何事*，而出
  错往往是悄无声息的。
- **`Miri`** — 一个在弱内存模型下运行 Rust 代码的解释器，用来*抓* `unsafe` 代码里的 UB
  （use-after-free、double-free、data race），这些正是 `cargo test` 会漏掉的。

*English: [`../en/00_index.md`](../en/00_index.md) · Tiếng Việt:
[`../vi/00_index.md`](../vi/00_index.md) · Deutsch: [`../de/00_index.md`](../de/00_index.md)*
