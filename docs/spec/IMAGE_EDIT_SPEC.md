# Quyết định 06 — Thay ảnh tự động, xem trước và xuất EPUB

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 11.

Ngày ghi nhận: **25/10/2026**.

Trạng thái: **ghi nhận thiết kế đã thống nhất đến thời điểm hiện tại**. Tài liệu mô tả hành vi cần triển khai, không khẳng định Go hoặc Rust đã có đầy đủ chức năng này. Đây là đặc tả sản phẩm; không thay đổi mã ứng dụng, command hoặc schema workspace trong lần ghi nhận này.

Tài liệu tiếp nối [xử lý EPUB đầu vào](EPUB_INPUT_SPEC.md), [giai đoạn Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md) và [rubric Review](REVIEW_RUBRIC_SPEC.md).

Cập nhật ngày **29/10/2026**: phần chữ và cơ sở bố cục của preview/export theo [đặc tả EPUB đầu ra](EPUB_OUTPUT_SPEC.md), mặc định tiếng Việt đọc ngang; cách thể hiện ruby và các nội dung chữ được giải quyết trong Dịch–Review.

Phạm vi là xử lý ảnh của một tập EPUB sau khi hoàn tất phần chữ: dùng ảnh người dùng đã chỉnh/dịch trong thư mục `edit/` để tự thay tài nguyên tương ứng, xem trước bản có chữ và ảnh đã chọn, rồi xuất EPUB. Không yêu cầu người dùng nhập và duyệt từng ảnh bằng biểu mẫu.

## 1. Các quyết định chính

| Nội dung | Quyết định |
| --- | --- |
| Ảnh gốc | Tách ra `images/source/` ngay khi import EPUB, giữ nguyên nội dung gốc |
| Ảnh người dùng chỉnh | Đặt trong `images/edit/`, khớp đường dẫn tương đối và tên file của ảnh nguồn |
| Chọn ảnh để xuất | Có ảnh thay hợp lệ thì dùng ảnh thay; không có thì dùng ảnh gốc |
| Thời điểm áp dụng | Khi chuẩn bị bản xem trước hoặc chuẩn bị xuất EPUB |
| Cách xác nhận | Đặt file tương ứng vào `edit/` chính là quyết định thay ảnh; không duyệt thủ công từng ảnh |
| Ảnh dùng nhiều chỗ | Thay tại mọi vị trí tham chiếu đến cùng tài nguyên ảnh trong tập |
| File thừa trong `edit/` | Liệt kê là không được sử dụng; không tự thêm vào sách |
| Ảnh thay hỏng | Báo đúng file và chặn xuất; không âm thầm quay về ảnh gốc |
| Xem trước | Tùy chọn, hiển thị ảnh đã thay trong nội dung chương đã dịch |
| TUI | Quản lý tác vụ và mở trang xem thử bằng trình duyệt; chưa cần web UI cho toàn ứng dụng |
| Nhất quán | Preview và EPUB cuối dùng cùng một bộ nội dung/ảnh đã chuẩn bị |
| Bảo toàn | Giữ EPUB nguồn và ảnh gốc; thay thế diễn ra trong bản chuẩn bị xuất và EPUB đầu ra |
| Định dạng và file xuất | Chuẩn hóa đầu ra EPUB 3; mỗi lần xuất tạo file phiên bản mới theo đặc tả đầu ra, giữ bản cũ |

Thiết kế này dùng trạng thái suy ra từ thư mục và kết quả kiểm tra. Không cần quy trình bắt buộc Chưa xử lý → Chọn ảnh → Duyệt ảnh cho từng tài nguyên.

## 2. Vị trí trong luồng làm việc

```text
Import EPUB
  ├── Nhận diện chương, block và mapping
  └── Tách ảnh gốc vào images/source/, giữ vị trí tham chiếu
        ↓
Dịch và Review phần chữ
        ↓
Người dùng chỉnh ảnh bằng công cụ riêng, đặt kết quả vào images/edit/
        ↓
Chuẩn bị bản xuất: ghép chữ đã duyệt và chọn ảnh tự động
        ├── Xem trước → Có thể chỉnh tiếp và nạp lại
        └── Xuất trực tiếp nếu không cần xem trước
        ↓
Kiểm tra bản xuất và tạo EPUB đầu ra
```

Ảnh có sẵn sau import nên người dùng có thể chuẩn bị ảnh chỉnh trước khi phần chữ hoàn tất. Bước hoàn thiện bản xuất theo luồng đã chốt diễn ra sau khi các chương cần dịch đã qua Review hoặc có quyết định ngoại lệ được ghi nhận.

Các điều kiện duyệt phần chữ tiếp tục theo đặc tả Review; điều kiện xuất toàn tập theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md). Giai đoạn đầu chỉ xuất EPUB hoàn chỉnh, không xuất bản dở dang. Việc ảnh hợp lệ không tự làm phần chữ đạt, và việc phần chữ đạt không tự chứng minh chữ bên trong ảnh đã dịch đúng.

## 3. Thư mục và mapping ảnh

### 3.1. Cây thư mục

```text
vol01_translation/
├── chapters/
│   └── <chapter-id>.md
└── images/
    ├── source/
    │   ├── anh01.png
    │   ├── anh02.jpg
    │   └── anh03.png
    └── edit/
        ├── anh01.png
        └── anh03.png
```

Các tên file chỉ minh họa. `source/` chứa ảnh nguyên bản được tách từ snapshot EPUB; `edit/` chứa kết quả chỉnh của người dùng, có thể trống hoặc chưa có file nào. Tên thư mục chốt cho thiết kế này là `edit/`.

Kết quả ví dụ: `anh01.png` và `anh03.png` dùng bản trong `edit/`; `anh02.jpg` dùng bản trong `source/`.

### 3.2. Dữ liệu cần giữ khi import

- Liên kết mỗi file trong `source/` với tài nguyên ảnh trong snapshot EPUB.
- Đường dẫn tương đối ổn định trong thư mục ảnh nguồn.
- Hash nguồn, định dạng và thông tin kích thước phục vụ kiểm tra.
- Các vị trí tham chiếu: tập, chương, vị trí inline/trang minh họa hoặc vai trò bìa nếu có.
- Quan hệ với các tài nguyên khác cần bảo toàn khi dựng sách.

Một tài nguyên có thể xuất hiện nhiều lần. Danh sách vị trí dùng để xem ảnh nằm ở đâu và để giữ đúng tham chiếu, không buộc người dùng tự gán ID khi thay ảnh.

Không làm phẳng thư mục theo cách khiến hai ảnh cùng tên ghi đè nhau. Đường dẫn xuất ra phải phân biệt được tài nguyên và truy ngược được về nguồn. Mở lại cùng snapshot không được tự ghi đè các ảnh người dùng đã đặt trong `edit/`.

### 3.3. Bảo toàn nguồn

Không chỉnh trực tiếp ảnh trong `source/` hoặc EPUB nguồn để thực hiện thay thế. Người dùng lấy bản sao để chỉnh hoặc đặt ảnh đã dịch sẵn vào `edit/`.

Nếu `source/` bị thay đổi ngoài ý muốn so với snapshot, báo chênh lệch và xử lý nguồn trước. Không tự hiểu đó là yêu cầu thay ảnh hoặc âm thầm lấy nội dung bị sửa làm ảnh gốc mới.

Ảnh gốc, ảnh chỉnh, bản chuẩn bị xuất, preview và EPUB đầu ra là dữ liệu workspace, tiếp tục nằm ngoài Git theo ranh giới dự án.

## 4. Quy tắc chọn ảnh tự động

### 4.1. Khớp theo đường dẫn tương đối

Với mỗi ảnh nguồn có đường dẫn tương đối `p`, tìm đúng `images/edit/p`:

```text
Nếu có file edit tương ứng và file hợp lệ:
    Dùng ảnh edit trong bản xuất.
Nếu không có:
    Dùng ảnh gốc.
Nếu có nhưng không hợp lệ:
    Báo lỗi file đó, chưa xuất EPUB.
```

Khớp gồm tên file và phần mở rộng. Quy ước so khớp chính xác cách viết tên giúp kết quả nhất quán giữa các môi trường; không tự ghép theo tên gần giống hoặc chỉ theo phần tên bỏ đuôi.

Ví dụ khi có thư mục con:

```text
source/ch01/anh01.png  ←→  edit/ch01/anh01.png
source/ch02/anh01.png  ←→  edit/ch02/anh01.png
```

`edit/anh01.png` không tự thay hai file trên. `edit/anh01.jpg` không tự thay `source/anh01.png`. File không khớp được đưa vào danh sách không sử dụng để người dùng kiểm tra tên/đường dẫn.

### 4.2. Bảng kết quả

| Tình trạng | Kết quả |
| --- | --- |
| Chỉ có ảnh nguồn | Dùng ảnh gốc |
| Có ảnh edit khớp và hợp lệ | Tự dùng ảnh edit |
| Không có thư mục edit hoặc thư mục trống | Dùng toàn bộ ảnh gốc; không chặn xuất vì chưa chỉnh ảnh |
| File edit không khớp tài nguyên nguồn | Liệt kê là không dùng; không tự thêm trang hoặc tài nguyên đó vào sách |
| File edit khớp nhưng hỏng/không đọc được | Báo lỗi, chặn xuất; không bỏ qua âm thầm |
| Xóa hoặc chuyển file edit ra khỏi vị trí khớp | Lần chuẩn bị mới trở về dùng ảnh gốc |
| Một ảnh nguồn được tham chiếu nhiều lần | Mọi tham chiếu dùng cùng ảnh đã chọn |

Phạm vi thay thế chốt ở cấp tài nguyên trong tập. Không có lựa chọn thay riêng một lần xuất hiện trong luồng đơn giản này. Nếu cần hành vi đó về sau, phải có quyết định thiết kế riêng, không suy ra từ tên file.

## 5. Kiểm tra ảnh trước khi dựng bản xuất

### 5.1. Kiểm tra kỹ thuật

- File đọc được và nội dung là ảnh hợp lệ theo định dạng được hỗ trợ.
- Định dạng thực phù hợp với tên file và thông tin tài nguyên sẽ dùng khi xuất; đổi đuôi file không được coi là chuyển đổi ảnh.
- Có thể lấy thông tin kích thước và dựng ảnh vào vị trí nguồn tương ứng.
- Mapping nguồn và các vị trí tham chiếu còn hợp lệ với snapshot đang dùng.
- File được chọn có thể đọc ổn định để tạo bộ dữ liệu chuẩn bị xuất.

File khớp nhưng lỗi phải được chỉ rõ bằng đường dẫn và lý do. Người dùng có thể sửa file hoặc chuyển nó ra khỏi vị trí khớp để dùng lại ảnh gốc ở lần chuẩn bị tiếp theo. Tool không tự thực hiện lựa chọn thay người dùng.

### 5.2. Kích thước, tỷ lệ và hình thức

Ảnh có kích thước khác nguồn không mặc nhiên bị loại. Nếu tỷ lệ hoặc độ trong suốt thay đổi có thể ảnh hưởng bố cục, hiển thị thông tin đó trong danh sách/xem trước.

Giữ vị trí, thứ tự và quan hệ bố cục của ảnh với nội dung; phần chữ được điều chỉnh sang tiếng Việt đọc ngang theo đặc tả đầu ra. Không tự cắt, kéo giãn hoặc sửa bitmap để ép vừa mà không có yêu cầu. Việc hiển thị vẫn có thể co ảnh theo bố cục; cần xem trước để nhận biết chữ nhỏ, ảnh lệch hoặc khoảng trống bất thường.

Các thông tin cần xem như khác tỷ lệ không tạo một bước duyệt bắt buộc cho từng ảnh. Chỉ lỗi khiến file/tài nguyên/bản xuất không hợp lệ mới chặn xuất theo kiểm tra kỹ thuật.

### 5.3. Giới hạn của kiểm tra nội dung ảnh

Luồng này nhận ảnh do người dùng đã chỉnh/dịch. Không bắt buộc OCR, gọi model dịch ảnh, vẽ lại ảnh hoặc chấm chất lượng phần chữ trong ảnh. Xem trước giúp người dùng tự kiểm tra nội dung và hình thức.

Caption, alt text hoặc văn bản XHTML nằm ngoài bitmap thuộc phần chữ, được xử lý theo lựa chọn nội dung ở [đặc tả đầu vào](EPUB_INPUT_SPEC.md). Chế độ toàn bộ phần chữ phải dịch các phần này; chế độ chỉ truyện dịch phần liên quan đến truyện và giữ nguyên phần chỉ thuộc trang phụ. Chỉ chữ nằm trong bitmap được tách sang luồng ảnh, thay bitmap không tự dịch văn bản bên ngoài ảnh.

## 6. Chuẩn bị một bộ dữ liệu chung cho preview và export

Core chuẩn bị một bản gồm nội dung chữ, cấu trúc và ảnh đã chọn trước khi mở preview hoặc đóng gói EPUB. Bản này cần xác định được:

- Snapshot EPUB nguồn và mapping đang dùng.
- Lựa chọn toàn bộ phần chữ/chỉ truyện và danh sách phần chữ giữ nguyên theo nguồn nếu có.
- Revision phần chữ đã được chấp nhận, cùng quyết định ngoại lệ nếu có.
- Bản dịch tiêu đề/mục lục/chú thích và kết quả thể hiện ruby cùng quyết định còn hiệu lực theo đặc tả đầu ra.
- Thông tin sách đầu ra đã lưu cho tập: tên sách/tập, tác giả, ngôn ngữ và người dịch/nhóm dịch nếu có.
- Danh sách ảnh nguồn và kết quả chọn source/edit cho từng tài nguyên.
- Tài nguyên ảnh gốc cần cho thao tác so sánh trong preview, phân biệt với lựa chọn ảnh dùng trong EPUB xuất.
- Đường dẫn, hash và nội dung ảnh đã chốt cho lần chuẩn bị.
- Các tham chiếu, thông tin bìa và bố cục cần giữ.
- Cấu hình xuất có ảnh hưởng đến nội dung hoặc bố cục.
- Đầu ra EPUB 3 và cấu hình dựng/chuyển cấu trúc tương ứng, kể cả khi snapshot nguồn là EPUB 2.
- Kết quả kiểm tra và danh sách file edit không được sử dụng.

Ảnh và nội dung được chốt trong bản chuẩn bị, không để preview đọc một file rồi export lấy phiên bản khác từ thư mục đang được chỉnh. Bản chuẩn bị có thể dùng dữ liệu bất biến hoặc bản sao trong vùng làm việc; vị trí lưu và schema cụ thể được quyết định khi triển khai workspace mới theo đặc tả dữ liệu.

Core lưu đủ thông tin để biết bản xuất dùng ảnh nào. Không yêu cầu người dùng tự ghi manifest hoặc điền ID để vận hành luồng này.

### 6.1. Khi dữ liệu dùng để xuất thay đổi

Nếu thêm, sửa, xóa hoặc đổi tên file ảnh ảnh hưởng đến lựa chọn, bản chuẩn bị/preview trước đó trở thành cũ. Việc phát hiện phải xét cả danh sách file khớp và nội dung, không chỉ kiểm tra những ảnh edit đã được chọn trước đó.

Ví dụ: preview trước dùng `source/anh02.jpg`; sau đó người dùng thêm `edit/anh02.jpg`. Bản xem trước phải được coi là cũ dù tất cả file edit đã chọn ở lần trước vẫn không đổi.

Thay đổi revision phần chữ, nguồn, mapping, thông tin sách đầu ra hoặc cấu hình bố cục liên quan cũng làm mất hiệu lực bản chuẩn bị cũ. Sửa metadata thuần túy chỉ cần cập nhật và kiểm tra bộ preview/export; không tự yêu cầu dịch lại phần chữ không đổi.

Đổi lựa chọn nội dung toàn bộ phần chữ/chỉ truyện cũng cần tính lại điều kiện và chuẩn bị lại; preview phải cho thấy đúng phần được dịch hoặc giữ nguyên của bản xuất, không trộn hai phạm vi.

### 6.2. Nạp lại và xuất trực tiếp

- **Nạp lại bản xem trước:** quét lại source/edit, kiểm tra, tạo bản chuẩn bị mới và cập nhật nội dung đang xem.
- **Xuất sau khi xem trước:** dùng đúng bộ đã xem nếu còn hiện hành. Nếu đã thay đổi, hiển thị trạng thái bản xem trước cũ và yêu cầu nạp lại trước khi tiếp tục xuất từ luồng đó; không âm thầm xuất nội dung khác dưới danh nghĩa bản vừa xem.
- **Xuất trực tiếp:** tự chuẩn bị và kiểm tra bộ mới từ dữ liệu hiện tại, rồi xuất; không buộc người dùng mở preview hoặc duyệt từng ảnh.

Sau khi một tác vụ xuất đã chốt bộ dữ liệu hợp lệ, nó đóng gói đúng bộ đó. Thay đổi file bên ngoài trong lúc đóng gói không được làm EPUB trộn nhiều phiên bản; thay đổi mới thuộc lần chuẩn bị tiếp theo. Nếu file đang ghi hoặc chưa thể đọc ổn định trước khi chốt, báo chưa thể chuẩn bị file đó thay vì coi kết quả đọc dở là hợp lệ.

## 7. Xem trước bản xuất trong nội dung chương

### 7.1. Nội dung hiển thị

Xem trước phải hiển thị ảnh thực sự đã chọn cho bản xuất, đặt tại vị trí của nó giữa phần chữ đã dịch. Không chỉ hiển thị một thư viện ảnh rời hoặc mặc định vẫn dùng ảnh gốc.

```text
TẬP 01 — CHƯƠNG 4
Ảnh đang dùng: edit/anh01.png
Vị trí sử dụng: Chương 4, giữa b032 và b033

… Đoạn văn đã dịch trước ảnh …

┌───────────────────────────────┐
│                               │
│     ẢNH ĐÃ ĐƯỢC THAY THẾ      │
│                               │
└───────────────────────────────┘

… Đoạn văn đã dịch sau ảnh …

[Ảnh trước] [Ảnh tiếp] [So với ảnh gốc]
```

Các chức năng cần có:

- Di chuyển giữa các ảnh và mở đúng chương/vị trí đang xem.
- Nhìn rõ ảnh đang dùng đến từ source hay edit.
- So sánh với ảnh gốc khi cần; thao tác xem ảnh gốc không thay đổi lựa chọn xuất.
- Xem thông tin khác kích thước/tỷ lệ và các vị trí dùng cùng tài nguyên.
- Nhận biết bản xem trước đã cũ và nạp lại sau khi chỉnh ảnh.

Preview dùng cùng bộ nội dung/ảnh và cơ sở bố cục với bản xuất. Cách ngắt trang hoặc chi tiết hiển thị có thể khác giữa trình duyệt và các ứng dụng đọc EPUB; không cam kết số trang hay bố cục từng pixel giống nhau trên mọi trình đọc.

Phần chữ Việt trong preview đọc ngang từ trái sang phải, dùng tên chương, chú thích, nhấn mạnh và cách thể hiện ruby đã xử lý trong Dịch–Review. Không giữ dàn chữ Nhật ở preview rồi áp một bố cục Việt khác khi xuất.

### 7.2. TUI và trình duyệt

TUI quản lý việc chuẩn bị, trạng thái, lỗi, nạp lại và lệnh xuất. Thao tác **Xem trước bản xuất** mở trang xem thử cục bộ bằng trình duyệt. Trang có điều hướng/so sánh phục vụ xem; không cần biến toàn bộ ứng dụng thành web UI hoặc thêm trình chỉnh ảnh bên trong tool.

Các nút và thông tin phục vụ preview là giao diện của tool, không trở thành nội dung chương trong EPUB đầu ra. Cách mở trang và bố trí chính xác các nút được chốt khi triển khai giao diện.

Cập nhật ngày **30/10/2026**, lựa chọn **11A** theo [đặc tả key và preview](SECURITY_SPEC.md): trang chỉ phục vụ cục bộ trên máy đang chạy, có quyền truy cập riêng theo phiên; không mở toàn bộ workspace hoặc chia sẻ qua mạng. Chỉ phục vụ tài nguyên của bản chuẩn bị và giao diện tin cậy của tool, gồm ảnh gốc được phép dùng cho thao tác so sánh. Không chạy script/thuộc tính chủ động từ EPUB hoặc tự tải tài nguyên mạng qua XHTML/CSS/tài nguyên nhúng. Thiếu/sai quyền phiên hoặc đường dẫn ngoài danh sách tài nguyên phải bị từ chối.

Mã phiên không dùng API key và không được đưa vào EPUB xuất. Việc kiểm tra tài nguyên diễn ra trên bản chuẩn bị, giữ snapshot nguồn và cùng nội dung/bố cục giữa preview/export; không tự bỏ phần cần thiết rồi báo bản xuất đầy đủ. TUI mở trang và thao tác bằng bàn phím theo lựa chọn **12A** trong [đặc tả TUI/CLI](TUI_CLI_SPEC.md); màn Ảnh và xuất cùng các thao tác bàn phím theo mục 8 của đặc tả TUI/CLI.

### 7.3. Tổng quan trước khi xuất

Ví dụ TUI:

```text
Tập 01 — Chuẩn bị xuất
Phần chữ: Đã hoàn tất theo quyết định Review

Tổng tài nguyên ảnh: 12
Dùng ảnh gốc: 9
Dùng ảnh thay thế: 3
Ảnh lỗi: 0
File edit không sử dụng: 1

[Mở thư mục edit] [Xem danh sách ảnh]
[Xem trước bản xuất] [Nạp lại] [Xuất EPUB]
```

Số ảnh tính theo tài nguyên; số vị trí sử dụng có thể nhiều hơn và được hiển thị riêng khi xem chi tiết. File edit không khớp được liệt kê để người dùng tự kiểm tra, không tạo yêu cầu xác nhận bắt buộc và không được gói vào sách như tài nguyên mới.

Màn này còn hiển thị Sẵn sàng xuất, Còn việc cần xử lý hoặc Sẵn sàng xuất với ngoại lệ theo đặc tả đầu ra. Liệt kê việc chặn và đường dẫn mở đúng chương/đoạn/file; ngoại lệ đã được người dùng giữ phải thấy rõ lý do và phạm vi. Chưa thay ảnh hoặc còn S0 không tự làm xuất bị chặn, nhưng ảnh edit khớp bị hỏng vẫn phải sửa hoặc được người dùng chuyển ra khỏi vị trí khớp.

Mục **Thông tin sách** cho xem/sửa tên sách, tác giả và người dịch/nhóm dịch tùy chọn; ngôn ngữ bản xuất là Tiếng Việt. Tool điền sẵn và dùng lại lựa chọn đã lưu theo từng tập, chỉ rõ tên sách/tác giả còn thiếu cần bổ sung trước khi xuất theo đặc tả đầu ra.

## 8. Ghép và xuất EPUB

1. Kiểm tra điều kiện xuất toàn tập theo đặc tả đầu ra: đủ nội dung và Review hiện hành, quyết định bắt buộc đã giải quyết, ngoại lệ hợp lệ nếu có, cùng hiệu lực bộ dữ liệu chuẩn bị.
2. Lấy cấu trúc sách và mapping từ snapshot nguồn, ghép đúng revision bản dịch.
3. Dùng ảnh source/edit đã chọn tại đúng tài nguyên và mọi vị trí tham chiếu tương ứng.
4. Giữ thứ tự chương, ranh giới nội dung, ảnh inline, trang minh họa, chú thích và liên kết.
5. Cập nhật thông tin tài nguyên cần thiết, gồm tham chiếu bìa khi thay ảnh; áp dụng bố cục Việt đọc ngang và định dạng đã chốt, điều chỉnh CSS/XHTML liên quan và giữ các tài nguyên/tham chiếu còn cần dùng.
6. Dựng/chuẩn hóa cấu trúc đầu ra thành EPUB 3 theo đặc tả xuất; đóng gói và kiểm tra cấu trúc, tài nguyên, readback và EPUBCheck.
7. Công bố file phiên bản mới cùng hồ sơ xuất và chỉ báo hoàn tất khi file đã tạo, kiểm tra thành công.

Không thay trực tiếp nội dung của EPUB nguồn hoặc file trong `source/`. Theo quyết định ngày **30/10/2026** trong [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md), mỗi lần xuất mới tạo file phiên bản mới, ví dụ `Ten-bo_Tap-01_VI_r002.epub`, và giữ các bản trước dù lần mới thành công hay thất bại. Không ghi đè tên đã tồn tại. Thư mục đầu ra theo cấu hình workspace hiện có (`Output/` theo bố trí đã mô tả); tài liệu này không tự thêm command hay thay tên khóa cấu hình.

Ghép/xuất không gọi model để dịch thêm hoặc quyết định ý nghĩa ruby. Nếu phát hiện phần chữ/quyết định còn thiếu, chỉ ra vị trí cần xử lý trong Dịch–Review; không tự viết nội dung trong bản chuẩn bị. Mở trình sửa XML không phải bước bắt buộc của quy trình thay ảnh đã chốt.

Nếu ảnh không thay, giữ nội dung ảnh gốc. Nếu dùng ảnh edit, lấy đúng nội dung đã được chốt cho lần chuẩn bị, không tự nén lại, chỉnh màu hoặc sửa chữ trong bitmap. Các biến đổi ảnh khác cần yêu cầu thiết kế riêng.

Thay bitmap không tự làm mất hiệu lực Review phần chữ nếu chữ và các dependency liên quan không đổi. Bản chuẩn bị và EPUB xuất cần được tạo lại để phản ánh lựa chọn ảnh mới.

## 9. Lịch sử và khôi phục dữ liệu ảnh

Lưu được nguồn, revision chữ, danh sách chọn ảnh, hash ảnh và kết quả kiểm tra cho mỗi lần chuẩn bị/xuất. Đóng rồi mở tool không làm mất file người dùng đã chỉnh.

Hồ sơ xuất liên kết số phiên bản file, đường dẫn và EPUB 3 với đúng bộ dữ liệu chuẩn bị. Số phiên bản file xuất khác revision bản dịch hoặc generation; thay ảnh rồi xuất lại tạo file mới, không tự sửa nội dung trong EPUB đã xuất trước đó.

| Tình huống | Hành vi |
| --- | --- |
| Chưa từng có ảnh edit | Dùng nguồn, vẫn có thể xem trước/xuất |
| Chuẩn bị bị gián đoạn | Kiểm tra dữ liệu còn hợp lệ rồi tiếp tục hoặc dựng lại; không báo hoàn tất bản chưa đủ |
| Preview đã dựng nhưng ảnh thay đổi | Đánh dấu cũ, nạp lại để tạo bộ mới |
| Xuất thất bại | Giữ source/edit và bản xuất trước; ghi rõ bước/file lỗi |
| Người dùng muốn trở về ảnh gốc | Chuyển/xóa file edit tương ứng khỏi vị trí khớp, rồi chuẩn bị lại |
| Workspace mới được hỗ trợ thiếu ảnh source do import chưa hoàn tất | Phục hồi/tách lại từ snapshot nguồn hợp lệ và kiểm tra mapping trước preview/export; không bắt dịch lại chữ |
| Workspace được hỗ trợ chưa có edit | Xem như chưa có ảnh thay; không tự thay schema chỉ vì thiếu thư mục |
| Đổi snapshot EPUB nguồn | Kiểm tra lại mapping và phạm vi ảnh edit; không dùng ảnh cũ cho nguồn khác chỉ vì trùng tên |

Thiết kế dùng đường dẫn `images/source/` và `images/edit/` đã chốt trong đặc tả. Thời điểm tách source là bước import. Theo quyết định ngày **30/10/2026** tại [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), không cần giữ/chuyển workspace test cũ sang bản Rust. Workspace mới vẫn phải giữ file chỉnh của người dùng, mapping nguồn và lịch sử xuất; schema và cách nâng cấp về sau cần được mô tả rõ.

Không xây luồng chuyển các thư mục chỉnh ảnh của workspace test cũ trong phạm vi lần chuyển sang Rust. Nếu người dùng chủ động đưa một ảnh đã chỉnh vào `images/edit/` của workspace mới, vẫn kiểm tra đường dẫn, file và mapping như ảnh thay mới; không tự suy ra dữ liệu ảnh/lịch sử cũ đã được tiếp nhận.

## 10. Tiêu chí nghiệm thu khi triển khai

Dùng EPUB và ảnh mẫu tự tạo, offline và deterministic cho kiểm thử tự động. Luồng thay ảnh và chuẩn bị preview không cần gọi model trả phí.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Import EPUB có ảnh | Tách ảnh gốc vào source, giữ nguyên nội dung và mapping vị trí |
| Hai ảnh trùng tên ở thư mục khác nhau | Không ghi đè; edit chỉ khớp đúng đường dẫn tương đối |
| Có file edit đúng tên/đuôi/đường dẫn, hợp lệ | Tự dùng ảnh đó, không yêu cầu duyệt từng ảnh |
| Không có edit hoặc edit trống | Dùng toàn bộ ảnh gốc, không chặn chỉ vì chưa chỉnh ảnh |
| File edit sai đường dẫn hoặc khác đuôi | Báo không sử dụng, không tự đoán thay ảnh nào |
| File edit khớp nhưng hỏng | Chặn xuất và chỉ đúng file; không âm thầm dùng nguồn |
| File edit đổi tỷ lệ nhưng vẫn hợp lệ | Hiển thị thông tin để xem trước, không tự cắt hoặc sửa bitmap |
| Một tài nguyên xuất hiện nhiều chỗ | Thay đồng nhất mọi tham chiếu, giữ đúng vị trí |
| Thay ảnh bìa | Ảnh và các thông tin tham chiếu bìa nhất quán trong đầu ra |
| File edit thừa | Hiện trong danh sách không dùng, không thêm vào sách |
| Preview một ảnh đã thay | Hiện ảnh edit trong chương cùng chữ đã dịch trước/sau |
| Preview và xuất từ nguồn chữ Nhật dọc | Cùng dùng chữ Việt đọc ngang, nội dung/ruby đã được xử lý, đúng vị trí ảnh và nhấn mạnh |
| So với ảnh gốc | Chỉ đổi chế độ xem, không đổi lựa chọn xuất |
| Sửa/xóa/thêm file edit sau preview | Phát hiện bản cũ, kể cả ảnh trước đó đang dùng source |
| Xuất sau preview còn hiện hành | Dùng đúng bộ nội dung và hash ảnh đã xem |
| Xuất trực tiếp | Chuẩn bị, kiểm tra và xuất, không buộc mở preview |
| File đổi trong lúc đóng gói | Không trộn phiên bản vào bản đã chốt; lần sau dùng dữ liệu mới |
| Bản chuẩn bị hoặc ảnh lỗi | Không báo EPUB hoàn tất; giữ dữ liệu người dùng và bản xuất trước |
| Xuất thành công | Kiểm tra cấu trúc/tài nguyên/readback/EPUBCheck theo workflow dự án |
| Thay ảnh rồi xuất lại | Tạo file EPUB 3 phiên bản mới, giữ các bản trước và truy được ảnh/hash của từng bản |
| Chỉ thay ảnh, phần chữ không đổi | Không tự chạy lại toàn bộ dịch/Review chữ |
| Mở lại workspace mới được hỗ trợ sau gián đoạn | Giữ ảnh chỉnh, phục hồi ảnh source từ snapshot nếu cần và không yêu cầu dịch lại phần chữ còn hợp lệ |

Định dạng, kiểm tra ảnh và cố định nội dung đã chốt tại mục 11. Phiên preview theo [đặc tả bảo mật](SECURITY_SPEC.md), mục 8. Bộ chuẩn bị và hồ sơ xuất theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md), mục 12. Ảnh khớp và hợp lệ tự thay theo đường dẫn, preview vẫn tùy chọn.

## 11. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [30](DESIGN_DECISIONS_36.md#decision-30) của bản 36 mục áp dụng cho đặc tả này.

### 11.1. Kiểm tra và lựa chọn ảnh thay thế — mục 30

Nhận thay ảnh **JPEG, PNG, GIF, WebP và SVG tĩnh an toàn**. Khớp chính xác đường dẫn tương đối, chữ hoa/thường và đuôi file trong `images/edit/`. Kiểm tra loại nội dung thật, đọc được cấu trúc, kích thước, bộ nhớ giải mã và tham chiếu. Với SVG, chỉ nhận nội dung tĩnh thuộc tập cho phép, chặn script và tài nguyên ngoài. Ảnh thay khớp nhưng lỗi chặn xuất và chỉ đúng file cần sửa. Hiện chênh lệch kích thước/tỷ lệ trong preview. File hợp lệ giữ nguyên byte người dùng đã chuẩn bị.

*Mặc định thử nghiệm* cho mỗi ảnh là 64 MiB, mỗi chiều bitmap tối đa 16.384 pixel, tối đa 40 triệu pixel/khung và 100 triệu pixel cộng dồn khi kiểm tra ảnh động. Các giới hạn này điều chỉnh được theo cấu hình. Trước preview/export, đọc phiên file ổn định vào kho theo hash, kiểm tra lại và ghim cả ảnh được chọn lẫn ảnh gốc cần cho so sánh. File đổi trong lúc đọc thì chuẩn bị lại tài nguyên đó. [Đặc tả ảnh](IMAGE_EDIT_SPEC.md)

### 11.2. Liên kết và yêu cầu triển khai

Mở và kết thúc phiên preview theo [đặc tả bảo mật](SECURITY_SPEC.md), mục 8. Manifest bộ chuẩn bị, số bản xuất và phục hồi theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md), mục 12. Cùng hash ảnh được dùng cho preview và EPUB cuối.
