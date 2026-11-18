# Quyết định 14 — Tương tác TUI và phạm vi CLI

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 8.

Ngày ghi nhận: **30/10/2026**.

Trạng thái: **đã chốt lựa chọn 12A trong trao đổi**. Tài liệu mô tả hành vi cần triển khai, chưa xác nhận giao diện/lệnh hiện tại đã đáp ứng. Hợp đồng command, cờ, JSON, mã thoát, tám màn và các phím chuyên biệt đã chốt ngày **04/11/2026** tại mục 8.

Liên kết: [36 quyết định đã chốt](DESIGN_DECISIONS_36.md), [Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [Skills/duyệt nhiều thuật ngữ](SKILLS_GLOSSARY_SPEC.md), [model/ngân sách](MODEL_STRATEGY_SPEC.md), [điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), [ảnh và preview](IMAGE_EDIT_SPEC.md), [key và bảo vệ preview](SECURITY_SPEC.md).

## 1. Nguyên tắc tương tác

- TUI là giao diện chính, sử dụng được hoàn toàn bằng bàn phím. Chuột hỗ trợ thêm khi terminal cho phép, không có thao tác bắt buộc chỉ dùng chuột.
- CLI và TUI cùng dùng core, dữ liệu, điều kiện duyệt, ngân sách và cơ chế tiếp tục; không có một đường CLI bỏ qua quy tắc của TUI.
- Trình duyệt chỉ phục vụ preview đã chốt, chưa mở rộng thành web UI cho toàn ứng dụng.
- Chuyển màn, cuộn, đổi focus hoặc xem nguồn không tự gọi model hay thay trạng thái duyệt.
- Người dùng thấy rõ màn/vùng đang chọn, tập/chương/phạm vi và hành động của nút trước khi kích hoạt.

## 2. Kích thước terminal và bố cục

| Kích thước | Hành vi |
| --- | --- |
| Từ **80 cột × 24 dòng**, rộng dưới 120 cột | Đủ thao tác bằng bố cục hẹp: xem danh sách, chọn mục rồi mở chi tiết; thẻ nguyên tác/bản dịch/bằng chứng xếp dọc. |
| Từ **120 cột**, cao ít nhất 24 dòng | Màn phù hợp có thể đặt danh sách và chi tiết cạnh nhau; đây là khả năng bố cục, không bắt mọi màn đều chia đôi. |
| Dưới 80 cột hoặc 24 dòng | Báo kích thước chưa đủ và cách mở rộng cửa sổ; giữ trạng thái/nháp/tác vụ, không cố hiển thị các nút bị che hoặc làm mất dữ liệu. |

Đơn vị là ô hiển thị của terminal, không phải pixel. Nội dung Nhật, tiếng Việt có dấu và chuỗi dài phải được xuống dòng/cuộn phù hợp, không cắt hỏng ký tự hoặc sửa nội dung nguồn để vừa khung.

Bố cục rộng/hẹp giữ cùng chức năng và quyền quyết định. Danh sách, bộ lọc, vị trí chọn, vị trí xem và nội dung đang nhập được giữ khi resize. Màn hẹp vẫn mở được toàn bộ bằng chứng và phạm vi đề xuất; không bỏ thông tin cần duyệt để tiết kiệm chỗ.

Khi cửa sổ tạm nhỏ hơn mức hỗ trợ, việc resize tự nó không hủy tác vụ hoặc khởi động lại request. Tool tiếp tục theo trạng thái tác vụ đã có; sau khi mở rộng, khôi phục màn và dữ liệu đang thao tác.

## 3. Phím dùng chung

| Phím | Hành vi theo vùng đang chọn |
| --- | --- |
| Mũi tên | Di chuyển mục trong danh sách; trong vùng nhập thì di chuyển con trỏ theo chức năng nhập liệu. |
| `Tab` | Chuyển focus giữa các vùng/control có thể thao tác; focus phải nhìn thấy. |
| `Enter` | Mở mục hoặc kích hoạt nút đang focus; trong ô soạn nhiều dòng có hành vi nhập dòng, không tự xác nhận cả biểu mẫu. |
| `Space` | Đánh dấu/bỏ đánh dấu mục trong danh sách chọn nhiều; trong ô nhập vẫn là dấu cách. |
| `Esc` | Đóng lớp đang mở hoặc quay lại một cấp; không tự xóa nội dung chưa lưu, hủy tác vụ hay thoát toàn ứng dụng. |
| `F1` | Mở trợ giúp theo màn/ngữ cảnh hiện tại, có hướng dẫn phím và hành động đang khả dụng. |
| `PageUp` / `PageDown` | Cuộn vùng nội dung hiện tại theo một trang khi có phần vượt khung; không đổi route, focus hay mục chọn. Tại đầu/cuối hoặc khi nội dung vừa khung, phím không làm gì và không được quảng bá như hành động khả dụng. |

Thanh hướng dẫn thể hiện phím có hiệu lực trong màn/vùng hiện tại. Phím nhập liệu không kích hoạt một hành động ở màn phía sau. Khi quay lại, giữ vị trí và nội dung đang nhập. Editor dùng phím lưu theo mục 8.2 và giữ revision nền cho bản đang soạn.

Ngoài ô nhập dùng `/` tìm, `f` lọc, `e` mở editor, `p` tạm dừng và `r` tiếp tục khi thao tác có hiệu lực. Trong editor dùng `Ctrl+S` để lưu revision sửa tay. Control tương ứng đều dùng được bằng bàn phím. Quay lại màn trước giữ trạng thái tác vụ. Chi tiết tám màn và phạm vi phím ở mục 8.

## 4. Review và duyệt nhiều thuật ngữ

Màn Review giữ nguyên thẻ gồm nguyên tác, bản dịch, lỗi, mức ảnh hưởng, bằng chứng, giải thích, đề xuất sửa và phạm vi theo đặc tả Review. Chọn một vấn đề mở đúng phiên bản/đoạn liên quan; bố cục hẹp và rộng đều xem được đầy đủ.

Danh sách đề xuất thuật ngữ theo 7A cho phép chọn cách dịch và phạm vi từng mục, mở chi tiết/ngữ cảnh, đánh dấu bằng `Space`, rồi chuyển tới nút xác nhận. Nút phải thể hiện số mục đã chọn, ví dụ **Chốt 2 mục đã chọn**; bấm mở một dòng hoặc đánh dấu chưa phải thao tác duyệt.

Người dùng xác nhận đúng các mục/lựa chọn đang xét. Những mục không chọn hoặc xuất hiện thêm sau đó tiếp tục chờ; dữ liệu thay đổi/xung đột được kiểm tra theo đặc tả Skills. Chuột, nếu có, thực hiện cùng hành động và điều kiện; không có cách bấm nhanh bỏ qua bằng chứng hoặc quy tắc xác nhận.

Duyệt thuật ngữ không tự chấp nhận các thẻ lỗi hoặc kết luận chương đạt. Khi trở lại danh sách, giữ vị trí/bộ lọc và phản ánh đúng những quyết định đã lưu.

## 5. Tác vụ đang chạy và thao tác tạm dừng

Trong lúc có request, vùng trạng thái luôn thấy được vai trò, tên/ID model thực tế và bước hiện tại, kể cả khi cuộn hoặc chuyển màn. Hiển thị tiến độ nội dung, ngân sách/chi phí, thời gian đã chạy/thời hạn, đếm ngược retry hoặc lý do chờ theo đặc tả model. Cách chia dòng trên màn hẹp không được làm mất thông tin nhận diện model/bước đang chạy.

Phân biệt chưa dịch, chờ Review, cần sửa/xác minh, đạt và người dùng giữ ngoại lệ; trạng thái tạm dừng/lỗi vận hành hiển thị riêng với chất lượng. Số lần gọi hoàn tất không được dùng làm tỷ lệ chất lượng chương. Hoàn tất Dịch–Review EPUB và xuất EPUB thành công là hai mốc khác nhau.

Có control **Tạm dừng** rõ ràng và dùng được bằng bàn phím. Khi kích hoạt, ghi ý định dừng, ngăn gửi thêm request theo workflow và giữ tiến độ đã lưu; xử lý phản hồi đang chạy theo quy tắc đã chốt. Không hiển thị đã hủy xử lý/tính phí ở provider chỉ vì tool nhận lệnh dừng.

Trở lại màn trước hoặc resize không tự tiếp tục một tác vụ đã tạm dừng. Tiếp tục dùng lại bước/phạm vi, cấu hình, chi phí/bộ đếm và các quyết định đã lưu; không bắt chọn lại từng chương trong chế độ tự động. Không hỏi xác nhận thêm chỉ vì chuyển sang chương kế tiếp đã đủ điều kiện.

Theo **13A** chốt ngày **30/10/2026** trong [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), chỉ một tác vụ AI hoạt động, tác vụ khác xếp hàng. Màn tác vụ phân biệt Đang chạy, Chờ chạy, Chờ người dùng và Đã tạm dừng; xem hồ sơ B không đổi nhãn model đang chạy của A. Khi A tạm dừng/chờ xử lý, người dùng có thể chọn chạy B độc lập sau khi bàn giao quyền chạy hợp lệ. Xem nguồn, sửa ảnh hoặc duyệt kết quả đã có vẫn được phép theo quyền dữ liệu; yêu cầu AI mới phải chờ cùng hàng đợi.

TUI/CLI dùng chung quyền chạy, không tự khởi chạy hai tác vụ do bấm hai cửa sổ. Tiếp tục tác vụ A trong lúc B đang chạy đưa A về chờ quyền chạy, không tự ngắt B. Việc duyệt một mục hoặc đổi màn không tự tiếp tục tác vụ đã bị người dùng tạm dừng/hủy; giữ các quyết định và tiến độ khi trở lại.

## 6. Các nhóm thao tác CLI

| Nhóm | Phạm vi cần có |
| --- | --- |
| Workspace | Tạo/mở workspace mới, xem cấu hình/trạng thái theo định dạng được hỗ trợ. |
| Import EPUB | Đăng ký tập, import và xem cấu trúc/phạm vi nội dung. |
| Dịch | Chạy phần đã chọn hoặc tự động đến hết EPUB theo cấu hình và điểm chờ đã chốt. |
| Tiếp tục | Khôi phục tác vụ và bước đang chờ mà không làm mới chi phí/bộ đếm. |
| Review | Xem kết quả/bằng chứng và thực hiện các thao tác xử lý thuộc workflow. |
| Trạng thái/kiểm tra | Xem tiến độ, việc chặn, model/ngân sách và điều kiện tiếp tục/xuất. |
| Preview | Chuẩn bị/mở bản xem trước cục bộ theo cùng quyền phiên và dữ liệu với TUI. |
| Xuất EPUB | Kiểm tra điều kiện toàn tập, tạo EPUB 3 phiên bản mới theo đặc tả đầu ra. |
| Kết nối/cấu hình | Dùng kết nối đã lưu hoặc key từ nguồn môi trường được chọn, không đưa key vào kết quả. |
| Kết quả cho script | Có JSON thống nhất, `--json`, `--non-interactive` và mã thoát theo mục 8. |

Cú pháp đích đã chốt tại mục 8, làm hợp đồng cho bản Rust. CLI cũng không được tự duyệt thuật ngữ, tăng ngân sách/trần số lần gọi, đổi model hoặc bỏ điều kiện Review để hoàn tất một lệnh.

Lỗi cần nêu đúng tác vụ/phạm vi, lý do và hành động tiếp theo; không gọi trạng thái chờ duyệt là hoàn tất thành công hoặc in dữ liệu bí mật. Bảng lệnh, tham số, JSON, mã thoát và hành vi không tương tác đã chốt tại mục 8.

## 7. Tiêu chí nghiệm thu

Kiểm thử dùng dữ liệu/model giả lập, offline và deterministic; không gọi model trả phí trong CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Chạy ở 80×24 | Dùng được các thao tác đã chốt bằng bàn phím và mở đủ chi tiết/bằng chứng. |
| Mở rộng từ 119 lên 120 cột hoặc thu hẹp lại | Bố cục đổi phù hợp, giữ mục chọn, vị trí xem và nội dung đang nhập. |
| Thu xuống dưới mức hỗ trợ rồi mở rộng | Thông báo rõ, không mất nháp hoặc tự hủy/chạy lại tác vụ. |
| Nội dung Nhật/Việt hoặc dòng dài | Hiển thị/cuộn đúng, không cắt hỏng ký tự hoặc thay đổi nội dung lưu. |
| Dùng mũi tên/Tab/Enter/Esc/F1 | Đúng phạm vi focus và ngữ cảnh; không kích hoạt nút phía sau hoặc tự bỏ sửa tay. |
| Gõ Space/Enter trong ô soạn | Nhập nội dung, không tự đánh dấu/duyệt danh sách phía sau. |
| Chọn 2 trong nhiều thuật ngữ | Nút nêu đúng số mục; chỉ xác nhận các lựa chọn đã chọn khi được kích hoạt. |
| Không có hỗ trợ chuột | Vẫn thao tác được toàn bộ luồng yêu cầu. |
| Cuộn/chuyển màn khi gọi AI | Model/vai trò/bước đang chạy vẫn rõ; xem màn không tự tạo request mới. |
| Tạm dừng trong lúc gọi hoặc chờ retry | Giữ tiến độ, không gửi thêm lượt trái ý định, không tuyên bố provider đã hủy. |
| Cùng thao tác từ CLI và TUI | Cùng dữ liệu, kiểm tra và điểm chờ; không có đường tắt vượt quy tắc. |
| Mở hồ sơ B trong lúc A đang gọi model | Vẫn nhận diện đúng A/model đang hoạt động; B chỉ hiển thị Chờ chạy nếu đã được giao, không tự gọi model. |
| Chọn tiếp tục một tác vụ khi tác vụ khác đang chạy | Chờ quyền chạy theo 13A, giữ dữ liệu/ngân sách riêng, không tự ngắt tác vụ hiện tại. |
| CLI mở preview hoặc xuất | Đúng phiên cục bộ và điều kiện đầu ra, không mở toàn workspace hay ghi đè bản xuất. |

## 8. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [34](DESIGN_DECISIONS_36.md#decision-34), [35](DESIGN_DECISIONS_36.md#decision-35) của bản 36 mục áp dụng cho đặc tả này.

### 8.1. Hợp đồng lệnh CLI — mục 34

Dùng tên thực thi **`novel-translate-cli`**. Mở không có tham số vào TUI. Các lệnh nhận `--workspace <đường-dẫn>` khi cần workspace, `--json` cho script và `--non-interactive` khi cần trả kết quả mà không đợi nhập. Bảng dưới là cú pháp đã chốt để triển khai.

| Lệnh hoặc nhóm lệnh | Đích và tham số chính |
| --- | --- |
| `workspace create <path>`, `workspace open <path>` | Tạo bộ mới hoặc kiểm tra/mở workspace đang hỗ trợ. |
| `import <epub> --volume <id>`, `scope set --volume <id> --content all-text\|story` | Nhập nguồn, xem cây và lưu phạm vi nội dung của tập. |
| `translate --volume <id>` | Chọn `--chapter <id>` lặp được hoặc `--to-end`. Cho ghi đè lựa chọn nội dung và ngân sách của tác vụ bằng `--cost-limit <số> --currency <mã>` hoặc `--unlimited-cost`. |
| `review --volume <id> --chapter <id>`, `issues --task <id>` | Review đúng dữ liệu hiệu lực hoặc liệt kê lỗi/bằng chứng của tác vụ. |
| `terms list`, `terms confirm --file <json>` | File xác nhận liệt kê ID, phiên bản, cách dịch và phạm vi những mục đã chọn. Áp dụng quy tắc nhóm theo [Quyết định 15](DESIGN_DECISIONS_36.md#decision-15). |
| `status [--task <id>]`, `pause <task-id>`, `resume <task-id>` | Đọc trạng thái, tạm dừng hoặc tiếp tục đúng bước. |
| `check --volume <id>`, `preview --volume <id>`, `export --volume <id>` | Kiểm tra sẵn sàng, mở phiên preview hoặc xuất EPUB phiên bản mới. |
| `connection set <name> --key-env <tên-biến>`, `connection list`, `connection remove <name>` | Quản lý tham chiếu kết nối. Dùng kho khóa hệ điều hành khi nhập key trong TUI. |

JSON có `schema_version, command, state, task_id, scope, result, issues, next_action`. Dữ liệu máy đọc ở stdout, chẩn đoán đã che thông tin ở stderr. Mã thoát cho lệnh chạy công việc gồm **0** hoàn tất, **2** tham số/cấu hình sai, **3** lỗi vận hành, **4** chờ duyệt/xử lý xung đột, **5** chạm ngân sách/trần gọi, **6** đã xếp hàng/đang có tác vụ đó, **7** tạm dừng hoặc chưa hoàn tất bởi điều kiện khác. `status` trả 0 khi đọc trạng thái thành công, còn trạng thái công việc nằm trong kết quả. CLI không tương tác lưu checkpoint, trả ID và hành động tiếp theo, ví dụ xác nhận thuật ngữ rồi `resume <id>`. [Quy ước CLI/TUI](TUI_CLI_SPEC.md)

### 8.2. Chi tiết giao diện TUI — mục 35

**Cập nhật được người dùng duyệt 14/11/2026:** Sau khi tạo hoặc mở và vào workspace, **Tổng quan** và **Tập và chương** đều có cùng nút **＋ Nhập tập truyện**, khi trống lẫn có tập. Tab/Enter hoặc chuột khi terminal hỗ trợ kích hoạt nút. Trên Windows được hỗ trợ hiện hành, hộp chọn file gốc cho phép duyệt thư mục, chọn một `.epub` và Mở/OK để import cục bộ vào workspace gốc. Ứng dụng tự xác định/đăng ký Volume ID và truyền đường dẫn cùng ID vào core; người dùng không gõ path hay ID nội bộ trong luồng TUI thông thường. CLI `import <epub> --volume <id>` và hợp đồng core giữ nguyên.

Esc/hủy trở lại workspace với focus và dữ liệu đúng. File sai/đuôi sai hoặc chooser không khả dụng có chẩn đoán hữu ích, thử lại/hủy và không công bố nguồn dở dang. Thành công hiện cây và focus tập mới. Đổi workspace hoặc resize khi chooser/import đang chờ không được áp kết quả vào workspace sai. Kiểm đường dẫn Unicode/có khoảng trắng. Adapter/thư viện thuộc owner P02-D06/T12; không cam kết Linux native.


Giữ tám màn với nhiệm vụ chính sau đây.

| Màn | Thông tin và thao tác chính |
| --- | --- |
| Tổng quan | Bộ/tập, tiến độ, việc đang chờ và lối mở đúng vị trí. |
| Tập và chương | Cây sách, phân loại, điều chỉnh ranh giới, chọn chương hoặc tự động đến hết. |
| Tác vụ | Hàng đợi, model/bước thực tế, chi phí, thời gian, tạm dừng/tiếp tục. |
| Editor Nhật–Việt | Nguyên tác, bản Việt, revision nền, sửa/lưu và lịch sử. |
| Reviewer | Danh sách issue, thẻ bằng chứng, mức S/M, giải thích, mở editor đúng đoạn và Review lại revision mới. |
| Skills và ngữ cảnh | Mục chờ, các phương án, phạm vi, chọn nhiều và nút xác nhận số mục. |
| Ảnh và xuất | Ảnh đã chọn, lỗi, bản chuẩn bị, preview và file xuất. |
| Cấu hình, báo cáo và trợ giúp | Kết nối, model, ngân sách, cấu hình và hướng dẫn phím theo màn. |

Từ **80×24**, danh sách mở sang chi tiết theo chiều dọc. Từ **120 cột**, dùng hai vùng cho cây tập/chương, hàng đợi tác vụ, Editor, Reviewer, Skills và danh sách ảnh. Màn tổng quan/cấu hình bố trí theo nội dung. Mỗi màn nhớ mục chọn, bộ lọc, cuộn và nội dung đang soạn khi quay lại/resize, theo đúng bộ, tập và tài liệu đang xem.

Ngoài ô nhập, dùng `/` tìm, `f` lọc, `e` mở editor, `p` tạm dừng và `r` tiếp tục khi thao tác có hiệu lực. Trong editor, `Ctrl+S` lưu revision sửa tay và bảo vệ bản nền. Các nút tương ứng luôn dùng được qua bàn phím. Mũi tên, Tab, Enter, Space, Esc và F1 giữ hành vi đã chốt theo vùng focus. Trở lại từ editor giữ đúng thẻ Review.

### 8.3. Liên kết và yêu cầu triển khai

Kích thước từng control, màu, wrapping, xử lý tín hiệu terminal và mã kết nối core là công việc triển khai theo tám màn, quy tắc bàn phím, dữ liệu và quyền chạy đã chốt. Các thao tác bắt đầu/thoát có control dùng được bằng bàn phím và tuân theo lưu tiến độ.
