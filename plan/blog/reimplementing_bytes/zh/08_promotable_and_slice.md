# 第 8 部分——当需求长出来：`advance`、lazy-promote，以及 trilemma

第 7 部分搭了最简单的一版——cap-in-ctx——恰好满足*当前*需求：zero-copy、zero-alloc 的
`freeze`，O(1) 的 `slice`，lazy-promote。但真实软件很少停在原地。这一篇*一个一个*地加需
求，看什么会崩，并列出**每一种 `ctx` 编码**及各自的代价。最后以一条不可能定理收尾：在一个
4-word 的 struct 里，你无法全都拥有。

## 需求 A：就地 `advance`

**`advance` 是什么。** `bytes::Buf::advance(n)`——通过*挪动 view 指针*（`self.ptr += n`，
`len -= n`）**就地**"吞掉"开头 `n` 个字节，在一个还一个主人的 handle 上，*不 clone*。这
是一把*消费型 cursor* 的刀。

**何时需要。** 用一个游走的指针读网络帧；有些流式 decoder 直接在一个 owned buffer 上
walk。（注意：Ethereum 的 RLP 通常*不*需要——你是在借来的 `&[u8]` 上 walk cursor，不挪动
owned `Bytes` 的指针。这就是第 7 部分适合这个 `Bytes` 的原因。）

**为什么 cap-in-ctx 会崩。** `advance(3)` 之后，`self.ptr = buf + 3 ≠ buf`。但
`owned_drop` 用 `dealloc(self.ptr, cap)` = `dealloc(buf + 3, ...)` 来 free——释放一个
allocation *中间*的指针 → **UB / 堆损坏**。根源：cap-in-ctx *假设* `self.ptr == buf`，而
`advance` 恰好破坏了那个假设。

有了 `advance`，`self.ptr` 就不再能可靠地当 `buf`。我们必须把 `buf` 存在别处。有两条路，
每条一个代价。

## `advance` 的路 1：把 `buf` 存进 `ctx` → 生出 EVEN/ODD

如果 `self.ptr` 不可信，就把 *buffer 指针*塞进 `ctx`。但现在 `cap` 在 `ctx` 里没地方了
（格子已被指针占用）。我们靠**算术**恢复 `cap`：`cap = (ptr - buf) + len` = 从底部到 view
*末尾*的距离。这*只有在* view 始终触到 allocation 末尾时才对——而 `advance` 只 trim 开头
（view-end 不动），所以算术没问题……**前提是创建时 `cap == len`。** 强制 `cap == len` 就
是 `into_boxed_slice`（shrink Vec）→ **失去从 Vec 来的 zero-copy**（如果 Vec 有富余，就是
一次 realloc + memcpy）。

然后就轮到 bit 打包的技巧——因为 `ctx` 现在装的是*指针*，需要一个 bit 来区分 OWNED 和
ARC。`u8` buffer 指针（align 1）*没有*保证空闲的低位：

```
偶数 case (buf 0x1000): 置位 → ctx = 0x1001 → recover 需清掉 bit → 0x1000
奇数 case (buf 0x1001): 不动 → ctx = 0x1001 → recover 必须保留   → 0x1001
```

**两种 case 的 `ctx` 完全一样（0x1001）但 `buf` 不同** → 把 tag 打进最低位是*有损的*。你需
要**1 个额外的 bit**来存"原始是偶数还是奇数"——而 *vtable 指针*正是存它的地方：**`EVEN`**
（"recover 时 mask"）对 **`ODD`**（"原样保留"）。这就是**两张 EVEN/ODD vtable 诞生的时
刻——作为*存指针的代价*，也就是 `advance` 的代价。**

这正是真正的 `bytes` 那条"从 Vec"的路。**取舍：得到 `advance` + 保住 lazy-promote，但失去
从 Vec 来的 zero-copy（shrink）+ 背上 EVEN/ODD。**

## `advance` 的路 2：从一开始就 refcount

把 **`buf` 和 `cap` 都**存在 heap 上一个 `Shared` 块里，*从出生起*就带 refcount。`ctx`
*永远*是 `*mut Shared`。`self.ptr` 是 view（随便 advance），`Shared.buf` 是底部，
`Shared.cap` 是大小。所有操作都走 `Shared`：

- `advance`：`self.ptr += n`。`slice`：clone（ref++）+ 收窄。两个都简单。
- `freeze`：*复用*现成的 `Shared` → **0 alloc**——但只有当 `Shared` *在 freeze 之前就存
  在*时才行 → **`BytesMut` 必须从 `new()` 起就 refcount**。

**取舍：得到 `advance` + zero-alloc-freeze，但失去 lazy-promote**——每个堆上 buffer *从出
生起*就付一个 `Shared` + atomic 的代价，哪怕它永远不 clone。

## 需求 B：把 lazy-promote 当作一条硬约束

**是什么。** 一个从未 clone 过的单主 buffer **不**付任何 atomic，**不**分配任何 `Shared`。
**何时重要。** RLP decode 铸出*数百万*个一次性 blob；*每个 blob* 一个 atomic + 一次 alloc
是 hot path 上最大的可避免开销。cap-in-ctx（第 7 部分）和 EVEN/ODD *有* lazy-promote。
refcount-从头*没有*。

## 每一种 `ctx` 编码，并排放

| 做法 | `ctx` 未 promote 时装 | `buf` 来自 | `cap` 来自 | `advance` | zero-copy freeze | lazy-promote | 复杂度 |
|---|---|---|---|---|---|---|---|
| **cap-in-ctx**（第 7 部分） | `cap` | `self.ptr` | `ctx` | ❌ | ✅ | ✅ | 1 vtable |
| **buf-in-ctx EVEN/ODD**（`bytes`） | buf 指针（tagged） | `ctx`（mask） | 算术（`cap==len`） | ✅ | ❌（shrink） | ✅ | 2 vtable |
| **refcount-从头** | *永远* `*mut Shared` | `Shared` | `Shared` | ✅ | ✅¹ | ❌ | 2 种 repr，逻辑最简单 |

¹ zero-copy freeze 需要 `BytesMut` refcount-从头。

## Trilemma：为什么"全都支持"不可能

看 `advance` / zero-copy-freeze / lazy-promote 这三列：**没有哪一行三样全占。** 这不是实现
的局限——它是一条定理：

> 在一个 4-word 的 struct 里，{lazy-promote、`advance`、`cap>len` 时 zero-alloc-freeze}
> 你只能得 **三选二**。

具体证明：`advance` 把 view 挪离底部 → *必须*存 `buf`。freeze-`cap>len` → *必须*存真实的
`cap`。这是**两个独立的值**，而 `ctx` 这个格子只装得下*一个*。两个都留 → 需要 heap 上一个
`Shared` 块 → 要让 freeze *不*分配，`Shared` 必须*先于* freeze 就存在 → `BytesMut`
refcount-从头 → **失去 lazy-promote。**

整个 trilemma 归到**一个问题**：*view 会不会在**还没** promote 时就离开 buffer 底部（也就
是有没有 `advance`）？*

- **会** → 必须存 `buf` → `ctx` 里放指针 → EVEN/ODD，而 `cap` 要么靠算术恢复（失去从 Vec
  来的 zero-copy），要么 refcount（失去 lazy-promote）。
- **不会** → `ctx` 空出来 → 塞 `cap` → 一张 vtable，lazy-promote 和 zero-alloc-freeze 都
  保住。

## 结论："对"的设计 = *你的*需求

没有绝对最好的一版。挑一个契合真实需求的点：

- **给 Ethereum/RLP 的 `Bytes`**（本篇）：slice + clone + freeze，*不*对 owned handle 做
  advance → **cap-in-ctx**（第 7 部分）。保住 lazy-promote（hot path 便宜）+
  zero-alloc-freeze，换掉这种用法不用的 `advance`。这是对的选择。
- **作为一个 `Buf` 的 `bytes`**（网络）：需要 `advance` → **EVEN/ODD**（承受从 Vec 来的
  shrink）+ `BytesMut` refcount-从头以拿到 zero-copy freeze。*那*就是真正的 `bytes` 复杂
  的原因——它为一个更宽的 feature set 付代价。
- **最通用 / 最好推理的**：**一切都 refcount**（放弃 lazy-promote）——两种 repr STATIC +
  SHARED，没有 tag，没有 promotion。

可带走的一课："重写 `bytes`" *不是*把它逐行抄下来。它是理解整个**设计空间**，并为自己的需
求选对那个点——然后能讲清为什么。`bytes` 选 EVEN/ODD，因为它是一个 `Buf`；我们选
cap-in-ctx，因为这个 `Bytes` 只 slice 不 advance。两者都*对*——对各自的问题而言。

## 验证，以及系列终章

三种设计里的 bug——KIND 分支写反、`shared` 对 `actual`、ordering 错、dealloc 用错
`cap`/`buf`——都*编译干净*，而且在单 thread 上*看起来跑得对*。必备：**`miri`**
（`cargo +nightly miri test`，给 cap-in-ctx 加上 `-Zmiri-strict-provenance`），以及
**promotion 竞争测试**（N 个 thread 一起 `clone` 同一个 handle → 戳到 `Err(actual)`；用
`loom` 穷举 interleaving）。

从"一个字节从网线进来"（第 1 部分）到 trilemma（这一篇），每一块都被上一块*逼*出来，而最后
一块表明：连"怎么编码一个 8 字节的格子"也没有绝对答案——只有*有名字的*取舍，按需求来选。现
在你不只能读懂 `bytes`，还能在这个取舍空间的任意一点*重新设计*它，并为自己的选择辩护。

---

*返回：[第 7 部分](07_from_vec_and_bit_tagging.md) · [目录](00_index.md)*

*English: [`../en/08_promotable_and_slice.md`](../en/08_promotable_and_slice.md)*
