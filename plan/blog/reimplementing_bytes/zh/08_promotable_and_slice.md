# 第 8 部分——完整的 promotable，以及 O(1) 的 `slice`

我们已经有了造出 `promotable` 的 `Bytes` 的 `from_vec`（第 7 部分），也懂了 promotion
*为什么*存在（第 4 部分），以及它需要的*那些并发工具*（第 5 部分）。这最后一篇把所有东西
拼成代码：四个 `promotable_*` 函数，带 CAS 竞争的 `promote_vec` 函数，O(1) 的 `slice`
函数，以及那条默默撑起这一切的 invariant。

让人舒服的是：经过这么多准备，四个 dispatch 函数几乎自己就写出来了。难度全集中在一个函数
上——`promote_vec`——以及它的其中一个分支，那个*败者*分支。

## 四个 `promotable_*` 函数只是 dispatch

每个函数只做一件事：读 `ctx`，看 KIND（按第 7 部分那句"VEC 奇，ARC 偶"），然后分支。ARC
分支委托给第 6 部分写的 `shared` helper；VEC 分支做 Vec 自己的活。

```rust
fn promotable_even_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let tagged = ctx.load(Ordering::Acquire); // Acquire：可能刚有人 promote 并发布了 Shared
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) } // 已 promote → 同 share_clone
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;          // EVEN：mask 清掉 bit
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }            // 第一次 clone → promote
    }
}
```

`promotable_odd_clone` 完全一样，只是 VEC 分支的 recover 不 mask：
`let buf = tagged as *mut u8;`。而两个 drop 函数也一样，只是把两件事换成：ARC →
`release_shared`（减 counter），VEC → `free_boxed_slice`（直接释放 buffer，不用
atomic）：

```rust
fn promotable_even_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut(); // &mut = 独占 → 普通读，不用 atomic（想想第 5 部分）
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { release_shared(tagged as *mut Shared) }
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;
        unsafe { free_boxed_slice(buf, ptr, len) }
    }
}
```

> **致命陷阱：** 最容易犯的是把 KIND 条件*写反*。死死咬住"VEC 奇，ARC 偶"：只有
> `== KIND_ARC` 这一支才走 `Shared` 路；另一支（VEC）才 promote / free buffer。把
> `Shared` 路误写成 `== KIND_VEC`，就是把 buffer 强转成 `*mut Shared` → 无声的 UB。这
> 正是 `miri` 生来要抓的那类 bug。

注意 clone 里的 `load` 是 `Acquire`，而 drop 里是通过 `get_mut` 的普通读——正如第 5 部分
解释过的：clone 拿的是共享引用（可能有竞争），drop 是独占的（没有竞争）。

## `promote_vec`：分配 `Shared`、CAS，以及处理败者

这是心脏。它把第 4 部分那句"回头改写原件"和第 5 部分的 CAS 落成了代码。

```rust
unsafe fn promote_vec(
    ctx: &AtomicPtr<()>, tagged: *mut (), buf: *mut u8, ptr: *const u8, len: usize,
) -> Bytes {
    // 1. 恢复 allocation 的大小。见下文"为什么这个算术是安全的"。
    let cap = (ptr as usize - buf as usize) + len;

    // 2. 分配 Shared 块，ref_count = 2（原始 handle + 我们即将返回的那个 clone）。
    let shared = Box::into_raw(Box::new(Shared {
        buf, cap, ref_count: AtomicUsize::new(2),
    }));

    // 3. 发布它：把 ctx 从 `tagged` swap 成 `shared`。
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            // 别人先 promote 了。丢掉*自己的* Shared，抱住胜者的。
            drop(Box::from_raw(shared));                          // free 控制块外壳，*不* free buf
            shallow_clone_arc(actual as *mut Shared, ptr, len)   // 用 `actual`，*不* 用 `shared`
        }
    }
}
```

有三点要说。

**`ref_count = 2`，不是 1。** 那次 CAS 同时向*两个* handle 发布 `Shared`：原始 handle
（`b1`，我们刚 CAS 了它的 `ctx`）和我们正在返回的那个 clone。两者现在都指向这个
`Shared`，所以 counter 初始化为 2。用第 3 部分数数的思路验一下：两个 handle → 两次
drop → 归 0 → free 一次。平衡。

**`Ok` 分支——promotion 的漂亮之处。** 那次 CAS 写的是*原始 handle `b1`* 的 `ctx`（我们
收到的 `ctx: &AtomicPtr` 正是 `&b1.ctx`）。所以 `b1` *就地变成了共享*，尽管 `b1.vtable`
仍是 `PROMOTABLE_*`（改不了——第 5 部分）。下次 `b1` clone/drop 时，`promotable_*` 函数读
`ctx`，看到 KIND_ARC（bit 0），自动走 `Shared` 分支。而新的那个 clone 直接带
`SHARE_VTABLE`。两"款"由 arc 支撑的 handle 共存，共同数着同一个 counter。

**`Err(actual)` 分支——`actual` *不同于* `shared`。** 这就是第 4 部分说的"丢掉多余
counter 时必须小心"，也是一个经典 bug。`compare_exchange(expected, new)` 的意思是："*如
果* `ctx` 仍等于 `expected` 就改成 `new`，否则报告当前值"。当 `Err(actual)` 时：

- `shared` = *自己*刚分配的 `Shared` 块（比如 0xBBB）——竞争失败，*没用了*。
- `actual` = `ctx` 里此刻真正装着的值 = *胜者*的 `Shared` 块（比如 0xAAA）——地址*完全不
  同*，因为每个 thread 各 `Box::new` 一次 → 两块 heap。

所以我们必须 (a) 丢掉自己的 `shared`——而且*用对方法*丢：`Box::from_raw(shared)` 只释放
那个*控制块外壳*，**不**碰 `buf`（因为 `Shared` 没有 `Drop` impl；`buf` 现在归胜者的
`Shared` 所有）；然后 (b) `shallow_clone_arc(actual)` 给胜者的 counter 加一。在步骤 (b)
里误用 `shared`（已 free）就是当场 use-after-free，*而且*还把真正的 `Shared` 弄丢了 →
counter 失衡 → double-free。

在 3 个 thread 的竞争里验一下 counter：胜者 A 造 `Shared`，`ref=2`（原始 + A）；B 和 C
输了，各自 `shallow_clone_arc(actual)` +1 → 到 `4`？不——B/C 里只有一个"先输"，但两个都
+1，成 **4**……等等。重新数清楚：只有*一个*原始 handle 和*一次*胜出的 promote（A）。每
个 clone 的 thread 造*一个*新 handle。3 个 thread clone → 3 个新 handle + 1 个原始 = 4
个 handle。A 设 ref=2（原始 + A 的 handle），B +1 = 3（加上 B 的 handle），C +1 = 4（加
上 C 的 handle）。恰好 4 个活的 handle → 4 次 drop → free 一次。平衡。

### 为什么 `cap = (ptr - buf) + len` 这个算术是安全的

`promote_vec` 拿不到现成的 `cap`——它靠算术恢复。`(ptr - buf)` 是从 buffer 底部到 view
开头的距离；加上 `len` 得到到 view *末尾*的距离。这只有在 **view 始终触到 allocation 末
尾**时才恰好等于 allocation 的大小——也就是 buffer 从来没有在尾部被截短过。

而事实正是如此，靠一条 invariant：**一个 VEC handle 永远不会被 slice。** 因为 `slice`
（下一节）经过 `clone`，而 clone 一个 VEC 会把它*promote* 成 ARC。所以你永远不会拿着一个
切过的 VEC——一个 VEC 永远是完整的 buffer，`ptr == buf`，`cap == len`。这就是为什么
`free_boxed_slice` 也用同样的算术恢复 `cap`，而不必存 `cap`：

```rust
unsafe fn free_boxed_slice(buf: *mut u8, ptr: *const u8, len: usize) {
    let cap = (ptr as usize - buf as usize) + len;
    drop(Vec::from_raw_parts(buf, cap, cap));
}
```

（相反，`shared` repr *确实*在 `Shared` 里存了 `cap`，因为 promote *之后*你可以两头自由
地切，所以再也没法从 view 恢复 `cap` 了。一个靠算术恢复，一个显式存——这种不对称正是那条
invariant 的后果。）

## `slice`：O(1)，而且它*执行*了那条 invariant

整个 `Bytes` 生来就是为了让 `slice` 便宜。诀窍：**clone，然后收窄 view**——不拷贝任何东
西。

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... 算出 start、end，assert 在边界内 ...
    if start == end {
        return Bytes::from_static(&[]); // 空 → 不必持有 refcount
    }
    let mut sub = self.clone(); // 共享 backing（加 counter / 若是 VEC 则 promote）
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

妙处在于你只写*一次*就对*三种* repr 都成立，因为 `clone` 已经把各 repr 自己的部分办了：

- **static**：clone 很平凡（没有 counter）。收窄成一个 `'static` slice → 仍是 static，
  drop 仍是 no-op。不分配。
- **shared**：clone 原子地加 counter。收窄 view；`Shared.buf`/`cap` 不变，所以 drop 仍
  从底部 free。*这*就是 `Shared` 把 `buf`/`cap` 和 view 分开存的原因。
- **promotable**：clone 把它**promote** 成 shared，然后收窄那个。

最后一点正是最漂亮的地方：**slice 一个 promotable 的 `Bytes` 会 promote 它**——恰好是
`promote_vec` 和 `free_boxed_slice` 都赖以用算术恢复 `cap` 的那条"VEC 永远不会被 slice"
invariant。`slice` 不只是*遵守* invariant，它*执行* invariant，靠的是结构：唯一能切的路
径是经过 clone，而 clone 会 promote。一个闭环。

两个小的安全点：`ptr.add(start)` 在边界内，因为已经 assert 了 `start <= end <= len`；而
给一个非空指针加一个小 offset 不可能变成 null，所以 `new_unchecked` 依然正确。

## 完工。回看代码全景

三种 repr，四个多函数，一条 invariant：

```
static     clone: 拷贝结构体             drop: no-op            (free 0 次)
shared     clone: fetch_add Relaxed     drop: fetch_sub Release + fence(Acquire)  (free 1 次)
promotable clone: 未 promote → promote_vec (CAS);  已 promote → shallow_clone_arc
           drop:  未 promote → free_boxed_slice;   已 promote → release_shared

invariant:  slice ⇒ clone ⇒ (VEC 则 promote) ⇒ VEC 永远不会被切
            ⇒ VEC 永远 ptr==buf, cap==len ⇒ 用算术恢复 cap 是安全的
```

而读路径——`deref`、`len`、比较、hash——仍只碰 `ptr` + `len`，从不碰 `ctx`/`vtable`，所
以像 `Arc<[u8]>` 一样廉价。整套 `ctx`/`vtable`/tag/CAS/ordering 的机器*只*在 `clone` 或
`drop` 时才登场。

## 验证：别信，去测

这篇里的这类 bug——KIND 写反、`shared` 对 `actual`、ordering 错——都*编译干净*，而且在单
个 thread 上常常*看起来跑得对*。它们只在有竞争、或有工具照进内存模型时才暴露。所以有两样
是必需的：

- **`miri`**：`cargo +nightly miri test`——抓 use-after-free、double-free、读未初始化内
  存，以及 data race。上面四个 bug 里有三个会被 `miri` 当场逮住。
- **promotion 竞争测试**：让 N 个 thread 一起 `clone` *同一个*原始 handle，逼多次
  `promote_vec` 并行跑，好戳到 `Err(actual)` 分支；反复跑很多遍。`loom`（如果你想走得更
  远）会穷举所有可能的重排次序。

重复第 5 部分三句收尾话里的第三句：unsafe 里可怕的 bug 不是那个把程序搞崩的，而是那个*正
确运行*的——安全 Rust 的直觉被反转了，错误的默认状态是沉默。在 promotable 这里，那种沉默
最厚。永远带上 `miri`。

## 系列终章

从"一个字节从网线进来"（第 1 部分）到带着败者分支的 `promote_vec`（这一篇），每一块都被上
一块*逼*出来：`Arc<[u8]>` 给不了 O(1) 的 `freeze` → 把拥有权降到 vtable → 把读和拥有分
开 → 克隆独占件就是 double-free → promotion 回头改写 → `AtomicPtr` 解掉三条要求 → 最后，
用 tagged pointer、CAS 和一条自我执行的 invariant 把这一切落成代码。没有哪一块是凭空冒出
来的。

现在你不只能*读懂* `bytes`，你还能*重写*它——并为每一行辩护。

---

*返回：[第 7 部分](07_from_vec_and_bit_tagging.md) · [目录](00_index.md)*

*English: [`../en/08_promotable_and_slice.md`](../en/08_promotable_and_slice.md)*
