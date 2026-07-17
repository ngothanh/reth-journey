# Phần 2 — Một kiểu, nhiều hành vi

Phần 1 kết lại ở một mâu thuẫn. Ta cần một kiểu `Bytes` duy nhất (vì biên giới API
đòi vậy), nhưng kiểu đó phải biết ba cách dọn bộ nhớ khác nhau — không dọn gì với
hằng số, giải phóng với vùng nhớ độc quyền, đếm-rồi-giải-phóng với vùng chia sẻ — và
cách nào áp dụng thì chỉ biết lúc chạy. Phần này đi tìm cơ chế cho phép điều đó.

Nhưng trước khi phát minh gì mới, đáng để nhìn kỹ cách Rust *bình thường* giải bài
này. Vì hóa ra thiết kế của ta không mới lạ gì — nó chỉ là bắt chước cái Rust vẫn
làm, hạ xuống một tầng.

## Bình thường, "cách dọn" đã nằm sẵn trong kiểu dữ liệu

Hãy nhìn ba kiểu quen thuộc, và để ý điều gì quyết định cách chúng được dọn:

```rust
&'static [u8]   // một chuỗi byte hằng, sống suốt đời chương trình
Vec<u8>         // một mảng byte do mình sở hữu
Arc<[u8]>       // một mảng byte chia sẻ, có counter
```

Khi một `Vec<u8>` ra khỏi phạm vi và bị hủy, compiler chạy đoạn code giải phóng vùng
nhớ của nó. Khi một `Arc<[u8]>` bị hủy, compiler chạy đoạn code giảm counter, và chỉ
giải phóng nếu counter về 0. Khi một `&'static [u8]` bị hủy, compiler chẳng chạy gì
cả — nó là tham chiếu tới hằng số, chả có gì để dọn.

Điều đáng chú ý: **bạn không bao giờ phải nói với compiler chạy đoạn nào.** Nó nhìn
*kiểu* của giá trị là biết. `Vec` thì dùng cách của `Vec`, `Arc` thì dùng cách của
`Arc`. Quyết định này được chốt *lúc biên dịch*, và nó miễn phí — không có bước "hỏi
lúc chạy xem đây là loại gì" nào cả, vì kiểu đã trả lời từ trước.

Nói cách khác, trong Rust, một *kiểu dữ liệu* không chỉ là hình dạng của dữ liệu. Nó
còn *đính kèm* một chương trình con — cách clone, cách drop — mà compiler tra ra và
gắn vào đúng chỗ. Có thể xem kiểu như một tấm bảng tra cứu hành vi, vô hình và tĩnh.

Đây chính là lý do Rust nhanh: mỗi cách sở hữu là một kiểu riêng, mọi quyết định dọn
dẹp chốt xong từ lúc biên dịch, chương trình chạy không phải nghĩ ngợi gì.

Và đây cũng chính là điều mà phương án "ba kiểu riêng" ở cuối Phần 1 định tận dụng —
để compiler làm hộ. Nó chết chỉ vì một lý do: biên giới API cần một kiểu duy nhất.
Nên giờ ta phải tự tay làm cái việc mà compiler vẫn âm thầm làm — nhưng làm lúc
chạy.

## Chỗ va chạm: một kiểu thì chỉ có một hàm `Drop`

Ràng buộc "một kiểu" đâm thẳng vào cách Rust gắn hành vi vào kiểu. Trong Rust, để
định nghĩa "khi một giá trị bị hủy thì làm gì", bạn viết một khối `impl Drop`:

```rust
impl Drop for Bytes {
    fn drop(&mut self) {
        // Viết gì ở đây bây giờ?
    }
}
```

Nhưng khối này chỉ được viết *một lần* cho kiểu `Bytes`. Và bạn không viết nổi một
dòng nào đúng cho cả ba trường hợp:

- Nếu viết "giải phóng vùng nhớ", nó đúng với vùng độc quyền, nhưng với hằng số thì
  đây là giải phóng bộ nhớ mình không sở hữu — chương trình sập.
- Nếu viết "không làm gì", đúng với hằng số, nhưng với vùng độc quyền thì rò rỉ bộ
  nhớ (leak) — vùng nhớ không bao giờ được trả lại.
- Nếu viết "giảm counter", đúng với vùng chia sẻ, nhưng hằng số và vùng độc quyền
  làm gì có counter mà giảm.

Không tồn tại một dòng code *cố định* nào đúng cho cả ba, vì loại nào thì chỉ biết
lúc chạy, tùy từng giá trị — `b1` này là hằng số, `b2` kia là vùng độc quyền, `b3`
nọ là vùng chia sẻ. Compiler đã hết cách: nó chọn hành vi *theo kiểu*, mà ta chỉ có
một kiểu.

Vậy thì quyết định "chọn hành vi nào" phải rời khỏi compiler và chuyển xuống lúc
chạy. Câu hỏi là chuyển bằng cách nào.

## Ý tưởng: đem "cách dọn" nhét vào trong chính giá trị

Nếu compiler không chọn được lúc biên dịch, thì hãy để *bản thân giá trị mang theo
cách chọn*. Mà muốn chọn lúc chạy, cái để chọn phải là *dữ liệu nằm trong struct*,
không phải kiểu.

Cụ thể: ta lấy đúng cái mà "kiểu" vẫn mang theo một cách vô hình — chương trình con
clone/drop — và biến nó thành một trường hữu hình. Trong Rust, "một chương trình
con có thể lưu vào biến" chính là *con trỏ hàm* (function pointer). Nên ta gom hai
con trỏ hàm lại thành một cái bảng nhỏ:

```rust
struct Vtable {
    clone: /* con trỏ tới hàm: "clone loại này thì chạy đoạn này" */,
    drop:  /* con trỏ tới hàm: "drop  loại này thì chạy đoạn này" */,
}
```

("Vtable" là tên truyền thống cho loại bảng này — *virtual table*, bảng các hàm ảo.)

Rồi ta tạo sẵn ba cái bảng, mỗi bảng cho một cách sở hữu, và cho chúng sống suốt đời
chương trình:

```rust
static STATIC_VTABLE: Vtable = /* clone, drop kiểu hằng số   */;
static OWNED_VTABLE:  Vtable = /* clone, drop kiểu độc quyền  */;
static SHARED_VTABLE: Vtable = /* clone, drop kiểu chia sẻ    */;
```

Và mỗi giá trị `Bytes` mang theo một con trỏ, chỉ tới một trong ba bảng đó:

```rust
struct Bytes {
    /* ... byte nằm ở đâu ... */
    vtable: &'static Vtable,   // con trỏ này quyết định số phận của giá trị
}
```

Bây giờ hàm `Drop` của `Bytes` chỉ còn một việc: *đọc trường `vtable`, rồi gọi hàm
`drop` nằm trong đó*. Giá trị nào cầm bảng `STATIC_VTABLE` thì gọi vào hàm không-làm
-gì; giá trị nào cầm `OWNED_VTABLE` thì gọi vào hàm giải-phóng. Một khối `impl Drop`
duy nhất, ba hành vi khác nhau, chọn đúng theo từng giá trị lúc chạy. Đúng thứ ta
cần.

Cách nhìn gọn nhất về chuyện vừa xảy ra:

> Cái vtable *chính là* kiểu dữ liệu, chỉ bị hạ từ tầng biên-dịch xuống thành một
> giá trị lúc-chạy. Ba kiểu `&'static [u8]`, `Vec<u8>`, `Arc<[u8]>` không biến mất —
> chúng hóa thành ba giá trị `STATIC_VTABLE`, `OWNED_VTABLE`, `SHARED_VTABLE` mà ta
> gán được vào một trường, so sánh được, gọi được, lúc chạy.

### Thật ra bạn đã dùng vtable rồi

Nếu bạn từng viết `&dyn SomeTrait` trong Rust, bạn đã dùng vtable — chỉ là compiler
dựng bảng hộ. Một `&dyn Trait` bên trong là một cặp con trỏ: một trỏ tới dữ liệu,
một trỏ tới cái bảng các method của trait. Khi bạn gọi một method qua `dyn`,
chương trình tra bảng để tìm đúng hàm, rồi gọi. Đó đúng là cơ chế ta đang dựng tay.

Khác biệt duy nhất giữa `dyn` và cái ta làm:

- Với `dyn`, mỗi trường hợp là một *kiểu khác nhau* (một `u64`, một `String`...), nên
  compiler biết dựng bảng nào cho kiểu nào.
- Ở đây, cả ba trường hợp *đã là cùng một kiểu* `Bytes`. Compiler không còn gì để
  phân biệt mà dựng bảng — nên ta dựng tay, và tự tay gán bảng nào cho giá trị nào,
  vào đúng lúc tạo ra giá trị đó.

Điều này cho ta một quy tắc dùng được về sau: khi nào nên tự viết vtable? Chính xác
khi *một* kiểu cần *nhiều* hành vi, chọn tùy từng giá trị lúc chạy. Nếu là nhiều
kiểu, dùng `dyn`. Nếu chỉ một hành vi, dùng hàm thường. Vtable tay lấp đúng vào ô
"một kiểu, nhiều hành vi, chọn per-giá-trị".

## Vì sao cái bảng có đúng hai ô

Đây là câu hỏi mà bất kỳ ai thiết kế kiểu này cũng nên tự trả lời được, vì nó là
phép thử xem bạn *hiểu* thiết kế hay chỉ chép lại. Ta xét cả hai chiều: vì sao không
*ít hơn* hai, và vì sao không *nhiều hơn* hai.

Có một quy tắc đơn giản để quyết định. Một ô hàm chỉ đáng có mặt trong vtable khi
hành vi của thao tác đó *thay đổi tùy theo ai sở hữu vùng nhớ*. Nếu một thao tác làm
y hệt nhau bất kể sở hữu kiểu gì, đưa nó vào vtable là vô nghĩa — tệ hơn, là có hại,
như ta sẽ thấy.

Thử liệt kê *mọi* thứ một `Bytes` làm được, và hỏi từng cái "có phụ thuộc sở hữu
không":

- Đọc byte ra (lấy độ dài, lấy nội dung, so sánh, in ra): với cả ba loại, câu trả
  lời đều là "cứ nhìn vào con trỏ và độ dài mà đọc". *Không* phụ thuộc sở hữu.
- `clone`: hằng số thì copy nguyên cái struct; độc quyền thì phải làm một việc phức
  tạp (Phần 4); chia sẻ thì tăng counter. *Có* phụ thuộc sở hữu.
- `drop`: hằng số không làm gì; độc quyền giải phóng; chia sẻ giảm counter. *Có* phụ
  thuộc sở hữu.

Đúng hai thao tác phụ thuộc sở hữu. Nên đúng hai ô: `clone` và `drop`.

Vì sao không *ít hơn*? Có gộp `clone` và `drop` vào một ô được không? Không — chúng
là hai thao tác độc lập, xảy ra ở hai thời điểm khác nhau (một cái khi nhân bản
handle, một cái khi hủy handle), và hành vi của cái này không suy ra được từ cái
kia. Bỏ ô `drop` thì không biết cách giải phóng; bỏ ô `clone` thì không biết cách
nhân bản. Mỗi cái phụ thuộc sở hữu theo cách riêng, nên mỗi cái cần ô riêng.

Còn nửa "đọc byte" — vì sao *không* có ô? Vì nó không phụ thuộc sở hữu. Con trỏ và
độ dài đã trả lời trọn vẹn câu hỏi "byte nào", giống hệt nhau ở cả ba loại. Cho nó
một ô vtable nghĩa là bắt mỗi lần đọc phải trả giá một cú gọi hàm gián tiếp
(indirect call) — cho một thao tác chẳng cần biết ai sở hữu. Đây là hạt giống của
Phần 3, nhưng đã thấy được từ đây: một thao tác không phụ thuộc sở hữu thì *bị cấm*
khỏi vtable, vì đưa nó vào là bắt đường-nóng trả giá vô ích.

Vì sao không *nhiều hơn*? Có hai phản đề tự nhiên.

Thứ nhất, "thêm một ô `slice` chứ?" — vì cắt một đoạn con từ `Bytes` *có vẻ* phụ
thuộc sở hữu (cắt một hằng số ra một hằng số; cắt một vùng độc quyền không thể ra
một vùng độc quyền khác, vì thành hai kẻ cùng sở hữu một vùng). Nhưng `slice` **viết
lại được bằng `clone`**: cắt một đoạn con chẳng qua là "nhân bản cái handle (để
`clone` lo đúng phần sở hữu), rồi thu con trỏ với độ dài về đúng đoạn cần". `clone`
đã biết cách nhân bản đúng cho cả ba loại rồi. Một ô nào *dựng lại được từ ô khác*
thì không xứng đáng có mặt.

Thứ hai, "gộp hết thành một ô, trả về một cái enum cho biết loại, rồi tự phân nhánh
chứ?" — tức là một hàm `kind()` trả về loại, rồi `clone` và `drop` tự `match`. Chạy
được, nhưng bạn phải điều phối *hai lần*: một cú gọi hàm gián tiếp (gọi `kind`),
*rồi* một cú rẽ nhánh (`match`). Trong khi cả sức hấp dẫn của con trỏ hàm là *gọi nó
chính là điều phối rồi* — một bước. Chưa kể cách gộp-enum này *đóng*: thêm một cách
sở hữu thứ tư nghĩa là phải sửa *mọi* chỗ `match` trong codebase; còn với vtable,
bạn chỉ thêm một bảng `static` mới, không đụng chỗ nào khác.

Gom lại: đúng hai ô, vì đúng hai thao tác phụ thuộc sở hữu và không dựng lại được từ
nhau. Ít hơn thì mất khả năng; nhiều hơn thì hoặc thừa (dựng lại được) hoặc chậm
(điều phối hai lần). Quy tắc mang đi: một ô xứng đáng có mặt khi và chỉ khi nó phụ
thuộc vào trạng thái ẩn *và* không dựng lại được từ các ô khác.

(Ghi chú thực tế cho ai tò mò: thư viện `bytes` thật có tới năm ô, không phải hai.
Ba ô thêm đều là *tối ưu thuần túy* — mỗi cái né một cú copy đo được. Ví dụ, "biến
thành `Vec`" trên một buffer đang được sở hữu độc quyền thì có thể trao thẳng vùng
nhớ đi thay vì copy — nhưng chỉ khi hỏi được "tôi có đang độc quyền không?", mà chỉ
vtable mới biết. Mỗi ô thêm đều có thể dựng lại từ `clone` với `drop`; chúng tồn tại
vì ai đó đã *đo* được một cú copy đáng né. Mỗi ô thêm trả giá bằng một cú gọi gián
tiếp, nên mỗi cái phải tự *kiếm chỗ đứng* bằng một phép benchmark. Bản học này làm
hai ô — đó là bộ tối thiểu đúng đắn.)

## Cái đi kèm vtable: một trường dữ liệu phụ

Vtable nói *cách* clone/drop. Nhưng "cách" thường cần kèm *một mẩu dữ liệu*. Hàm
giải phóng của vùng độc quyền cần biết vùng nhớ dài bao nhiêu để trả lại đúng ngần
ấy. Hàm giảm counter của vùng chia sẻ cần biết cái counter nằm ở địa chỉ nào. Hàm
của hằng số thì chẳng cần gì.

Mẩu dữ liệu đó lấy ở đâu ra? Ta thêm một trường nữa, tạm đặt tên là `data`. Nó phải
mang được *ba loại thông tin khác nhau* tùy vào vtable:

- với hằng số, `data` không dùng tới;
- với vùng độc quyền, `data` chứa độ dài vùng nhớ đã cấp — một *con số*, chứ không
  phải địa chỉ;
- với vùng chia sẻ, `data` chứa *địa chỉ* của cái counter — một con trỏ thật.

Khi thì một con số, khi thì một con trỏ. Không có kiểu Rust "tử tế" nào diễn tả nổi
điều đó. Nên ta khai báo nó bằng kiểu vô-định-hình nhất có thể: **`*mut ()`**. Đây là
một *con trỏ thô* (raw pointer) trỏ tới `()` — kiểu "rỗng" của Rust; nói cách khác nó
là `void*` của C: chỉ đúng **8 byte** (trên máy 64-bit), không mang ý nghĩa gì cho
tới khi có kẻ diễn giải. Đừng đọc `data` là "một con trỏ"; hãy đọc nó là "8 byte, ý
nghĩa để dành". Với vùng độc quyền, ta nhét thẳng con số độ dài vào 8 byte đó (một số
nguyên đội lốt con trỏ); với vùng chia sẻ, 8 byte đó là một địa chỉ thật.

`data` và `vtable` luôn đi thành cặp: `data` là 8 byte tự thân vô nghĩa, còn `vtable`
là thứ *duy nhất* biết lần này 8 byte đó nghĩa là gì. Đây là lý do — bạn sẽ thấy khi
ngồi code — mọi hàm trong vtable đều nhận `data` làm tham số đầu tiên: ta trao 8 byte
vô nghĩa cho đúng cái hàm biết cách đọc chúng.

## Tổng kết: design đang ở đâu

Gộp lại, sau Phần 2 cái `Bytes` của ta trông thế này — lần đầu tiên với kiểu thật,
không còn placeholder:

```rust
struct Bytes {
    ptr:    NonNull<u8>,      // "byte nào": trỏ tới đầu dãy byte
    len:    usize,            // "byte nào": dài bao nhiêu
    data:   *mut (),          // "ai sở hữu": 8 byte, ý nghĩa do vtable quy định
    vtable: &'static Vtable,  // "ai sở hữu": dùng bộ clone/drop nào
}

struct Vtable {
    clone: /* con trỏ hàm */,   // clone loại này thì chạy đoạn này
    drop:  /* con trỏ hàm */,   // drop  loại này thì chạy đoạn này
}
```

Và ba giá trị `data`/`vtable` khả dĩ:

| vtable          | `data` chứa gì            | `drop` làm gì  |
|-----------------|---------------------------|----------------|
| `STATIC_VTABLE` | (không dùng)              | không làm gì   |
| `OWNED_VTABLE`  | độ dài vùng nhớ (con số)  | giải phóng     |
| `SHARED_VTABLE` | địa chỉ counter (con trỏ) | giảm counter   |

Giữ cái bảng này trong đầu; ở Phần 5 nó sẽ đổi đúng *một* dòng — kiểu của trường
`data` — và ta sẽ thấy vì sao `*mut ()` là chưa đủ. (Lý do nằm ở `clone`, chưa lộ ra
bây giờ.)

## Đã có gì, và Phần 3 giải tiếp gì

Đến đây ta đã giải xong nửa đầu câu hỏi của Phần 1: một kiểu `Bytes` duy nhất, mang
ba cách dọn khác nhau, chọn đúng cách tùy từng giá trị lúc chạy, nhờ một trường
`vtable` trỏ tới một trong ba bảng dựng sẵn.

Nhưng còn nửa sau, và nó quan trọng không kém: *đọc byte phải vẫn rẻ y như
`Arc<[u8]>`.* Ở mục về "hai ô" ta đã thoáng thấy: các thao tác đọc *không* nằm trong
vtable. Nhưng chỉ "không nằm trong vtable" thì chưa đủ. Còn phải sắp xếp các trường
của struct sao cho việc đọc *tuyệt đối* không đụng tới `data` hay `vtable`, kể cả một
cú rẽ nhánh. Vì sao cách sắp xếp đó thắng một cái `enum`, và vì sao nó khiến hai
trường vừa thêm vào trở nên *miễn phí* trên đường-nóng — là nội dung Phần 3.

Phần 3 cũng giới thiệu một cách nghĩ sẽ làm xương sống cho hai phần khó nhất còn lại:
mọi cách sở hữu bộ nhớ, rút gọn tận cùng, chỉ là một câu hỏi — *vùng nhớ này bị giải
phóng đúng mấy lần?*

---

*Tiếp theo: [Phần 3 — Tách "byte nào" khỏi "ai sở hữu"](03_split_and_counting.md) ·
[Mục lục](00_index.md)*

*English: [`../en/02_vtable.md`](../en/02_vtable.md)*
