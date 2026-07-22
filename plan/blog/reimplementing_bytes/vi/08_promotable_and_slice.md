# Phần 8 — Khi yêu cầu đẻ thêm: `advance`, lazy-promote, và trilemma

Phần 7 xây bản đơn giản nhất — cap-in-ctx — thoả đúng yêu cầu *hiện tại*: zero-copy,
zero-alloc `freeze`, `slice` O(1), lazy-promote. Nhưng phần mềm thật hiếm khi đứng yên.
Bài này thêm yêu cầu *từng cái một*, xem cái gì vỡ, và liệt kê **mọi cách mã hoá `ctx`**
cùng cái giá của từng cách. Kết lại bằng một định lý bất-khả: trong struct 4-từ, bạn
không thể có tất cả.

## Yêu cầu A: in-place `advance`

**`advance` là gì.** `bytes::Buf::advance(n)` — "nuốt" `n` byte đầu bằng cách *dời con
trỏ view* (`self.ptr += n`, `len -= n`) **in place**, trên một handle đang-một-chủ, *mà
không clone*. Đây là con dao của một *cursor tiêu thụ*.

**Khi nào cần.** Đọc frame mạng theo con trỏ chạy; vài decoder streaming walk thẳng
trên một owned buffer. (Chú ý: RLP của Ethereum thường *không* cần — bạn walk cursor
trên `&[u8]` mượn, không dời con trỏ của owned `Bytes`. Đó là lý do Phần 7 hợp cho
`Bytes` này.)

**Vì sao cap-in-ctx vỡ.** Sau `advance(3)`, `self.ptr = buf + 3 ≠ buf`. Nhưng
`owned_drop` free bằng `dealloc(self.ptr, cap)` = `dealloc(buf + 3, ...)` — giải phóng
một con trỏ *giữa* allocation → **UB / hỏng heap**. Gốc rễ: cap-in-ctx *giả định*
`self.ptr == buf`, và `advance` phá đúng giả định đó.

Có `advance` thì `self.ptr` không còn đáng tin làm `buf`. Ta phải cất `buf` chỗ khác.
Có hai lối, mỗi lối một cái giá.

## Lối 1 cho `advance`: cất `buf` vào `ctx` → sinh ra EVEN/ODD

Nếu `self.ptr` không đáng tin, nhét *con trỏ buffer* vào `ctx`. Nhưng giờ `cap` không
còn chỗ trong `ctx` (ô đã bận chứa con trỏ). Ta khôi phục `cap` bằng **số học**:
`cap = (ptr - buf) + len` = khoảng cách từ base tới *cuối* view. Đúng *chỉ khi* view
luôn chạm cuối allocation — mà `advance` chỉ trim đầu (view-end đứng yên), nên số học
ổn... **với điều kiện `cap == len` lúc tạo.** Ép `cap == len` = `into_boxed_slice`
(shrink Vec) → **mất zero-copy-từ-Vec** (một cú realloc + memcpy nếu Vec dư chỗ).

Rồi tới pointer tagging — vì `ctx` giờ chứa *con trỏ*, cần một bit phân biệt OWNED với
ARC. Con trỏ buffer `u8` (align 1) *không* có bit thấp trống đảm bảo:

```
Case chẵn (buf 0x1000): bật bit → ctx = 0x1001 → recover cần XOÁ bit → 0x1000
Case lẻ   (buf 0x1001): để nguyên → ctx = 0x1001 → recover phải GIỮ  → 0x1001
```

**Hai case `ctx` giống hệt (0x1001) nhưng `buf` khác nhau** → gắn tag vào bit thấp là
*lossy*. Bạn cần **1 bit thừa** cất "gốc chẵn hay lẻ" — và *con trỏ vtable* là chỗ cất
nó: **`EVEN`** ("recover thì mask") vs **`ODD`** ("giữ nguyên"). Đây là lúc **hai
vtable EVEN/ODD ra đời — như *cái giá của việc cất con trỏ*, tức cái giá của `advance`.**

Đây chính là đường "từ Vec" của `bytes` thật. **Tradeoff: được `advance` + giữ
lazy-promote, nhưng mất zero-copy-từ-Vec (shrink) + gánh EVEN/ODD.**

## Lối 2 cho `advance`: refcount ngay từ đầu

Cất **cả `buf` lẫn `cap`** trong một khối `Shared` trên heap, có refcount *từ lúc sinh*.
`ctx` *luôn* là `*mut Shared`. `self.ptr` là view (advance thoải mái), `Shared.buf` là
base, `Shared.cap` là kích thước. Mọi thao tác chạy qua `Shared`:

- `advance`: `self.ptr += n`. `slice`: clone (ref++) + thu hẹp. Cả hai đơn giản.
- `freeze`: *tái dùng* `Shared` sẵn có → **0 alloc** — nhưng chỉ nếu `Shared` *đã tồn
  tại trước freeze* → **`BytesMut` phải refcount ngay từ `new()`**.

**Tradeoff: được `advance` + zero-alloc-freeze, nhưng mất lazy-promote** — mọi buffer
trên heap trả tiền một `Shared` + atomic *từ lúc sinh*, kể cả khi không bao giờ clone.

## Yêu cầu B: lazy-promote như một ràng buộc cứng

**Là gì.** Một buffer một-chủ chưa từng clone thì **không** trả một atomic nào, **không**
cấp một `Shared` nào. **Khi nào quan trọng.** RLP decode đúc *hàng triệu* blob dùng một
lần; một atomic + một alloc *mỗi blob* là chi phí tránh-được lớn nhất trên hot path.
cap-in-ctx (Phần 7) và EVEN/ODD *có* lazy-promote. Refcount-từ-đầu *không*.

## Mọi cách mã hoá `ctx`, đặt cạnh nhau

| cách | `ctx` chưa-promote chứa | `buf` từ | `cap` từ | `advance` | zero-copy freeze | lazy-promote | độ phức tạp |
|---|---|---|---|---|---|---|---|
| **cap-in-ctx** (Phần 7) | `cap` | `self.ptr` | `ctx` | ❌ | ✅ | ✅ | 1 vtable |
| **buf-in-ctx EVEN/ODD** (`bytes`) | con trỏ buf (tagged) | `ctx` (mask) | số học (`cap==len`) | ✅ | ❌ (shrink) | ✅ | 2 vtable |
| **refcount-từ-đầu** | *luôn* `*mut Shared` | `Shared` | `Shared` | ✅ | ✅¹ | ❌ | 2 repr, đơn giản nhất về logic |

¹ zero-copy freeze cần `BytesMut` refcount-từ-đầu.

## Trilemma: vì sao "hỗ trợ tất cả" là bất khả

Nhìn ba cột `advance` / zero-copy-freeze / lazy-promote: **không hàng nào được cả ba.**
Đây không phải giới hạn cài đặt — nó là một định lý:

> Trong struct 4-từ, bạn chỉ được **2 trong 3** {lazy-promote, `advance`, zero-alloc-freeze
> với `cap>len`}.

Chứng minh cụ thể: `advance` dời view khỏi base → *phải* lưu `buf`. freeze-`cap>len` →
*phải* lưu `cap` thật. Đó là **hai giá trị độc lập**, mà ô `ctx` chỉ chứa *một*. Giữ cả
hai → cần khối `Shared` trên heap → để freeze *không* cấp phát, `Shared` phải tồn tại
*trước* freeze → `BytesMut` refcount-từ-đầu → **mất lazy-promote.**

Cả trilemma quy về **một câu hỏi**: *view có dời khỏi base của buffer khi **chưa** promote
không (tức có `advance` không)?*
- **Có** → phải lưu `buf` → con trỏ trong `ctx` → EVEN/ODD, và `cap` phải suy số học
  (mất zero-copy-từ-Vec) *hoặc* refcount (mất lazy-promote).
- **Không** → `ctx` rảnh → pack `cap` → một vtable, giữ cả lazy-promote lẫn zero-alloc-freeze.

## Kết luận: thiết kế "đúng" = yêu cầu của *bạn*

Không có bản tốt nhất tuyệt đối. Chọn điểm hợp với yêu cầu thật:

- **`Bytes` cho Ethereum/RLP** (bài này): slice + clone + freeze, *không* advance owned
  handle → **cap-in-ctx** (Phần 7). Giữ lazy-promote (hot path rẻ) + zero-alloc-freeze,
  đổi lấy `advance` mà kiểu này không dùng. Đây là lựa chọn đúng.
- **`bytes` như một `Buf`** (mạng): cần `advance` → **EVEN/ODD** (chịu shrink-từ-Vec) +
  `BytesMut` refcount-từ-đầu cho zero-copy freeze. *Đó* là vì sao `bytes` thật phức tạp
  — nó trả giá cho một feature set rộng hơn.
- **Tổng quát nhất / dễ suy luận nhất**: **refcount mọi thứ** (bỏ lazy-promote) — hai
  repr STATIC + SHARED, không tag, không promotion.

Bài học mang đi: "viết lại `bytes`" *không* phải chép nó dòng-đối-dòng. Nó là hiểu cả
**không gian thiết kế** và chọn đúng điểm cho yêu cầu của mình — rồi biện luận được vì
sao. `bytes` chọn EVEN/ODD vì nó là một `Buf`; ta chọn cap-in-ctx vì `Bytes` này slice
chứ không advance. Cả hai *đúng* — với bài toán của mình.

## Kiểm chứng, và hết loạt bài

Bug ở cả ba thiết kế — nhánh KIND đảo, `shared` vs `actual`, ordering sai, dealloc sai
`cap`/`buf` — *compile sạch* và *chạy có vẻ đúng* một thread. Bắt buộc: **`miri`**
(`cargo +nightly miri test`, thêm `-Zmiri-strict-provenance` cho cap-in-ctx), và **test
đua promotion** (N thread cùng `clone` một handle → chọc `Err(actual)`; `loom` để vét
cạn interleaving).

Từ "một byte đi vào từ dây mạng" (Phần 1) tới trilemma (bài này), mỗi mảnh bị *ép* bởi
mảnh trước, và mảnh cuối cho thấy: ngay cả "cách mã hoá một ô 8 byte" cũng không có đáp
án tuyệt đối — chỉ có những đánh đổi *có tên*, chọn theo yêu cầu. Giờ bạn không chỉ đọc
được `bytes`, mà *thiết kế lại* được nó ở bất kỳ điểm nào trên không gian đánh đổi, và
biện luận được cho lựa chọn của mình.

---

*Quay lại: [Phần 7](07_from_vec_and_bit_tagging.md) · [Mục lục](00_index.md)*

*English: [`../en/08_promotable_and_slice.md`](../en/08_promotable_and_slice.md)*
