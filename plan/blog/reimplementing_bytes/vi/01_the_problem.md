# Phần 1 — Một byte đi từ dây mạng vào chương trình

Hãy bắt đầu từ một chương trình rất đời thường: một node Ethereum đang chạy. Nó mở
một socket, và ở đầu bên kia, cả thế giới gửi dữ liệu tới — block mới, transaction,
message trao đổi giữa các peer. Công việc của node, ở tầng thấp nhất, là đọc mớ
byte đó lên, hiểu chúng, rồi hoặc lưu lại, hoặc chuyển tiếp cho peer khác.

Nghe đơn giản, nhưng ngay ở bước "đọc byte lên" đã có một vấn đề mà bài viết này
xoay quanh. Ta sẽ đi rất chậm, vì mọi thứ về sau đều mọc ra từ đây.

## Byte không đến gọn gàng

Khi bạn đọc từ một socket, dữ liệu không tới thành một khối đẹp đẽ có sẵn độ dài.
Nó tới từng mẩu, lúc 40 byte, lúc 1500 byte, tùy mạng. Một message hoàn chỉnh —
chẳng hạn một block header — có thể phải ghép từ năm, sáu lần đọc như vậy. Và bạn
**không biết trước** message dài bao nhiêu cho tới khi đọc gần hết.

Vậy bạn cần một chỗ để hứng: một vùng nhớ mà bạn cứ ghi thêm byte vào, và nó **tự
lớn lên** khi đầy. Trong thư viện chuẩn của Rust, thứ gần nhất là `Vec<u8>` — một
mảng byte co giãn được. Bạn `push` byte vào, khi hết chỗ nó tự xin thêm bộ nhớ.

Trong bài này, ta gọi cái thùng chứa ghi-được đó là **`BytesMut`** (`Mut` là
*mutable* — sửa được). Bạn có thể hình dung nó gần như `Vec<u8>`: một con trỏ tới
một vùng nhớ trên heap, cộng với hai con số — đã ghi bao nhiêu byte (`len`), và
vùng nhớ hiện cấp được bao nhiêu (`cap`, viết tắt của *capacity*).

```
BytesMut đang hứng một block header, ghi được 7 byte, vùng nhớ cấp 1024:

   con trỏ ──────────► [ 68 65 61 64 65 72 21 · · · · · · · · ]
                       └─ 7 byte đã ghi ─┘└─ chỗ trống còn lại ─┘
   len = 7
   cap = 1024
```

Điểm mấu chốt của `BytesMut`: nó **ghi được**, và nó **lớn dần**. Đó chính xác là
thứ bạn cần khi đang đọc dở một message từ socket.

## Nhưng đọc xong rồi thì `BytesMut` lại phiền

Giả sử message đã đọc xong. Giờ bạn muốn làm gì với nó?

- đưa cho bộ giải mã (decoder) để hiểu nội dung,
- cho vào cache để lát nữa tra lại,
- gửi lại cho một peer khác.

Và thường là **cả ba cùng lúc**: cùng một block header, decoder giữ một bản để
parse, cache giữ một bản, hàng đợi gửi giữ một bản.

Ở đây `BytesMut` trở nên phiền, vì hai lý do.

Thứ nhất, nó **ghi được**. Một khi message đã hoàn chỉnh, ta *không muốn* ai sửa nó
nữa. Nếu decoder và cache cùng cầm một `BytesMut` mà một trong hai lỡ ghi đè lên,
bên kia đọc phải dữ liệu hỏng. Dữ liệu bất biến (immutable) là điều kiện để chia sẻ
an toàn: nếu không ai sửa được, thì nhiều nơi cùng đọc không bao giờ đá nhau.

Thứ hai, cho nhiều nơi cùng giữ một `BytesMut` là **đắt**. `BytesMut` sở hữu vùng
nhớ của nó; muốn đưa cho ba nơi, cách an toàn duy nhất là copy ra ba bản. Mà copy
thì tốn — ta sẽ thấy tốn cỡ nào ở cuối bài.

Nói cách khác, `BytesMut` giỏi việc *xây* một message, nhưng dở việc *chia sẻ* một
message đã xây xong.

## `Bytes`: cái handle chỉ-đọc, chia sẻ được

Nên ta cần một kiểu thứ hai, cho giai đoạn sau: khi message đã xong và chỉ cần được
đọc, chuyền tay khắp nơi. Ta gọi nó là **`Bytes`** (không có `Mut` — không sửa
được).

`Bytes` là một *handle chỉ-đọc* trỏ tới một mớ byte. Nó có ba tính chất ta cần:

- **bất biến** — không ai ghi qua `Bytes` được, nên chia sẻ vô tư;
- **clone rẻ** — cho thêm một nơi cùng giữ thì gần như miễn phí, không copy nội
  dung;
- **tự dọn** — khi nơi cuối cùng giữ nó hủy đi, vùng nhớ tự được giải phóng.

Và thao tác biến một `BytesMut` (đã xây xong) thành một `Bytes` (sẵn sàng chia sẻ)
có một cái tên: **`freeze`** — "đóng băng". Bạn ngừng ghi, đóng băng cái buffer
lại, và từ đó nó chỉ còn được đọc.

```
   BytesMut  ──freeze──►  Bytes
   (ghi được,             (chỉ đọc,
    đang xây)              chia sẻ được)
```

Toàn bộ vòng đời của một message inbound, vẽ lại, là:

```
socket ──► BytesMut ──► freeze ──► Bytes ──► decoder / cache / gửi đi
          (hứng &         (đóng      (chuyền tay
           lớn dần)        băng)      khắp nơi)
```

Giờ ta đã có đủ từ vựng để đặt câu hỏi trung tâm.

## `freeze` làm gì bên trong, và tại sao nó có thể rất chậm

`freeze` nghe như một thao tác vô thưởng vô phạt — chỉ là "đổi nhãn" một cái buffer
từ ghi-được sang chỉ-đọc. Nhưng nó là chỗ mà toàn bộ hiệu năng của đường nhận dữ
liệu được quyết định, vì một lý do đơn giản: **mọi message inbound đều đi qua nó.**
Một node bận rộn `freeze` hàng trăm nghìn, hàng triệu lần mỗi giây.

Câu hỏi là: khi `freeze` chạy, nó có phải *copy* nội dung buffer sang một chỗ mới
không?

Cách làm ngây thơ nhất thì **có**. Nó cấp một vùng nhớ mới, copy toàn bộ byte từ
`BytesMut` sang, rồi trả về một `Bytes` trỏ vào vùng mới. Ta hãy tính xem cái copy
đó tốn bao nhiêu.

Giả sử một burst block header cỡ 1 MiB. Tốc độ copy bộ nhớ (memcpy) trên phần cứng
hiện đại khoảng 5 GiB/s. Vậy copy 1 MiB mất:

```
1 MiB ÷ 5 GiB/s ≈ 200 micro-giây
```

200 micro-giây nghe nhỏ, nhưng đó là **200 micro-giây CPU đứng yên**, không làm gì
ngoài chuyển byte từ chỗ này sang chỗ khác — cho *một* lần `freeze`. Nhân với số
message mỗi giây, và cái node của bạn đang đốt một phần đáng kể thời gian chỉ để
*chép lại* dữ liệu mà nó vừa mới đọc lên.

Với một hệ thống mà thông lượng là tất cả, một cú copy trên mỗi `freeze` không phải
"chưa tối ưu" — nó là thứ loại thẳng thiết kế khỏi cuộc chơi. Nên ta đặt ra một yêu
cầu cứng, và cả loạt bài này là câu chuyện về cái giá phải trả để giữ đúng nó:

> `freeze` phải chạy trong thời gian **hằng số**, bất kể buffer to hay nhỏ. Vùng
> nhớ chứa payload **không được di chuyển**; cái duy nhất được phép chuyển là *quyền
> sở hữu* nó — từ `BytesMut` sang `Bytes`.

"Không di chuyển vùng nhớ, chỉ chuyển quyền sở hữu" là ý tưởng ta sẽ đào suốt phần
còn lại. Nhưng trước khi xây, đáng để hỏi: tại sao cách làm hiển nhiên nhất lại
*không* đạt được điều đó? Vì trả lời câu này sẽ cho thấy vấn đề thật nằm ở đâu.

## Thử #1: `Bytes` bọc quanh `Vec<u8>`

Ý tưởng đầu tiên: `freeze` cứ trả thẳng cái buffer ra dưới dạng `Vec<u8>`.

Cái này *có* tránh được copy — `Vec<u8>` và `BytesMut` đều là một khối nhớ phẳng
trên heap, nên `Vec` có thể "nhận" luôn vùng nhớ của `BytesMut` mà không chép lại.
Vậy về mặt copy thì đạt. Nhưng nó hỏng ở hai chỗ khác, và chính hai chỗ đó định
hình mọi thứ về sau.

Thứ nhất, `Vec<u8>` **ghi được**. Ta lại quay về đúng vấn đề của `BytesMut`: không
có gì đảm bảo nội dung bất biến, nên không chia sẻ an toàn được.

Thứ hai, `Vec<u8>` **không clone rẻ**. `Vec::clone()` copy toàn bộ nội dung. Mà "cho
nhiều nơi cùng giữ" là chuyện xảy ra liên tục. Nếu mỗi lần "cùng giữ" lại là một
cú memcpy, thì ta chỉ *dời* cái copy từ `freeze` sang `clone` — chứ không giết được
nó.

Bài học từ thử #1: ta cần một kiểu vừa **bất biến**, vừa **clone rẻ**, vừa **nhận
được vùng nhớ có sẵn**. Ba tính chất này là thước đo để chấm mọi phương án tiếp
theo.

## Thử #2: `Bytes` bọc quanh `Arc<[u8]>`

Đây là phản xạ thứ hai của gần như mọi lập trình viên Rust, và cũng là nơi rất
nhiều thiết kế ngoài đời thật khởi đầu.

`Arc` (viết tắt của *Atomically Reference-Counted*) là công cụ chuẩn của Rust cho
"nhiều nơi cùng sở hữu một dữ liệu". Bên trong nó giữ một **counter** đếm xem hiện
có bao nhiêu nơi đang cầm. Mỗi lần `clone`, counter tăng một; mỗi lần một bản bị
hủy, counter giảm một; khi counter về 0, dữ liệu tự được giải phóng. `Arc<[u8]>`
nghĩa là "một mảng byte, được đếm-tham-chiếu".

```rust
struct Bytes(Arc<[u8]>);
```

Cái này cho ta ngay **hai trong ba** tính chất cần thiết:

- **bất biến** — `Arc<[u8]>` chỉ cho mượn đọc, không ai ghi được;
- **clone rẻ** — `clone` chỉ tăng counter lên một, không chép nội dung. Đây đúng là
  "nhiều nơi cùng giữ một message" mà ta cần.

Và nó **chạy được**. Rất nhiều codebase khởi đầu đúng bằng thiết kế này. Vấn đề chỉ lộ ra ở
tính chất thứ ba — `freeze` không copy — và để thấy tại sao nó hỏng, phải nhìn vào
*cách bộ nhớ được sắp xếp*.

### Tại sao `Arc<[u8]>` buộc phải copy khi `freeze`

`Arc<[u8]>` là **một** khối nhớ duy nhất, trong đó cái counter nằm **dính liền**
ngay trước dãy byte:

```
Arc<[u8]>:   [ counter | b0 b1 b2 ... bN ]
             └ phần đầu ┘└──── payload ────┘
                một khối nhớ duy nhất
```

Còn buffer của `BytesMut` thì chỉ có payload, **không có** phần counter ở đầu:

```
BytesMut:    [ b0 b1 b2 ... bN ]
             └──── payload ────┘
```

Hai cách sắp xếp này khác hình dạng, và không đời nào khớp được. Muốn biến buffer
của `BytesMut` thành một `Arc<[u8]>`, bạn phải nhét cái counter vào *ngay trước* con
trỏ hiện tại. Nhưng khoảng nhớ ngay trước đó **không thuộc về bạn** — bộ cấp phát
(allocator) chưa từng giao nó cho bạn, ghi vào đó là giẫm lên bộ nhớ của phần khác
trong chương trình. Không có cách nào "chèn thêm" một phần đầu vào trước một khối đã
cấp phát rồi.

Nên khi bạn đưa buffer cho `Arc`, nó **buộc phải**:

1. xin allocator một khối *mới*, đủ chỗ cho `counter + N byte`;
2. copy N byte payload từ buffer cũ sang khối mới;
3. giải phóng buffer cũ.

Bước 2 chính là cú memcpy mà ta thề sẽ giết. Và đây là điểm quan trọng: nó **không
phải lỗi code**, mà là hệ quả tất yếu của *hình dạng* `Arc<[u8]>`. `Arc<[u8]>` chỉ
biết một kiểu sở hữu duy nhất — đếm tham chiếu — và kiểu đó đòi counter phải sống
*bên trong* cùng khối với payload. Một kiểu mà counter dính liền payload thì **không
thể nhận** một payload đã nằm sẵn ở nơi khác.

Chốt lại bằng một câu, đây là bản lề của cả loạt bài:

> `Arc<[u8]>` không thể *nhận* một vùng nhớ có sẵn. Cách duy nhất đưa byte vào một
> `Arc<[u8]>` là cấp khối mới rồi chép vào. Mà thứ ta cần lại là điều ngược lại: một
> handle trỏ thẳng vào vùng nhớ *của kẻ khác* (ở đây là buffer của `BytesMut`), rồi
> nhận lấy trách nhiệm dọn dẹp nó.

## Điều gì lộ ra khi handle trỏ vào "vùng nhớ của kẻ khác"

Ngay khi ta chấp nhận ý tưởng "một handle trỏ vào buffer có sẵn", một câu hỏi mới
xuất hiện — câu hỏi mà `Arc<[u8]>` chưa bao giờ phải trả lời.

`Arc<[u8]>` luôn biết chính xác phải làm gì khi một bản bị hủy: giảm counter, về 0
thì giải phóng. Luôn luôn, không ngoại lệ — vì nó chỉ có *một* kiểu sở hữu. Nhưng
một handle trỏ-tự-do có thể đang trỏ vào ba loại vùng nhớ với ba số phận trái ngược
nhau:

- Nó trỏ vào một **hằng số nằm sẵn trong file chạy** (ví dụ một chuỗi byte hard-code
  trong chương trình). Vùng này chưa từng được allocator cấp; khi hủy, ta **không
  được** làm gì cả — giải phóng nó là giải phóng bộ nhớ mà mình không sở hữu.
- Nó trỏ vào một **buffer vừa cướp từ `BytesMut`**. Vùng này *có* được cấp phát, và
  đúng một handle sở hữu nó; khi hủy, ta phải giải phóng.
- Nó trỏ vào một **vùng đang được nhiều nơi chia sẻ**. Lúc này cần một counter; ai
  kẻ nào bị hủy sau cùng thì mới giải phóng.

Đây là mâu thuẫn trung tâm của toàn bộ vấn đề, phát biểu lần đầu:

> Cùng *một* kiểu `Bytes`, nhưng *ba* cách dọn dẹp khác nhau, và cách nào áp dụng
> thì chỉ biết được lúc chương trình chạy, tùy từng giá trị cụ thể.

`Arc<[u8]>` né được mâu thuẫn này bằng cách chỉ hỗ trợ *một* cách dọn dẹp (và trả
giá bằng cú memcpy ở `freeze`). Ta thì không được né: ta cần cả ba cách *trong cùng
một kiểu* — để vừa có `freeze` không-copy (cách "sở hữu độc quyền"), vừa có `clone`
rẻ (cách "chia sẻ"), vừa để không tốn gì cho các hằng số byte cố định (cách "hằng
tĩnh", dùng cho những chuỗi byte hard-code như hằng số genesis hay bytecode của các
precompile).

## Vì sao không thể chỉ dùng ba kiểu riêng

Đến đây có thể bạn nghĩ: "Vậy làm ba kiểu riêng đi — `StaticBytes`, `OwnedBytes`,
`SharedBytes`, mỗi kiểu một cách dọn, để compiler tự lo."

Về mặt sở hữu, đây thực ra là cách *đúng đắn* — và ở Phần 2 ta sẽ thấy Rust bình
thường *muốn* bạn làm đúng như vậy. Nó chết vì một lý do khác hẳn: **biên giới API**.

Hãy nhìn một hàm điển hình tiêu thụ byte:

```rust
fn decode(data: Bytes) -> Header;
```

Hàm `decode` này — và hàng trăm hàm như nó — phải nuốt được byte *bất kể chúng từ
nguồn nào*. Nếu có ba kiểu riêng, thì:

- bạn phải viết `decode` ba lần (hoặc generic hóa mọi hàm tiêu thụ byte — bùng nổ số
  lượng);
- một `Vec<Bytes>` không thể chứa lẫn lộn ba kiểu;
- một channel gửi `Bytes` giữa các thread không thể gửi lẫn ba kiểu;
- một struct có trường `Bytes` phải chọn cứng *một* trong ba, mất hết linh hoạt.

Toàn bộ hạ tầng phía dưới `Bytes` ngầm giả định chỉ có **một** kiểu. Ràng buộc "một
kiểu" không phải do ta tự đặt cho vui — nó đến từ chính những đoạn code *dùng*
`Bytes`.

Và đây là cái bẫy: một kiểu duy nhất nghĩa là một hàm dọn dẹp (`Drop`) duy nhất,
tức một hành vi *cố định*. Mà ta vừa chứng minh rằng cần *ba* hành vi, chọn tùy từng
giá trị lúc chạy. Mâu thuẫn giữa "một kiểu, do API bắt buộc" và "ba cách dọn, do
freeze-không-copy cộng clone-rẻ cộng hằng-số-miễn-phí bắt buộc" — chính là bài toán
thật.

## Vậy Phần 2 sẽ giải gì

Gom lại, ta có ba phương án và ba lý do chúng chết:

- `Vec<u8>`: đạt được freeze-không-copy, nhưng không bất biến và clone thì copy.
- `Arc<[u8]>`: bất biến và clone rẻ, nhưng freeze buộc phải memcpy vì counter dính
  liền payload.
- ba kiểu riêng: đạt cả ba cách dọn, nhưng biên giới API đòi một kiểu duy nhất.

Cô lại thành một câu hỏi, đây là thứ Phần 2 phải trả lời:

> Làm sao để một kiểu `Bytes` duy nhất mang được ba cách dọn dẹp khác nhau, chọn
> đúng cách tùy từng giá trị lúc chạy — mà việc đọc byte vẫn rẻ y như `Arc<[u8]>`?

Vế "một kiểu, nhiều hành vi, chọn lúc chạy" nghe quen: đó chính là bài toán mà cơ
chế *dynamic dispatch* sinh ra để giải. Phần 2 sẽ cho thấy dùng nó thế nào cho
đúng, và một câu hỏi phụ nhưng quan trọng không kém — vì sao cái bảng điều phối
(vtable) mà ta sắp dựng lại có đúng *hai* ô, không hơn không kém.

---

*Tiếp theo: [Phần 2 — Một kiểu, nhiều hành vi](02_vtable.md) · [Mục lục](00_index.md)*

*English: [`../en/01_the_problem.md`](../en/01_the_problem.md)*
