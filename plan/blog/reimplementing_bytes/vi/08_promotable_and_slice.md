# Phần 8 — promotable đầy đủ, và `slice` O(1)

Ta đã có `from_vec` tạo ra một `Bytes` `promotable` (Phần 7), và đã hiểu *vì sao*
promotion tồn tại (Phần 4) cùng *những công cụ concurrency* nó cần (Phần 5). Bài cuối
này ráp tất cả thành code: bốn hàm `promotable_*`, hàm `promote_vec` với cuộc đua CAS,
hàm `slice` O(1), và cái invariant lặng lẽ chống đỡ cho tất cả.

Điều dễ chịu: sau ngần ấy chuẩn bị, bốn hàm dispatch gần như tự viết. Cái khó dồn hết
vào đúng một hàm — `promote_vec` — và đúng một nhánh của nó, nhánh *thua cuộc*.

## Bốn hàm `promotable_*` chỉ là dispatch

Mỗi hàm làm đúng một việc: đọc `ctx`, xem KIND (theo câu "VEC lẻ, ARC chẵn" của Phần
7), rồi rẽ. Nhánh ARC uỷ cho các helper `shared` đã viết ở Phần 6; nhánh VEC làm việc
riêng của Vec.

```rust
fn promotable_even_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let tagged = ctx.load(Ordering::Acquire); // Acquire: có thể vừa có kẻ promote & công bố Shared
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) } // đã promote → như share_clone
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;          // EVEN: mask bỏ bit
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }            // cú clone đầu → promote
    }
}
```

`promotable_odd_clone` giống hệt, chỉ khác nhánh VEC recover không mask:
`let buf = tagged as *mut u8;`. Còn hai hàm drop giống thế, chỉ đảo hai việc: ARC →
`release_shared` (giảm counter), VEC → `free_boxed_slice` (giải phóng buffer thẳng,
không atomic):

```rust
fn promotable_even_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut(); // &mut = độc quyền → đọc thường, khỏi atomic (nhớ Phần 5)
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { release_shared(tagged as *mut Shared) }
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;
        unsafe { free_boxed_slice(buf, ptr, len) }
    }
}
```

> **Bẫy chết người:** dễ nhất là *đảo ngược* điều kiện KIND. Cứ bám chặt "VEC lẻ, ARC
> chẵn": nhánh `== KIND_ARC` mới đi đường `Shared`; nhánh còn lại (VEC) mới promote /
> free buffer. Viết nhầm thành `== KIND_VEC` cho đường `Shared` là ép buffer thành
> `*mut Shared` → UB im lặng. Đây đúng là loại bug mà `miri` sinh ra để bắt.

Chú ý cú `load` trong clone là `Acquire`, còn trong drop là đọc thường qua `get_mut`
— đúng như Phần 5 đã lý giải: clone chia sẻ tham chiếu (có thể đua), drop độc quyền
(không đua).

## `promote_vec`: cấp `Shared`, CAS, và xử lý kẻ thua

Đây là trái tim. Nó hiện thực đúng "sửa ngược vào cái gốc" của Phần 4 và cú CAS của
Phần 5.

```rust
unsafe fn promote_vec(
    ctx: &AtomicPtr<()>, tagged: *mut (), buf: *mut u8, ptr: *const u8, len: usize,
) -> Bytes {
    // 1. Khôi phục kích thước allocation. Xem "vì sao số học này an toàn" bên dưới.
    let cap = (ptr as usize - buf as usize) + len;

    // 2. Cấp khối Shared, ref_count = 2 (handle gốc + bản clone ta sắp trả về).
    let shared = Box::into_raw(Box::new(Shared {
        buf, cap, ref_count: AtomicUsize::new(2),
    }));

    // 3. Công bố nó: swap ctx từ `tagged` sang `shared`.
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            // Kẻ khác đã promote trước. Vứt Shared của MÌNH, bám vào của kẻ thắng.
            drop(Box::from_raw(shared));                          // free vỏ điều khiển, KHÔNG free buf
            shallow_clone_arc(actual as *mut Shared, ptr, len)   // dùng `actual`, KHÔNG dùng `shared`
        }
    }
}
```

Ba điểm cần nói.

**`ref_count = 2`, không phải 1.** Cú CAS công bố `Shared` cho *hai* handle cùng lúc:
handle gốc (`b1`, mà `ctx` của nó ta vừa CAS) và bản clone ta đang trả về. Cả hai giờ
đều trỏ vào `Shared` này, nên counter khởi tạo bằng 2. Kiểm bằng cách nghĩ đếm-số-lần
của Phần 3: hai handle → hai lần drop → về 0 → free một lần. Cân.

**Nhánh `Ok` — cái đẹp của promotion.** Cú CAS ghi vào `ctx` của *handle gốc `b1`*
(ta nhận `ctx: &AtomicPtr` chính là `&b1.ctx`). Nên `b1` *biến thành chia sẻ ngay tại
chỗ*, dù `b1.vtable` vẫn là `PROMOTABLE_*` (không đổi được — Phần 5). Lần sau `b1`
clone/drop, hàm `promotable_*` đọc `ctx`, thấy KIND_ARC (bit 0), tự đi nhánh `Shared`.
Bản clone mới thì mang thẳng `SHARE_VTABLE`. Hai "vị" handle arc-backed cùng tồn tại,
cùng đếm đúng một counter.

**Nhánh `Err(actual)` — `actual` KHÁC `shared`.** Đây là chỗ Phần 4 gọi là "phải cẩn
thận khi vứt counter thừa", và là bug kinh điển. `compare_exchange(expected, new)`
nghĩa: "*nếu* `ctx` vẫn bằng `expected` thì đổi thành `new`, không thì báo giá trị
hiện tại". Khi `Err(actual)`:

- `shared` = khối `Shared` **của mình** vừa cấp (ví dụ 0xBBB) — thua cuộc, *vô dụng*.
- `actual` = giá trị đang thật sự nằm trong `ctx` = khối `Shared` **của kẻ thắng** (ví
  dụ 0xAAA) — địa chỉ *khác hẳn*, vì mỗi thread `Box::new` một lần → hai vùng heap.

Nên ta phải (a) vứt `shared` của mình — và vứt *đúng cách*: `Box::from_raw(shared)`
chỉ giải phóng cái *vỏ điều khiển*, **không** đụng `buf` (vì `Shared` không có `Drop`
impl; `buf` giờ thuộc về `Shared` của kẻ thắng); rồi (b) `shallow_clone_arc(actual)`
để tăng counter của kẻ thắng. Dùng nhầm `shared` (đã free) ở bước (b) là use-after-free
tức thì, *và* bỏ rơi luôn `Shared` thật → counter lệch → double-free.

Kiểm counter trong cuộc đua 3 thread: kẻ thắng A tạo `Shared` với `ref=2` (gốc + A);
B và C thua, mỗi đứa `shallow_clone_arc(actual)` +1 → về `4`? Không — chỉ một trong B/C
"thua trước", nhưng cả hai đều +1, thành **4**... khoan. Đếm lại cho đúng: chỉ có *một*
handle gốc và *một* cú promote thắng (A). Mỗi thread clone tạo *một* handle mới. 3
thread clone → 3 handle mới + 1 gốc = 4 handle. A đặt ref=2 (gốc + handle của A), B +1
= 3 (thêm handle B), C +1 = 4 (thêm handle C). Đúng 4 handle sống → 4 lần drop → free
một lần. Cân.

### Vì sao số học `cap = (ptr - buf) + len` an toàn

`promote_vec` không được cho `cap` sẵn — nó khôi phục bằng số học. `(ptr - buf)` là
khoảng cách từ đáy buffer tới đầu view; cộng `len` ra khoảng cách tới *cuối* view.
Điều này chỉ đúng bằng kích thước allocation **nếu view luôn chạm cuối allocation** —
tức buffer chưa bao giờ bị cắt ngắn ở đuôi.

Và đúng là vậy, nhờ một invariant: **một handle VEC không bao giờ bị slice.** Vì
`slice` (mục sau) đi qua `clone`, mà clone một VEC thì *promote* nó thành ARC. Nên bạn
không bao giờ cầm một VEC đã-cắt — một VEC luôn là buffer nguyên vẹn, `ptr == buf`,
`cap == len`. Đó là lý do `free_boxed_slice` cũng khôi phục `cap` bằng đúng số học ấy,
thay vì phải lưu `cap`:

```rust
unsafe fn free_boxed_slice(buf: *mut u8, ptr: *const u8, len: usize) {
    let cap = (ptr as usize - buf as usize) + len;
    drop(Vec::from_raw_parts(buf, cap, cap));
}
```

(Trái lại, repr `shared` *có* lưu `cap` trong `Shared`, vì *sau khi* promote bạn được
cắt tự do cả hai đầu, nên không khôi phục `cap` từ view được nữa. Một cái khôi phục
bằng số học, một cái lưu tường minh — sự bất đối xứng đó chính là hệ quả của invariant.)

## `slice`: O(1), và nó *thực thi* invariant

Cả `Bytes` sinh ra là để `slice` rẻ. Bí quyết: **clone, rồi thu hẹp view** — không
copy gì.

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... tính start, end, assert trong biên ...
    if start == end {
        return Bytes::from_static(&[]); // rỗng → khỏi giữ refcount
    }
    let mut sub = self.clone(); // chia sẻ backing (tăng counter / promote nếu đang VEC)
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

Cái hay là bạn viết *một lần* mà đúng cho *cả ba* repr, vì `clone` đã lo phần
repr-riêng:

- **static**: clone tầm thường (không counter). Thu hẹp vào slice `'static` → vẫn
  static, drop vẫn no-op. Không cấp phát.
- **shared**: clone tăng counter atomic. Thu hẹp view; `Shared.buf`/`cap` không đổi
  nên drop vẫn free từ đáy. *Đây* là lý do `Shared` lưu `buf`/`cap` tách khỏi view.
- **promotable**: clone **promote** thành shared, rồi thu hẹp cái đó.

Chính điểm cuối là chỗ đẹp nhất: **`slice` một `Bytes` promotable sẽ promote nó** —
đúng cái invariant "VEC không bao giờ bị slice" mà cả `promote_vec` lẫn
`free_boxed_slice` dựa vào để khôi phục `cap` bằng số học. `slice` không chỉ *tuân*
invariant, nó *thực thi* invariant, bằng cấu trúc: đường duy nhất để cắt là qua clone,
và clone promote. Một vòng khép kín.

Hai điểm an toàn nhỏ: `ptr.add(start)` nằm trong biên vì đã assert `start <= end <=
len`; và cộng một offset nhỏ vào con trỏ non-null không thể ra null, nên
`new_unchecked` vẫn đúng.

## Xong. Nhìn lại toàn cảnh code

Ba repr, bốn-cộng hàm, một invariant:

```
static     clone: copy struct         drop: no-op            (free 0 lần)
shared     clone: fetch_add Relaxed    drop: fetch_sub Release + fence(Acquire)  (free 1 lần)
promotable clone: chưa promote → promote_vec (CAS);  đã rồi → shallow_clone_arc
           drop:  chưa promote → free_boxed_slice;   đã rồi → release_shared

invariant:  slice ⇒ clone ⇒ (VEC thì promote) ⇒ VEC không bao giờ bị cắt
            ⇒ VEC luôn ptr==buf, cap==len ⇒ khôi phục cap bằng số học là an toàn
```

Và đường đọc — `deref`, `len`, so sánh, hash — vẫn chỉ chạm `ptr` + `len`, không bao
giờ đụng `ctx`/`vtable`, nên rẻ y như `Arc<[u8]>`. Toàn bộ cỗ máy `ctx`/`vtable`/tag/
CAS/ordering *chỉ* vào cuộc khi `clone` hoặc `drop`.

## Kiểm chứng: đừng tin, hãy đo

Loại bug ở bài này — KIND đảo ngược, `shared` vs `actual`, ordering sai — *compile
sạch* và thường *chạy có vẻ đúng* trên một thread. Chúng chỉ lộ ra khi có đua hoặc khi
một công cụ soi vào mô hình bộ nhớ. Nên hai thứ bắt buộc:

- **`miri`**: `cargo +nightly miri test` — bắt use-after-free, double-free, đọc bộ
  nhớ chưa khởi tạo, và data race. Ba trong bốn bug ở trên bị `miri` tóm ngay.
- **Test đua promotion**: cho N thread cùng `clone` *một* handle gốc, ép nhiều cú
  `promote_vec` chạy song song để chọc vào nhánh `Err(actual)`; chạy lặp lại nhiều
  lần. `loom` (nếu bạn muốn đi xa hơn) sẽ vét cạn các thứ tự sắp-xếp-lại có thể.

Nhắc lại câu thứ ba trong ba câu chốt của Phần 5: con bug đáng sợ trong unsafe không
phải con làm sập chương trình, mà là con *chạy đúng* — trực giác Rust an toàn bị đảo,
mặc định của cái sai là im lặng. Ở promotable, cái im lặng đó dày nhất. Luôn mang theo
`miri`.

## Hết loạt bài

Từ "một byte đi vào từ dây mạng" (Phần 1) tới `promote_vec` với nhánh thua cuộc của nó
(bài này), mỗi mảnh đều bị *ép* bởi mảnh trước: `Arc<[u8]>` không cho `freeze` O(1) →
hạ sở hữu xuống vtable → tách đọc khỏi sở hữu → clone-độc-quyền là double-free →
promotion sửa-ngược → `AtomicPtr` giải ba đòi hỏi → và cuối cùng, code hoá tất cả với
tagged pointer, CAS, và một invariant tự-thực-thi. Không mảnh nào từ trên trời rơi
xuống.

Giờ bạn không chỉ *đọc* được `bytes`, bạn *viết lại* được nó — và biện luận được cho
từng dòng.

---

*Quay lại: [Phần 7](07_from_vec_and_bit_tagging.md) · [Mục lục](00_index.md)*

*English: [`../en/08_promotable_and_slice.md`](../en/08_promotable_and_slice.md)*
