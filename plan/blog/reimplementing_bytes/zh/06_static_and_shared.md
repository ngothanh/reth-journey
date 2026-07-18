# 第 6 部分——从模型到代码：`static` 和 `shared`

前五部分把*模型*搭完了：一个 `Bytes` 由 `ptr` + `len`（哪些字节）和 `data` +
`vtable`（谁拥有）组成，有三种拥有方式——`static`、`promotable`、`shared`。从这一部分
起我们*动手写*。而让人舒服的是：三种拥有方式里有两种写起来几乎不值一提。`static` 是热
身，`shared` 只有一个地方难——但那个地方恰是第 5 部分*还没*触及的、最重要的一课 memory
ordering。

我们要写头四个 vtable 函数：`static_clone`、`static_drop`、`share_clone`、
`share_drop`。并回答第 5 部分留下的一个问题：promotion 的 ordering 是为了*发布*一个
`Shared` 块；而 `share_drop` 的 ordering 是为了*释放*一个共享 buffer——一种完全不同的危
险，名叫 *free-while-read*。

## 地图：读 `ctx` → 知道 repr → 该跑哪个函数

整个实现阶段都围着一个动作转：每个 vtable 函数读 `ctx`，推断出当前在哪个 repr，然后分
岔。进代码前先把这个钉在脑子里：

```
vtable = STATIC       ctx = null              clone: copy struct   · drop: no-op        (释放 0 次)

vtable = SHARE        ctx = *mut Shared       clone: +refcount     · drop: -refcount    (释放 1 次)

vtable = PROMOTABLE   ctx 奇  (KIND_VEC)      clone: promote_vec   · drop: free_boxed_slice
                      ctx 偶 (KIND_ARC)     clone/drop: 走 Shared（同 SHARE 那行）

     唯一的状态转移，单向：
        PROMOTABLE/VEC ──(首次 clone: promote_vec, CAS)──► PROMOTABLE/ARC
```

第 6 部分写头两行（`STATIC`、`SHARE`）。第 7 部分负责怎么给 `PROMOTABLE` *编码* `ctx`
（奇/偶的技巧）。第 8 部分写那个状态转移（`promote_vec`）和两个 `PROMOTABLE` 函数。记
住：`vtable` 在出生时就冻结了；promote 时变的只是 *`ctx` 里的 KIND bit*——所以
"PROMOTABLE/ARC" 用的仍是 promotable vtable，只是分岔到 Shared 那一支。

## `static`：热身

回想一下：一个 `static` 的 `Bytes` 指向永生的内存（`&'static [u8]`），所以没有什么可
数，也没有什么可释放。`data` 留成 null。它的两个函数是整个系列里最短的两个答案：

```rust
fn static_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    // 没有 refcount。clone 只是重建一个指向同一处的 handle。
    unsafe {
        Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8),
            len,
            ctx: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }
}

fn static_drop(_ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    // 什么都不做。字节永生，没有什么可释放。
}
```

`static_drop` *有意*留空——它正是那五个可带走的问题里第一个的化身：*这块内存被释放了恰
好几次？* 对 `static` 来说，答案是 **0**。一个空的 `drop` 函数不是没写完；它是把"0
次"写成了代码。注意这里 `ctx` 是 null，所以绝对不能 deref 它——而幸运的是，没有哪一行去
deref 它。

## `shared`：`Shared` 块和它的三个字段

`shared` 是一个自己写的 `Arc<[u8]>`。我们需要一个堆上的控制块，装着 counter：

```rust
struct Shared {
    buf: *mut u8,          // allocation 的原始地址——以后好还给 allocator
    cap: usize,            // allocation 的大小——和 buf 一起构成"如何释放"
    ref_count: AtomicUsize,
}
```

一个第 4 部分预告过、现在落到实处的细节：`Shared.buf` 是 allocation 的*原始*地址，**不
是** handle 手里拿着的那个指针（`Bytes.ptr`）。对一个还没切过的 handle，两者相等；但
`slice` 之后，`Bytes.ptr` 指向 buffer *中间*，而 `buf` 仍必须是起点——因为你只能把
allocator 交给你的那个指针原样还回去。这就是为什么 `buf`/`cap` 住在 `Shared` 里，和
handle 的 `ptr`/`len` 分开。（第 8 部分会正是用这个性质来做 O(1) 的 `slice`。）

## `share_clone`：给 counter 加一，以及为什么 `Relaxed` 就够

```rust
fn share_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { shallow_clone_arc(shared, ptr, len) }
}

unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    let old = (*shared).ref_count.fetch_add(1, Ordering::Relaxed);
    if old > isize::MAX as usize / 2 {
        abort(); // 见下文"为什么用 abort 而不是 panic"
    }
    Bytes {
        ptr: NonNull::new_unchecked(ptr as *mut u8),
        len,
        ctx: AtomicPtr::new(shared as *mut ()),
        vtable: &SHARE_VTABLE,
    }
}
```

这里有*两个*原子操作，两个都是 `Relaxed`。这是最容易让新手困惑的地方，所以细说。

**`load` 那个 `shared` 指针：`Relaxed`。** 记住第 5 部分的原则——ordering 保护的不是*原
子值本身*，而是那个操作*周围的其他内存*。这里 `shared` 指针是一个*稳定*的地址：它在
handle 出生时就定下，整个 handle 生命期都不变。我们并没有把这次读当作一个*信号*，说有什
么新内存刚被发布——我们只是在取一个*本来就属于自己*的地址。没有任何 happens-before 边需
要建立，所以 `Relaxed` 是诚实的最低强度。

（对照着记：第 5 部分 `promotable_clone` 里读 `data` 必须 `Acquire`，因为那里它*可能*是
一个信号"刚 promote 完，这是新的 `Shared`"——而我们接着会去*读*那个 `Shared` 块的*内
容*。同样是 `load`，ordering 不同，因为一个是"取已拥有的地址"，另一个是"接收刚发布的内
存"。）

**`fetch_add` 给 counter 加一：`Relaxed`。** 增加 refcount *不向任何人发布*任何内存。能
调 `clone`，你手里就已经拿着一个活的 handle → payload 和 `Shared` 块早已存在、且对你可
见。这次加一只是对一个计数器做算术；没有什么要同步。所以 `Relaxed`。

**封顶 overflow——以及为什么用 `abort` 而不是 `panic`。** 因为 `fetch_add` 用
`Relaxed` 非常便宜，一个变态的 `mem::forget` 循环（或者 clone 风暴）*理论上*能让
`usize` 溢出回一个小数字 → 过早释放 → use-after-free。所以我们封顶：如果 counter 越过阈
值就硬停。用 `abort` 停而不是 `panic`，因为到那一刻内存安全已经坏了——而 `panic` *可能被
`catch_unwind` 接住*，并且它会 *unwind 穿过各个 `Drop`*，而 `Drop` 又恰好碰那个已经不可
信的 counter。`abort` 是无条件停机。（我们用 *`fetch_add` 的返回值*来检查阈值，而不是单
独一次 `load`——好避免"读"和"加"之间的 TOCTOU 缝隙。）

## `share_drop`：free-while-read 的危险

这是整篇最值钱的部分。`share_drop` 给 counter 减一，如果自己是最后一个，就释放 buffer +
`Shared` 块。

```rust
fn share_drop(ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { release_shared(shared) }
}

unsafe fn release_shared(shared: *mut Shared) {
    if (*shared).ref_count.fetch_sub(1, Ordering::Release) != 1 {
        return; // 还不是最后一个——收工
    }
    core::sync::atomic::fence(Ordering::Acquire);
    let cap = (*shared).cap;
    drop(Vec::from_raw_parts((*shared).buf, cap, cap)); // 从*原始地址*释放 buffer
    drop(Box::from_raw(shared));                        // 释放 Shared 块
}
```

注意 `release_shared` **不需要 handle 的 `ptr`/`len`**——它从 `Shared.buf`/`Shared.cap`
释放整个 allocation。正是这一点让 `slice` 安全：不管 handle 切到哪里，drop 总是还回那个
原始指针。（`Vec::from_raw_parts` 的长度*和* capacity 都用 `cap`——我们描述的是
*allocation*，不是 *view*。`u8` 没有 destructor，所以长度只影响"跑几个 destructor"，但
如实描述 allocation 是必须保持的习惯：哪天 buffer 装的是带 `Drop` 的类型，误用 view 的
`len` 就会跑错数量的 destructor。）

现在来说 ordering，以及为什么它和第 5 部分的 ordering **截然不同**。

### 问题：在别的 thread 还在读时就释放

第 5 部分担心的是 *publish-before-read* 的危险：在 `Shared` 块的内容还没来得及出现之前就
发布了它的地址。这里的危险正好相反：**在另一个 thread 还在读 buffer 时就把它释放了**——
free-while-read。

搭个场景：`b1` 和 `b2` 是共享同一个 buffer 的两个 handle，在两个不同的 thread 上。
Thread A 读几个字节然后 drop `b2`；thread B drop `b1`。Counter 走 `2 → 1 → 0`。你的顺序
直觉说："counter 归 0 意味着没人还在用 → 释放是安全的"。对——*如果只有一个 thread*。但
跨多个 thread、在会重排的硬件上，**"counter 归 0"和"所有读都已完成"并不自动是同一个时
刻。** CPU/compiler 被允许把 thread A 读 buffer 那一下挪到它自己减 counter 那一下*之后*。

看它在*没有* ordering 时怎么崩（假设两次减都是 `Relaxed`）：

```
Thread A                              Thread B
  fetch_sub → 2→1 (Relaxed)
  ...读 b2[0] 被挪到这里                   fetch_sub → 1→0，看到 0
       │                                 free(buf)         ← buffer 消失
       └── 现在才读 b2[0] ←──────────────────────── USE-AFTER-FREE
```

A 的读被挪到了它那次减之后，所以 B 看到 counter 为 0 就释放了，*而此时* A 的读还悬着。读
到了已死的内存。

### 解法：减时用 `Release`，释放前用 `Acquire` fence

- 每个执行 drop 的人用 **`Release`** 减 counter → "发布：我对 buffer 的一切访问都*在*这
  次减*之前*，不许滑到后面去。"
- 最后一个（`fetch_sub` 返回 1 的那个）在释放*之前*跑一个 **`fence(Acquire)`** → "订
  阅：与其他 thread 的*每一次* `Release` 减都同步，所以他们对 buffer 的一切访问现在都
  happens-before 我的这次释放。"

这一对 `Release`/`Acquire` 正是把"counter 归 0"*粘*到"所有读者真的都读完了"上的东西。缺
了它，counter 对，但内存可见性错。

一个微妙的细节，让*一个* fence 就足以和*所有*减操作同步：每个 `fetch_sub` 都是一个读-改-
写操作，所以最后那次减读到的值处在一条由之前每一次 `Release` 减带头的*release 链*里——正
是这一点让一个 `fence(Acquire)` 能和所有减配成对。

### 为什么用单独的 `fence(Acquire)` 而不是 `fetch_sub(AcqRel)`？

你*可以*把那次减改成 `AcqRel` 并去掉 fence——依然正确。但 `AcqRel` 会把 `Acquire` 强加到
*每一次*减上，包括那些不是最后一次的（只是 return，什么都不释放）。用单独的 fence 是为了
让 **只有最后一个** 付 `Acquire` 屏障的代价；其他人只做更便宜的 `Release` 减。这是性能问
题，不是对错问题——而这正是真正的 `Arc` 被这样写的原因。

## 对照本系列里的两种 ordering

这是一个值得带走的点，因为它把两种人们常常混为一谈的危险分开了：

| | 第 5 部分（promotion） | 第 6 部分（`share_drop`） |
|---|---|---|
| 危险 | publish-before-read：内容出现前就发布了指针 | free-while-read：别人还在读时就释放 |
| 操作 | CAS 把 `data` 写成新的 `Shared` | `fetch_sub` counter |
| "发布"方 | CAS 成功 → `Release` | 每次减 → `Release` |
| "接收"方 | `load`/CAS 失败 → `Acquire` | 最后一个的 `fence(Acquire)` |

同一对 `Release`/`Acquire`，两个不同的问题。通用原则依然成立：*每当你的一个原子操作被另
一个 thread 当作信号，用来决定"现在我可以碰（或释放）那块共享内存了"，那么围绕那个操作的
内存访问就必须通过 Release/Acquire 这一对来排定次序。*

## 已经有了什么，第 7 部分做什么

四个函数完成：`static_*`（释放 0 次），`share_*`（加时用 `Relaxed` 的纪律，减时用
`Release`+`fence(Acquire)`）。要点：`share_drop` 的 ordering 不是第 5 部分那个
ordering——它防的是 free-while-read，不是 publish-before-read。

但我们还没能*造出*一个 `shared` 的 `Bytes`。还没有什么东西调到 `SHARE_VTABLE`。缺的那块
是 `from_vec`——把一个 `Vec<u8>` 变成 `Bytes`。而正当写 `from_vec` 时，我们撞上第 5 部分
在一条边注里有意留下的东西：怎么让*一个 8 字节的格子*既装得下一个 buffer 指针、又装得下
一个 `Shared` 指针，还能区分这两类？那就是 bit 打包的技巧，第 7 部分把它剖到底。

---

*下一部分：[第 7 部分——`from_vec` 和 bit 打包的技巧](07_from_vec_and_bit_tagging.md) ·
[目录](00_index.md)*

*English: [`../en/06_static_and_shared.md`](../en/06_static_and_shared.md)*
