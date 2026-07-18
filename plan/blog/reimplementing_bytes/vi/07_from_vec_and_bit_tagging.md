# Phần 7 — `from_vec` và mẹo nhồi-bit: một ô 8 byte, hai ý nghĩa

Phần 6 viết xong `share_*` nhưng chưa có cách nào *tạo* ra một `Bytes` đi vào đường
đó. Cửa vào là `from_vec` — nhận một `Vec<u8>` mà *không* copy. Và đúng lúc viết
`from_vec`, ta đâm vào cái Phần 5 để dành trong một ghi chú ngoài lề: `data` của một
`Bytes` `promotable` phải chứa được *hai loại* con trỏ khác nhau — con trỏ buffer
(chưa promote) *hoặc* con trỏ `Shared` (đã promote) — trong cùng 8 byte, và mọi hàm
sau này phải phân biệt được đang chứa loại nào.

Bài này mổ đúng cái mẹo đó tới đáy. Nó là chỗ "tủn mủn" nhất của cả thiết kế, nên ta
đi thật chậm, và cuối bài chỉ cho một câu để nhớ khiến mọi thứ gọn lại.

## `from_vec`: chuẩn hoá rồi để dành

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // rỗng → đi thẳng repr static, không cấp phát
    }
    let boxed: Box<[u8]> = bytes.into_boxed_slice(); // chuẩn hoá: cap == len
    let len = boxed.len();
    let buf = Box::into_raw(boxed) as *mut u8;       // NHẬN quyền sở hữu — giờ ta lo việc free
    // ... nhồi-bit rồi dựng Bytes (bên dưới) ...
}
```

Ba việc, mỗi việc có lý do:

**`is_empty` → `from_static(&[])`.** `into_boxed_slice` của một `Vec` rỗng cho một
con trỏ *dangling* mà ta không muốn nhồi-bit hay giải phóng. Cho rỗng đi thẳng repr
`static` (buffer sống-mãi rỗng) là sạch nhất — không cấp phát cho 0 byte.

**`into_boxed_slice()` — chuẩn hoá `cap == len`.** Đây là chi tiết mấu chốt mà sau này
ta sẽ *dựa vào để khỏi phải lưu `cap`*. Một `Vec` có thể có `cap > len` (thừa chỗ);
`into_boxed_slice` co lại cho `cap == len`. Cái giá: nếu `Vec` có chỗ thừa, thao tác
này *cấp phát lại và memcpy*. Đúng, và `bytes` thật cũng làm y vậy — nên cứ nhớ là cú
realloc đó có thể xảy ra với `Vec` còn dư capacity.

**`Box::into_raw` — nhận quyền sở hữu.** Trước dòng này, `Box` sẽ tự giải phóng buffer
khi ra khỏi scope. Sau `into_raw`, `Box` biến mất và *không gì* tự giải phóng nữa —
**bạn** đã ký nhận việc free (sau này qua `free_boxed_slice`/`release_shared`). `buf`
giờ là địa chỉ heap của byte đầu tiên. Nếu bỏ rơi `buf` ở đây là leak.

## Bài toán: một ô, hai ý nghĩa

Một `Bytes` `promotable` cần `data` (ta gọi trường này là `ctx`) chứa:

- lúc **chưa** promote: con trỏ tới **buffer** thô,
- lúc **đã** promote: con trỏ tới khối **`Shared`**.

Và Phần 5 đã kết luận vì sao cái nhãn-phân-loại này *phải* nằm trong `ctx`: promotion
đổi trạng thái *giữa chừng vòng đời* qua một cú CAS một-từ trên `ctx` — mà `vtable`
thì đóng băng lúc sinh ra, không CAS chung một lượt được. Nên ta cần một cách, đọc
*chỉ mình `ctx`*, biết nó đang là loại nào.

Cách: mượn **bit thấp nhất** của con trỏ làm cờ KIND.

```rust
const KIND_ARC: usize = 0b0; // bit thấp = 0 → ctx là *mut Shared
const KIND_VEC: usize = 0b1; // bit thấp = 1 → ctx là con trỏ buffer
const KIND_MASK: usize = 0b1;
```

Vì sao bit thấp *là chỗ trống để mượn*? Vì **căn lề (alignment)**. Một giá trị kiểu
`T` căn lề `A` luôn nằm ở địa chỉ chia hết cho `A` — bội của 8 trong nhị phân luôn tận
cùng `000`. Khối `Shared` chứa con trỏ + `usize` + `AtomicUsize` nên căn lề ≥ 8 → địa
chỉ của nó **luôn tận cùng bit 0**. Vậy `Shared` *tự nhiên* là `KIND_ARC`, khỏi làm
gì.

## Câu để nhớ: **VEC luôn LẺ, ARC luôn CHẴN**

Mọi thứ suy ra từ đúng một dòng đó. Khi bất kỳ hàm nào **nhìn vào `ctx`** để giải mã
trạng thái:

- **`ctx` lẻ (bit = 1) → VEC** (còn là buffer, chưa promote),
- **`ctx` chẵn (bit = 0) → ARC** (đã là `Shared`).

- *ARC luôn chẵn*: `Shared` căn lề 8 → tự nhiên bit 0. Miễn phí.
- *VEC phải lẻ*: để **không đụng hàng với ARC**. Nếu một con trỏ buffer chẵn được cất
  thẳng vào `ctx`, hàm `clone`/`drop` sau này nhìn thấy bit 0 → tưởng "đã promote,
  đây là `Shared`" → ép buffer thành `*mut Shared` rồi đọc `ref_count`... tức đọc mấy
  byte dữ liệu của bạn tưởng là bộ đếm → tan nát. Nên ta *ép* trạng thái VEC luôn đọc
  ra lẻ.

## Cái rắc rối: buffer `u8` có thể chẵn *hoặc* lẻ

Đây là chỗ khiến bài này khác một tagged pointer trong sách giáo khoa. Buffer là
`u8`, **alignment = 1**, nên địa chỉ của nó **không** đảm bảo bit thấp = 0 — nó có thể
chẵn hoặc lẻ. Nhưng ta *muốn* nó luôn đọc ra lẻ (KIND_VEC). Nên:

- **buffer chẵn** (bit 0): phải *bật* bit lên (`buf | 1`) để đánh dấu VEC. Muốn lấy
  lại địa chỉ thật, phải *xoá* bit đó (`& !1`). → dùng **`PROMOTABLE_EVEN_VTABLE`**.
- **buffer lẻ** (bit 1 sẵn): đã đọc ra VEC rồi, KHÔNG cần bật. Nhưng bit 1 này *là
  phần thật của địa chỉ*, nên khi lấy lại tuyệt đối KHÔNG được xoá. Cất nguyên. → dùng
  **`PROMOTABLE_ODD_VTABLE`**.

Code của `from_vec` phần còn lại chính là cái rẽ nhánh đó:

```rust
    if buf as usize & KIND_MASK == 0 {
        // EVEN: bật bit làm dấu VEC; sau này recover bằng cách MASK bỏ bit.
        let ctx = (buf as usize | KIND_VEC) as *mut ();
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(ctx), vtable: &PROMOTABLE_EVEN_VTABLE }
    } else {
        // ODD: bit thấp đã là 1 == VEC; cất con trỏ NGUYÊN VĂN, sau này KHÔNG mask.
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(buf as *mut ()), vtable: &PROMOTABLE_ODD_VTABLE }
    }
```

Chú ý `ptr` cất **con trỏ sạch** (`buf` chưa nhồi bit); chỉ `ctx` mang cái tag. Nhờ
vậy `deref` (đọc qua `ptr`) không bao giờ thấy cái bit — đường đọc luôn dùng địa chỉ
thật.

## Vì sao *hai* vtable, không phải một?

Đây là câu hỏi hay nhất, và câu trả lời chạm vào một sự thật về thông tin. Thử hình
dung bạn chỉ có một vtable và chỉ mình `ctx`:

```
Case 1 (buffer chẵn 0x1000): bật bit → ctx = 0x1001 → recover cần xoá bit  → 0x1000
Case 2 (buffer lẻ   0x1001): để nguyên → ctx = 0x1001 → recover phải giữ  → 0x1001
```

**Hai trường hợp có `ctx` giống hệt nhau (0x1001), nhưng địa chỉ buffer thật khác
nhau (0x1000 vs 0x1001).** Chỉ nhìn `ctx`, bạn *không thể* biết buf thật là cái nào —
đã mất 1 bit thông tin. Việc nhồi tag vào bit thấp là **lossy với địa chỉ lẻ**.

Nên bạn cần **1 bit thừa** cất ở đâu đó để nhớ "buffer gốc chẵn hay lẻ" — tức "recover
có phải mask không". Và **con trỏ vtable chính là chỗ cất bit đó**, miễn phí, vì bạn
vốn đã mang theo nó. `EVEN` = "recover thì mask", `ODD` = "recover thì giữ nguyên".
Một vtable + mình `ctx` thì *thiếu thông tin*, chấm hết.

(Không phải 4 nhánh khác nhau đâu: hai case ARC — dù EVEN hay ODD — *giống hệt nhau*,
đều đọc `ctx` thẳng thành `*mut Shared` không mask, vì `Shared` luôn bit 0. EVEN/ODD
*chỉ* khác nhau ở nhánh VEC.)

Bảng đầy đủ, hai thời điểm khác nhau — lúc *encode* (`from_vec` nhìn `buf`) và lúc
*decode* (`clone`/`drop` nhìn `ctx`):

| buffer gốc | encode (nhìn `buf`) | decode (nhìn `ctx`) | vtable |
|---|---|---|---|
| chẵn | bật bit `\| 1` | `ctx` chẵn → **ARC**, `ctx` lẻ → **VEC (mask để recover)** | EVEN |
| lẻ | để nguyên | `ctx` chẵn → **ARC**, `ctx` lẻ → **VEC (giữ nguyên)** | ODD |

## Một ghi chú thực tế: `ODD` gần như không bao giờ chạy

Trên thực tế, allocator hệ thống *căn lề dư* — `malloc`/allocator của Rust thường trả
con trỏ căn lề ≥ 16 kể cả cho buffer `u8` (vốn chỉ cần lề 1). Nên `buf` gần như luôn
chẵn, và `PROMOTABLE_ODD_VTABLE` gần như là code chết trên allocator thường. Nhưng lề
của `u8` *không đảm bảo* chẵn (một allocator tuỳ biến, arena, hay sub-allocation có
thể trả địa chỉ lẻ), nên nhánh ODD tồn tại thuần tuý như một *lưới an toàn về tính
đúng*. Muốn thật sự chạy qua `promotable_odd_*`, bạn phải cố tình dựng một `Bytes`
trên buffer địa chỉ lẻ — đường `from_vec` thường có thể chẳng bao giờ tới đó.

## Một lối thoát: nếu thấy nhồi-bit là thừa

Cảm giác "tủn mủn" là *đúng*. Và nó chỉ ra một điều: nhồi-bit là công cụ cho *tính
tổng quát*, không bắt buộc cho `Bytes` tối giản. `bytes` thật nhét `buf` vào `ctx` vì
nó hỗ trợ `advance`/`split` — những thao tác dời `ptr` ra khỏi `buf` mà *không*
promote, nên nó buộc phải nhớ `buf` gốc ở chỗ khác → sinh ra tag + EVEN/ODD.

Nhưng nếu `Bytes` của bạn có invariant "VEC không bao giờ bị slice" (Phần 8 sẽ dựng
nó — `slice` luôn promote), thì một handle VEC *luôn* có `ptr == buf` và `cap == len`.
Nghĩa là `buf`/`cap` đã nằm sẵn trong `ptr`/`len` rồi — nhồi lại vào `ctx` là *thừa*.
Lúc đó bạn có thể gộp về **một vtable duy nhất**, phân biệt VEC/ARC bằng null:

```
ctx == null  → VEC (lấy buf từ self.ptr, cap từ self.len)
ctx != null  → ARC (ctx là *mut Shared)
```

`null` không bao giờ đụng hàng với con trỏ `Shared`, nên là sentinel tuyệt đối an
toàn, và toàn bộ EVEN/ODD biến mất. Đây là một quyết định thiết kế có thật: giữ tag
để mirror `bytes` 1:1 và sẵn sàng cho `advance` sau này, hay bỏ tag cho gọn với feature
set hiện tại. Cả hai đều đúng — biết mình đang trả giá cho cái gì mới là điều quan
trọng.

## Đã có gì, và Phần 8 làm gì

`from_vec` xong: chuẩn hoá về boxed slice (`cap == len`), nhận quyền free, rồi nhồi
bit theo chẵn/lẻ để chọn `EVEN`/`ODD`. Câu để mang đi: **VEC lẻ, ARC chẵn** — và
chẵn/lẻ của buffer *chỉ* quyết định cách *recover* (mask hay không), được ghi nhớ qua
việc chọn vtable.

Giờ ta đã tạo được một `Bytes` `promotable`, nhưng bốn hàm `promotable_*` vẫn trống,
và cú `clone` đầu tiên — cái *promotion* mà Phần 4 và 5 dựng cả mô hình để giải thích
— vẫn chưa viết. Phần 8 viết nốt: bốn hàm dispatch, cuộc đua CAS với nhánh thua, và
`slice` O(1) — cái vừa dùng promotion vừa *thực thi* cái invariant "VEC không bao giờ
bị slice" đã hứa ở trên.

---

*Tiếp theo: [Phần 8 — promotable đầy đủ và `slice`](08_promotable_and_slice.md) ·
[Mục lục](00_index.md)*

*English: [`../en/07_from_vec_and_bit_tagging.md`](../en/07_from_vec_and_bit_tagging.md)*
