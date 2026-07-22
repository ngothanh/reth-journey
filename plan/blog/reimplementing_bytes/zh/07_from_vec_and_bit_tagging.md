# 第 7 部分——最简单的一版：zero-copy、zero-alloc 的 `freeze`

第 6 部分给了我们两种 repr——`static` 和 `shared`——但还没有什么能*造出*一个拥有堆上
buffer 的 `Bytes`，而整个系列那条标题级的要求仍未达成：**`freeze` 必须是 O(1)——
zero-copy、zero-allocation。** 这一篇搭出满足那条要求的**最简单的一版**，而且*只*满足那
一条。

这是一个有意的选择：我们*不*为还没出现的需求（就地 advance、进阶的 lazy-promote 优化）提
前搭架子。我们从能跑的最小版起步。第 8 部分才问"如果还要更多呢？"——并展示每一个额外需求
都*逼*出一个取舍。

## 一个主人的问题

一个刚从 `from_vec` 或 `BytesMut::freeze` 出来的 `Bytes` **独自拥有一个 buffer**。它必须
能做两件事：

- **drop** → 释放 buffer。`dealloc` 需要 *allocation 的底部* + *`cap`*（好重建出正确的
  `Layout::array::<u8>(cap)`）。
- **clone** → 升级成 shared（第 4 部分）：分配一个带 refcount 的 `Shared`。

这两件事都需要信息，而我们只有*一个*格子来存：`ctx`。而且 `ctx` 必须能和 `Shared` 指针
（已 promote 状态）区分开。那么我们往 `ctx` 里塞什么？

## 关键的简化：`self.ptr` 本就是 buffer 底部

这里一切都收拢起来。对一个 *view 永不离开底部* 的拥有型 handle 来说，**`self.ptr` 就是
buffer 底部（`buf`）**。所以 `ctx` **不需要**存指针——它存的正是 `drop` *无法*从
`ptr`/`len` 推出来的那样东西：**`cap`**。

（"view 不离开底部"这个条件成立，是因为把 `ptr` 挪走的唯一路径是 `slice`，而 `slice` 会
*promote*——见文末。所以一个 OWNED handle *永远*有 `self.ptr == buf`。这是整个设计的底层
invariant。）

## 编码：`cap` 放进 `ctx`

```rust
const OWNED_TAG: usize = 1;
//   ctx 奇  (bit 0 = 1)  → OWNED: ctx = (cap << 1) | 1;  buf = self.ptr
//   ctx 偶 (bit 0 = 0)  → ARC:   ctx = *mut Shared  (Shared 对齐 ≥ 8 → 永远偶数)
```

一个最低位区分两种状态。heap 上的 `Shared` 永远是偶数（对齐），所以我们用 `(cap << 1) |
1` *强制* OWNED 永远是奇数——`cap` 是我们自己掌控的数，左移再置位就完了。**唯一一张
`OWNED_VTABLE`。**（没有"buffer 奇/偶"，没有 EVEN/ODD——那是第 8 部分的事，等到我们被迫
存*指针*而不是 *cap* 的时候。）

## `from_vec` 和 `from_owned_parts`

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // 空 → static，0 分配（空 Vec 正常 drop）
    }
    // 原样保留 Vec 的 cap——不 into_boxed_slice，不 realloc。
    let mut bytes = core::mem::ManuallyDrop::new(bytes);
    let (buf, len, cap) = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
    unsafe { Self::from_owned_parts(NonNull::new_unchecked(buf), len, cap) }
}

pub(crate) unsafe fn from_owned_parts(ptr: NonNull<u8>, len: usize, cap: usize) -> Self {
    if cap == 0 { return Bytes::from_static(&[]); } // 例如 BytesMut::new(0) → ptr dangling
    Bytes {
        ptr, len,                                    // self.ptr = buf
        // cap 当作不带 provenance 的地址塞进 ctx（我们只会再读 .addr()，不 deref）
        ctx: AtomicPtr::new(ptr::without_provenance_mut((cap << 1) | OWNED_TAG)),
        vtable: &OWNED_VTABLE,
    }
}
```

两点就是这一版全部的漂亮之处：

- **不 `into_boxed_slice`。** 真正的 `bytes` 把 Vec 缩到 `cap == len`（如果 Vec 有富余，
  就是一次 realloc + memcpy）。我们*不*这么做——原样保留 buffer，`cap` 可能 > `len`。因此
  `BytesMut::freeze` 一个 `cap 1024 / len 7` 的 buffer 是 **zero-copy**（指针不变）*且*
  `from_owned_parts` **什么都不分配**（连 control-block 都不分配）→ **zero
  allocation**。这正是标题级的要求，达成。
- **`without_provenance_mut` + `.addr()`**：我们把一个*整数*存在 `AtomicPtr` 格子里。因
  为从不把它当指针 deref，这是正确的 strict-provenance API——Miri
  `-Zmiri-strict-provenance` 干净。

## `owned_clone` / `owned_drop`——只是 dispatch

```rust
fn owned_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let raw = ctx.load(Ordering::Acquire); // Acquire：可能刚有人 promote 并发布了 Shared
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { shallow_clone_arc(raw as *mut Shared, ptr, len) } // 已 promote → 同 share_clone
    } else {
        let cap = raw.addr() >> 1;                                 // cap 直接读，不靠算术恢复
        unsafe { promote_owned(ctx, raw, ptr, cap, len) }          // 第一次 clone → promote
    }
}

fn owned_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, _len: usize) {
    let raw = *ctx.get_mut(); // &mut = 独占 → 普通读，不用 atomic（第 5 部分）
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { release_shared(raw as *mut Shared) }
    } else {
        let cap = raw.addr() >> 1;
        unsafe { dealloc(ptr as *mut u8, Layout::array::<u8>(cap).unwrap()) } // buf = self.ptr
    }
}
```

`buf` 是 `self.ptr`（不 mask），`cap` 是 `ctx.addr() >> 1`（直接读）。对比第 8 部分的
EVEN/ODD——mask 指针 + 靠算术恢复 `cap`——这精简多了。

> **陷阱：** 把 KIND 分支写反。死死咬住：`ctx` **偶 = ARC**，`ctx` **奇 = OWNED**。搞错就
> 是把一个 cap-数强转成 `*mut Shared` 再 deref → 无声的 UB。`miri` 抓的正是这类。

## `promote_owned`——分配 `Shared`、CAS、处理败者

这一篇的心脏：把"回头改写原件"（第 4 部分）+ CAS（第 5 部分）落成代码。

```rust
unsafe fn promote_owned(
    ctx: &AtomicPtr<()>, tagged: *mut (), ptr: *const u8, cap: usize, len: usize,
) -> Bytes {
    let shared = Box::into_raw(Box::new(Shared {
        buf: ptr as *mut u8, cap, ref_count: AtomicUsize::new(2), // 原始 handle + 那个 clone
    }));
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            drop(Box::from_raw(shared));                        // free 外壳，*不* free buf
            shallow_clone_arc(actual as *mut Shared, ptr, len) // 用 `actual`，*不*用 `shared`
        }
    }
}
```

- **`ref_count = 2`**：CAS 向*两个* handle 发布 `Shared`——原始的 `b1`（我们刚 CAS 了它
  的 `ctx`）+ 返回的那个 clone。两次 drop → 归 0 → free 一次。平衡。
- **`Ok`——漂亮之处**：CAS 写的是*原始 handle* 的 `ctx`，所以 `b1` *就地*变成共享，尽管
  `b1.vtable` 仍是 `OWNED_VTABLE`；下次它读 `ctx` 看到 bit 偶 → 自动走 Shared 分支。
- **`Err(actual)`——经典 bug**：`actual` = **胜者**的 `Shared`（不同于自己的 `shared`，因
  为每个 thread 各 `Box::new` 一块独立的 heap）。必须丢掉自己的 `shared`
  （`Box::from_raw` 只 free *外壳*，不碰 `buf`，因为 `Shared` 没有 `Drop`），再抱住
  `actual`。误用 `shared`（已 free）就是当场 use-after-free。

## `slice`——O(1)，而且它*执行*了 invariant

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... 算出 start、end，assert 在边界内 ...
    if start == end { return Bytes::from_static(&[]); }
    let mut sub = self.clone();  // 共享 backing（加 counter / 若是 OWNED 则 promote）
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

只写*一次*，对三种 repr 都成立，因为 `clone` 已经把各 repr 自己的部分办了。关键点：
**slice 一个 OWNED 的 `Bytes` 会 `clone` 它 → clone 一个 OWNED 会把它 *promote* 成
SHARED。** 所以切出来的结果永远是 SHARED（用 `Shared.buf` 作底部，自由地切），而原始的
OWNED handle *永远不会*被挪动 `ptr`。这就是 invariant `self.ptr == buf` 如何*靠结构被执
行*：挪动 `ptr` 的唯一路径是 `slice`，而 `slice` 会 promote。`owned_drop` 因此
`dealloc(self.ptr, cap)` 总是命中底部。

## 最简单的一版完工

我们有了一个完整、正确、并**达成标题级要求**的 `Bytes`：`freeze` zero-copy + zero-alloc，
`slice` O(1)，`clone` lazy-promote，读起来像 `Arc<[u8]>` 一样便宜。Miri
`-Zmiri-strict-provenance` 干净，`freeze` 测试确认 0 alloc / 0 dealloc。

```
static  ctx = null                 clone: copy      drop: no-op                (free 0)
shared  ctx = *mut Shared          clone: +refcount drop: -refcount+fence      (free 1)
OWNED   ctx = (cap<<1|1) 或 Shared;  buf = self.ptr;  clone: promote/arc  drop: dealloc/arc
```

**但是**——这是一个只对*当前需求*正确的版本。现实会长出更多需求：*就地 advance*（何时？花
多少？怎么写？）和*把 lazy-promote 当作一条硬约束*。第 8 部分逐个剖开：每个新需求都*逼*出
一种不同的编码，牵出 EVEN/ODD 或 refcount-从头——而最后是 **trilemma**，它表明为什么在一
个 4-word 的 struct 里"全都支持"不可能。

---

*下一部分：[第 8 部分——当需求长出来：advance、lazy-promote，以及 trilemma](08_promotable_and_slice.md) ·
[目录](00_index.md)*

*English: [`../en/07_from_vec_and_bit_tagging.md`](../en/07_from_vec_and_bit_tagging.md)*
