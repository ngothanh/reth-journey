# Phần 6 — Từ mô hình xuống code: `static` và `shared`

Năm phần đầu dựng xong *mô hình*: một `Bytes` gồm `ptr` + `len` (byte nào) và `data`
+ `vtable` (ai sở hữu), với ba cách sở hữu — `static`, `promotable`, `shared`. Từ
phần này trở đi ta *ngồi viết*. Và điều dễ chịu: hai trong ba cách sở hữu viết ra
gần như tầm thường. `static` là bài khởi động, `shared` chỉ khó đúng một chỗ — nhưng
cái chỗ đó lại là bài học memory ordering quan trọng nhất mà Phần 5 *chưa* chạm tới.

Ta sẽ viết bốn hàm vtable đầu tiên: `static_clone`, `static_drop`, `share_clone`,
`share_drop`. Và ta trả lời một câu Phần 5 để dành: ordering của promotion là để
*công bố* một khối `Shared`; còn ordering của `share_drop` là để *giải phóng* một
buffer chia sẻ — một mối nguy hoàn toàn khác, tên là *free-while-read*.

## Bản đồ: đọc `ctx` → biết repr → chạy hàm nào

Cả chặng hiện thực xoay quanh một động tác: mỗi hàm vtable đọc `ctx`, suy ra đang ở
repr nào, rồi rẽ. Neo cái này trong đầu trước khi vào code:

```
vtable = STATIC       ctx = null              clone: copy struct   · drop: no-op        (free 0 lần)

vtable = SHARE        ctx = *mut Shared       clone: +refcount     · drop: -refcount    (free 1 lần)

vtable = PROMOTABLE   ctx LẺ  (KIND_VEC)      clone: promote_vec   · drop: free_boxed_slice
                      ctx CHẴN (KIND_ARC)     clone/drop: đi qua Shared (như hàng SHARE)

     chuyển trạng thái DUY NHẤT, một chiều:
        PROMOTABLE/VEC ──(clone lần đầu: promote_vec, CAS)──► PROMOTABLE/ARC
```

Phần 6 viết hai hàng đầu (`STATIC`, `SHARE`). Phần 7 lo cách *encode* `ctx` cho
`PROMOTABLE` (mẹo lẻ/chẵn). Phần 8 viết cú chuyển trạng thái (`promote_vec`) và hai
hàm `PROMOTABLE`. Nhớ: `vtable` đóng băng lúc sinh ra; chỉ *bit KIND trong `ctx`* đổi
khi promote — nên "PROMOTABLE/ARC" vẫn dùng vtable promotable, chỉ rẽ sang nhánh
Shared.

## `static`: bài khởi động

Nhớ lại: một `Bytes` `static` trỏ vào vùng nhớ sống mãi (`&'static [u8]`), nên không
có gì để đếm, không có gì để giải phóng. `data` để null. Hai hàm của nó là hai câu
trả lời ngắn nhất trong cả loạt bài:

```rust
fn static_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    // Không có refcount. Clone chỉ là dựng lại một handle trỏ vào cùng chỗ.
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
    // Không làm gì. Byte sống mãi, không có gì để giải phóng.
}
```

`static_drop` rỗng *có chủ đích* — nó chính là hiện thân của câu hỏi thứ nhất trong
năm câu hỏi mang-đi: *vùng nhớ này giải phóng đúng mấy lần?* Với `static`, câu trả
lời là **0**. Một hàm `drop` rỗng không phải là chưa-viết-xong; nó là "0 lần" viết
thành code. Chú ý `ctx` là null ở đây, nên tuyệt đối không được deref nó — và may
thay, chẳng có dòng nào deref cả.

## `shared`: khối `Shared` và ba trường của nó

`shared` là bản `Arc<[u8]>` tự-viết. Ta cần một khối điều khiển trên heap chứa
counter:

```rust
struct Shared {
    buf: *mut u8,          // địa chỉ GỐC của allocation — để sau này trả lại cho allocator
    cap: usize,            // kích thước allocation — cùng buf tạo thành "cách giải phóng"
    ref_count: AtomicUsize,
}
```

Một chi tiết Phần 4 đã báo trước, giờ thành cụ thể: `Shared.buf` là địa chỉ *gốc*
của allocation, **không** phải con trỏ mà handle đang cầm (`Bytes.ptr`). Với một
handle chưa cắt, hai cái bằng nhau; nhưng sau khi `slice`, `Bytes.ptr` trỏ vào *giữa*
buffer, trong khi `buf` vẫn phải là điểm đầu — vì bạn chỉ được trả về allocator đúng
cái con trỏ nó đã giao. Đó là lý do `buf`/`cap` sống trong `Shared`, tách khỏi
`ptr`/`len` của handle. (Phần 8 sẽ dùng đúng tính chất này để làm `slice` O(1).)

## `share_clone`: tăng counter, và vì sao `Relaxed` là đủ

```rust
fn share_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { shallow_clone_arc(shared, ptr, len) }
}

unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    let old = (*shared).ref_count.fetch_add(1, Ordering::Relaxed);
    if old > isize::MAX as usize / 2 {
        abort(); // xem "Vì sao abort chứ không panic" bên dưới
    }
    Bytes {
        ptr: NonNull::new_unchecked(ptr as *mut u8),
        len,
        ctx: AtomicPtr::new(shared as *mut ()),
        vtable: &SHARE_VTABLE,
    }
}
```

Có *hai* thao tác nguyên tử ở đây, và cả hai đều `Relaxed`. Đây là chỗ dễ khiến người
mới hoang mang, nên nói kỹ.

**Cú `load` cái con trỏ `shared`: `Relaxed`.** Nhớ nguyên tắc từ Phần 5 — ordering
không bảo vệ *bản thân giá trị nguyên tử*, nó bảo vệ *bộ nhớ khác nằm quanh* thao tác
đó. Ở đây con trỏ `shared` là một địa chỉ *ổn định*: nó được đặt lúc handle sinh ra
và không đổi suốt đời handle. Ta không dùng cú đọc này như một *cờ báo* rằng có bộ
nhớ mới nào đó vừa được công bố — ta chỉ đang lấy một địa chỉ mà *mình vốn đã sở hữu*.
Không có cạnh happens-before nào cần dựng, nên `Relaxed` là mức tối thiểu trung thực.

(Đối chiếu để nhớ: cú đọc `data` trong `promotable_clone` ở Phần 5 phải `Acquire`,
vì ở đó nó *có thể* là một cờ báo "vừa promote xong, đây là `Shared` mới" — và ta sẽ
đi *đọc nội dung* khối `Shared` đó. Cùng là `load`, khác ordering, vì một cái là "lấy
địa chỉ đã sở hữu", cái kia là "nhận bộ nhớ vừa công bố".)

**Cú `fetch_add` tăng counter: `Relaxed`.** Tăng refcount *không công bố* bộ nhớ nào
cho ai. Để gọi được `clone`, bạn đã đang cầm một handle sống → payload và khối
`Shared` đã hiện hữu và nhìn thấy được với bạn rồi. Cú tăng chỉ là số học trên một bộ
đếm; không có gì để đồng bộ. Nên `Relaxed`.

**Chốt overflow — và vì sao `abort` chứ không `panic`.** Vì `fetch_add` dùng
`Relaxed` rất rẻ, một vòng `mem::forget` bệnh hoạn (hoặc bão clone) *về lý thuyết* có
thể làm `usize` tràn về số nhỏ → giải phóng non → use-after-free. Nên ta chốt: nếu
counter vượt ngưỡng thì dừng cứng. Dừng bằng `abort` chứ không `panic`, vì tới lúc đó
an toàn bộ nhớ đã hỏng — mà `panic` thì *có thể bị `catch_unwind` bắt lại* và nó
*unwind qua các `Drop`*, mà `Drop` lại đụng đúng cái counter không còn tin được. `abort`
là dừng vô điều kiện. (Ta kiểm ngưỡng bằng *giá trị trả về của `fetch_add`*, không
phải một cú `load` riêng — để tránh khe TOCTOU giữa "đọc" và "tăng".)

## `share_drop`: mối nguy free-while-read

Đây là phần đáng giá của cả bài. `share_drop` giảm counter, và nếu mình là kẻ cuối
cùng thì giải phóng buffer + khối `Shared`.

```rust
fn share_drop(ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { release_shared(shared) }
}

unsafe fn release_shared(shared: *mut Shared) {
    if (*shared).ref_count.fetch_sub(1, Ordering::Release) != 1 {
        return; // chưa phải kẻ cuối — xong
    }
    core::sync::atomic::fence(Ordering::Acquire);
    let cap = (*shared).cap;
    drop(Vec::from_raw_parts((*shared).buf, cap, cap)); // giải phóng buffer từ ĐỊA CHỈ GỐC
    drop(Box::from_raw(shared));                        // giải phóng khối Shared
}
```

Chú ý `release_shared` **không cần `ptr`/`len` của handle** — nó giải phóng cả
allocation từ `Shared.buf`/`Shared.cap`. Chính điều này khiến `slice` an toàn: dù
handle đã cắt tới đâu, drop luôn trả về đúng con trỏ gốc. (Dùng `cap` cho *cả* độ dài
lẫn capacity của `Vec::from_raw_parts` — ta mô tả *allocation*, không phải *view*.
`u8` không có destructor nên độ dài chỉ ảnh hưởng "chạy mấy destructor", nhưng mô tả
đúng allocation là thói quen phải giữ: ngày buffer chứa kiểu có `Drop`, dùng nhầm
`len` của view sẽ chạy sai số destructor.)

Giờ tới cái ordering, và vì sao nó **khác hẳn** ordering của Phần 5.

### Vấn đề: giải phóng trong khi thread khác còn đọc

Phần 5 lo mối nguy *publish-before-read*: công bố địa chỉ khối `Shared` trước khi nội
dung nó kịp hiện ra. Ở đây mối nguy ngược lại: **giải phóng buffer trong khi một
thread khác còn đang đọc nó** — free-while-read.

Dựng cảnh: `b1` và `b2` là hai handle chia sẻ cùng một buffer, ở hai thread khác
nhau. Thread A đọc vài byte rồi drop `b2`; thread B drop `b1`. Counter đi `2 → 1 → 0`.
Trực giác tuần tự của bạn nói: "counter về 0 nghĩa là không ai còn dùng → giải phóng
an toàn". Đúng — *nếu chỉ có một thread*. Nhưng qua nhiều thread, trên phần cứng
sắp-xếp-lại, **"counter về 0" và "mọi cú đọc đã xong" KHÔNG tự động là cùng một thời
điểm.** CPU/compiler được phép dời cú đọc buffer của thread A xuống *sau* cú giảm
counter của chính nó.

Xem nó vỡ khi *không* có ordering (giả sử cả hai giảm đều `Relaxed`):

```
Thread A                              Thread B
  fetch_sub → 2→1 (Relaxed)
  ...đọc b2[0] BỊ DỜI xuống đây          fetch_sub → 1→0, thấy 0
       │                                 free(buf)         ← buffer biến mất
       └── đọc b2[0] NGAY BÂY GIỜ ←──────────────────────── USE-AFTER-FREE
```

Cú đọc của A bị dời qua cú giảm của nó, nên B thấy counter 0 và giải phóng *trong khi*
cú đọc của A còn treo. Đọc phải bộ nhớ đã chết.

### Cách chữa: `Release` khi giảm, `Acquire` fence trước khi giải phóng

- Mỗi kẻ drop giảm counter bằng **`Release`** → "công bố: mọi truy cập buffer của tôi
  *nằm trước* cú giảm này, không được trượt xuống sau."
- Kẻ cuối cùng (cú `fetch_sub` trả về 1) chạy một **`fence(Acquire)`** *trước* khi
  giải phóng → "đăng ký nhận: đồng bộ với *mọi* cú giảm `Release` của các thread
  khác, nên mọi truy cập buffer của họ giờ happens-before cú giải phóng của tôi."

Cặp `Release`/`Acquire` này chính là cái *dán* "counter về 0" vào "mọi kẻ đọc đã thật
sự xong". Thiếu nó, counter đúng nhưng khả-kiến-bộ-nhớ sai.

Một chi tiết tinh tế khiến *một* cái fence đủ đồng bộ với *tất cả* các cú giảm: mỗi
`fetch_sub` là một thao tác đọc-sửa-ghi, nên cú giảm cuối cùng đọc một giá trị nằm
trong *chuỗi release* dẫn dắt bởi mọi cú giảm `Release` trước đó — đó là điều cho phép
một `fence(Acquire)` bắt cặp với tất cả.

### Vì sao `fence(Acquire)` riêng thay vì `fetch_sub(AcqRel)`?

Bạn *có thể* làm cú giảm thành `AcqRel` và bỏ fence — vẫn đúng. Nhưng `AcqRel` ép
`Acquire` lên *mọi* cú giảm, kể cả những cú không-cuối (chỉ return, chẳng giải phóng
gì). Fence riêng để **chỉ kẻ cuối cùng** trả cái giá của rào `Acquire`; những kẻ khác
chỉ giảm `Release` rẻ hơn. Đây là chuyện hiệu năng, không phải đúng-sai — và là đúng
lý do `Arc` thật được viết như vậy.

## Đối chiếu hai loại ordering trong loạt bài

Đây là điểm để mang đi, vì nó tách bạch hai mối nguy mà người ta hay gộp làm một:

| | Phần 5 (promotion) | Phần 6 (`share_drop`) |
|---|---|---|
| Mối nguy | publish-before-read: công bố con trỏ trước khi nội dung hiện | free-while-read: giải phóng trong khi kẻ khác còn đọc |
| Thao tác | CAS ghi `data` = `Shared` mới | `fetch_sub` counter |
| Bên "công bố" | CAS thành công → `Release` | mỗi cú giảm → `Release` |
| Bên "nhận" | cú `load`/CAS-thất-bại → `Acquire` | `fence(Acquire)` của kẻ cuối |

Cùng một cặp `Release`/`Acquire`, hai bài toán khác nhau. Nguyên tắc chung vẫn đúng:
*mỗi khi một thao tác nguyên tử của bạn được thread khác dùng làm tín hiệu để quyết
định "giờ tôi được đụng vào (hoặc giải phóng) vùng nhớ chung", thì các truy cập bộ
nhớ quanh thao tác đó phải được sắp thứ tự qua cặp Release/Acquire.*

## Đã có gì, và Phần 7 làm gì

Bốn hàm xong: `static_*` (0 lần giải phóng), `share_*` (kỷ luật `Relaxed` khi tăng,
`Release`+`fence(Acquire)` khi giảm). Điểm cốt: ordering của `share_drop` không phải
cái ordering của Phần 5 — nó chống free-while-read, không phải publish-before-read.

Nhưng ta vẫn chưa *tạo* được một `Bytes` `shared`. Chưa có gì gọi tới `SHARE_VTABLE`.
Mảnh còn thiếu là `from_vec` — biến một `Vec<u8>` thành `Bytes`. Và đúng lúc viết
`from_vec`, ta đâm vào cái Phần 5 cố tình để dành trong một ghi chú ngoài lề: làm sao
*một ô 8 byte* vừa chứa được một con trỏ buffer vừa chứa được một con trỏ `Shared`, và
phân biệt được hai loại? Đó là mẹo nhồi-bit, và Phần 7 mổ nó tới tận đáy.

---

*Tiếp theo: [Phần 7 — `from_vec` và mẹo nhồi-bit](07_from_vec_and_bit_tagging.md) ·
[Mục lục](00_index.md)*

*English: [`../en/06_static_and_shared.md`](../en/06_static_and_shared.md)*
