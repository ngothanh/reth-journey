# Phần 7 — Bản đơn giản nhất: zero-copy, zero-alloc `freeze`

Phần 6 cho ta hai repr — `static` và `shared` — nhưng chưa có gì *tạo* ra một `Bytes`
sở-hữu-buffer-trên-heap, và cái yêu cầu tiêu đề của cả loạt vẫn chưa đạt: **`freeze`
phải O(1) — zero-copy, zero-allocation.** Bài này xây **bản đơn giản nhất** thoả yêu
cầu đó, và *chỉ* yêu cầu đó.

Đây là một lựa chọn có chủ đích: ta *không* xây sẵn cho những nhu cầu chưa có (in-place
advance, tối ưu lazy-promote nâng cao). Ta bắt đầu từ cái tối thiểu chạy được. Phần 8
mới hỏi "nếu cần thêm thì sao?" — và cho thấy mỗi nhu cầu thêm *ép* một đánh đổi.

## Bài toán một-chủ

Một `Bytes` vừa ra từ `from_vec` hay `BytesMut::freeze` **sở hữu một buffer, một mình**.
Nó phải làm được hai việc:

- **drop** → giải phóng buffer. `dealloc` cần *allocation base* + *`cap`* (để dựng lại
  đúng `Layout::array::<u8>(cap)`).
- **clone** → thăng cấp lên shared (Phần 4): cấp một `Shared` có refcount.

Cả hai việc đó cần thông tin, mà ta chỉ có *một* ô để cất: `ctx`. Và `ctx` phải phân
biệt được với con trỏ `Shared` (trạng thái đã-promote). Vậy ta nhét gì vào `ctx`?

## Đơn giản hoá then chốt: `self.ptr` đã là base của buffer

Đây là chỗ mọi thứ gọn lại. Với một handle sở-hữu mà *view không bao giờ dời khỏi base*,
**`self.ptr` chính là base của buffer (`buf`)**. Nên `ctx` **không cần** cất con trỏ — nó cất
đúng cái thứ mà `drop` *không* suy được từ `ptr`/`len`: **`cap`**.

(Điều kiện "view không dời khỏi base" đúng vì đường duy nhất để dời `ptr` là `slice`, mà
`slice` sẽ *promote* — xem cuối bài. Nên một handle OWNED *luôn* có `self.ptr == buf`.
Đây là invariant nền của cả thiết kế.)

## Mã hoá: `cap` trong `ctx`

```rust
const OWNED_TAG: usize = 1;
//   ctx LẺ  (bit 0 = 1)  → OWNED: ctx = (cap << 1) | 1;  buf = self.ptr
//   ctx CHẴN (bit 0 = 0)  → ARC:   ctx = *mut Shared  (Shared căn lề ≥ 8 → luôn chẵn)
```

Một bit thấp phân biệt hai trạng thái. `Shared` trên heap luôn chẵn (căn lề), nên ta
*ép* OWNED luôn lẻ bằng `(cap << 1) | 1` — `cap` là số ta tự kiểm soát, dịch trái rồi
bật bit là xong. **Một `OWNED_VTABLE` duy nhất.** (Không có "buffer chẵn/lẻ", không
EVEN/ODD — đó là chuyện của Phần 8, khi ta buộc phải cất *con trỏ* thay vì *cap*.)

## `from_vec` và `from_owned_parts`

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // rỗng → static, 0 cấp phát (Vec rỗng drop bình thường)
    }
    // Giữ NGUYÊN cap của Vec — KHÔNG into_boxed_slice, KHÔNG realloc.
    let mut bytes = core::mem::ManuallyDrop::new(bytes);
    let (buf, len, cap) = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
    unsafe { Self::from_owned_parts(NonNull::new_unchecked(buf), len, cap) }
}

pub(crate) unsafe fn from_owned_parts(ptr: NonNull<u8>, len: usize, cap: usize) -> Self {
    if cap == 0 { return Bytes::from_static(&[]); } // vd BytesMut::new(0) → ptr dangling
    Bytes {
        ptr, len,                                    // self.ptr = buf
        // cap pack vào ctx như địa-chỉ-không-provenance (ta chỉ đọc .addr() lại, không deref)
        ctx: AtomicPtr::new(ptr::without_provenance_mut((cap << 1) | OWNED_TAG)),
        vtable: &OWNED_VTABLE,
    }
}
```

Hai điểm là cả cái đẹp của bản này:

- **Không `into_boxed_slice`.** `bytes` thật shrink Vec về `cap == len` (một cú realloc + memcpy nếu Vec dư chỗ). Ta *không* — giữ nguyên buffer, `cap` có thể > `len`. Nhờ
đó `BytesMut::freeze` một buffer `cap 1024 / len 7` là **zero-copy** (con trỏ không
đổi) *và* `from_owned_parts` **không cấp phát gì** (không cả control-block) → **zero
allocation**. Đây chính là yêu cầu tiêu đề, đạt.
- **`without_provenance_mut` + `.addr()`**: ta cất một *số nguyên* trong ô `AtomicPtr`.
  Vì không bao giờ deref nó như con trỏ, đây là API strict-provenance đúng — Miri
  `-Zmiri-strict-provenance` sạch.

## `owned_clone` / `owned_drop` — chỉ là dispatch

```rust
fn owned_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let raw = ctx.load(Ordering::Acquire); // Acquire: có thể vừa có kẻ promote & công bố Shared
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { shallow_clone_arc(raw as *mut Shared, ptr, len) } // đã promote → như share_clone
    } else {
        let cap = raw.addr() >> 1;                                 // cap ĐỌC THẲNG, không suy số học
        unsafe { promote_owned(ctx, raw, ptr, cap, len) }          // cú clone đầu → promote
    }
}

fn owned_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, _len: usize) {
    let raw = *ctx.get_mut(); // &mut = độc quyền → đọc thường, khỏi atomic (Phần 5)
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { release_shared(raw as *mut Shared) }
    } else {
        let cap = raw.addr() >> 1;
        unsafe { dealloc(ptr as *mut u8, Layout::array::<u8>(cap).unwrap()) } // buf = self.ptr
    }
}
```

`buf` là `self.ptr` (không mask gì), `cap` là `ctx.addr() >> 1` (đọc thẳng). So với
EVEN/ODD của Phần 8 — mask con trỏ + suy `cap` bằng số học — đây gọn hơn hẳn.

> **Bẫy:** đảo nhánh KIND. Bám chặt: `ctx` **chẵn = ARC**, `ctx` **lẻ = OWNED**. Nhầm
> là ép một cap-số thành `*mut Shared` rồi deref → UB im lặng. `miri` bắt đúng loại này.

## `promote_owned` — cấp `Shared`, CAS, xử lý kẻ thua

Trái tim của bài: hiện thực "sửa ngược vào cái gốc" (Phần 4) + cú CAS (Phần 5).

```rust
unsafe fn promote_owned(
    ctx: &AtomicPtr<()>, tagged: *mut (), ptr: *const u8, cap: usize, len: usize,
) -> Bytes {
    let shared = Box::into_raw(Box::new(Shared {
        buf: ptr as *mut u8, cap, ref_count: AtomicUsize::new(2), // handle gốc + bản clone
    }));
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            drop(Box::from_raw(shared));                        // free vỏ, KHÔNG free buf
            shallow_clone_arc(actual as *mut Shared, ptr, len) // dùng `actual`, KHÔNG `shared`
        }
    }
}
```

- **`ref_count = 2`**: CAS công bố `Shared` cho *hai* handle — gốc `b1` (ta vừa CAS `ctx`
  của nó) + bản clone trả về. Hai drop → về 0 → free một lần. Cân.
- **`Ok` — cái đẹp**: CAS ghi vào `ctx` của *handle gốc* nên `b1` thành shared *in-place*,
  dù `b1.vtable` vẫn `OWNED_VTABLE`; lần sau nó đọc `ctx` thấy bit chẵn → tự đi nhánh Shared.
- **`Err(actual)` — bug kinh điển**: `actual` = `Shared` của **kẻ thắng** (khác `shared`
  của mình, vì mỗi thread `Box::new` một vùng heap riêng). Phải vứt `shared` của mình
  (`Box::from_raw` chỉ free *vỏ*, không đụng `buf` vì `Shared` không có `Drop`) rồi bám
  vào `actual`. Dùng nhầm `shared` (đã free) là use-after-free tức thì.

## `slice` — O(1), và nó *thực thi* invariant

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... tính start, end, assert trong biên ...
    if start == end { return Bytes::from_static(&[]); }
    let mut sub = self.clone();  // chia sẻ backing (tăng counter / promote nếu đang OWNED)
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

Viết *một lần*, đúng cho cả ba repr vì `clone` đã lo phần repr-riêng. Điểm mấu chốt:
**`slice` một `Bytes` OWNED sẽ `clone` nó → `clone` một OWNED sẽ *promote* nó thành
SHARED.** Nên kết quả cắt luôn là SHARED (dùng `Shared.buf` làm base, cắt tự do), còn
handle OWNED gốc *không bao giờ* bị dời `ptr`. Đó là cách invariant `self.ptr == buf`
được *thực thi bằng cấu trúc*: đường duy nhất để dời `ptr` là `slice`, mà `slice`
promote. `owned_drop` nhờ đó `dealloc(self.ptr, cap)` luôn trúng base.

## Xong bản đơn giản nhất

Ta có một `Bytes` hoàn chỉnh, đúng, và **đạt yêu cầu tiêu đề**: `freeze` zero-copy +
zero-alloc, `slice` O(1), `clone` lazy-promote, đọc rẻ như `Arc<[u8]>`. Miri
`-Zmiri-strict-provenance` sạch, test `freeze` khẳng định 0 alloc / 0 dealloc.

```
static  ctx = null                 clone: copy      drop: no-op                (free 0)
shared  ctx = *mut Shared          clone: +refcount drop: -refcount+fence      (free 1)
OWNED   ctx = (cap<<1|1) HOẶC Shared;  buf = self.ptr;  clone: promote/arc  drop: dealloc/arc
```

**Nhưng** — đây là bản cho đúng *các yêu cầu hiện tại*. Đời thường đẻ thêm yêu cầu:
*in-place advance* (when, tradeoff, how) và *lazy-promote như một ràng buộc cứng*.
Phần 8 mổ từng cái: mỗi yêu cầu mới **ép** một cách mã hoá khác, kéo theo EVEN/ODD hay
refcount-từ-đầu — và cuối cùng là **trilemma** cho thấy vì sao "hỗ trợ tất cả" là bất
khả trong một struct 4-từ.

---

*Tiếp theo: [Phần 8 — Khi yêu cầu đẻ thêm: advance, lazy-promote, và trilemma](08_promotable_and_slice.md) ·
[Mục lục](00_index.md)*

*English: [`../en/07_from_vec_and_bit_tagging.md`](../en/07_from_vec_and_bit_tagging.md)*
