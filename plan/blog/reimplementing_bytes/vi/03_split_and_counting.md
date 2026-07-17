# Phần 3 — Tách "byte nào" khỏi "ai sở hữu"

Phần 2 để lại một món nợ. Ta đã có cơ chế cho một kiểu `Bytes` mang ba cách dọn khác
nhau, nhưng chưa trả lời được nửa sau của yêu cầu: đọc byte phải vẫn *nhanh*. Một
`Bytes` bây giờ có tới bốn trường — con trỏ, độ dài, `data`, `vtable` — nhiều hơn
hẳn `Arc<[u8]>` vốn chỉ có một con trỏ. Liệu cái sự "nhiều hơn" đó có làm việc đọc
chậm đi?

Câu trả lời là không, và lý do vì sao lại không chính là ý tưởng đẹp nhất của cả
thiết kế. Nó cũng là điều khiến thiết kế này *thắng* một cách làm hiển nhiên hơn mà
ta sẽ so sánh cuối mục.

## Một handle trả lời hai câu hỏi chẳng liên quan gì nhau

Hãy nhìn lại một `Bytes` và để ý rằng bốn trường của nó thật ra tách thành hai nhóm,
trả lời hai câu hỏi hoàn toàn tách biệt.

Câu hỏi thứ nhất: *byte nào?* — chúng nằm ở đâu, dài bao nhiêu. Trả lời bằng con trỏ
và độ dài.

Câu hỏi thứ hai: *ai sở hữu?* — vùng byte này là hằng số, là độc quyền, hay đang
chia sẻ; và chính điều đó quyết định khi clone hay khi hủy thì phải xử lý ra sao.
Trả lời bằng `data` và `vtable`. (Lưu ý: `clone` không "dọn" gì cả — nó nhân bản một
handle; chỉ `drop` mới giải phóng. Cả hai cùng nằm ở nhóm này vì cả hai đều *phụ
thuộc vào ai sở hữu*, chứ không phải vì cả hai đều dọn dẹp.)

```rust
struct Bytes {
    ptr:    /* con trỏ */,     // ┐ "byte nào"
    len:    /* độ dài */,      // ┘
    data:   /* 8 byte phụ */,  // ┐ "ai sở hữu"
    vtable: /* con trỏ bảng */,// ┘
}
```

Điều mấu chốt: nội dung byte *luôn luôn* chỉ là một dãy byte thô, bất kể nó tới từ
hằng số, từ vùng độc quyền, hay từ vùng chia sẻ. Không có "hình dạng ẩn" nào phải
khám phá sau. Nên con trỏ với độ dài đã trả lời *trọn vẹn* câu hỏi thứ nhất, và câu
hỏi thứ nhất *không bao giờ cần biết* đáp án của câu thứ hai.

Chiều ngược lại thì không đối xứng: `clone` và `drop` cần `data`/`vtable`, nhưng
chúng *cũng* cần con trỏ và độ dài (để dựng handle mới, để giải phóng đúng địa chỉ).
Nên nhóm "ai sở hữu" đọc cả hai nhóm; nhưng nhóm "byte nào" chỉ đọc mỗi nhóm của nó.
Chính sự lệch đó là chỗ để khai thác.

Một cách hình dung: con trỏ và độ dài giống như *vị trí trên kệ và số trang* của một
cuốn sách. Còn `data` với `vtable` giống như *tấm phiếu mượn* dán sau bìa: ai đang
mượn bản này, trả thì xử lý ra sao. Tấm phiếu mượn không cho bạn biết trong sách
viết gì — và bạn đọc trọn cuốn sách mà chẳng cần liếc tấm phiếu lấy một lần.

## Nhờ vậy, việc đọc miễn phí

Hệ quả trực tiếp: mọi thao tác đọc — lấy nội dung, lấy độ dài, so sánh, băm (hash),
in ra — chỉ đụng tới con trỏ và độ dài. Không tra `vtable`. Không rẽ nhánh theo loại
sở hữu. Không đụng counter. Lấy nội dung ra chỉ là "từ con trỏ này, đọc ngần này
byte" — đúng một dòng, và là *chính xác* cái mà `Arc<[u8]>` cũng biên dịch ra.

Đây là toàn bộ lý do việc đọc vẫn rẻ. Hai trường vừa thêm (`data`, `vtable`) không
tốn gì trên đường-nóng, đơn giản vì đường-nóng chỉ đọc, mà đọc thì hai trường đó vô
hình. Cái giá của sự linh hoạt — một kiểu gánh ba cách dọn — được dồn *hết* vào
`clone` và `drop`, hai thao tác *lạnh*, ít khi chạy; và *không rò rỉ* sang việc đọc,
thao tác *nóng*, chạy suốt.

Đây là một nguyên tắc thiết kế dùng được ở khắp nơi, không riêng `Bytes`: khi bạn
thêm trạng thái để có thêm linh hoạt, hãy xếp bố cục sao cho trạng thái đó *nằm
ngoài đường-nóng*. Nếu đường-nóng buộc phải *nhìn vào* trạng thái mới — dù chỉ một cú
rẽ nhánh — thì sự linh hoạt đã rò chi phí vào đúng chỗ đắt nhất.

## Vì sao không dùng một cái `enum` cho xong

Đến đây hẳn nhiều người sẽ hỏi: sao phải cầu kỳ vtable với con trỏ hàm, trong khi
Rust có sẵn `enum` để biểu diễn "một trong ba khả năng"?

```rust
enum Bytes {
    Static { /* ... */ },
    Owned  { /* ... */ },
    Shared { /* ... */ },
}
```

Cách này *đúng*. Nó còn *an toàn hơn* (không phải viết code `unsafe`). Vậy vì sao
thiết kế thật không chọn nó?

Vì một `enum` nhét cái nhãn-phân-loại (câu hỏi "ai sở hữu") *chung* với dữ liệu (câu
hỏi "byte nào"). Mỗi lần đọc, bạn phải `match` cái nhãn đó — một cú rẽ nhánh — để
lôi con trỏ với độ dài ra, *dù việc đọc byte chẳng liên quan gì tới nhãn cả*. Bạn
trả giá cho câu hỏi "ai sở hữu" ở *mọi* lần hỏi câu "byte nào".

Với cách bố trí phẳng của ta — con trỏ và độ dài luôn nằm ở cùng một vị trí cố định
cho cả ba loại — việc đọc lấy thẳng ra, không rẽ nhánh. Con trỏ `vtable` thì nằm
tách ra một bên, chỉ được đụng tới bởi `clone` và `drop`.

Đánh đổi ở đây rất thật, và đáng nói thẳng: chọn cách phẳng nghĩa là bạn mất đi sự
an toàn tĩnh của `enum` (phải viết code `unsafe` và tự tay giữ đúng cái bất biến
"`data` phải khớp với `vtable`"), để đổi lấy việc đọc không rẽ nhánh. Với một kiểu
mà thao tác đọc bị gọi liên tục trong vòng lặp nóng, đổi vậy là đáng. Với một kiểu
ít khi đọc, thì `enum` mới là lựa chọn đúng. Biết mình đang ở đâu trên cái phổ đó
chính là một phần của kỹ năng thiết kế — không phải lúc nào "nhanh hơn" cũng thắng.

## Cách nghĩ xương sống: đếm số lần giải phóng

Giờ chuyển sang nhóm "ai sở hữu", và đặt nền cho hai phần khó nhất. Ba cách sở hữu nghe
thì khác nhau, nhưng thật ra chúng chỉ là ba câu trả lời cho *cùng một* câu hỏi:

> Vùng nhớ này bị giải phóng (`dealloc`) đúng bao nhiêu lần, và do ai làm?

- Hằng số: giải phóng **0** lần. Nó chưa từng được cấp phát; chưa xin thì trả cho
  ai.
- Vùng độc quyền: giải phóng **1** lần, do chính handle làm, khi nó bị hủy.
- Vùng chia sẻ: giải phóng **1** lần, do handle *cuối cùng* làm, khi counter về 0.

Con số đúng luôn là như vậy. Và đây là điều biến câu hỏi này thành một công cụ chứ
không phải khẩu hiệu: *mọi bug trong thiết kế này đều quy về việc đếm sai con số đó.*
Đếm ra **0** khi đáng lẽ phải 1 là rò rỉ bộ nhớ. Đếm ra **2** khi đáng lẽ 1 là
double-free hoặc use-after-free. Suốt Phần 4 và 5, mỗi khi phân vân "chỗ này đúng
không", bạn chỉ cần hỏi: *với đúng vùng nhớ này, tôi vừa làm con số đếm thành mấy?*

Từ cách nghĩ này, hai điều quan trọng lộ ra.

## `Drop` không dọn cái struct — nó hoàn tác một lần cấp phát

Nhìn lại bốn trường của `Bytes`: một con trỏ, một con số, `data` (con trỏ hoặc số),
`vtable` (một tham chiếu tĩnh). *Không trường nào sở hữu gì cả.* Nếu bạn xóa hẳn khối
`impl Drop for Bytes`, việc hủy một `Bytes` đã là một thao tác rỗng hoàn hảo — cái
struct tự biến mất khỏi stack, không cần ai giúp.

Vậy `Drop` tồn tại để làm gì? *Chỉ* để trả lại cái vùng nhớ trên heap mà struct trỏ
tới. Đây là chỗ mà trực giác hay sai: `Drop` không phải để "dọn dẹp bản thân giá
trị" — bản thân giá trị tự tan. `Drop` chỉ tồn tại để *hoàn tác một lần cấp phát
trước đó*. Nếu chưa từng cấp phát, thì chẳng có gì để hoàn tác.

Đây chính là lý do hàm `drop` của hằng số là một hàm *rỗng*, và cái rỗng đó *đúng*.
Một hằng số byte chưa từng được cấp phát, nên nó phải được giải phóng 0 lần, nên
hàm dọn của nó không làm gì. Nói cách khác: không cấp phát thì không `Drop`. Rò rỉ
một hằng số là *đúng* — nó sống suốt đời chương trình dù bạn có làm gì đi nữa.

## Cái bẫy im lặng: bài học lớn nhất về code unsafe

Hàm giải phóng của vùng độc quyền phải trả lại đúng số byte đã *cấp*, chứ không phải
số byte đã *ghi*. Nhớ lại từ Phần 1: một `BytesMut` có thể đã cấp 1024 byte nhưng
mới ghi 7. Khi giải phóng, bộ cấp phát đòi lại đúng cái khối 1024 byte nó đã giao —
nó khớp theo *kích thước đã cấp*, không phải theo nội dung.

Chuyện gì xảy ra nếu bạn lỡ trả lại theo số byte đã ghi (7) thay vì đã cấp (1024)?

Trên Linux và macOS, lệnh giải phóng cuối cùng gọi xuống `free(ptr)` của C — mà
`free` chỉ nhận *một* tham số là con trỏ; nó tự tra kích thước từ metadata giấu ngay
trước khối, và *vứt đi* cái kích thước bạn truyền vào. Hậu quả: chương trình **không
sập**. Test qua hết. Chạy mười triệu lần vẫn qua. Chạy trên production hai năm vẫn
qua.

Nhưng nó là undefined behavior. Cái hợp đồng của lệnh giải phóng đòi kích-thước-lúc
-trả phải bằng kích-thước-lúc-cấp. Ngày nó nổ là ngày ai đó thay bộ cấp phát mặc
định bằng một cái khác — chẳng hạn `jemalloc` hay `mimalloc` — loại *tin* vào kích
thước bạn truyền và dùng nó để chọn ngăn chứa. Nó trả cái khối 1024 byte vào ngăn
dành cho khối 8 byte; vài nghìn lần cấp phát sau, hai chỗ trong chương trình cùng
ghi lên một vùng nhớ; và bạn có heap corruption ở một chỗ hoàn toàn không liên quan,
không cách nào lần ra.

Đây là bài học đáng khắc cốt nhất về code unsafe, và nó ngược hẳn trực giác:

> Con bug đáng sợ trong Rust unsafe không phải con làm chương trình sập, mà là con
> chạy *đúng*. Trực giác từ Rust an toàn — "sai thì panic ngay" — bị đảo ngược ở
> đây: mặc định của cái sai là *im lặng*.

Công cụ bắt được nó là `miri` — một trình thông dịch không chạy `free` thật mà *kiểm
tra hợp đồng*: nó ghi nhớ kích thước lúc cấp, so lại lúc trả, và la lên "kích thước
giải phóng không khớp" ngay lập tức, đúng dòng. Đây cũng là lý do mọi trường kiêm-
nhiệm — như `data` khi thì là số, khi thì là con trỏ — phải được chú thích ngay tại
chỗ và kiểm tra bằng test: cái sai không tự lộ ra lúc chạy.

## Ba hành vi "dễ" tự rơi ra

Cầm cách nghĩ đếm-số-lần-giải-phóng, ba trong bốn hành vi tự lộ ra — và điểm hay là
*chưa cần một dòng nào dính tới đa luồng*. Đây là lý do nên xây chúng trước; Phần 4
và 5 mới là phần khó.

Với **hằng số**, giải phóng 0 lần. Tạo ra một `Bytes` hằng số chỉ là dựng một handle
trỏ vào chuỗi byte có sẵn, gắn `vtable` là bảng hằng số, `data` để trống. Hàm dọn
rỗng. Xong.

Với **vùng độc quyền**, đây là chỗ ta cuối cùng giết được cú memcpy của Phần 1. Ta
muốn vùng nhớ được giải phóng 1 lần, không phải 2. Vấn đề: cái `BytesMut` cũ *sẽ*
tự giải phóng vùng nhớ khi nó bị hủy (nó có sẵn hành vi dọn của mình); rồi cái
`Bytes` mới *cũng sẽ* giải phóng. Vậy là 2 — double-free. Để về 1, ta phải chặn
không cho `BytesMut` chạy hành vi dọn của nó.

Rust có sẵn công cụ cho việc này: `mem::forget`. Nghe tên thì tưởng nó "xóa biến",
nhưng thật ra nó là một lời tuyên bố:

> "Tôi đã trao vùng nhớ này cho người khác rồi. Đừng chạy hàm dọn của tôi nữa."

Đó chính là *định nghĩa* của một cú bàn giao zero-copy: bên nhận lấy luôn vùng nhớ
của bên trao (không copy), và một vùng nhớ thì chỉ được một người dọn. Cái buffer
không hề nhúc nhích; chỉ có *trách nhiệm dọn* nó chuyển từ `BytesMut` sang `Bytes`.
Quy trình `freeze` do đó là: đọc con trỏ / độ dài / kích-thước-đã-cấp ra khỏi
`BytesMut`; gọi `mem::forget` để `BytesMut` không tự dọn nữa; rồi dựng một `Bytes`
độc quyền trỏ vào đúng cái buffer đó, với `data` là kích-thước-đã-cấp.

Một điểm tinh tế: `mem::forget` *bình thường là rò rỉ bộ nhớ* — đó mới là công dụng
chính (và nguy hiểm) của nó. Ở đây nó *không* gây rò rỉ chỉ vì ta đã kịp đọc con trỏ
ra và trao cho `Bytes` *trước*. `mem::forget` không tự kiểm tra điều đó; *bạn* phải
đảm bảo có người nhận việc. Nên thứ tự đọc-ra-trước-rồi-mới-forget là bắt buộc; đảo
lại thì compiler chặn ngay (vì bạn đã trao `self` vào `forget` mất rồi). Chỗ này khó
viết sai — đúng kiểu code mà ta thích.

Và hàm giải phóng của vùng độc quyền, như đã bàn ở trên: trả lại vùng nhớ theo đúng
kích-thước-đã-cấp lấy từ `data`. Nhớ cái bẫy im lặng — đã cấp, không phải đã ghi.

## Đã có gì, và bức tường đang chờ

Sau Phần 3, ta có một `Bytes` chạy được cho *hai trong ba* loại, đường đọc miễn phí,
và chưa đụng gì tới đa luồng. Hằng số thì clone là copy struct, dọn là hàm rỗng.
Vùng độc quyền thì `freeze` là cú bàn giao thời-gian-hằng-số — cú memcpy của Phần 1
đã chết, tức yêu cầu cứng ta đặt ra ở Phần 1 đã đạt — và dọn là giải phóng theo
kích-thước-đã-cấp. Cách nghĩ đếm-số-lần-giải-phóng thì sẵn sàng làm công cụ chẩn
đoán.

Còn đúng một hành vi: `clone` một vùng độc quyền. Và nó phá vỡ mọi thứ. Cầm cách
nghĩ đếm mà thử: clone một vùng độc quyền, rồi để *cả hai* handle cùng là độc quyền,
thì cả hai cùng giải phóng — đếm ra 2 — double-free.

Vì sao không tránh được, và vì sao lối thoát của nó lại buộc ta làm một điều rất bất
thường trong Rust — sửa ngược vào một giá trị đang tồn tại — là nội dung Phần 4, và
là phần khó nhất của cả loạt bài.

---

*Tiếp theo: [Phần 4 — Bức tường: khi clone làm hỏng mọi thứ](04_promotion.md) ·
[Mục lục](00_index.md)*

*English: [`../en/03_split_and_counting.md`](../en/03_split_and_counting.md)*
