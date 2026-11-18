# Quyết định 13 — API key, kết nối và bảo vệ preview cục bộ

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 8.

Ngày ghi nhận: **30/10/2026**.

Trạng thái: **đã chốt lựa chọn 11A trong trao đổi**. Đây là yêu cầu sản phẩm cần triển khai, chưa phải xác nhận hành vi hoặc mức bảo vệ hiện có của bản Go/Rust. Ưu tiên Windows theo phạm vi dự án; adapter kho thông tin xác thực được hiện thực theo 11A. Hợp đồng phiên preview đã chốt tại mục 8, quyền truy cập và xử lý nội dung phải được kiểm chứng khi triển khai.

Tài liệu liên kết [chiến lược model](MODEL_STRATEGY_SPEC.md), [TUI và CLI](TUI_CLI_SPEC.md), [dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md), [EPUB đầu vào](EPUB_INPUT_SPEC.md), [ảnh và preview](IMAGE_EDIT_SPEC.md), [đầu ra EPUB](EPUB_OUTPUT_SPEC.md) và [Research](RESEARCH_STAGE_SPEC.md).

## 1. Các quyết định đã chốt

| Nội dung | Quyết định |
| --- | --- |
| Nhập API key | Nhập một lần khi thiết lập kết nối, các lần sau chọn kết nối đã lưu. |
| Lưu API key | Dùng kho thông tin xác thực của hệ điều hành; workspace chỉ giữ tham chiếu kết nối. |
| Hiển thị và ghi nhận | Che key khi nhập/hiển thị; không đưa giá trị key vào log, lịch sử, prompt, bản dịch hoặc EPUB đầu ra. |
| CLI | Cho phép lấy key từ biến môi trường theo cấu hình kết nối, không bắt nhập lại bằng TUI. |
| Phạm vi preview | Chỉ phục vụ trên máy đang chạy, có mã truy cập riêng cho phiên; không mở cho máy khác trong mạng. |
| Nội dung nguồn | Không thực thi script của EPUB, không tự tải tài nguyên ngoài Internet khi xem trước. |
| Tài nguyên được phục vụ | Chỉ giao diện tin cậy của tool và tài nguyên của bản chuẩn bị xem trước được phép truy cập. |

TUI/CLI vẫn là nơi thao tác chính. Preview phục vụ xem ảnh trong chương, so sánh và điều hướng theo đặc tả ảnh; quyết định này không mở rộng thành web UI quản lý ứng dụng.

## 2. Thiết lập và dùng lại kết nối

Người dùng đặt tên kết nối, chọn cấu hình provider/model liên quan và cung cấp key. Phần thông tin không bí mật được lưu trong cấu hình kết nối; giá trị key được giao cho kho thông tin xác thực của hệ điều hành. Workspace và hồ sơ tác vụ tham chiếu kết nối thay vì sao chép key.

```text
Người dùng thiết lập kết nối
  → nhập key trong ô được che
  → lưu key vào kho thông tin xác thực của hệ điều hành
  → lưu tham chiếu kết nối
  → chọn kết nối/model cho Dịch, Review hoặc Research
  → khi có request được phép chạy: lấy key để xác thực với kết nối đã chọn
```

Một kết nối có thể được các vai trò dùng chung theo đặc tả model. Không bắt nhập key lại ở mỗi chương, lần Review hoặc lần mở tác vụ. Việc lưu key thành công chỉ xác nhận đã lưu được; chưa chứng minh key còn hợp lệ hay tài khoản được phép gọi model. Mở màn cấu hình hoặc chọn kết nối không tự gọi model trả phí.

Khi không đọc/lưu được kho thông tin xác thực, báo đúng kết nối và phần cần xử lý, chưa gửi request phụ thuộc. Không báo đã lưu khi còn lỗi, không tự chuyển sang ghi key dạng văn bản trong workspace, `.env` hoặc log. Người dùng có thể sửa kết nối hoặc chủ động dùng nguồn biến môi trường được hỗ trợ.

Thay key/kết nối là thao tác cấu hình rõ ràng của người dùng. Áp dụng cho request chưa gửi và giữ lịch sử công việc, ngân sách, bộ đếm; không đổi tài khoản/model tự động khi xác thực thất bại. Hồ sơ thay đổi chỉ ghi định danh kết nối và sự kiện, không ghi giá trị key cũ/mới.

## 3. API key từ biến môi trường cho CLI

CLI có thể dùng giá trị biến môi trường được chỉ định cho kết nối. Cấu hình được phép ghi tên biến hoặc nguồn thông tin xác thực; không ghi lại giá trị biến vào file, hồ sơ tác vụ hoặc kết quả CLI.

Nguồn key có hiệu lực phải xác định rõ. Một biến môi trường tình cờ tồn tại không tự ghi đè kết nối/kho key đã chọn; nếu người dùng chủ động chọn nguồn môi trường cho lần chạy thì sử dụng nguồn đó. Thiếu hoặc rỗng phải báo cấu hình thiếu, không tự thử một key/kết nối khác.

Key từ môi trường không được tự chép vào kho hệ điều hành hoặc `.env`. Sau khi mở lại, tool lấy lại key từ nguồn đã cấu hình; việc thiếu key chỉ chặn request cần xác thực, không xóa kết quả dịch hoặc tiến độ đã lưu. CLI dùng `connection set <name> --key-env <tên-biến>`, lưu tên biến do người dùng chọn theo [hợp đồng CLI](TUI_CLI_SPEC.md).

`.env` không còn là nơi tool mặc định tạo/lưu key trong workspace mới. Nếu người dùng có file riêng bên ngoài luồng này, file đó vẫn phải nằm ngoài Git; không tự quét và tiếp nhận nội dung file chỉ vì nó tồn tại.

## 4. Che thông tin bí mật

| Vị trí | Yêu cầu |
| --- | --- |
| Ô nhập và màn kết nối | Che giá trị key, dùng tên kết nối và trạng thái đã cấu hình để nhận diện. |
| Log, chẩn đoán và lỗi provider | Loại thông tin xác thực khỏi nội dung trước khi lưu/hiển thị, kể cả khi lỗi phản chiếu dữ liệu request. |
| Lịch sử, usage và hồ sơ tác vụ | Lưu vai trò/model/kết nối và kết quả, không lưu key hoặc bản sao toàn bộ môi trường tiến trình. |
| Prompt Dịch/Review/Research | Không đưa key vào nội dung gửi model; key chỉ dùng ở lớp xác thực kết nối tương ứng. |
| Preview và EPUB xuất | Không đưa key vào trang, tài nguyên, metadata, đường dẫn hoặc file đã ghép. |
| Kết quả CLI dạng chữ/JSON | Cùng quy tắc che thông tin; chế độ chẩn đoán không mở quyền in key. |

Thông tin xác thực chỉ được dùng với kết nối đã cấu hình. Nội dung EPUB, skill, kết quả model hoặc trang Research không được dùng để lấy key hay đổi đích gửi thông tin xác thực. Dữ liệu bí mật và dữ liệu workspace tiếp tục nằm ngoài Git.

## 5. Phiên preview chỉ chạy cục bộ

### 5.1. Phạm vi truy cập

Khi người dùng mở **Xem trước bản xuất**, tool tạo hoặc dùng phiên preview hợp lệ gắn với bộ dữ liệu đã chuẩn bị. Dịch vụ chỉ lắng nghe trên địa chỉ cục bộ của chính máy chạy; không lắng nghe trên toàn bộ giao diện mạng hoặc mở chia sẻ qua LAN/Internet.

Phiên có mã truy cập riêng do tool tạo, không dùng lại API key. Yêu cầu trang và tài nguyên phải thuộc phiên được phép; thiếu/sai mã hoặc phiên đã đóng thì không được trả nội dung sách. Mã cần khó đoán, giới hạn theo phiên và hết hiệu lực khi phiên kết thúc; cách truyền/lưu phía trình duyệt và thời hạn chi tiết được thiết kế khi triển khai.

Tool tự mở trình duyệt với quyền truy cập phiên, không bắt người dùng nhập lại mã mỗi ảnh. Không đưa mã phiên vào log công khai, EPUB xuất hoặc yêu cầu gửi tới website bên ngoài. Chỉ chạy cục bộ không thay cho kiểm tra quyền truy cập phiên.

### 5.2. Giới hạn file và thao tác

Preview phục vụ từ danh sách tài nguyên được phép của bản chuẩn bị, không mở toàn bộ workspace như một thư mục web. Danh sách gồm phần chữ/bố cục, ảnh đã chọn và ảnh gốc phục vụ thao tác so sánh; ảnh gốc dùng để so sánh không tự thay lựa chọn ảnh trong bản xuất.

Không phục vụ file cấu hình bí mật, `.env`, lịch sử nội bộ hoặc file tùy ý trên máy. Đường dẫn tài nguyên phải được đối chiếu sau khi chuẩn hóa/giải quyết đường dẫn; tên file, đường dẫn tuyệt đối, đường dẫn đi ra ngoài hoặc liên kết filesystem không được giúp đọc ra ngoài danh sách được phép.

Trang xem cho phép điều hướng, so ảnh gốc và nhận biết bản xem đã cũ. Nó không cung cấp quyền gọi model, đổi cấu hình, duyệt thuật ngữ hoặc ghi file tùy ý. Nạp lại/chuẩn bị và xuất tuân theo luồng TUI/CLI, không bỏ điều kiện Review hoặc làm phát sinh một bản xuất chỉ vì mở trang.

## 6. XHTML, CSS và tài nguyên từ EPUB

Nội dung EPUB là dữ liệu để đọc, không phải mã ứng dụng được tin cậy. Bản xem trước không được thực thi script, thuộc tính xử lý sự kiện, URL có khả năng chạy mã hoặc nội dung nhúng chủ động từ sách. Các điều khiển do chính tool cung cấp được xử lý riêng với nội dung nguồn; không vô hiệu hóa chức năng điều hướng chỉ vì sách không được chạy script.

XHTML, CSS, ảnh/font và tham chiếu tài nguyên phải được kiểm tra trước khi đưa vào bản chuẩn bị. Ngăn việc tự tải tài nguyên mạng, kể cả tham chiếu gián tiếp trong CSS hoặc tài nguyên lồng nhau. Liên kết chương/chú thích và ảnh cục bộ hợp lệ vẫn cần hoạt động; không xóa nội dung chữ hoặc phá mapping để bỏ qua bước kiểm tra.

Nếu tài nguyên cần thiết không thể chuẩn bị an toàn trong phạm vi hỗ trợ, chỉ rõ vị trí/tài nguyên và lý do để xử lý. Không âm thầm bỏ ảnh hoặc phần nội dung rồi trình bày như bản xuất đầy đủ. EPUB cần script/tính năng ngoài phạm vi để đọc đúng tiếp tục theo giới hạn hỗ trợ trong đặc tả đầu vào.

Việc chuẩn hóa hoặc loại bỏ hành vi chủ động diễn ra trên bản chuẩn bị; snapshot EPUB nguồn vẫn là căn cứ bất biến. Preview và export dùng cùng nội dung, ảnh và cơ sở bố cục đã kiểm tra theo đặc tả ảnh/đầu ra. Cơ chế mã truy cập và điều khiển trang của tool không được đóng gói vào EPUB.

Quyền dùng Internet của Research là luồng riêng có tác vụ/model và bằng chứng theo đặc tả Research. Nó không cấp quyền cho một trang EPUB trong preview tự mở kết nối hoặc đọc dữ liệu cấu hình.

## 7. Tiêu chí nghiệm thu

Kiểm thử dùng key giả, kho thông tin xác thực giả lập, EPUB/tài nguyên tự tạo và dịch vụ cục bộ kiểm soát được; offline, deterministic, không gọi model trả phí hoặc đưa key thật vào CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Lưu kết nối và mở lại | Lấy được key qua kho hệ điều hành/tham chiếu, không có giá trị key trong workspace. |
| Kho key lỗi hoặc bị khóa | Báo cần xử lý, không gửi request hoặc tự ghi key dạng văn bản. |
| CLI dùng biến môi trường | Dùng đúng nguồn đã chọn, không in/sao chép giá trị hoặc yêu cầu mở TUI. |
| Nguồn key thiếu/sai | Không tự chọn key/kết nối/model khác. |
| Provider trả lỗi chứa thông tin xác thực | Log, màn hình và JSON không lộ key. |
| Mở/xem lịch sử hoặc preview | Không gọi AI hay lấy key đưa vào trang. |
| Yêu cầu preview từ ngoài phạm vi cục bộ hoặc không có quyền phiên | Không phục vụ nội dung sách. |
| Đường dẫn cố truy cập ra ngoài tài nguyên chuẩn bị | Từ chối, không đọc cấu hình hoặc file ngoài danh sách. |
| EPUB có script/sự kiện hoặc CSS trỏ ra mạng | Không chạy nội dung chủ động hoặc tự gửi yêu cầu mạng. |
| Preview có ảnh thay và so ảnh gốc | Cả hai được phục vụ đúng quyền, thao tác so sánh không đổi ảnh xuất. |
| Phiên kết thúc hoặc dữ liệu được chuẩn bị lại | Không dùng quyền phiên đã hết hiệu lực; dữ liệu xem vẫn tuân theo kiểm tra phiên bản/bản cũ. |
| Đóng gói EPUB | Không chứa key, mã truy cập phiên hoặc điều khiển riêng của trang preview. |

## 8. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [31](DESIGN_DECISIONS_36.md#decision-31) của bản 36 mục áp dụng cho đặc tả này.

### 8.1. Bảo mật và vận hành Preview cục bộ — mục 31

Mở phiên trên loopback, cổng tạm và danh sách tài nguyên theo bản chuẩn bị. Sinh bí mật ngẫu nhiên 256 bit dùng một lần, truyền ở phần fragment của URL mở trình duyệt. Trang điều khiển tin cậy đổi mã đó qua POST cùng origin, xóa fragment khỏi thanh địa chỉ và giữ quyền phiên trong bộ nhớ của trang. Tài nguyên sách chỉ được trang điều khiển lấy qua yêu cầu có quyền, theo ID nội bộ. Kiểm tra Host/origin hợp lệ, tránh ghi thông tin phiên vào log/referrer. Phiên kết thúc khi bấm đóng, tiến trình phục vụ đóng, hoặc sau **30 phút không hoạt động** theo mặc định thử nghiệm.

Dựng tài liệu xem cô lập với giao diện tin cậy. Parser chỉ cho phép các phần tử/thuộc tính/CSS cần cho sách, chặn script, sự kiện, form/iframe chủ động, tham chiếu mạng, đường dẫn file tùy ý và nội dung lồng có khả năng thực thi. Tài nguyên theo ID mở từ bản bất biến đã kiểm tra. Khi ảnh/chữ/bố cục đổi, hiện **Bản xem trước đã cũ** và nút nạp lại. Bản xuất lấy cùng bộ dữ liệu đã kiểm tra. [Đặc tả bảo mật](SECURITY_SPEC.md)

### 8.2. Liên kết và yêu cầu triển khai

Cập nhật/xóa key và tham chiếu kết nối theo [chiến lược model](MODEL_STRATEGY_SPEC.md), mục 10. CLI chọn tên biến môi trường theo [hợp đồng CLI](TUI_CLI_SPEC.md), mục 8. Ảnh SVG tĩnh theo [đặc tả ảnh](IMAGE_EDIT_SPEC.md), mục 11. Adapter kho hệ điều hành, thư viện làm sạch XHTML/CSS, chính sách trình duyệt và che log được kiểm chứng bằng fixture khi triển khai.
