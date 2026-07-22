# Phần 5 — `AtomicPtr`: sửa ngược một cách an toàn

Phần 4 khép lại ở một bài toán và ba đòi hỏi. Bài toán: khi clone một `Bytes` độc
quyền, ta phải *sửa ngược* vào cái gốc để thăng cấp nó lên chia sẻ, nếu không thì
double-free. Ba đòi hỏi để làm được việc sửa-ngược đó: phải có đường đi tới trường
`data` của cái gốc; phải ghi được qua một tham chiếu chỉ-đọc; và phải an toàn khi
nhiều thread cùng làm.

Phần này giải cả ba. Và điều đẹp là cả ba, dù thuộc ba thế giới khác nhau, đều được
giải bởi một lựa chọn kiểu duy nhất cho trường `data`: biến nó thành một `AtomicPtr`
— một con trỏ nguyên tử. Đây là phần trừu tượng nhất của loạt bài, nên ta sẽ mổ từng
đòi hỏi một, và với mỗi cái, khái niệm concurrency tương ứng sẽ hiện ra như một câu
trả lời cho một vấn đề *cụ thể* ta đang thật sự vướng, chứ không phải lý thuyết trên
không.

## Vì sao lại là một con trỏ nguyên tử

Nhắc lại ba đòi hỏi của Phần 4, vì điều đáng chú ý là chúng thuộc ba chuyện chẳng
liên quan gì nhau. Đòi hỏi thứ nhất — "phải có đường tới trường `data` của cái gốc"
— là chuyện *truyền tham số*: hàm nhận một bản sao hay một tham chiếu. Đòi hỏi thứ
hai — "phải ghi được qua một tham chiếu chỉ-đọc" — là chuyện *luật mượn* của
compiler. Đòi hỏi thứ ba — "phải an toàn khi nhiều thread cùng làm" — là chuyện *mô
hình bộ nhớ* của phần cứng. Ba chuyện ở ba tầng khác nhau, không cái nào biết cái
nào.

Nhớ lại từ Phần 2, trường `data` đang có kiểu **`*mut ()`** — một con trỏ thô, 8
byte, "ý nghĩa để dành". Cái kiểu đó *không* đáp ứng nổi ba đòi hỏi trên: một `*mut
()` truyền vào hàm là truyền *một bản sao* (trượt đòi hỏi một); và ngay cả khi có
tham chiếu tới nó, Rust *cấm* ghi qua một tham chiếu chỉ-đọc (trượt đòi hỏi hai); và
đọc/ghi một `*mut ()` từ nhiều thread cùng lúc là data race, tức undefined behavior
(trượt đòi hỏi ba).

Cả ba trượt cùng được sửa bởi một thay đổi duy nhất: đổi kiểu của `data` từ `*mut ()`
thành **`AtomicPtr<()>`** — vẫn là 8 byte đó, cùng vai trò "ý nghĩa để dành", nhưng
giờ là một con trỏ *nguyên tử*. Nó cho phép lấy tham chiếu tới nó (giải đòi hỏi một).
Nó cho ghi qua một tham chiếu chỉ-đọc, nhờ một tính chất tên là *interior mutability*
(giải đòi hỏi hai). Và nó cung cấp các thao tác nguyên tử để nhiều thread không giẫm
lên nhau (giải đòi hỏi ba).

Đây là điều đáng mang đi như một bài học riêng: khi ai đó hỏi "vì sao trường này lại
là atomic?", câu trả lời đúng *không* phải "vì đa luồng" chung chung. Ở đây nó là ba
yêu cầu cụ thể, tách bạch nhau, tình cờ được giải bởi cùng một thứ. Nhận ra lúc nào
nhiều yêu cầu khác nhau hội tụ về một cơ chế — đó là một nửa của kỹ năng thiết kế
systems.

Ba mục tiếp theo mổ từng đòi hỏi. Đòi hỏi thứ nhất (lấy tham chiếu) thì tầm thường —
chỉ cần đổi chữ ký hàm để truyền `&data` thay vì `data`. Hai đòi hỏi còn lại mới đáng
bàn, và mỗi cái dẫn ta tới một khái niệm concurrency.

## Đòi hỏi hai — ghi qua một tham chiếu chỉ-đọc: interior mutability

Có một quy tắc trong Rust, đơn giản đến mức đáng thuộc lòng: một tham chiếu chỉ-đọc
(`&T`) thì *chỉ được đọc*. Muốn ghi qua nó, bên trong `T` *phải* chứa một thứ tên là
`UnsafeCell`.

`UnsafeCell` là *thứ duy nhất* trong toàn bộ Rust cho phép "sửa dữ liệu qua một tham
chiếu chỉ-đọc". Nó là một cái lỗ được compiler cho phép, đục thẳng vào luật mượn.
Mọi công cụ khác mà bạn từng dùng để "ghi qua tham chiếu chung" đều là `UnsafeCell`
cộng với một kỷ luật riêng để dùng nó cho an toàn:

- `Mutex` là `UnsafeCell` cộng với "phải khóa trước khi vào".
- `RefCell` là `UnsafeCell` cộng với "đếm số lượt mượn lúc chạy, sai thì panic".
- `Cell` là `UnsafeCell` cộng với "chỉ được copy vào/ra, không cho mượn ruột".
- Và một con trỏ nguyên tử là `UnsafeCell` chứa một con trỏ, cộng với "chỉ được
  đọc/ghi bằng lệnh nguyên tử của CPU".

Nên khi `clone` chỉ có một tham chiếu chỉ-đọc tới `b1` mà lại cần ghi vào `b1.data`,
cái trường đó *bắt buộc* phải chứa `UnsafeCell`. Con trỏ nguyên tử vừa đúng là điều
ta cần: nó mở được cái lỗ ghi-qua-tham-chiếu-chỉ-đọc (đòi hỏi hai), đồng thời lo
luôn phần đa luồng (đòi hỏi ba).

Có một sự đối xứng đáng nhớ ở đây. `Arc<T>` cũng chỉ cho bạn một tham chiếu chỉ-đọc
tới ruột của nó. Đó chính là *lý do* ai cũng phải viết `Arc<Mutex<T>>` — `Arc` lo
phần *chia sẻ*, còn `Mutex` lo phần *ghi*. `Bytes` gặp đúng bài toán đó, chỉ giải
khác đi: cả hai đều là "chia sẻ, và cần ghi", nhưng `Arc<Mutex<Vec>>` dùng một cái
khóa (vì `Vec` to, không nguyên-tử-hóa được), còn `Bytes` dùng một con trỏ nguyên tử
(vì thứ cần ghi chỉ đúng 8 byte).

Và đây là chỗ đáng dừng lại: vì sao `Bytes` được dùng thao tác nguyên tử thay vì một
cái khóa? Vì thứ cần bảo vệ *đúng bằng một từ máy* (8 byte trên máy 64-bit). Đây là
một sự thật của phần cứng, tra một lần rồi nhớ, không cần suy luận: một CPU 64-bit có
lệnh đọc, ghi, và "so-sánh-rồi-đổi" *nguyên tử* cho đúng phần dữ liệu 8 byte trở
xuống. Cái gì to hơn 8 byte thì *không lệnh nào* làm nguyên tử được — lúc đó mới cần
khóa (khóa cho phép đọc/ghi nhiều từ máy tuần tự dưới sự bảo vệ của một cờ). Vì
`data` chỉ 8 byte, nó tự thân "vừa là khóa vừa là dữ liệu", không cần một `Mutex`
riêng bên cạnh. Chính điều này giữ cho `Bytes` gọn nhẹ, không khóa, mà vẫn an toàn
đa luồng. Nếu thứ cần thăng cấp mà to hơn 8 byte, cả thiết kế này sụp và bạn phải
quay lại dùng khóa.

## Đòi hỏi ba — nhiều thread cùng thăng cấp: Send, Sync, và CAS

Rust có hai khái niệm về việc dữ liệu đi qua ranh giới thread. Một giá trị "gửi
được" (`Send`) nếu nó được phép *chuyển* sang thread khác. Một kiểu là "chia sẻ được"
(`Sync`) nếu một tham chiếu tới nó được phép *dùng chung* giữa các thread. Compiler
tự suy ra hai tính chất này; và vì `Bytes` chứa con trỏ thô (mà Rust mặc định coi là
không-gửi-được, không-chia-sẻ-được, vì nó bi quan về con trỏ), nên `Bytes` mặc định
*không* có hai tính chất đó.

Nhưng codebase *cần* chúng. Cache được chia sẻ giữa nhiều worker, message được gửi
qua channel giữa các thread. Nếu `Bytes` không gửi-được và chia-sẻ-được thì code
không biên dịch nổi — và bạn nhận đúng lỗi này:

```
error[E0277]: `*mut ()` cannot be sent between threads safely
```

Nên ta phải *hứa* với compiler rằng `Bytes` an toàn để gửi và chia sẻ. Lời hứa này
đúng, vì: payload thì bất biến (nhiều nơi cùng đọc không đá nhau), và trạng thái duy
nhất còn sửa được — `data` — thì là nguyên tử. Đây đúng là cái làm cho `Arc<[u8]>`
gửi-được và chia-sẻ-được, và `Bytes` có cùng hình dạng đó.

Nhưng lời hứa có cái giá của nó. Ngay khi `Bytes` chia-sẻ-được, hai thread có thể
cùng cầm một tham chiếu tới `b1` (một vùng độc quyền) và cùng gọi `clone`. Nếu ta
thăng cấp theo cách ngây thơ — đọc `data` ra, rồi ghi giá trị mới vào, thành hai
bước rời nhau — thì kịch bản này xảy ra:

```
Thread 1: đọc data, thấy "chưa thăng cấp"
Thread 2: đọc data, thấy "chưa thăng cấp"     ← chen vào giữa
Thread 1: cấp counter A, ghi data = A
Thread 2: cấp counter B, ghi data = B          ← ĐÈ mất A
```

Kết quả: hai counter được tạo, một cái (A) bị bỏ rơi — rò rỉ, hoặc nếu logic đếm
lệch thì use-after-free. Đây là một lỗi kinh điển tên là *lost update* — mất mát do
ghi đè — của kiểu "đọc rồi ghi" không nguyên tử. Hai bước rời nhau chừa ra một khe
cho thread kia lọt vào.

Cách chữa là một thao tác gộp "kiểm tra" và "ghi" lại thành một bước không thể tách,
tên là *compare-and-swap*, viết tắt CAS. Dịch nó ra tiếng người:

> "Này `data`, *nếu* mày vẫn đang là cái giá trị cũ (chưa thăng cấp) thì hãy đổi
> thành con trỏ counter của tao — và làm hai việc này *dính liền*, không thread nào
> chen vào giữa được. Còn nếu đứa khác đã đổi mày trước rồi, thì *đừng* đổi, và báo
> cho tao biết mày đang chứa gì."

Phần cứng đảm bảo, khi nhiều thread cùng lao vào, *đúng một* cú CAS thắng. Kẻ thắng
cài counter của mình vào `data`; ngay khoảnh khắc đó, `b1` trở thành chia sẻ (vì
`b1` và cái ô `data` là một). Kẻ thua nhận được tín hiệu "đứa khác đã đổi rồi", bèn
vứt cái counter mà mình lỡ cấp đi, và dùng luôn counter của kẻ thắng. Cuối cùng chỉ
còn một counter, và vùng nhớ được giải phóng đúng một lần — cách nghĩ đếm-số-lần của
Phần 3 lại cân bằng.

Một điểm cần cẩn thận trong đường-của-kẻ-thua: khi vứt cái counter thừa đi, ta phải
vứt sao cho nó *không* kéo theo việc giải phóng payload — vì payload giờ thuộc về
counter của kẻ thắng. Ta giải phóng cái *vỏ* của khối thừa mà bỏ qua hàm dọn payload
của nó. Nếu quên chi tiết này, payload bị giải phóng hai lần.

Có một cách gọi tên đẹp cho vai trò của CAS ở đây: nó là *điểm tuyến-tính-hóa*. Dù
hai thread lao vào song song, cú CAS là cái mốc biến mớ hỗn loạn đó thành một trình
tự rõ ràng — "đứa nào thắng CAS thì coi như xảy ra trước". Bất cứ khi nào bạn cần
"đúng một trong nhiều kẻ đang đua được phép làm việc X", CAS là công cụ, và cái mốc
thắng chính là điểm tuyến-tính-hóa.

## Đòi hỏi ba, phần còn lại — memory ordering: "ghi được" chưa đủ, còn phải "thấy đúng thứ tự"

CAS mới giải một nửa của đa luồng: nó đảm bảo đúng một thread *cài* được counter.
Nhưng còn một mối nguy thứ hai, tinh vi hơn và tách biệt hẳn — và đây là chỗ hầu hết
mọi người thấy khó nhất. Ta dựng vấn đề trước, rồi mổ đúng từng thao tác một để xem
mỗi cái cần "độ mạnh" đồng bộ nào.

### Vấn đề: các lần ghi bị sắp xếp lại

Kẻ thắng cuộc đua thăng cấp làm hai việc, theo thứ tự này *trong code*: trước hết nó
khởi tạo nội dung khối `Shared` (ghi vào đó địa chỉ gốc của vùng nhớ và độ dài), rồi
sau đó nó *công bố* địa chỉ khối `Shared` bằng cú CAS ghi vào `data`.

Vấn đề là phần cứng lẫn compiler đều được phép *sắp xếp lại* các lần ghi vào bộ nhớ
để chạy nhanh hơn — chúng đệm, gộp, đảo thứ tự. Với một thread đơn thì vô hại, vì kết
quả cuối cùng nhìn vẫn đúng. Nhưng với nhiều thread, một thread khác có thể nhìn thấy
các lần ghi của kẻ thắng *theo một thứ tự khác* với thứ tự trong code.

Thảm họa cụ thể: một thread thứ hai gọi `clone`, đọc `data`, thấy nó đã là địa chỉ
khối `Shared`, bèn truy cập vào khối đó để tăng counter — tức đọc nội dung của nó.
Nếu thread thứ hai nhìn thấy *cái địa chỉ* nhưng *chưa* nhìn thấy phần *nội dung* mà
kẻ thắng vừa khởi tạo — điều hoàn toàn hợp lệ dưới luật sắp-xếp-lại — thì nó đọc phải
một khối `Shared` toàn rác, và mọi thứ sau đó là undefined behavior. Cái địa chỉ đã
"chạy nhanh hơn" cái nội dung nó trỏ tới.

Ta cần một sự đảm bảo: *kẻ nào đã thấy địa chỉ khối `Shared` thì cũng phải thấy luôn
phần nội dung đã khởi tạo xong của nó.* Đây là việc của *memory ordering* — những
"nhãn" ta gắn kèm mỗi thao tác nguyên tử để quy định nó được phép bị sắp xếp lại tới
đâu.

### Bốn độ mạnh, và cách hình dung

Rust có bốn nhãn ta sẽ dùng tới: `Relaxed`, `Acquire`, `Release`, và `AcqRel`. Cách
hình dung dễ nhất cho hai cái ở giữa là "công bố" và "đăng ký nhận":

- Một thao tác **ghi** kiểu **`Release`** là một sự *công bố*: mọi thứ tôi đã ghi
  *trước* thao tác này, hễ ai đọc được giá trị tôi vừa ghi thì sẽ thấy hết.
- Một thao tác **đọc** kiểu **`Acquire`** là một sự *đăng ký nhận*: một khi tôi đọc
  được giá trị đã công bố, tôi thấy luôn mọi thứ mà kẻ công bố đã ghi *trước khi* nó
  công bố.
- **`Relaxed`** là "chỉ cần thao tác này nguyên tử thôi, không hứa gì về thứ tự với
  các lần ghi khác" — rẻ nhất.
- **`AcqRel`** là "vừa `Acquire` vừa `Release`", dành cho một thao tác *vừa đọc vừa
  ghi* (như CAS, vốn vừa đọc giá trị cũ vừa ghi giá trị mới).

Mấu chốt: `Release` và `Acquire` chỉ có tác dụng khi đi *thành cặp*, trên *cùng một
biến*. Một bên công bố, một bên đăng ký nhận; cặp đó dựng nên mối liên kết thứ tự nối
hai thread. Thiếu một bên thì cặp gãy, đảm bảo biến mất.

### Mổ từng thao tác trên `data`

Giờ áp vào đúng những chỗ code chạm vào `data` trong lúc thăng cấp, và với mỗi chỗ
hỏi: nó cần nhãn nào, vì sao.

**Cú đọc `data` đầu tiên, mở màn `clone`.** Trước khi quyết định có cần thăng cấp
không, ta đọc `data` xem nó đang là gì. Nhãn: **`Acquire`**. Vì sao? Vì có khả năng
`data` đã bị một thread khác thăng cấp trước rồi, tức nó đã là địa chỉ một khối
`Shared`; khi đó ta đi thẳng vào nhánh "đã chia sẻ" và *truy cập* khối `Shared` đó
để tăng counter. Muốn truy cập an toàn, ta phải thấy nội dung đã khởi tạo của nó —
nên cú đọc này phải là `Acquire`, để bắt cặp với cú `Release` của kẻ đã thăng cấp.

**Cú CAS — đây là câu trả lời cho "tại sao AcqRel".** Một điều nhiều người không để
ý: `compare_exchange` mang *hai* nhãn ordering, không phải một — một cho trường hợp
*thành công*, một cho trường hợp *thất bại*. Lý do là CAS có hai kết cục khác hẳn
nhau, mỗi kết cục cần một đảm bảo khác nhau.

- *Khi CAS thất bại* (đứa khác đã thăng cấp trước): CAS trả về cho ta giá trị hiện
  tại của `data` — chính là địa chỉ khối `Shared` của kẻ thắng. Và ngay sau đó ta sẽ
  *dùng* cái địa chỉ đó (tăng counter của khối `Shared` kia). Nghĩa là ta lại sắp
  truy cập một khối `Shared` do thread khác khởi tạo — nên nhánh thất bại phải là
  **`Acquire`**, đúng cùng lý do với cú đọc mở màn.
- *Khi CAS thành công* (ta là kẻ thắng): ta vừa *công bố* cái khối `Shared` mà *chính
  ta* dựng nên ngay trước đó. Để một thread khác sau này đọc được địa chỉ này rồi
  truy cập khối `Shared` mà không vớ phải rác, cú ghi này phải là **`Release`**.

Vậy nhánh thất bại cần `Acquire`, nhánh thành công cần `Release`. Và đây là điểm
chốt: Rust bắt buộc nhãn của nhánh *thành công* không được yếu hơn nhãn của nhánh
*thất bại*. Mà `Release` một mình thì *không* bao gồm `Acquire` (chúng là hai hướng
khác nhau — một cái lo phần ghi, một cái lo phần đọc). Nên để nhánh thành công vừa có
`Release` (cho phần công bố của chính nó), vừa đủ mạnh so với nhánh thất bại
`Acquire`, nó phải mang *cả hai* — và cái nhãn "vừa Acquire vừa Release" đó chính là
**`AcqRel`**.

Nói gọn: `AcqRel` cho CAS không phải chọn cho "chắc ăn", mà là nhãn duy nhất thỏa mãn
đồng thời hai việc trên cùng một lệnh — kẻ thua phải *nhận* được khối `Shared` của kẻ
thắng (`Acquire`), và kẻ thắng phải *công bố* khối `Shared` của mình (`Release`).

**Cú đọc `data` trong `drop` — không cần nguyên tử gì cả.** Vì `drop` nhận tham chiếu
*độc quyền* tới giá trị (nhớ lại Phần 4), nó biết chắc không thread nào khác còn giữ
giá trị này — không có đua, nên đọc thường là đủ. Tham chiếu độc quyền tự nó đã là
bằng chứng không có race. Đây chính là lý do chữ ký `drop` dùng tham chiếu độc quyền
còn `clone` dùng tham chiếu chia sẻ: không phải phong cách, mà là "drop có độc quyền,
clone thì không".

**Trường hợp rìa: `Bytes` hằng số.** `data` để rỗng, chẳng đồng bộ hóa gì với ai —
nên mọi cú chạm vào nó chỉ cần **`Relaxed`**, nhãn rẻ nhất.

### Vì sao không dùng `SeqCst` cho chắc

`SeqCst` (sequential consistency) là nhãn mạnh nhất Rust có — nó bắt *mọi* thao tác
`SeqCst` trong toàn chương trình xếp thành một trình tự toàn cục duy nhất mà mọi
thread đều đồng ý. Nghe thì an toàn, và nhiều người phản xạ dùng nó cho "chắc". Nhưng
ở đây nó vừa *thừa* vừa *đắt*. Thừa, vì thứ ta cần chỉ là một mối liên kết *từng-cặp*
giữa kẻ công bố và kẻ nhận trên đúng một biến `data` — không cần cả chương trình đồng
ý về một trình tự chung. Đắt, vì `SeqCst` thường phải chèn thêm memory fence mạnh hơn
hẳn, làm chậm đúng cái đường mà cả thiết kế này sinh ra để giữ cho nhanh. Chọn đúng
độ mạnh tối thiểu cần thiết — `Acquire`/`Release`/`AcqRel` đúng chỗ, `Relaxed` ở chỗ
không cần — chính là một phần của việc viết code lock-free cho tử tế.

Nguyên tắc để mang đi: `Release` và `Acquire` luôn đi thành cặp trên cùng một biến,
nối một sự "công bố" với một sự "đăng ký nhận"; một thao tác *vừa đọc vừa ghi* mà cả
hai vai đều quan trọng (như CAS) thì cần `AcqRel`; và hễ khi nào bạn công bố cho
thread khác một con trỏ trỏ tới dữ liệu vừa-mới-khởi-tạo, bạn *luôn* cần cặp này — nếu
không, dữ liệu có thể "đến sau" con trỏ.

## Một hệ quả gọn: vì sao cái nhãn đổi thành "promotable"

Chi tiết cuối cùng, và nó là hệ quả trực tiếp của việc "sửa ngược" đụng phải một
giới hạn.

Khi thăng cấp, `clone` ghi được `b1.data` (nhờ con trỏ nguyên tử) nhưng *không* ghi
được `b1.vtable` — vì `vtable` là một trường thường, *không* có interior mutability,
mà `clone` chỉ có tham chiếu chỉ-đọc. Nên sau khi thăng cấp:

```
b1.vtable = vẫn là bảng "độc quyền"   ← kẹt cứng, sửa không được, giờ nói dối
b1.data   = đã là địa chỉ khối Shared  ← đã đổi (qua CAS), nói thật
```

`b1` mãi mãi điều phối qua cái bảng cũ, dù thực chất nó đã là chia sẻ. Nên cái bảng
đó, ở *lệnh đầu tiên của mỗi lần gọi* (cả `clone` lẫn `drop`), phải tự hỏi: `data`
của tôi bây giờ là một con số độ dài (chưa thăng cấp) hay là một địa chỉ khối Shared
(đã thăng cấp)? — rồi rẽ nhánh cho đúng. Chính vì lý do này mà cái nhãn *không phải*
là "độc quyền" mà là "promotable" — "kẻ này *có thể đã* thành chia sẻ rồi".

Nhìn sâu hơn một chút, đây là một quy luật chung: cái đánh dấu-phân-loại *phải nằm
trong cái ô sửa-được* (`data`, nguyên tử), *không phải trong con trỏ vtable bất
biến*. Đó là hệ quả tất yếu của việc sở hữu bị đổi giữa chừng vòng đời: con trỏ
vtable đóng băng ngay lúc giá trị sinh ra, nên nó không thể là nơi lưu trạng thái
động. Hễ khi nào trạng thái của một giá trị đổi *sau khi* nó sinh ra, cái đánh dấu
trạng thái đó phải nằm ở phần *sửa-được*, không phải phần *bất biến* — và ở đây, chỉ
có `data` là sửa-được.

(Một ghi chú ngoài lề, *không* thuộc về mô hình: làm sao `data` vừa chứa được một con
số độ dài vừa chứa được một địa chỉ khối Shared trong cùng 8 byte, và cái bảng
"promotable" phân biệt được hai loại? Có một thủ thuật mượn *bit thấp nhất*: địa chỉ
của khối Shared luôn là số chẵn — do quy tắc căn lề của bộ nhớ — nên bit thấp nhất
của nó luôn là 0; ta bèn bật bit thấp lên 1 khi cất một con số vào, và chỉ cần nhìn
bit thấp là biết đang chứa loại nào. Đây thuần túy là một *tối ưu không gian* — thay
nó bằng một trường riêng để chứa độ dài cũng hoàn toàn đúng, chỉ tốn thêm một từ máy.
Thư viện `bytes` dùng pointer tagging vì nó đếm từng byte; một bản làm-để-học thì không nhất
thiết.)

## Tổng kết: design hoàn chỉnh

Đây là toàn bộ thiết kế, đặt cạnh nhau một lần. So với bản tổng kết cuối Phần 2, chỉ
*một* dòng đổi — kiểu của `data` — nhưng cả Phần 4 và 5 là để giải thích đúng dòng đó.

```rust
struct Bytes {
    ptr:    NonNull<u8>,      // "byte nào": trỏ tới đầu dãy byte
    len:    usize,            // "byte nào": dài bao nhiêu
    data:   AtomicPtr<()>,    // "ai sở hữu": 8 byte nguyên tử; *mut () ở P2, giờ atomic
    vtable: &'static Vtable,  // "ai sở hữu": dùng bộ clone/drop nào
}

struct Vtable {
    clone: unsafe fn(&AtomicPtr<()>, ptr, len) -> Bytes,  // & để clone sửa được cái gốc
    drop:  unsafe fn(&mut AtomicPtr<()>, ptr, len),       // &mut: độc quyền, đọc khỏi atomic
}
```

Ba cách sở hữu, và `data` mang gì trong mỗi cách:

| vtable              | `data` chứa gì                     | `clone` làm gì            | `drop` làm gì  |
|---------------------|------------------------------------|---------------------------|----------------|
| `STATIC_VTABLE`     | null (không dùng)                  | copy struct               | không làm gì   |
| `PROMOTABLE_VTABLE` | độ dài (con số), *hoặc* địa chỉ counter sau khi bị thăng cấp | nếu chưa thăng cấp: dựng counter, CAS lên chia sẻ; nếu rồi: tăng counter | nếu chưa: giải phóng; nếu rồi: giảm counter |
| `SHARED_VTABLE`     | địa chỉ counter (con trỏ thật)     | tăng counter              | giảm counter   |

Đọc dữ liệu (`deref`, `len`, so sánh, hash) chỉ chạm `ptr` + `len` — không bao giờ
đụng `data` hay `vtable`, nên rẻ y như `Arc<[u8]>`. Còn `data` + `vtable` chỉ vào
cuộc khi `clone` hoặc `drop`. Đó là toàn bộ thiết kế.

Đường đi từ đầu tới đây, tóm trong một hình:

```
Arc<[u8]>            P1: counter dính liền payload ⇒ freeze BẮT BUỘC copy
   │
   ▼ cần freeze O(1) ⇒ Bytes phải NHẬN vùng nhớ, không copy
Bytes{ ptr, len, data, vtable }
   │  P2: hạ "sở hữu" từ type xuống vtable (một kiểu, ba hành vi)
   │      data: *mut ()  — 8 byte "ý nghĩa để dành"
   │  P3: tách "byte nào" (ptr,len) khỏi "ai sở hữu" (data,vtable) ⇒ đọc miễn phí
   │      + cách nghĩ: mỗi vùng nhớ giải phóng đúng mấy lần? (0/1/1)
   │
   ▼ P4: clone một vùng độc quyền = double-free
   │      ⇒ promotion: sửa NGƯỢC vào cái gốc để đẩy nó lên chia sẻ
   │
   ▼ P5: sửa-ngược cần &data + ghi-qua-& + an-toàn-đa-luồng
          ⇒ data: *mut ()  ➜  AtomicPtr<()>
          interior mutability · CAS · Acquire/Release/AcqRel
```

## Năm câu hỏi mang sang mọi bài toán về sau

Gấp loạt bài lại, quên vtable, atomic, CAS đi. Cái đáng mang theo — sang những bài
sau này về write-ahead log, về chia sẻ node trong skiplist, về cache khối dữ liệu, và
về mọi thiết kế có dính unsafe, sở hữu, hay tối ưu — là năm câu hỏi này. Chúng là
thứ chắt lại từ toàn bộ câu chuyện.

Thứ nhất: *mỗi vùng nhớ bị giải phóng đúng mấy lần?* 0 là rò rỉ, 2 là double-free, 1
là đúng. Mọi bug đều quy về con số này.

Thứ hai: *cái gì khác nhau giữa các trường hợp?* Chỉ cái đó mới cần điều phối. Cái gì
giống nhau thì để yên — đó chính là lý do đường-nóng được miễn phí.

Thứ ba: *đường-nóng có đụng vào điều phối không?* Nếu có thì thiết kế sai. Việc đọc
phải rẻ y như `Arc<[u8]>`.

Thứ tư: *có ghi qua một tham chiếu chỉ-đọc không?* Nếu có thì cần interior
mutability. Một tham chiếu chỉ-đọc trần trụi thì luôn chỉ được đọc.

Thứ năm: *có nhiều thread không?* Nếu có thì dùng thao tác nguyên tử thay cho cái
đọc-ghi thường; thao tác kiểu "đúng một kẻ thắng" thì dùng CAS; và mỗi lần công bố
một con trỏ-trỏ-tới-dữ-liệu cho thread khác đọc thì dùng cặp release/acquire trên
cùng một biến.

Và ba câu để chốt lại về cách nghĩ, rải rác suốt loạt bài. `Drop` không dọn cái
struct — cái struct tự tan; `Drop` chỉ hoàn tác một lần cấp phát, nên không cấp phát
thì không `Drop`. Một vtable là kiểu-dữ-liệu bị hạ từ tầng biên-dịch xuống thành một
giá trị lúc-chạy, dùng khi một kiểu cần nhiều hành vi chọn theo từng giá trị. Và con
bug đáng sợ trong unsafe không phải con làm sập chương trình, mà là con chạy đúng —
trực giác từ Rust an toàn bị đảo ngược, mặc định của cái sai là im lặng, nên hãy
luôn mang theo `miri` để bắt nó phải lên tiếng.

Đến đây bạn đã có đủ mô hình trong đầu để ngồi xuống viết lại `Bytes` từ số 0 — bốn
hành vi cho ba cách sở hữu, thao tác `freeze`, và promotion — và biện luận được cho
từng lựa chọn. Các chi tiết code còn lại sẽ tự lộ ra khi bạn viết, vì bạn đã hiểu lý
do tồn tại của từng cái.

---

*Quay lại: [Phần 4](04_promotion.md) · [Mục lục](00_index.md)*

*English: [`../en/05_atomics.md`](../en/05_atomics.md)*
