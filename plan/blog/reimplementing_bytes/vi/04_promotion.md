# Phần 4 — Bức tường: khi clone làm hỏng mọi thứ

Ba trong bốn hành vi đã xong, và chúng dễ đến bất ngờ. Còn đúng một cái: clone một
`Bytes` đang sở hữu vùng nhớ độc quyền. Nghe cũng chẳng có gì ghê gớm. Nhưng đây là
chỗ cả thiết kế đâm vào tường, và cái tường này khó không phải vì cú pháp — nó khó
vì nó đi ngược một giả định mà ta mặc nhiên tin suốt khi viết Rust.

Ta sẽ đi rất chậm ở phần này, vì nếu hiểu được nó, mọi thứ ở Phần 5 (atomic, CAS,
memory ordering) sẽ thành hệ quả tự nhiên chứ không còn là một mớ khái niệm rời rạc.

## "Độc quyền" là một lời hứa, không phải một mô tả

Cầm cách nghĩ đếm-số-lần-giải-phóng của Phần 3, và thử cách clone ngây thơ nhất:
clone một vùng độc quyền thì trả về một vùng độc quyền nữa, y như cách ta copy một
hằng số.

Giả sử `b1` là một `Bytes` độc quyền, trỏ vào vùng nhớ ở địa chỉ `0xAAAA`. Ta clone
nó ra `b2`. Cách ngây thơ cho `b2` cũng là độc quyền, cũng trỏ vào `0xAAAA` — *cùng
một vùng nhớ*. Giờ đếm:

```
b2 bị hủy → nó là độc quyền → giải phóng 0xAAAA   ← lần 1
b1 bị hủy → nó là độc quyền → giải phóng 0xAAAA   ← lần 2 💥
```

Đếm ra 2. Double-free. Vì sao không "khéo" hơn để tránh?

Vì chữ "độc quyền" mang nghĩa "tôi là kẻ sở hữu *duy nhất*". Đây không phải một cái
nhãn thụ động mô tả trạng thái — nó là một *lời hứa* mà hàm giải phóng *tin tưởng* để
dám giải phóng. Hàm giải phóng của vùng độc quyền không đi kiểm tra "còn ai khác giữ
không"; nó *mặc định* mình là kẻ duy nhất, vì đó là điều khoản của cái nhãn độc
quyền.

So với hằng số: ta copy một hằng số thoải mái được, vì "không ai sở hữu" nhân đôi
lên vẫn là "không ai sở hữu". Nhưng copy một *kẻ-sở-hữu-duy-nhất* thì ra *hai kẻ*, và
cả hai vẫn đeo cái nhãn "duy nhất", cả hai vẫn sẽ giải phóng.

Đây là chỗ mấu chốt, và cũng là chỗ nhiều người trượt: hành động clone *làm cho lời
hứa "tôi là kẻ duy nhất" của `b1` trở thành sai* — dù ta chẳng hề đụng vào `b1`. Chỉ
riêng việc `b2` ra đời đã biến `b1` thành kẻ nói dối. Người ta thường nghĩ nhiệm vụ
duy nhất của `clone` là "tạo một bản sao". Nhưng ở đây `clone` còn một nhiệm vụ thứ
hai, ẩn đi: *sửa lại trạng thái của cái gốc, để nó thôi tự mâu thuẫn.* Ai gây ra
chuyện thì người đó phải dọn.

## Lối thoát duy nhất: thăng cấp lên chia sẻ

Không có cách nào viết `clone` mà trả về một vùng độc quyền cho an toàn. Muốn thoát,
`b1` phải *thôi làm kẻ sở hữu duy nhất*. Cụ thể, ta chuyển cả hai handle sang cách
sở hữu thứ ba — chia sẻ, có counter. Quá trình này gọi là *promotion*, thăng cấp từ
độc quyền lên chia sẻ.

Ta cấp một khối nhớ nhỏ mới để chứa counter — tạm gọi nó là khối `Shared` — và bọc
nó trong một `Arc` để `Arc` lo phần đếm nguyên tử và giải-phóng-khi-về-0. Điểm quan
trọng: khối `Shared` này chỉ *trỏ tới* payload; bản thân payload *không hề nhúc
nhích*. Nên đây vẫn là zero-copy — ta không copy lại dãy byte, chỉ cấp thêm một chỗ
nhỏ để đếm.

Rồi ta chuyển *cả* `b1` *lẫn* `b2` sang chia sẻ, cả hai cùng trỏ vào khối `Shared`
đó, counter đặt bằng 2.

```
        payload (không nhúc nhích)
           ▲              ▲
           │              │
     b1: chia sẻ     b2: chia sẻ
           │              │
           └──► Shared ◄──┘      counter = 2
                (Arc)
```

Giờ đếm lại: mỗi handle khi bị hủy sẽ giảm counter một. Vùng nhớ chỉ được giải
phóng đúng một lần, khi counter về 0. Con số về đúng 1.

Chú ý một chi tiết sẽ quan trọng về sau: khối `Shared` phải nhớ *địa chỉ gốc* của
vùng nhớ đã cấp, chứ không nhất thiết là con trỏ mà handle đang cầm — vì thao tác
cắt đoạn con có thể đẩy con trỏ của handle tiến lên, nhưng khi trả về cho bộ cấp
phát thì vẫn phải trả đúng con trỏ gốc nó đã giao.

## Thăng cấp là con đường một chiều

Có một chuỗi trạng thái ở đây:

```
hằng số ─────────────────────────────────────  (không bao giờ đổi)

độc quyền ──(clone lần đầu)──► chia sẻ ──(clone tiếp)──► chia sẻ ──► ...
            promotion
            ◄─── không có chiều ngược ───
```

Vì sao *chia sẻ không bao giờ quay về độc quyền*? Vì một khi đã có từ hai handle
cùng chia sẻ, không handle nào tự biết mình có phải kẻ cuối cùng không nếu thiếu
counter. Muốn quay về độc quyền thì phải bỏ counter đi — nhưng bỏ nó là mất luôn khả
năng đếm, mà nếu vẫn còn từ hai handle thì đó là một cú double-free đang chờ. Nên
một khi đã lên chia sẻ, ở lại chia sẻ.

(Về lý thuyết, nếu counter tụt xuống lại còn 1, ta *có thể* hạ cấp về độc quyền để né
chi phí atomic. Thư viện `bytes` thật không làm — độ phức tạp không bõ so với cái
lợi. Đây là một quyết định *không làm* đáng ghi nhận: đôi khi thiết kế tốt là biết
dừng lại đúng chỗ.)

Điều này cũng lý giải một cái tên. Cái nhãn của `b1` lúc đầu là "độc quyền", nhưng nó
*có thể sẽ* thành "chia sẻ". Ở Phần 5 ta sẽ thấy cái nhãn này không tự sửa được, nên
thay vì gọi nó là "độc quyền" (owned), ta sẽ gọi nó bằng một cái tên phản ánh khả
năng biến đổi — *promotable*, "có thể thăng cấp". Nhưng lý do sâu xa của cái tên nằm
ở một ràng buộc kỹ thuật của Phần 5; bây giờ chỉ cần hiểu: "độc quyền" ở đây nghĩa
là "đang một mình, nhưng sẵn sàng lên chia sẻ".

## Điều bất thường: sửa ngược vào một giá trị đang tồn tại

Giờ tới cái làm phần này khó *về mặt khái niệm*, chứ không phải về code.

Trong Rust bình thường, cách sở hữu của một giá trị được *cố định ngay lúc nó sinh
ra*. Một `Vec` sinh ra là `Vec` cho tới lúc chết. Một `Arc` sinh ra là `Arc`. Bạn
không bao giờ "biến" một giá trị đang sống từ cách sở hữu này sang cách khác — bạn
tạo một giá trị *mới* và bỏ cái cũ đi.

Ở đây thì khác hẳn. `b1` sinh ra là độc quyền, nhưng *bị biến thành chia sẻ giữa
chừng vòng đời*, bởi *một giá trị khác* — là `b2` — trong đúng lúc `b2` đang được tạo
ra. `b1` không tự đổi; nó *bị* `b2` đổi.

Chính điều bất thường này đẻ ra toàn bộ độ phức tạp còn lại. Để `clone` (đang chạy
nhân danh việc tạo `b2`) sửa được `b1`, nó phải thỏa mãn ba đòi hỏi *độc lập* với
nhau.

Đòi hỏi thứ nhất là phải có *đường đi tới* cái trường của `b1`. Hiện tại, khi
`clone` chạy, nó nhận `data` của `b1` dưới dạng một *bản sao* — 8 byte được chép ra.
Gán giá trị mới cho bản sao đó thì `b1` gốc chẳng hề hay biết. Muốn sửa được `b1`,
`clone` phải nhận *tham chiếu tới* cái trường thật, không phải bản sao.

Đòi hỏi thứ hai là phải *ghi được* qua cái đường đó. Ngay cả khi đã có tham chiếu
tới trường của `b1`, `clone` chỉ có một tham chiếu *chia sẻ, chỉ đọc* tới `b1` (vì
chữ ký của `clone` trong Rust là `&self`). Ghi qua một tham chiếu chỉ-đọc là điều
Rust *cấm* mặc định. Cần một cơ chế đặc biệt.

Đòi hỏi thứ ba là phải *an toàn khi nhiều thread cùng làm*. `Bytes` sẽ buộc phải gửi
được qua thread (Phần 5 sẽ giải thích vì sao bắt buộc). Khi đó, hai thread có thể
cùng cầm tham chiếu tới `b1` và cùng gọi `clone`, cùng cố thăng cấp nó. Làm ngây thơ
thì hai thread cấp ra hai counter, một cái bị bỏ rơi — rò rỉ, hoặc tệ hơn.

Ba đòi hỏi này thuộc ba phạm trù hoàn toàn khác nhau — một cái về chuyện truyền tham
số (bản sao hay tham chiếu), một cái về luật mượn của Rust (ghi qua tham chiếu chỉ-
đọc), một cái về mô hình bộ nhớ đa luồng. Chúng không hề biết tới nhau. Vậy mà, như
Phần 5 sẽ cho thấy, cả ba cùng chỉ về *một* thay đổi duy nhất về kiểu của trường
`data`.

## Chữ ký phải đổi

Cụ thể, các hàm trong vtable phải chuyển từ "nhận `data` dưới dạng bản sao" sang
"nhận tham chiếu tới `data`". Và có một chi tiết nhỏ mà thú vị: hàm `clone` sẽ nhận
tham chiếu *chia sẻ* (chỉ đọc, vì nó có `&self`), còn hàm `drop` được nhận tham
chiếu *độc quyền* (`&mut self`, vì khi một giá trị đang bị hủy thì chắc chắn không
thread nào khác còn giữ nó). Sự khác biệt "clone thì chia sẻ, drop thì độc quyền"
này nghe nhỏ, nhưng ở Phần 5 nó sẽ có hệ quả rất cụ thể: `drop` đọc `data` mà không
cần atomic, còn `clone` thì cần.

## Đã có gì, và Phần 5 giải nốt gì

Phần 4 kết lại ở đây: clone một vùng độc quyền là double-free, nên *promotion* —
thăng cấp cả hai handle lên chia sẻ — là lối thoát duy nhất; ta cấp một khối
`Shared` có counter, chuyển cả hai handle sang đó, payload không di chuyển. Thăng cấp
là một chiều. Và điều bất thường cốt lõi: `clone` phải *sửa ngược* vào `b1` — một giá
trị đang tồn tại — vì chính hành động clone làm lời hứa của `b1` thành sai.

Việc sửa-ngược đó đặt ra ba đòi hỏi độc lập: có đường tới trường, ghi được qua tham
chiếu chỉ-đọc, và an toàn đa luồng. Phần 5 sẽ cho thấy cả ba hội tụ vào một kiểu
trường duy nhất, và mỗi đòi hỏi tương ứng với một mảnh của bài toán concurrency —
interior mutability (ghi qua tham chiếu chỉ đọc), CAS (chọn đúng một kẻ thắng khi
đua), và memory ordering (đảm bảo thread kia *nhìn thấy* được cái vừa thăng cấp). Đó
là phần trừu tượng nhất của loạt bài, và ta sẽ đi thật chậm.

---

*Tiếp theo: [Phần 5 — `AtomicPtr`: sửa ngược một cách an toàn](05_atomics.md) ·
[Mục lục](00_index.md)*

*English: [`../en/04_promotion.md`](../en/04_promotion.md)*
