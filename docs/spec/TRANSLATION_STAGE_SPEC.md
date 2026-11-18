# Quyết định 02 — Giai đoạn Dịch: từ nhận lệnh đến lưu bản nháp

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 11.

Ngày ghi nhận: **25/10/2026**.

Cập nhật ngày **27/10/2026**: liên kết [mô hình dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md) và [Skills, quy ước riêng, duyệt thuật ngữ](SKILLS_GLOSSARY_SPEC.md). Thuật ngữ/quy ước mới phải được người dùng duyệt trước khi áp dụng vào bản nháp hiện hành.

Cập nhật ngày **28/10/2026** theo [chiến lược model](MODEL_STRATEGY_SPEC.md): không tự đổi model; tối đa 2 lần thử lại lỗi dịch vụ tạm thời và 2 lượt yêu cầu sửa cấu trúc, có báo lỗi và lưu tiến độ. Người dùng chọn đặt hạn mức chi phí tác vụ hoặc chạy không giới hạn chi phí; cả hai vẫn giữ các giới hạn xử lý lỗi/chất lượng.

Cập nhật ngày **29/10/2026** theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md): tiêu đề/mục lục/chú thích và cách thể hiện ruby được xử lý trong Dịch–Review; bước xuất chỉ ghép nội dung đã được chấp nhận.

Trạng thái: **ghi nhận thiết kế đã thống nhất đến thời điểm hiện tại**. Tài liệu mô tả hành vi cần triển khai trong core Rust; không khẳng định toàn bộ chức năng đã có trong bản Rust hoặc Go.

Tài liệu tiếp nối [đặc tả xử lý EPUB đầu vào](EPUB_INPUT_SPEC.md). Cách nhận diện chương, giữ cấu trúc sách, chia lượt và hai giới hạn dung lượng tiếp tục theo đặc tả đó. Mốc 3.000 token nguồn/lượt vẫn là mức thử nghiệm.

Phạm vi ở đây gồm đúng năm bước: **nhận lệnh Dịch → chuẩn bị dịch và ngữ cảnh → tạo bản dịch → kiểm tra cấu trúc → lưu bản nháp**. Đầu ra là bản nháp hợp lệ về cấu trúc, chờ Review. Rubric đánh giá chất lượng, vòng sửa theo Review và xuất EPUB được bàn ở các phần riêng.

Hợp đồng JSON Dịch đã chốt tại mục 11. Lưu request, phản hồi, revision và tiến độ theo [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), với một SQLite cho bộ và các file bất biến có hash. Workspace Rust mới bắt đầu schema 1 và có cơ chế nâng cấp, giữ sửa tay/lịch sử. Quyết định ngày **30/10/2026** bỏ yêu cầu chuyển workspace test cũ tiếp tục có hiệu lực.

## 1. Phân chia trách nhiệm

TUI tiếp nhận thao tác, gửi yêu cầu và hiển thị tiến độ. Core thực hiện nghiệp vụ; CLI gọi cùng core. Lớp kết nối provider chuyển request sang API phù hợp, còn lớp lưu trữ chịu trách nhiệm ghi trạng thái và bản dịch.

| Bước | Đầu vào | Đầu ra |
| --- | --- | --- |
| 1. Nhận lệnh Dịch | Workspace, tập, chương và ý định bắt đầu/tiếp tục | Hồ sơ tác vụ, phạm vi và tiến độ đã xác định |
| 2. Chuẩn bị dịch và ngữ cảnh | Hồ sơ tác vụ, nguồn, quy tắc và dữ liệu ngữ cảnh | Request cho một lượt, kèm phạm vi nguồn và dependency |
| 3. Tạo bản dịch | Request đã chuẩn bị | Phản hồi ứng viên và thông tin lần gọi model |
| 4. Kiểm tra cấu trúc | Phản hồi ứng viên và phạm vi đã giao | Kết quả hợp lệ hoặc danh sách lỗi cấu trúc |
| 5. Lưu bản nháp | Kết quả hợp lệ và phiên bản dữ liệu lúc chuẩn bị | Bản nháp đã lưu, revision và trạng thái chờ review |

Bước 1 thực hiện khi bắt đầu hoặc tiếp tục tác vụ. Các bước 2–5 áp dụng cho từng lượt dịch. Theo [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md) chốt ngày **28/10/2026**, khi một lượt trả về bản nháp đã lưu, bước tiếp theo là Review lượt đó; chỉ dịch lượt kế tiếp khi điều kiện cục bộ đã được giải quyết. Sau mọi lượt còn kiểm tra tổng thể chương trước khi chuyển chương. Không tạo nháp toàn chương hoặc cả nhóm chương rồi mới bắt đầu Review.

## 2. Bước 1 — Nhận lệnh Dịch

### 2.1. Ý nghĩa đơn giản

Bước này trả lời ba câu hỏi:

1. Người dùng muốn dịch tập/chương nào?
2. Dữ liệu và cấu hình đã đủ để chuẩn bị công việc chưa?
3. Trong phạm vi đó, phần nào đã có kết quả còn hợp lệ và phần nào còn thiếu?

Ví dụ người dùng chọn **Tập 01 → Chương 2**, rồi bấm **Dịch**. Tool ghi một “phiếu công việc”:

```text
Tác vụ: Dịch Chương 2
Tập: 01
Phạm vi: Chương 2
Đã có nháp hợp lệ: đoạn 001–020
Còn thiếu bản dịch: đoạn 021–060
Trạng thái: Đang chuẩn bị
```

Phiếu này ghi nhận việc cần làm để tránh tạo tác vụ trùng và có thể tiếp tục khi mở lại ứng dụng. Nó chưa quyết định đoạn 021–060 sẽ cần bao nhiêu request; việc đó thuộc bước chuẩn bị từng lượt.

Ngoài chọn một/nhóm chương, người dùng có thể chọn **Dịch tự động đến hết EPUB** ở cấp tập theo [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md). Bước nhận lệnh khi đó ghi chế độ tự động và phạm vi toàn tập, giữ phần đã hoàn tất còn phù hợp, lập hàng đợi phần còn thiếu. Tool tự chuyển chương sau khi đủ Review, không yêu cầu người dùng chọn/bấm Dịch lại ở từng chương.

“Đã có nháp” chưa đồng nghĩa “đã được review chấp nhận”. Hồ sơ phải giữ riêng hai trạng thái này. Phần đã có nháp có thể đang chờ Review, còn phần thiếu được ghi nhận để bộ điều phối xử lý đúng thứ tự.

### 2.2. Yêu cầu từ giao diện

Thông tin gửi cho core có thể được diễn đạt như sau:

```json
{
  "volume_id": "vol01",
  "chapter_ids": ["chapter-02"],
  "mode": "continue"
}
```

Workspace được xác định từ workspace đang mở. ID trong ví dụ là ký hiệu minh họa; khi chạy thật phải dùng ID từ dữ liệu import.

`continue` diễn đạt ý định dùng lại dữ liệu còn hợp lệ và tiếp tục phần chưa xong. Nếu chương chưa có bản dịch, cùng ý định này sẽ bắt đầu từ nội dung đầu tiên cần dịch. Dịch lại một bản đã có là ý định riêng, cần chỉ rõ phạm vi và bảo toàn lịch sử.

### 2.3. Các kiểm tra core phải thực hiện

| Kiểm tra | Hành vi khi chưa đạt |
| --- | --- |
| Workspace, tập và chương tồn tại | Báo đúng phạm vi không tìm thấy; không tạo request model |
| Chương đã import, có snapshot và mapping đọc được | Chỉ rõ cần import hoặc xử lý nguồn |
| Snapshot, generation và dữ liệu nguồn phù hợp | Chuyển sang xử lý revision/mapping, không áp dữ liệu cũ vào nguồn khác |
| Provider/model được cấu hình | Báo cấu hình còn thiếu |
| Thông tin xác thực cần thiết đã được thiết lập | Báo thiếu cấu hình xác thực; chưa kết luận key thực sự dùng được trước khi provider trả lời |
| Các giới hạn tác vụ đang áp dụng cho phép tiếp tục | Hiển thị giới hạn đã chạm hoặc cấu hình cần điều chỉnh |
| Có lựa chọn ngân sách rõ ràng, hợp lệ | Tác vụ mới dùng mặc định/kế thừa 8A hoặc lựa chọn riêng; tác vụ tiếp tục dùng ngân sách đã lưu, chưa có thì cần người dùng chọn trước request |
| Không có tác vụ trùng đang hoạt động | Nếu trùng, mở tiến độ tác vụ hiện có thay vì tạo lần dịch thứ hai |
| Có quyền chạy AI theo 13A trước khi gửi | Nếu tác vụ khác đang hoạt động, ghi yêu cầu vào hàng đợi và hiển thị Chờ chạy; chưa gửi model |
| Có quyền ghi theo cơ chế một writer của workspace | Chờ hoặc báo tác vụ đang giữ quyền ghi; không để hai tác vụ ghi cạnh tranh |
| Bản thảo trên đĩa không có sửa tay chưa được ghi nhận | Yêu cầu xử lý theo cơ chế đồng bộ/xung đột, giữ nguyên bản sửa |

Các kiểm tra ở bước này là kiểm tra cục bộ. Lỗi xác thực thực tế hoặc lỗi dịch vụ được xử lý khi gọi provider ở bước 3.

### 2.4. Đọc tiến độ đã lưu

Core phân loại nội dung trong chương:

| Tình trạng | Cách ghi nhận vào công việc |
| --- | --- |
| Chưa có bản dịch | Cần tạo bản dịch |
| Có bản nháp hợp lệ, dependency còn phù hợp | Dùng lại bản nháp; giữ trạng thái chờ review nếu chưa duyệt |
| Đã được duyệt và còn hợp lệ | Giữ kết quả đã chấp nhận |
| Nguồn hoặc dependency đã thay đổi | Đánh dấu cần xử lý lại theo chính sách revision; không âm thầm dùng bản cũ |
| Có sửa tay hoặc xung đột | Giữ dữ liệu và nêu rõ việc cần giải quyết |
| Có phản hồi đã lưu nhưng chưa kiểm tra | Có thể tiếp tục kiểm tra cục bộ trước khi cân nhắc gọi lại model |

### 2.5. Hồ sơ tác vụ và tiến độ giao diện

Hồ sơ cần biểu diễn ID tác vụ, chế độ chạy theo phần đã chọn hoặc tự động đến hết EPUB, tập/phạm vi và hàng đợi chương, ý định thực hiện, thời điểm tạo, snapshot nguồn, revision hiện tại, cấu hình model và phiên bản quy tắc liên quan. Thông tin xác thực không được chép vào hồ sơ/log tác vụ.

Theo lựa chọn 8A chốt ngày **30/10/2026** trong [chiến lược model](MODEL_STRATEGY_SPEC.md), tác vụ mới xác định và hiển thị ngân sách từ lựa chọn riêng của tác vụ, cấu hình riêng của bộ hoặc mặc định tool, rồi lưu cấu hình thực tế và nguồn kế thừa vào hồ sơ. Tiếp tục tác vụ giữ ngân sách, chi phí/dự phòng và bộ đếm đã lưu; thay mặc định không tự thay ngân sách của tác vụ này.

Theo cập nhật ngày **30/10/2026** trong [đặc tả đầu vào](EPUB_INPUT_SPEC.md), lưu thêm lựa chọn nội dung toàn bộ phần chữ hoặc chỉ truyện cùng phân loại phần cần dịch/giữ nguyên. Lựa chọn nội dung, chương xử lý trong tác vụ và chế độ chi phí là các lựa chọn riêng; chạy tự động không thay đổi phạm vi nội dung đã chọn.

Core ghi nhận công việc trước khi gọi model. Các công việc tốn thời gian chạy nền; TUI vẫn điều hướng được và hiển thị trạng thái **Đang chuẩn bị dịch Chương 2**.

Bấm Dịch lần nữa với cùng phạm vi đang chạy hoặc đã xếp hàng phải được nhận diện ở core, bên cạnh việc vô hiệu hóa hoặc đổi hành vi nút trên TUI. CLI cũng cần được bảo vệ bởi cùng cơ chế tác vụ và writer lock.

Theo **13A** chốt ngày **30/10/2026** trong [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), chỉ một tác vụ AI hoạt động. Yêu cầu khác được ghi nhận để chờ; khi được chạy phải kiểm tra lại nguồn, sửa tay, quy ước, ngân sách và quyền ghi. Người dùng có thể chọn tác vụ độc lập khác sau khi tác vụ trước đã tạm dừng/chờ và bàn giao hợp lệ; không tạo tác vụ mới cho chương sau để vượt bước Review hoặc quyết định đang chờ.

**Đầu ra bước 1:** một hồ sơ tác vụ đã xác định được phạm vi và tiến độ, hoặc lỗi chuẩn bị có nguyên nhân rõ. Chưa gửi nội dung cho AI ở bước này.

## 3. Bước 2 — Chuẩn bị dịch và ngữ cảnh

### 3.1. Xác định phần nguồn được giao

Core lấy các block cần dịch theo thứ tự của chương và lập một lượt theo hai giới hạn trong đặc tả EPUB. Một lượt chỉ được giao phạm vi nguồn xác định; những đoạn dùng để tham khảo được đánh dấu riêng.

Nếu một block phải chia thành các phần con, request cần giữ ID và mapping phạm vi con đủ để kiểm tra và ghép lại. ID của block nguồn không thay đổi chỉ vì điều chỉnh kích thước lượt dịch.

Ví dụ:

```text
Lượt dịch 2
├── Phạm vi được giao: b021–b040
├── Tham khảo nguồn: b019–b020
├── Bản dịch đã duyệt của b019–b020, nếu có
├── Quy tắc dịch và quy ước bộ truyện
├── Glossary liên quan đến b021–b040
├── Ngữ cảnh nhân vật/cảnh có căn cứ
└── Yêu cầu trả kết quả cho đúng các ID được giao
```

Các số đoạn là minh họa. Kế hoạch chia thực tế phải được tính trên nội dung và model cụ thể.

### 3.2. Tạo gói ngữ cảnh

| Thành phần | Nguồn dữ liệu | Quy tắc lựa chọn |
| --- | --- | --- |
| Nguyên văn cần dịch | Snapshot EPUB | Lấy đầy đủ nội dung thuộc các ID/phạm vi đã giao |
| Ruby và dấu định dạng | Mapping import | Giữ cả chữ chính, chữ nhỏ, quan hệ và ngữ cảnh; áp dụng cách thể hiện đầu ra đã được xác định/duyệt |
| Hướng dẫn dịch chung | Thư viện skill chung, các file đang bật | Chọn hướng dẫn và phần tra cứu liên quan trong phạm vi đang xử lý |
| Quy ước riêng của truyện | File skill riêng và quyết định đã duyệt của bộ | Áp dụng đúng phạm vi; quy ước riêng ưu tiên hơn mặc định chung khi phù hợp |
| Glossary | Thuật ngữ đã xác nhận | Chọn tên, biến thể và thuật ngữ liên quan đến nội dung hiện tại |
| Xưng hô, quan hệ | Dữ liệu ngữ cảnh đã xác nhận | Lọc theo vị trí xuất bản và thời điểm cảnh, kể cả hồi tưởng |
| Nguồn gần trước | Snapshot nguồn | Lấy đoạn liên quan giúp nối mạch trong ngân sách cho phép |
| Bản dịch gần trước | Revision đã được duyệt, còn hợp lệ | Dùng làm tham khảo về cách diễn đạt và tính liên tục |
| Tài liệu tham khảo | Nguồn đã lưu/cấu hình và kết quả Research | Chọn phần có căn cứ, đúng mốc/phạm vi; giữ nguồn, trạng thái kiểm chứng và tách khỏi chỉ dẫn điều khiển |
| Cấu trúc trả về | Hợp đồng đầu ra của ứng dụng | Nêu rõ ID cần trả, trường bắt buộc và quy tắc marker |

Không tự dùng bản nháp chưa duyệt làm dữ liệu ngữ cảnh đã xác nhận. Những đề xuất do model vừa tạo cũng chưa được đưa thành glossary hoặc quan hệ chính thức.

Khi người dùng sửa file skill đã bật, áp dụng cơ chế 2B trong [đặc tả Skills](SKILLS_GLOSSARY_SPEC.md): tự nhận thay đổi rõ ràng, hợp lệ và dùng từ lượt tiếp theo. Gói ngữ cảnh đã chuẩn bị nhưng chưa gửi cần kiểm tra lại phiên bản liên quan; request đã gửi giữ phiên bản cũ và kết quả trả về phải được xét hiệu lực. Chỗ mơ hồ/xung đột chuyển phần phụ thuộc sang chờ xử lý; cập nhật do người dùng cung cấp không cho AI tự áp dụng đề xuất mới chưa duyệt.

Nếu gặp thuật ngữ/quy ước mới cần người dùng chốt, lưu nguyên ngữ, ngữ cảnh, các phương án và lý do theo đặc tả Skills. Phần phụ thuộc chuyển sang **Chờ duyệt thuật ngữ/quy ước**, chưa tự dùng một phương án trong bản nháp. Có thể xử lý phần độc lập đã đủ điều kiện; không bắt mọi từ thông thường phải thành một mục glossary.

Nếu đây là phần đầu truyện hoặc chưa có dữ liệu trước đó, sử dụng những thông tin hiện có. Những quan hệ chưa biết được giữ là chưa xác định; không tự tạo người nói/người nghe hoặc cách xưng hô chỉ để điền đủ trường.

Khi cần tìm thêm thông tin, phần Dịch nêu câu hỏi, vị trí/ngữ cảnh nguồn, quy ước hiện hành và mốc liên quan để [Research](RESEARCH_STAGE_SPEC.md) xử lý. Research được tra Internet và trả bằng chứng; phần Dịch không tự mở một luồng tìm web riêng. Kết quả chưa được kiểm chứng hoặc chứa dữ kiện tương lai không tự thành ngữ cảnh xác nhận; lựa chọn thuật ngữ mới vẫn chờ người dùng. Nếu phát hiện thiếu căn cứ sau khi nhận phản hồi model, giữ ứng viên và tiếp tục từ bước cần xử lý theo cùng quy tắc.

Tiêu đề chương/part, nhãn mục lục, chú thích, caption và alt text thuộc phạm vi đã chọn cũng đi qua Dịch–Review, dùng mapping để không bỏ sót hoặc dịch lặp. Với chế độ toàn bộ phần chữ, xử lý cả bản quyền, giới thiệu tác giả và quảng cáo; chế độ chỉ truyện giữ các trang phụ cùng nội dung phụ trợ riêng của chúng theo nguồn, ghi rõ trạng thái giữ nguyên. Nhãn mục lục khác tiêu đề trong nguồn phải được đối chiếu theo nội dung thực, không tự viết lại cho giống nhau.

Với ruby, phân biệt phần chỉ hướng dẫn phát âm với phần có dụng ý/ý nghĩa riêng theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md). Cách thể hiện mới của ruby có nghĩa riêng phải đưa bằng chứng và phương án để người dùng duyệt trước khi áp dụng; chưa đủ căn cứ thì giữ Cần xác minh. Quy tắc không hiện lại chú âm thuần phát âm không cho phép xóa dữ liệu nguồn hoặc bỏ ruby khỏi gói đối chiếu.

### 3.3. Thứ tự ưu tiên khi dung lượng không đủ

Nguồn được giao, mapping và hướng dẫn bắt buộc phải được giữ đầy đủ. Chọn tiếp các quy ước và bằng chứng cần thiết cho nội dung đó, rồi các đoạn tham khảo hữu ích trong dung lượng còn lại.

Khi request vượt giới hạn, giảm tài liệu tham khảo ít liên quan trước. Nếu vẫn chưa vừa, giảm số block được giao cho lượt và tính lại ngữ cảnh. Không cắt bớt nguyên văn bên trong một block mà vẫn giữ nguyên phạm vi khai báo là đã giao đủ block đó.

Nếu riêng hướng dẫn bắt buộc hoặc một thành phần không thể tách đã vượt dung lượng, trả lỗi chuẩn bị có vị trí/nguyên nhân để điều chỉnh cấu hình hoặc xử lý riêng.

### 3.4. Kiểm tra request hoàn chỉnh

Sau khi ghép hướng dẫn, dữ liệu và cấu trúc đầu ra, core kiểm tra lại:

- số token phần nguồn được giao so với giới hạn vận hành;
- toàn bộ đầu vào và ngân sách đầu ra so với giới hạn kỹ thuật;
- những ID nào thuộc phần phải dịch và những ID nào chỉ tham khảo;
- các phiên bản nguồn, skill, rule, glossary và ngữ cảnh đã dùng;
- model và cấu hình request dự kiến.

Việc đếm/ước lượng token phải phù hợp model, có dự phòng khi cần. `max_input_bytes` của bản Go có ý nghĩa byte; cấu hình token của bản Rust phải có tên/đơn vị rõ ràng, không diễn giải byte thành token. Bắt đầu workspace mới không yêu cầu tiếp nhận khóa cấu hình cũ.

Chuẩn bị một lượt là công việc đọc, lọc và ghép dữ liệu của core. Thiết kế hiện tại chưa yêu cầu một request AI riêng chỉ để phân tích hoặc tóm tắt toàn chương trước khi dịch.

Lượt tiếp theo được chuẩn bị khi đến lượt chạy để sử dụng dữ liệu mới nhất đã được chấp nhận. Tại thời điểm một request đã được chốt, dependency được ghi nhận để có thể kiểm tra lại khi lưu kết quả.

**Đầu ra bước 2:** request cho một lượt, phạm vi nguồn được giao, ngữ cảnh chỉ đọc, dependency và các ngân sách đã được kiểm tra.

## 4. Bước 3 — Tạo bản dịch

### 4.1. Gửi request qua lớp kết nối provider

Core chuyển request đã chuẩn bị cho adapter của model. Adapter chịu trách nhiệm chuyển định dạng thông điệp, thiết lập giới hạn đầu ra, timeout và các tham số đã được cấu hình.

Theo 9A chốt ngày **30/10/2026** trong [chiến lược model](MODEL_STRATEGY_SPEC.md), mặc định mỗi lần gọi chờ tối đa 10 phút, retry lỗi tạm thời chờ 10 rồi 30 giây và không tự đổi model; người dùng có thể tùy chỉnh thời gian. Trước mỗi lần gửi, gồm retry/sửa, kiểm tra ngân sách tiền và trần tổng số lần gọi tùy chọn. Lưu lần đã gửi/kết quả chưa rõ để tiếp tục không làm mất bộ đếm; không coi phản hồi dở hoặc timeout là bản dịch hoàn chỉnh.

Nếu provider hỗ trợ ràng buộc cấu trúc đầu ra phù hợp, adapter có thể sử dụng khả năng đó. Kiểm tra cấu trúc ở core vẫn phải chạy trên mọi phản hồi.

Trước mỗi lần gọi mạng, ghi nhận một lần thử gồm tác vụ, lượt, phạm vi nguồn, model và thời điểm bắt đầu. Giới hạn request/ngân sách đang áp dụng phải được kiểm tra trước khi bắt đầu lần thử.

Phân biệt ba mức:

| Mức | Ý nghĩa |
| --- | --- |
| Tác vụ | Yêu cầu dịch/tiếp tục phạm vi chương của người dùng |
| Lượt dịch | Phạm vi nội dung vừa dung lượng được giao cho model |
| Lần thử | Một lần thực sự gọi provider cho lượt đó, kể cả retry hoặc sửa cấu trúc |

Một lượt có thể có nhiều lần thử. Các lần thử vẫn cần ghi nhận usage, lỗi và model thực tế; chúng không được tạo nhiều bản dịch hiện hành trùng nhau cho cùng phần nguồn.

### 4.2. Hợp đồng đầu ra

Model trả bản dịch theo ID. Ví dụ rút gọn cho một lượt có hai đoạn:

```json
{
  "units": [
    {
      "id": "b021",
      "target": "Mưa vẫn chưa ngớt."
    },
    {
      "id": "b022",
      "target": "Cô khép chiếc ô rồi bước vào nhà ga."
    }
  ],
  "uncertainties": [],
  "continuity_candidates": [],
  "rule_candidates": []
}
```

| Trường | Vai trò |
| --- | --- |
| `units` | Bản dịch cho từng ID/phạm vi được giao |
| `uncertainties` | Chỗ chưa chắc, có ID và đoạn nguồn làm bằng chứng |
| `continuity_candidates` | Đề xuất về người nói, người nghe, quan hệ hoặc dữ liệu ngữ cảnh, có căn cứ nguồn |
| `rule_candidates` | Đề xuất thuật ngữ/quy ước mới, có căn cứ nguồn |

Nguyên văn và tài liệu tham khảo được đưa vào như dữ liệu cần xử lý. Model được yêu cầu chỉ trả bản dịch cho phạm vi được giao; phần nguồn trước đó dùng để tham khảo không được xuất thêm thành bản dịch mới.

ID tác vụ, hash nguồn, hash bản dịch và số revision là metadata do chương trình quản lý. Không giao việc tính hoặc tự suy đoán các hash này cho model.

### 4.3. Thu nhận kết quả

Adapter thu nhận phản hồi, trạng thái kết thúc, model thực tế, thông tin lỗi và usage nếu provider trả về. Nếu giá hoặc usage chưa xác định, giữ trạng thái chưa xác định thay vì ghi chi phí bằng không.

Phản hồi nhận được là ứng viên, chưa phải bản nháp hợp lệ. Cần lưu đủ dữ liệu để tiếp tục kiểm tra cục bộ khi có gián đoạn, trong khu vực dữ liệu nội bộ của workspace. Nội dung request/response không được đưa vào Git hay log công khai.

Nếu phản hồi tự dùng thuật ngữ hoặc cách xưng hô mới chưa được duyệt, giữ nội bộ để trình đề xuất và chờ người dùng. JSON hợp lệ không cho phép công bố phần phụ thuộc thành bản nháp hiện hành.

**Đầu ra bước 3:** phản hồi ứng viên gắn với đúng lượt/lần thử và thông tin provider liên quan.

## 5. Bước 4 — Kiểm tra cấu trúc

### 5.1. Nội dung cần kiểm tra

Với lượt được giao `b021–b040`, core thực hiện các kiểm tra bằng chương trình:

| Nhóm | Điều kiện cần đạt |
| --- | --- |
| Trạng thái kết thúc | Phản hồi không bị provider đánh dấu cắt do giới hạn; lỗi provider được xử lý rõ |
| Parse và schema | JSON hợp lệ, đủ trường bắt buộc, đúng kiểu và đúng hợp đồng đầu ra |
| Tập ID | Mỗi ID được giao có đúng một kết quả; không thiếu, trùng hoặc có ID lạ |
| Nội dung | Đoạn cần dịch có target không rỗng |
| Thứ tự | Bản dịch được gắn và dựng lại theo thứ tự nguồn, không phụ thuộc thứ tự model tự liệt kê |
| Inline token | Đủ token bắt buộc, cặp mở/đóng và quan hệ định dạng hợp lệ |
| Ruby, ảnh, chú thích và liên kết | Mapping cấu trúc liên quan vẫn hợp lệ |
| Block có phần con | Các phạm vi con phủ nguồn đầy đủ, không chồng lấn, giữ được thứ tự ghép |
| Bằng chứng của uncertainty/candidate | ID thuộc phạm vi được giao; quote tham chiếu tồn tại trong nguồn tương ứng |

Có thể dựng thử đoạn XHTML từ target và token mapping để phát hiện lỗi ghép cấu trúc trước khi lưu nháp. Đây là kiểm tra phần nội dung liên quan đến request; kiểm tra toàn bộ EPUB vẫn thuộc workflow xuất.

Mapping ruby phải còn đầy đủ để truy về nguồn và áp dụng đúng quyết định thể hiện. Không hiển thị chú âm thuần phát âm theo chính sách đầu ra khác với làm mất token/mapping hoặc bỏ một ý nghĩa chưa được xử lý; kiểm tra cấu trúc cần phân biệt các trường hợp này.

Đủ ID và hợp lệ về XML/marker không chứng minh bản dịch đúng nghĩa hoặc đủ ý. Việc đánh giá chất lượng đó thuộc Review.

### 5.2. Xử lý lỗi theo đúng nguyên nhân

| Loại lỗi | Cách xử lý |
| --- | --- |
| Mạng tạm thời hoặc lỗi dịch vụ có thể thử lại | Báo ngay, chờ rồi retry cùng model tối đa 2 lần ngoài lần gọi đầu, lưu từng lần thử |
| Xác thực/provider/model cấu hình sai | Dừng và báo cấu hình cần sửa |
| Hết hạn mức tài khoản hoặc chưa rõ nguyên nhân lỗi | Dừng phần bị lỗi và chờ người dùng; không tự đổi model hoặc thử liên tục |
| Chạm giới hạn request hoặc không đủ ngân sách khi bật hạn mức chi phí | Chưa gửi request mới, giữ tiến độ để tiếp tục sau khi người dùng xử lý; không tự nâng hạn mức hoặc chuyển sang chạy không giới hạn |
| Phản hồi bị cắt vì giới hạn token | Tính lại đầu ra hoặc chia nhỏ phạm vi chưa hoàn thành |
| JSON sai hoặc thiếu/trùng ID | Gửi lỗi cụ thể để cùng model sửa theo hợp đồng, tối đa 2 lượt sửa cấu trúc |
| Marker/inline cấu trúc sai | Yêu cầu sửa đúng lỗi đã phát hiện; không tự đoán bản dịch thay thế |
| Thành phần nguồn không thể chia mà vẫn quá dài | Báo vị trí và giới hạn liên quan |
| Hết số lần sửa cấu trúc | Lưu chẩn đoán, chuyển lượt sang trạng thái cần xử lý |

Ví dụ phản hồi sửa cấu trúc phải chỉ rõ lỗi:

```text
Lượt đang xử lý: b021–b040
- Thiếu ID b027.
- ID b030 xuất hiện hai lần.
- Token t2 của b033 chưa có cặp đóng hợp lệ.
```

Giới hạn đã chốt là tối đa 2 lượt yêu cầu sửa cấu trúc và 2 lần retry mạng ngoài lần gọi đầu, theo [chiến lược model](MODEL_STRATEGY_SPEC.md). Đây là các bộ đếm riêng, không phải vòng sửa chất lượng của giai đoạn Review. Mọi lần gửi đều được ghi nhận, không tự đổi model khi lỗi; những thông điệp sửa cấu trúc bổ sung cũng phải được tính vào ngân sách request.

Một lượt chỉ được công nhận hợp lệ khi toàn bộ phạm vi của lượt đã qua kiểm tra. Phản hồi thiếu hoặc hỏng có thể được giữ làm dữ liệu chẩn đoán, nhưng không được công bố như bản nháp hoàn chỉnh của lượt đó.

Qua kiểm tra cấu trúc chưa thay thế điều kiện duyệt thuật ngữ/quy ước. Trạng thái chờ người dùng không phải lỗi cấu trúc hoặc lỗi mạng; không retry liên tục và không tự chọn phương án để vượt bước chờ.

Nếu chia lại phạm vi, kế hoạch mới phải giữ coverage nguồn và không ghi lặp những phần đã được lưu hợp lệ ở trước đó.

**Đầu ra bước 4:** kết quả qua kiểm tra cấu trúc hoặc danh sách lỗi gắn với lượt và block cụ thể.

## 6. Bước 5 — Lưu bản nháp

### 6.1. Dữ liệu cần lưu

| Nhóm dữ liệu | Thông tin cần biểu diễn |
| --- | --- |
| Phạm vi | Tập/chương, ID block, phần con nếu có, ID tác vụ và lượt |
| Bản dịch | Target theo ID và thứ tự ghép về nguồn |
| Nguồn | Snapshot, source hash, phiên bản parser/mapping |
| Dependency | Phiên bản skill, rule, glossary và ngữ cảnh thực sự dùng |
| Model | Provider/model thực tế, tham số có liên quan, liên kết đến các lần thử |
| Lịch sử | Revision mới và quan hệ với revision/generation trước |
| Kiểm tra | Kết quả kiểm tra cấu trúc, hash target do chương trình tính |
| Nội dung cần xem xét | Uncertainties, continuity candidates và rule candidates |
| Trạng thái | Đã lưu nháp, chờ review; chưa được coi là bản dịch đã chấp nhận |

Các đề xuất được lưu trong hồ sơ riêng với trạng thái chưa xác nhận, gắn phạm vi nguồn và revision liên quan nếu đã có. Phần còn phụ thuộc vào thuật ngữ/quy ước chưa duyệt chỉ được giữ dưới dạng ứng viên, chưa thành bản nháp hiện hành. Sau khi người dùng chốt, chuẩn bị lại phần phụ thuộc với quyết định đã lưu rồi kiểm tra/lưu nháp và Review. Việc lưu nháp không tự duyệt các đề xuất ngữ cảnh đi kèm.

### 6.2. Kiểm tra phiên bản trước khi ghi

Trong thời gian chờ AI, dữ liệu liên quan có thể thay đổi. Trước khi ghi, core đối chiếu generation hiện tại, nguồn và dependency với phiên bản lúc chuẩn bị request.

Nếu có bản sửa tay mới, revision nguồn mới hoặc thay đổi liên quan làm kết quả không còn phù hợp, giữ kết quả trong dữ liệu phục hồi/xung đột và nêu rõ nguyên nhân. Không ghi đè revision hiện hành bằng phản hồi được tạo từ phiên bản cũ.

Writer lock kiểm soát tác vụ ghi, còn kiểm tra phiên bản bảo vệ việc commit đúng nền dữ liệu. Cần kiểm tra cả bản thảo có chỉnh sửa bên ngoài để thao tác dựng Markdown không làm mất sửa tay.

### 6.3. Trình tự ghi dữ liệu

Chốt ngày **04/11/2026** theo hợp đồng generation và SQLite trong [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), mục 11.

```text
Kết quả hợp lệ
  → đối chiếu phiên bản, quyền ghi và sửa tay
  → ghi/kiểm tra file bất biến theo hash nếu có
  → một giao dịch SQLite
      → revision, tham chiếu Review/quyết định và dependency
      → generation, checksum, con trỏ hiện hành và tiến độ
      → công việc dựng Markdown cần hoàn thành
  → commit và phát trạng thái dữ liệu đã lưu
  → dựng/cập nhật Markdown, bảo vệ chỉnh sửa ngoài
  → ghi hoàn tất việc đồng bộ và cập nhật giao diện
```

Generation hiện hành và tiến độ được công bố cùng transaction. Mỗi generation có ID cha cùng các tham chiếu thay đổi. File bất biến được ghi xong trước khi transaction tham chiếu nó. File chưa được tham chiếu có thể được thu gom sau.

Nếu commit đã thành công nhưng dựng Markdown thất bại, hiện **Đã lưu bản dịch, chờ dựng Markdown**. Phục hồi từ dữ liệu đã commit, đối chiếu hash/bản nền và bảo vệ sửa tay theo hợp đồng đồng bộ Markdown, rồi tiếp tục việc dựng đang chờ.

Khi transaction chưa commit, giữ kết quả có thể phục hồi và báo đúng lỗi lưu trữ. Request, phản hồi thô và kế hoạch nhiều đơn vị có ID riêng trong dữ liệu mới. Quyết định không chuyển workspace test cũ cho phép cấu trúc SQLite này, còn revision, quyết định và lịch sử mới phải được bảo toàn.

### 6.4. Kết quả trả về cho bộ điều phối

Hàm xử lý một lượt trả về thông tin tương đương:

```text
Tác vụ: dịch Chương 2
Lượt: 1
Phạm vi: b001–b020
Revision/generation: bản vừa commit
Kiểm tra cấu trúc: hợp lệ
Trạng thái: đã lưu nháp, chờ review
```

TUI cập nhật từ sự kiện sau khi lưu, chẳng hạn **Chương 2 — Đã lưu nháp lượt 1, chờ review**. Không đánh dấu chương hoàn tất chất lượng chỉ vì các lượt đã có nháp.

Trong suốt lúc gọi model, giao diện luôn hiển thị tên/ID model và vai trò của request đang chạy theo [chiến lược model](MODEL_STRATEGY_SPEC.md). Khi chuyển sang kiểm tra/lưu cục bộ hoặc chờ người dùng, đổi trạng thái tương ứng để không hiển thị như AI vẫn đang chạy.

**Đầu ra bước 5:** ID/revision bản nháp đã lưu, phạm vi nguồn và trạng thái chờ review. Giai đoạn Dịch của lượt đó kết thúc tại đây.

## 7. Tiếp tục khi bị gián đoạn

| Trạng thái lúc dừng | Cách tiếp tục |
| --- | --- |
| Đã có hồ sơ tác vụ, chưa gửi request | Đọc hồ sơ, kiểm tra lại dữ liệu và chuẩn bị request |
| Request đang chạy, chưa có phản hồi được lưu | Ghi nhận kết quả lần thử chưa xác định; xử lý retry theo chính sách |
| Phản hồi đã được lưu, chưa kiểm tra cấu trúc | Tiếp tục kiểm tra cục bộ trước khi gọi model lại |
| Đã qua kiểm tra, chưa commit | Kiểm tra lại phiên bản và thử lưu |
| Generation đã commit, Markdown chưa dựng được | Phục hồi bản thảo từ generation, bảo vệ sửa tay |
| Bản nháp đã lưu còn hợp lệ | Dùng lại bản nháp, giữ trạng thái chờ review |
| Đề xuất thuật ngữ/quy ước đang chờ | Khôi phục hồ sơ và phần phụ thuộc; tiếp tục chờ quyết định, không tự áp dụng |
| Nguồn/dependency đã đổi | Xử lý revision/xung đột trước khi dùng lại |

Không bảo đảm provider chỉ tính một lần khi kết nối bị mất sau lúc gửi request. Mỗi lần retry có thể là một lần gọi mới; usage chưa xác định phải được giữ đúng trạng thái. Ứng dụng cần bảo đảm việc tiếp tục không ghi lặp hoặc ghi đè sai kết quả trong workspace.

Pause bình thường thực hiện tại ranh giới an toàn của công việc đã lưu. Khi dừng đột ngột, dữ liệu đã commit được dùng làm căn cứ khôi phục. Phần đang chạy nhưng chưa có kết quả hợp lệ vẫn chưa hoàn thành.

## 8. Ví dụ xuyên suốt

Giả sử Chương 2 có 60 đoạn. Các số lượng dưới đây chỉ minh họa, không thay cách tính token thực tế.

```text
Người dùng bấm Dịch Chương 2
│
├── Bước 1: nhận yêu cầu
│   ├── Xác định Tập 01 / Chương 2
│   ├── Kiểm tra nguồn, cấu hình và tiến độ
│   └── Tạo hồ sơ tác vụ
│
├── Bước 2: chuẩn bị lượt đầu
│   ├── Giao b001–b020 theo dung lượng thực tế
│   ├── Ghép quy tắc, glossary và ngữ cảnh liên quan
│   └── Kiểm tra hai giới hạn, ghi dependency
│
├── Bước 3: tạo bản dịch
│   ├── Ghi lần gọi đang chạy
│   ├── Gọi model
│   └── Nhận phản hồi ứng viên cùng trạng thái và usage
│
├── Bước 4: kiểm tra cấu trúc
│   ├── Kiểm tra đủ ID, marker và mapping
│   └── Nếu lỗi: xử lý đúng nguyên nhân trong giới hạn cho phép
│
└── Bước 5: lưu bản nháp
    ├── Kiểm tra phiên bản trước khi ghi
    ├── Commit generation và dựng bản thảo
    └── Trả kết quả: b001–b020 đã có nháp, chờ review
```

Bộ điều phối nhận kết quả lượt đầu và chạy Review cho đúng revision vừa lưu. Khi lượt này đạt điều kiện cục bộ, chuẩn bị request dịch lượt tiếp theo bằng trạng thái và ngữ cảnh còn phù hợp tại thời điểm đó. Hết các lượt phải kiểm tra tổng thể chương theo đặc tả điều phối.

Nếu ứng dụng mở lại sau lúc nháp b001–b020 đã lưu, bước nhận lệnh phải hiển thị phần này là đã có nháp. Việc mở lại tác vụ không tự biến nó thành phần chưa dịch.

## 9. Tiêu chí nghiệm thu cho giai đoạn Dịch

Kiểm thử tự động dùng model giả lập và dữ liệu mẫu offline, deterministic; không gọi provider trả phí.

| Tình huống | Điều cần chứng minh |
| --- | --- |
| Bấm Dịch với chương hợp lệ | Tạo đúng phạm vi và hồ sơ tác vụ |
| Bấm lặp khi tác vụ đang chạy | Không tạo hai công việc trùng |
| Thiếu nguồn/cấu hình hoặc có writer khác | Báo rõ nguyên nhân trước khi gửi request |
| Tiếp tục chương đã có nháp | Dùng lại kết quả còn hợp lệ, phân biệt nháp với đã duyệt |
| Có sửa tay chưa đồng bộ | Không ghi đè hoặc tự dịch lại bản sửa |
| Chuẩn bị ngữ cảnh | Chỉ dùng dữ liệu đúng phạm vi, đúng thời điểm; candidate chưa duyệt không thành quy ước chính thức |
| Model dùng thuật ngữ/quy ước mới chưa duyệt | Giữ ứng viên và đề xuất, chưa ghi phần phụ thuộc thành bản nháp hiện hành |
| Người dùng chốt thuật ngữ | Lưu quyết định đúng phạm vi, tiếp tục dịch và Review; không tự coi bản dịch đã đạt |
| Chưa có ngữ cảnh trước đó | Vẫn chuẩn bị được phần đầu với dữ liệu thực có, không tự bịa thông tin |
| Request gần giới hạn | Tính toàn bộ request và ngân sách đầu ra, giảm phạm vi khi cần |
| Model trả thiếu/trùng/ID lạ hoặc target rỗng | Không công nhận lượt hợp lệ |
| Marker, ruby hoặc mapping con hỏng | Chỉ đúng vị trí lỗi và giữ nguồn nguyên vẹn |
| Ruby có ý nghĩa riêng hoặc chưa rõ | Giữ cả hai lớp nguồn, trình bằng chứng/phương án và chờ xử lý; không tự bỏ hoặc dùng cách thể hiện mới chưa duyệt |
| Tiêu đề, mục lục và chú thích thuộc phạm vi dịch | Có bản dịch/mapping tương ứng trước bước xuất, dùng lại kết quả đã duyệt khi cùng nội dung nguồn |
| Bằng chứng candidate không có trong nguồn | Kiểm tra thất bại, không công bố candidate |
| Response bị cắt hoặc provider lỗi | Phân loại đúng, retry/sửa có giới hạn |
| Mất kết nối sau gửi request | Lần thử được ghi nhận, không giả định chắc chắn chưa phát sinh usage |
| Nguồn/revision thay đổi trong khi chờ AI | Kết quả được giữ để xử lý xung đột, không ghi đè sai |
| Commit thành công nhưng dựng Markdown lỗi | Khôi phục từ generation, không bắt dịch lại |
| Dừng và mở lại ở từng bước | Tiếp tục từ dữ liệu đã lưu, không ghi lặp phần đã hoàn tất |
| Lưu nháp thành công | TUI và bộ điều phối nhận trạng thái chờ review, không nhầm với hoàn tất chất lượng |
| Lượt vừa lưu nháp nhưng chưa Review | Chạy Review cho lượt đó trước khi gửi dịch lượt kế tiếp |

## 10. Đối chiếu với mã hiện tại

Go đã có nhiều thành phần tương ứng: phạm vi chạy, snapshot skill/ngữ cảnh, hợp đồng JSON, kiểm tra bản dịch, request usage, generation và bản thảo chương. Runner hiện gắn dịch và review trong cùng luồng và thường gọi theo block, có xử lý block dài.

Tài liệu này xác lập ranh giới Dịch và request nhiều block theo ngân sách cho bản Rust. Hợp đồng đã chốt ngày **04/11/2026** tại mục 11 và đặc tả dữ liệu là căn cứ triển khai. Giữ đường chạy Go hoạt động đến khi phần thay thế đạt yêu cầu. Workspace Rust có schema 1, bảo toàn dữ liệu và nâng cấp về sau, không cần chuyển dữ liệu test cũ.

- [Đặc tả EPUB đầu vào và chia lượt](EPUB_INPUT_SPEC.md).
- [Runner và payload hiện có](../../internal/translation/engine/runner.go).
- [Nạp skill](../../internal/translation/skills/loader.go) và [chọn nội dung prompt](../../internal/translation/skills/prompt.go).
- [Ngữ cảnh đã được chấp nhận](../../internal/translation/continuity/context.go).
- [Adapter model và ghi usage](../../internal/translation/model/adapter.go).
- [Hợp đồng đầu ra](../../internal/translation/model/contracts.go).
- [Generation và commit](../../internal/translation/state/store.go).
- [Bản thảo theo chương và khôi phục](../../internal/translation/revision/edit.go).
- [Sự kiện tiến độ](../../internal/translation/progress/events.go).

## 11. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [20](DESIGN_DECISIONS_36.md#decision-20) của bản 36 mục áp dụng cho đặc tả này.

### 11.1. Cấu trúc request và response của phần Dịch — mục 20

Chọn hợp đồng JSON có `schema_version`. Request gồm danh sách đơn vị được giao theo thứ tự, ID/phạm vi và nguyên văn từng đơn vị, marker được bảo vệ, ngữ cảnh chỉ đọc, Skills/glossary/xưng hô đã duyệt, hướng dẫn văn phong và hợp đồng đầu ra. Chương trình giữ snapshot, hash, phiên bản và model của lần gửi. Model trả `units[{id, target}]`, `uncertainties[]`, `continuity_candidates[]` và `rule_candidates[]`. Mỗi đề xuất/điểm chưa rõ có vị trí và căn cứ nguồn.

Core kiểm tra đủ ID, đúng thứ tự, mỗi ID xuất hiện một lần, bản dịch có chữ và các marker, ruby, liên kết, ảnh, chú thích còn đối chiếu được. Lỗi cấu trúc tạo chẩn đoán theo ID và tối đa hai yêu cầu sửa. Kết quả bị cắt cần lập lại ngân sách/phạm vi. Đơn vị dùng thuật ngữ mới chờ quyết định ở vùng ứng viên. Chỉ bản đã qua các điều kiện mới thành nháp hiện hành. [Hợp đồng Dịch](TRANSLATION_STAGE_SPEC.md)

### 11.2. Liên kết và yêu cầu triển khai

Thực hiện công bố revision theo SQLite/generation ở mục 6.3 và [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), mục 11. Phản hồi muộn theo [chiến lược model](MODEL_STRATEGY_SPEC.md), checkpoint theo [điều phối](WORKFLOW_ORCHESTRATION_SPEC.md).
