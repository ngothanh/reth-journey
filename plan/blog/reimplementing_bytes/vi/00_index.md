# Viết lại `Bytes`: một kiểu dữ liệu, ba cách sở hữu bộ nhớ

Đây là một loạt bài về một mẩu code nhỏ nhưng nổi tiếng là khó: một *handle byte
zero-copy*. Nếu bạn từng dùng thư viện `bytes` trong Rust, hay `IOBuf` của Facebook,
hay `ByteBuf` của Netty, thì đây chính là thứ nằm bên trong chúng — chỉ có điều ta
sẽ tự xây lại từ đầu để hiểu vì sao nó được thiết kế như vậy.

Loạt bài không phải hướng dẫn code kiểu "gõ theo từng dòng". Nó là một cuộc *điều
tra*: ta bắt đầu từ một nhu cầu rất đời thường (một chương trình mạng đọc dữ liệu
lên rồi chuyền đi), gặp một vấn đề hiệu năng, thử vài cách hiển nhiên, thấy chúng
hỏng, và mỗi lần đâm vào tường thì một mảnh của thiết kế thật lại lộ ra. Không có
mảnh nào từ trên trời rơi xuống; mỗi quyết định đều bị *ép* bởi cái trước nó.

Bạn không cần biết trước `Bytes`, `BytesMut`, hay `freeze` là gì — Phần 1 sẽ dựng
mọi thứ từ số 0.

## Loạt bài gồm năm phần

**[Phần 1 — Một byte đi từ dây mạng vào chương trình.](01_the_problem.md)**
Ta dựng bối cảnh: một chương trình mạng hứng dữ liệu, cần một buffer ghi-được
(`BytesMut`), rồi cần biến nó thành một handle chỉ-đọc chia-sẻ-được (`Bytes`) qua
một thao tác tên là `freeze`. Ta phát hiện `freeze` có thể rất chậm nếu nó copy, và
đặt ra yêu cầu: `freeze` phải không copy. Rồi ta thử hai cách hiển nhiên (`Vec<u8>`
và `Arc<[u8]>`) và xem chúng hỏng ở đâu — từ đó lộ ra mâu thuẫn trung tâm: *một kiểu
dữ liệu, ba cách dọn bộ nhớ.*

**[Phần 2 — Một kiểu, nhiều hành vi.](02_vtable.md)**
Trong Rust, "cách dọn bộ nhớ" bình thường gắn liền với *kiểu dữ liệu*, và compiler
lo hết. Nhưng ta chỉ có một kiểu mà cần ba hành vi. Phần này cho thấy cách hạ cái
quyết-định-dọn-dẹp từ compiler xuống thành *dữ liệu trong struct* — tức một cái bảng
điều phối (vtable) tự viết. Kèm một câu hỏi mà bất kỳ ai thiết kế kiểu này đều phải
trả lời được: vì sao cái bảng đó có đúng *hai* ô.

**[Phần 3 — Tách "byte nào" khỏi "ai sở hữu".](03_split_and_counting.md)**
Bí quyết khiến thiết kế này vừa linh hoạt vừa *nhanh*: xếp các trường của struct sao
cho việc đọc byte không bao giờ phải nhìn tới phần thông tin sở hữu. Phần này cũng
giới thiệu một cách nghĩ đơn giản mà xuyên suốt phần khó về sau — mọi cách sở hữu bộ
nhớ, rút gọn lại, chỉ là một câu hỏi: *vùng nhớ này bị giải phóng đúng mấy lần?*

**[Phần 4 — Bức tường: khi clone làm hỏng mọi thứ.](04_promotion.md)**
Đây là chỗ khó nhất. Ba trong bốn hành vi thì dễ, nhưng `clone` một handle "sở hữu
độc quyền" lại gây double-free. Lối thoát duy nhất — gọi là *promotion* — buộc một
điều rất bất thường trong Rust: một giá trị phải *sửa ngược* vào một giá trị khác
đang tồn tại, giữa chừng vòng đời của nó.

**[Phần 5 — `AtomicPtr`: sửa ngược một cách an toàn.](05_atomics.md)**
Việc sửa-ngược ở Phần 4 đặt ra ba đòi hỏi độc lập, và cả ba tình cờ được giải bởi
một kiểu trường duy nhất. Phần này đi qua ba khái niệm concurrency mà nhiều người
thấy trừu tượng nhất — interior mutability, CAS, và memory ordering — nhưng lần này
mỗi khái niệm gắn với một vấn đề cụ thể ta đang thật sự phải giải, chứ không phải
lý thuyết suông. Khép lại bằng năm câu hỏi có thể mang sang mọi bài toán systems sau
này.

## Đọc thế nào

Đọc tuần tự — phần sau dựa trực tiếp vào cái phần trước vừa dựng. Mỗi phần dài
khoảng 15 phút, tự chứa, mở đầu bằng chỗ phần trước bỏ dở và khép lại bằng câu hỏi
mà phần sau sẽ nhặt lên.

## Phạm vi

Loạt bài bàn về *thiết kế*, không phải chi tiết cài đặt. Ta cố tình bỏ qua vài thứ
chỉ liên quan tới code (chữ ký hàm chính xác, vài mẹo về `Layout`, một thủ thuật
nhồi-bit để tiết kiệm bộ nhớ). Những cái đó tự lộ ra khi bạn ngồi viết, nếu đã nắm
được năm phần này.

*English version: [`../en/00_index.md`](../en/00_index.md)*
