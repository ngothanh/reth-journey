# 第 7 部分——`from_vec` 和 bit 打包的技巧：一个 8 字节的格子，两种含义

第 6 部分写完了 `share_*`，但还没有办法*造出*一个走那条路的 `Bytes`。入口是
`from_vec`——接收一个 `Vec<u8>` 而*不*拷贝。而正当写 `from_vec` 时，我们撞上第 5 部分在
一条边注里留下的东西：一个 `promotable` 的 `Bytes` 的 `data` 必须能装下*两种*不同的指
针——buffer 指针（未 promote）*或* `Shared` 指针（已 promote）——都在同样 8 个字节里，而
且以后每个函数都必须能区分现在装的是哪一种。

这篇就把那个技巧剖到底。它是整个设计里最"琐碎抠细节"的地方，所以我们走得很慢，并在结尾给
出一句好记的话，让一切都收拢起来。

## `from_vec`：先规范化，再存起来

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // 空 → 直接走 static repr，不分配
    }
    let boxed: Box<[u8]> = bytes.into_boxed_slice(); // 规范化：cap == len
    let len = boxed.len();
    let buf = Box::into_raw(boxed) as *mut u8;       // *接管*所有权——现在 free 由我们负责
    // ... bit 打包，然后构造 Bytes（见下文）...
}
```

三件事，每件都有理由：

**`is_empty` → `from_static(&[])`。** 一个空 `Vec` 的 `into_boxed_slice` 会给出一个
*dangling* 指针，我们不想对它做 bit 打包或释放。让空的直接走 `static` repr（永生的空
buffer）最干净——不为 0 个字节分配。

**`into_boxed_slice()`——规范化 `cap == len`。** 这是一个关键细节，后面我们会*靠它来省
掉存 `cap`*。一个 `Vec` 可能 `cap > len`（有多余的空间）；`into_boxed_slice` 把它缩到
`cap == len`。代价：如果 `Vec` 有多余空间，这个操作会*重新分配并 memcpy*。没错，真正的
`bytes` 也一样——所以记住，对一个还有富余 capacity 的 `Vec`，那次 realloc 可能发生。

**`Box::into_raw`——接管所有权。** 在这一行之前，`Box` 会在离开 scope 时自动释放
buffer。`into_raw` 之后，`Box` 消失，*没有任何东西*再自动释放——**你**已经签下了 free
这件事（以后通过 `free_boxed_slice`/`release_shared`）。`buf` 现在是第一个字节的堆地址。
此处若把 `buf` 弄丢了就是 leak。

## 问题：一个格子，两种含义

一个 `promotable` 的 `Bytes` 需要 `data`（我们把这个字段叫 `ctx`）装着：

- **还没** promote 时：指向原始 **buffer** 的指针，
- **已经** promote 后：指向 **`Shared`** 块的指针。

而第 5 部分已经得出结论，为什么这个分类标记*必须*住在 `ctx` 里：promotion 通过对 `ctx`
的一次单字 CAS 在*生命中途*改变状态——而 `vtable` 在出生时就冻住了，没法和它一起一次 CAS
掉。所以我们需要一种办法，*只读 `ctx`*，就知道它现在是哪一类。

办法：借指针的**最低位**当 KIND 标志。

```rust
const KIND_ARC: usize = 0b0; // 低位 = 0 → ctx 是 *mut Shared
const KIND_VEC: usize = 0b1; // 低位 = 1 → ctx 是 buffer 指针
const KIND_MASK: usize = 0b1;
```

为什么最低位*是可以借的空位*？因为 **alignment（对齐）**。一个 `A` 对齐的 `T` 类型的值总
是落在能被 `A` 整除的地址上——8 的倍数在二进制里总是以 `000` 结尾。`Shared` 块装着指针 +
`usize` + `AtomicUsize`，所以对齐 ≥ 8 → 它的地址**总是以 bit 0 结尾**。于是 `Shared` *天
然*就是 `KIND_ARC`，什么都不用做。

## 一句好记的话：**VEC 永远是奇数，ARC 永远是偶数**

一切都从这一句推出来。当任何函数**看 `ctx`**来解码状态时：

- **`ctx` 是奇数（bit = 1）→ VEC**（还是 buffer，未 promote），
- **`ctx` 是偶数（bit = 0）→ ARC**（已经是 `Shared`）。

- *ARC 永远是偶数*：`Shared` 对齐 8 → 天然 bit 0。免费。
- *VEC 必须是奇数*：好**不和 ARC 撞车**。如果一个偶数的 buffer 指针被直接存进 `ctx`，以
  后 `clone`/`drop` 函数看到 bit 0 → 以为"已 promote，这是 `Shared`" → 把 buffer 强转成
  `*mut Shared` 再读 `ref_count`……也就是把你的几个数据字节当成计数器来读 → 全毁。所以我
  们*强制* VEC 状态永远读出来是奇数。

## 麻烦之处：`u8` buffer 可能是偶数*也可能*是奇数

这就是让这篇不同于教科书里那种 tagged pointer 的地方。buffer 是 `u8`，**alignment =
1**，所以它的地址**不**保证低位 = 0——它可能是偶数，也可能是奇数。但我们*想要*它永远读出
来是奇数（KIND_VEC）。所以：

- **偶数 buffer**（bit 0）：必须把 bit *置起来*（`buf | 1`）来标记 VEC。想拿回真实地址，
  必须*清掉*那个 bit（`& !1`）。→ 用 **`PROMOTABLE_EVEN_VTABLE`**。
- **奇数 buffer**（bit 1 本来就有）：已经读出来是 VEC 了，*不*需要置位。但这个 bit 1 *是
  地址真实的一部分*，所以拿回时绝对*不能*清掉。原样存。→ 用 **`PROMOTABLE_ODD_VTABLE`**。

`from_vec` 剩下的代码就是这个分支：

```rust
    if buf as usize & KIND_MASK == 0 {
        // EVEN：置位作 VEC 标记；以后 recover 时用 MASK 清掉 bit。
        let ctx = (buf as usize | KIND_VEC) as *mut ();
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(ctx), vtable: &PROMOTABLE_EVEN_VTABLE }
    } else {
        // ODD：低位已经是 1 == VEC；把指针*原样*存，以后*不* mask。
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(buf as *mut ()), vtable: &PROMOTABLE_ODD_VTABLE }
    }
```

注意 `ptr` 存的是**干净的指针**（没打过 bit 的 `buf`）；只有 `ctx` 带着那个 tag。这样
`deref`（通过 `ptr` 读）永远看不到那个 bit——读路径永远用真实地址。

## 为什么是*两张* vtable，而不是一张？

这是最好的问题，而答案触及一个关于信息的事实。设想你只有一张 vtable，手里只有 `ctx`：

```
Case 1（偶数 buffer 0x1000）：置位 → ctx = 0x1001 → recover 需清掉 bit → 0x1000
Case 2（奇数 buffer 0x1001）：不动   → ctx = 0x1001 → recover 需保留   → 0x1001
```

**两种情况的 `ctx` 完全一样（0x1001），但真实的 buffer 地址不同（0x1000 对 0x1001）。**
只看 `ctx`，你*无法*知道真实的 buf 是哪一个——丢了 1 bit 信息。把 tag 打进最低位这件事，
**对奇数地址是有损的**。

所以你需要**1 个额外的 bit**存在某处，来记住"原始 buffer 是偶数还是奇数"——也就是
"recover 要不要 mask"。而 **vtable 指针正是存那个 bit 的地方**，免费，因为你本来就带着
它。`EVEN` = "recover 时 mask"，`ODD` = "recover 时原样保留"。一张 vtable + 光一个 `ctx`
就是*信息不够*，就这么简单。

（并不是 4 个不同的分支：两种 ARC 情况——不管 EVEN 还是 ODD——*完全一样*，都把 `ctx` 直接
读成 `*mut Shared` 而不 mask，因为 `Shared` 永远 bit 0。EVEN/ODD *只*在 VEC 分支上不
同。）

完整的表格，两个不同的时刻——*encode* 时（`from_vec` 看 `buf`）和 *decode* 时
（`clone`/`drop` 看 `ctx`）：

| 原始 buffer | encode（看 `buf`） | decode（看 `ctx`） | vtable |
|---|---|---|---|
| 偶数 | 置位 `\| 1` | `ctx` 偶 → **ARC**，`ctx` 奇 → **VEC（mask 以 recover）** | EVEN |
| 奇数 | 不动 | `ctx` 偶 → **ARC**，`ctx` 奇 → **VEC（原样保留）** | ODD |

## 一条现实的注记：`ODD` 几乎永远不会跑到

实际上，系统 allocator *对齐是有富余的*——`malloc`/Rust 的 allocator 通常返回对齐 ≥ 16
的指针，即使对 `u8` buffer（本来只需要对齐 1）也如此。所以 `buf` 几乎总是偶数，
`PROMOTABLE_ODD_VTABLE` 在常规 allocator 上几乎是死代码。但 `u8` 的对齐*并不保证*是偶数
（一个自定义 allocator、arena，或 sub-allocation 可能返回奇数地址），所以 ODD 分支纯粹作
为一张*正确性安全网*存在。要真正跑过 `promotable_odd_*`，你得故意在一个奇数地址的 buffer
上造一个 `Bytes`——常规的 `from_vec` 路径可能永远不会走到那里。

## 一条出路：如果你觉得 bit 打包是多余的

那种"抠细节"的感觉是*对的*。它指出一件事：bit 打包是为了*通用性*的工具，对一个极简的
`Bytes` 并非必需。真正的 `bytes` 把 `buf` 塞进 `ctx`，是因为它支持 `advance`/`split`——
那些把 `ptr` 移离 `buf` 而*不* promote 的操作，所以它不得不把原始的 `buf` 记在别处 → 于
是有了 tag + EVEN/ODD。

但如果你的 `Bytes` 有一条 invariant"VEC 永远不会被 slice"（第 8 部分会立起它——`slice`
总是 promote），那么一个 VEC handle *永远*有 `ptr == buf` 和 `cap == len`。也就是说
`buf`/`cap` 已经现成地躺在 `ptr`/`len` 里了——再打包进 `ctx` 是*多余*的。那时你可以合并
成**唯一一张 vtable**，用 null 来区分 VEC/ARC：

```
ctx == null  → VEC（buf 取自 self.ptr，cap 取自 self.len）
ctx != null  → ARC（ctx 是 *mut Shared）
```

`null` 永远不会和 `Shared` 指针撞车，所以是一个绝对安全的 sentinel，整个 EVEN/ODD 都消
失。这是一个真实的设计决定：保留 tag 以 1:1 镜像 `bytes`、并为将来的 `advance` 做好准
备，还是丢掉 tag 好配合当前的 feature set 更精简。两者都对——知道自己在为什么付代价，才
是重要的。

## 已经有了什么，第 8 部分做什么

`from_vec` 完成：规范化成 boxed slice（`cap == len`），接管 free 的责任，然后按奇偶做
bit 打包来选 `EVEN`/`ODD`。可带走的一句：**VEC 奇，ARC 偶**——而 buffer 的奇偶*只*决定
*recover* 的方式（mask 与否），这一点通过选哪张 vtable 来记住。

现在我们能造出一个 `promotable` 的 `Bytes` 了，但四个 `promotable_*` 函数还是空的，而第
一次 `clone`——就是第 4、5 部分搭了整个模型去解释的那个 *promotion*——还没写。第 8 部分把
它写完：四个 dispatch 函数，带败者分支的 CAS 竞争，以及 O(1) 的 `slice`——它既用到
promotion，又*执行*了上面承诺的"VEC 永远不会被 slice"那条 invariant。

---

*下一部分：[第 8 部分——完整的 promotable 和 O(1) 的 `slice`](08_promotable_and_slice.md) ·
[目录](00_index.md)*

*English: [`../en/07_from_vec_and_bit_tagging.md`](../en/07_from_vec_and_bit_tagging.md)*
