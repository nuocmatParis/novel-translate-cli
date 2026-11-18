# Quyết định 07 — Mô hình dữ liệu, phiên bản và trạng thái công việc

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 11.

Ngày ghi nhận: **27/10/2026**.

Trạng thái: **đã chốt hướng thiết kế trong trao đổi**. Tài liệu mô tả dữ liệu và hành vi sản phẩm cần có; không khẳng định implementation hiện tại đã đáp ứng. Hợp đồng lưu trữ, generation, dependency và ba trục trạng thái đã chốt tại mục 11. Tên struct Rust và câu lệnh tạo bảng được hiện thực theo hợp đồng đó.

Tài liệu nối các quyết định về [đầu vào EPUB](EPUB_INPUT_SPEC.md), [Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [rubric](REVIEW_RUBRIC_SPEC.md), [xưng hô](ADDRESSING_CONTINUITY_SPEC.md), [Skills và duyệt thuật ngữ](SKILLS_GLOSSARY_SPEC.md), [ảnh và xuất EPUB](IMAGE_EDIT_SPEC.md).

Cập nhật ngày **29/10/2026**: dữ liệu phục vụ [bố cục EPUB tiếng Việt](EPUB_OUTPUT_SPEC.md) phải truy được tiêu đề/mục lục/chú thích, nguồn ruby, cách thể hiện đã xác định/duyệt và cấu hình bố cục của từng bản chuẩn bị.

Cập nhật ngày **30/10/2026**, mục 10 trong danh sách cần làm rõ: workspace cũ chỉ dùng thử nghiệm và không cần giữ/chuyển dữ liệu sang bản Rust mới. Chốt bắt đầu bằng workspace mới, bỏ yêu cầu tương thích/migration từ workspace thử nghiệm cũ; phạm vi cụ thể tại mục 9.1. Cơ chế bảo toàn dữ liệu, revision và phục hồi của workspace mới vẫn là yêu cầu sản phẩm.

## 1. Mục tiêu và các nguyên tắc đã chốt

Sau mỗi thao tác, tool phải xác định được: dữ liệu nào vừa thay đổi, kết quả nào còn dùng được, phần nào cần kiểm tra lại và công việc tiếp theo là gì.

- Giữ bản nguồn làm căn cứ; bản dịch và các quyết định có lịch sử riêng.
- Kết quả Review gắn với đúng phiên bản đã được kiểm tra.
- Chất lượng nội dung, tính còn phù hợp của dữ liệu và trạng thái tác vụ là ba trục riêng.
- Thay đổi một phần chỉ làm xét lại các kết quả chịu ảnh hưởng; không mặc định dịch lại cả bộ.
- Dừng, mở lại hoặc đổi cách chia lượt không làm mất kết quả đã lưu hợp lệ.
- Quyết định dịch thuật mới phải qua người dùng duyệt theo đặc tả Skills; trạng thái chờ duyệt phải được lưu và phục hồi.
- Bảo toàn sửa tay, lịch sử và dữ liệu của workspace mới được hỗ trợ. Không yêu cầu giữ định dạng hoặc chuyển workspace thử nghiệm cũ sang bản Rust mới; schema mới bắt đầu ở phiên bản 1 và nâng cấp có sao lưu, kiểm tra, phục hồi theo mục 11.

## 2. Các đối tượng cần quản lý

| Đối tượng | Thông tin và quan hệ cần biểu diễn |
| --- | --- |
| Bộ truyện | Các tập, skill riêng, tham chiếu thư viện skill chung, quy ước và ngữ cảnh dùng xuyên tập. |
| Tham chiếu kết nối model | Định danh kết nối/cấu hình không bí mật dùng cho các vai trò; giá trị key nằm ngoài dữ liệu workspace theo 11A. |
| Tập | EPUB nguồn, snapshot đang dùng, cây chương, tài nguyên, tiến độ và thông tin sách đầu ra đã lưu riêng cho tập. |
| Lựa chọn nội dung của tập | Toàn bộ phần chữ hoặc chỉ truyện và phần liên quan; phiên bản phân loại phần cần dịch, giữ nguyên, ảnh và chưa rõ theo mapping nguồn. |
| Chương | Thứ tự block, tiêu đề/part, vị trí tranh và kết quả kiểm tra nhất quán cấp chương. |
| Block nguồn | ID, nguyên văn, vị trí trong snapshot, các thành phần được bảo vệ và mapping để ghép lại. |
| Revision bản dịch | Nội dung dịch của một phạm vi, nguồn tương ứng, phiên bản quy tắc/ngữ cảnh đã dùng và lịch sử thay đổi. |
| Kết quả Review | Revision được kiểm tra, độ phủ, lỗi, bằng chứng, kết luận và các quyết định xử lý. |
| Đề xuất quy ước | Vấn đề chưa được chốt, phương án, căn cứ, phạm vi và trạng thái chờ người dùng. |
| Quyết định đã duyệt | Cách dịch/quy tắc được chọn, người xác nhận, phạm vi hiệu lực, căn cứ và phiên bản. |
| Ngữ cảnh có căn cứ nguồn | Nhân vật, sự kiện, thời điểm và phần chưa biết; tách dữ kiện nguồn khỏi lựa chọn diễn đạt tiếng Việt. |
| Yêu cầu/kết quả Research | Câu hỏi, bước đang chờ, vị trí/snapshot nguồn, bằng chứng nội bộ/URL ngoài, mốc sử dụng, điều chưa rõ và model/lịch sử; chưa tự thành ngữ cảnh đã xác nhận. |
| Tác vụ | Ý định người dùng, chế độ theo phần đã chọn hoặc tự động đến hết EPUB, phạm vi/hàng đợi chương, cấu hình ngân sách thực tế/nguồn kế thừa, usage/chi phí/dự phòng, cấu hình thời gian chờ/thử lại, trần số lần gọi tùy chọn/tổng đã dùng, tiến độ đã lưu, việc đang chờ, công việc tiếp theo và lịch sử các lần thử. |
| Hàng đợi tác vụ AI | Các tác vụ đã được yêu cầu chạy/tiếp tục, thứ tự, lý do chờ và một tác vụ đang giữ quyền chạy theo 13A; phân biệt với hàng đợi chương của từng tác vụ. |
| Bộ dữ liệu chuẩn bị xuất | Tập hợp revision bản dịch gồm tiêu đề/mục lục/chú thích, snapshot và mapping nguồn, kết quả thể hiện ruby/quyết định liên quan, thông tin sách đầu ra, ảnh được chọn và cấu hình trình bày dùng chung cho preview/export. |
| Bản EPUB đã xuất | Tập, số phiên bản file xuất, đường dẫn/thời điểm/hash, định dạng EPUB 3, bộ dữ liệu chuẩn bị và kết quả kiểm tra; giữ độc lập các bản đã xuất. |

Một quyết định về thuật ngữ, văn phong hoặc xưng hô phải có thao tác duyệt của người dùng. Việc Reviewer xác nhận một sự kiện trong nguồn không tự quyết định cách dịch thuật ngữ hoặc cách xưng hô tiếng Việt.

Theo 7A chốt ngày **30/10/2026** trong [đặc tả Skills](SKILLS_GLOSSARY_SPEC.md), một thao tác có thể xác nhận nhiều đề xuất được chọn. Mỗi đề xuất vẫn có quyết định, phạm vi, bằng chứng và phiên bản riêng; cần truy được đúng các mục/lựa chọn đã được xác nhận. Các mục chưa chọn hoặc chưa lưu thành công không tự thành quy tắc có hiệu lực. Nhóm được chọn công bố cùng thành công theo mục 15 của [36 quyết định đã chốt](DESIGN_DECISIONS_36.md#decision-15). Ghi Skill chưa xong thì giữ trạng thái chờ đồng bộ, rồi kích hoạt cả nhóm một lần.

Theo 8A cùng ngày trong [chiến lược model](MODEL_STRATEGY_SPEC.md), mặc định ngân sách của tool/cấu hình riêng của bộ là nguồn điền cho tác vụ mới. Hồ sơ tác vụ lưu cấu hình có hiệu lực và nguồn kế thừa; sửa mặc định không thay ngân sách tác vụ đã tồn tại. Tiếp tục hoặc chủ động chỉnh ngân sách của tác vụ giữ usage, chi phí/dự phòng, bộ đếm và lịch sử. Hồ sơ và sổ chi phí nằm trong SQLite của bộ. Hợp đồng tính giá/dự phòng theo mục 18 của [36 quyết định đã chốt](DESIGN_DECISIONS_36.md#decision-18).

Theo 9A cùng ngày, mỗi lần gọi cần truy được model/vai trò, request logic, số lần thử, thời điểm gửi/thời hạn, trạng thái đã gửi hoặc chưa rõ, kết quả và usage. Tác vụ giữ tổng số lần gọi và mốc chờ retry qua gián đoạn; không dùng số request làm số revision hoặc mức chất lượng. Cần đối chiếu phản hồi đến muộn để không áp kết quả hai lần; lưu riêng request logic và từng attempt trong SQLite theo mục 19 của [36 quyết định đã chốt](DESIGN_DECISIONS_36.md#decision-19).

Theo **11A** cùng ngày trong [đặc tả key và preview](SECURITY_SPEC.md), workspace/hồ sơ tác vụ chỉ giữ tham chiếu kết nối, không sao chép key từ kho hệ điều hành hoặc môi trường. Quyền phiên preview gắn với phạm vi tài nguyên được phép trong bộ chuẩn bị; mã truy cập không thuộc nội dung sách, lịch sử dịch hoặc file xuất. Thiết kế lưu schema không được đưa các giá trị bí mật này vào generation hoặc bản thảo.

Cây quan hệ minh họa, không phải bố trí thư mục mới:

```text
Bộ truyện
├── Quy ước đã duyệt + lịch sử đề xuất
└── Tập 01
    ├── Snapshot nguồn
    ├── Chương 1
    │   ├── Block b001
    │   │   ├── Bản dịch r1 → Review r1
    │   │   └── Bản dịch r2 → Review r2
    │   ├── Block b002
    │   └── Review mạch chương → bộ revision được kiểm tra
    ├── Ảnh nguồn / ảnh thay
    └── Tác vụ và bộ dữ liệu chuẩn bị xuất
```

Các lượt dịch chỉ tham chiếu tới block/phạm vi nguồn. Chia một chương từ ba lượt thành bốn lượt không tạo chương mới và không đổi ID nguồn đã xác lập. Revision tham chiếu các đơn vị nguồn/phạm vi con đã định danh theo mục 04 của [36 quyết định đã chốt](DESIGN_DECISIONS_36.md#decision-04). Review chương tham chiếu đúng tập hợp revision của chương.

## 3. Snapshot, revision, generation và dependency

### 3.1. Snapshot — nguồn cố định để đối chiếu

Snapshot giữ bản EPUB đã import cùng hash và thông tin phân tích/mapping. Dịch, Review và ghép sách cùng tham chiếu nguồn này.

Khi người dùng đưa EPUB mới vào thay nguồn, tạo hoặc đăng ký snapshot mới, kiểm tra mapping và xác định kết quả có thể tái sử dụng. Không sửa nội dung snapshot cũ rồi tiếp tục coi đó là cùng nguồn.

### 3.2. Revision — một phiên bản nội dung

Khi bản dịch thay đổi, lưu revision mới và giữ lịch sử bản cũ. Ví dụ `r1` đã được review, người dùng sửa câu thì tạo `r2`; kết luận của `r1` vẫn có giá trị lịch sử nhưng chưa chứng minh `r2` đạt.

Quy ước cũng có phiên bản riêng. Đổi cách dịch một thuật ngữ tạo phiên bản quyết định mới; không sửa lịch sử để làm như mọi bản dịch trước đây đã dùng lựa chọn mới.

Phản hồi thô của model hoặc một phương án chưa được duyệt có thể được lưu nội bộ để phục hồi. Việc lưu đó không biến nó thành revision bản nháp hiện hành được phép dùng.

### 3.3. Generation — một mốc trạng thái lưu nhất quán

Generation là mốc dữ liệu đã lưu để các thành phần liên quan cùng chỉ tới một trạng thái nhất quán: revision hiện hành, quyết định liên quan, dependency và tiến độ đã commit.

Ví dụ, khi một bản nháp mới được lưu, hệ thống không được báo revision mới đã hiện hành trong khi tiến độ hoặc tham chiếu dữ liệu vẫn ở một trạng thái không tương thích. Chỉ công bố mốc mới sau khi phần dữ liệu bắt buộc đã ghi thành công theo cơ chế lưu trữ của workspace.

Generation không phải số lần gọi AI, số chương đã dịch hay phiên bản nội dung của riêng một đoạn. Bản thảo Markdown và màn hình là cách biểu diễn từ dữ liệu đã lưu, đồng thời phải bảo vệ các sửa tay chưa được đồng bộ.

### 3.4. Dependency — dữ liệu mà một kết quả đã dựa vào

Mỗi kết quả cần truy được nguồn, mapping, skill, glossary, quy tắc xưng hô và ngữ cảnh đã sử dụng, theo đúng phiên bản/phạm vi. Thông tin model và các lần gọi được lưu để giải thích quá trình tạo kết quả.

Ví dụ, một đoạn được dịch với quy ước xưng hô phiên bản 3. Khi phiên bản 4 được duyệt, tool xét cặp nhân vật, mốc hiệu lực và phạm vi thay đổi để xác định đoạn đó có cần kiểm tra lại hay không.

Thay đổi dependency không tự đồng nghĩa bản dịch sai. Nó làm phát sinh việc đánh giá lại tính còn phù hợp trước khi tiếp tục dùng kết quả như đã được xác nhận.

## 4. Ba trục trạng thái riêng

### 4.1. Chất lượng nội dung

| Trạng thái minh họa | Ý nghĩa |
| --- | --- |
| Chờ Review | Có bản nháp hiện hành đủ điều kiện vào Review, chưa có kết luận chất lượng. |
| Cần sửa | Có lỗi đã xác nhận, phải xử lý theo phạm vi/rubric. |
| Cần xác minh | Chưa đủ nguồn hoặc căn cứ để đưa ra kết luận đáng tin cậy. |
| Đạt | Phạm vi đã đáp ứng điều kiện Review tương ứng. |
| Người dùng giữ — có ngoại lệ | Người dùng chấp nhận giữ vấn đề với lý do; không đồng nhất với Reviewer xác nhận hoàn toàn đúng. |

Phần chưa có bản nháp được hiển thị là chưa dịch. Ứng viên đang chờ duyệt thuật ngữ chưa được tính là bản nháp hoàn chỉnh của phần phụ thuộc.

### 4.2. Tính còn phù hợp

| Trạng thái minh họa | Ý nghĩa |
| --- | --- |
| Còn phù hợp | Các căn cứ bắt buộc vẫn tương thích với nguồn và quyết định hiện hành. |
| Cần kiểm tra lại | Nguồn, mapping, bản dịch hoặc quy tắc liên quan đã thay đổi; phải đánh giá ảnh hưởng. |

Ví dụ, kết quả có thể hiển thị **Đã đạt ở lần Review trước — cần kiểm tra lại do đổi glossary**. Tool giữ kết luận cũ trong lịch sử và chưa trình bày nó như kết luận đạt cho bộ dữ liệu mới.

### 4.3. Trạng thái tác vụ

| Trạng thái minh họa | Hành vi |
| --- | --- |
| Chờ chạy | Công việc đã được ghi nhận, chưa bắt đầu phần cần xử lý. |
| Đang chạy | Đang thực hiện bước đã xác định của tác vụ. |
| Đang tạm dừng | Đã nhận yêu cầu dừng; đang xác lập mốc có thể phục hồi. |
| Đã tạm dừng | Giữ tiến độ đã lưu và bước tiếp theo. |
| Chờ người dùng | Cần quyết định cụ thể, ví dụ duyệt thuật ngữ hoặc xử lý xung đột. |
| Gặp lỗi vận hành | Có lỗi provider, mạng, lưu trữ hoặc điều kiện chạy; chưa được suy thành lỗi nghĩa của bản dịch. |
| Hoàn tất | Mục tiêu và phạm vi của chính tác vụ đã hoàn thành. |
| Đã hủy | Dừng phần công việc còn lại, giữ lịch sử và các kết quả đã lưu. |

`Chờ duyệt thuật ngữ` là lý do chờ người dùng của phạm vi bị ảnh hưởng. Task tạm dừng không làm mất trạng thái đạt của những phần còn hợp lệ. Task chỉ tạo nháp hoàn tất cũng không tự làm cả chương đạt Review.

Theo **13A** chốt ngày **30/10/2026** trong [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), bộ điều phối chỉ có một tác vụ AI hoạt động. Tác vụ đã được yêu cầu nhưng chưa có quyền chạy là Chờ chạy; khác với Chờ người dùng do thiếu quyết định hoặc Đã tạm dừng theo ý định người dùng. Cần truy được chủ thể giữ quyền chạy và mốc bàn giao, không suy từ việc đang mở màn hình nào. Dịch, Review và Research là các bước tuần tự thuộc cùng tác vụ.

## 5. Các chuyển trạng thái quan trọng

| Sự kiện | Dữ liệu/trạng thái cần thay đổi | Công việc tiếp theo |
| --- | --- | --- |
| Nhận lệnh Dịch | Ghi tác vụ, phạm vi và phiên bản đầu vào | Chuẩn bị phần chưa hoàn thành theo đặc tả Dịch. |
| Gặp thuật ngữ/quy ước mới chưa chốt | Lưu đề xuất, bằng chứng và phạm vi đang chờ | Chờ người dùng; có thể xử lý phần độc lập. |
| Người dùng chốt đề xuất | Lưu quyết định, phiên bản và phạm vi đã duyệt | Chuẩn bị lại phần phụ thuộc bằng quy tắc mới. |
| Phản hồi qua cấu trúc và điều kiện duyệt quy tắc | Lưu revision bản nháp hiện hành | Review revision đó. |
| Review xác nhận lỗi | Lưu lỗi và bằng chứng gắn đúng revision | Sửa/dịch lại theo rubric, rồi Review lại. |
| Review chưa đủ căn cứ | Ghi phần cần xác minh | Mở rộng nguồn hoặc chuyển người dùng, chưa đoán để sửa. |
| Review đạt và dependency còn phù hợp | Ghi kết luận cho đúng phạm vi/revision | Tiếp tục theo workflow; còn kiểm tra mạch chương nếu chưa làm. |
| Người dùng sửa bản dịch | Lưu revision mới, bảo toàn bản trước | Kiểm tra cấu trúc, Review và xét ảnh hưởng liên quan. |
| Quy tắc đã duyệt thay đổi | Tạo phiên bản quyết định mới | Xác định phần phụ thuộc cần kiểm tra lại. |
| Người dùng sửa file skill đã bật, thay đổi rõ ràng và hợp lệ | Tự lưu phiên bản cùng quyết định do người dùng cung cấp theo cơ chế 2B | Dùng từ lượt tiếp theo phù hợp và xét lại phần phụ thuộc, không yêu cầu xác nhận lại. |
| File skill đổi nhưng chưa rõ hoặc có xung đột | Giữ phiên bản hợp lệ và cả dữ liệu thay đổi, nêu phần đang vướng | Chờ xử lý phần phụ thuộc, không tự quyết bên thắng hoặc trộn phiên bản. |
| Mất kết nối/provider lỗi | Ghi lỗi lần thử và usage nếu xác định được | Retry/resume theo giới hạn, không tự đánh trượt nội dung. |
| Tạm dừng hoặc hủy | Ghi ý định và tiến độ đã commit | Không tiếp tục phần còn lại trái ý định; giữ dữ liệu để xem hoặc tiếp tục phù hợp. |

Các ràng buộc chuyển trạng thái được cụ thể hóa bởi [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), chốt ngày **28/10/2026**: Review ngay sau từng lượt, kiểm tra tổng thể cuối chương rồi mới chuyển chương. Batch là nhóm chương được chọn, không cho phép đợi dịch hết nhóm mới Review. Bổ sung ngày **30/10/2026**, 13A chỉ cho một tác vụ AI hoạt động; tác vụ khác xếp hàng và chỉ được nhận quyền sau khi tác vụ trước kết thúc hoặc bàn giao hợp lệ. Chọn tác vụ khác không xóa dependency, bước chờ, ngân sách hoặc bộ đếm của tác vụ cũ.

Chế độ **Dịch tự động đến hết EPUB** xác lập phạm vi toàn tập một lần và giữ hàng đợi qua các chương, các điểm chờ và lần tiếp tục tác vụ. Việc chuyển chương không cần chọn lại phạm vi và không đặt lại chi phí đã dùng. Các điều kiện Review, duyệt quy tắc, lỗi và ngân sách vẫn quyết định khi nào được tiếp tục.

## 6. Ví dụ revision và Review đến muộn

```text
b012 / r1
  → kiểm tra cấu trúc
  → lưu nháp
  → Review r1 đạt

Người dùng sửa b012
  → tạo r2
  → kiểm tra cấu trúc và Review r2
  → giữ Review r1 trong lịch sử
```

Nếu Reviewer đang kiểm tra `r1` mà `r2` đã được tạo, kết quả trả về vẫn gắn với `r1`. Không tự gán kết luận hoặc áp bản sửa của `r1` lên `r2`.

Trước khi ghi kết quả vào trạng thái hiện hành, core kiểm tra nền revision và dependency đã dùng. Nếu nền đã đổi, giữ kết quả để đối chiếu/xử lý xung đột hoặc chuẩn bị lại; không ghi đè sửa tay.

## 7. Xác định phạm vi ảnh hưởng khi dữ liệu thay đổi

| Thay đổi | Phần phải xem lại |
| --- | --- |
| Sửa một đoạn dịch | Revision đoạn đó, Review cấp chương liên quan và phần sau thực sự đã dùng nội dung đó làm ngữ cảnh. |
| Đổi cách dịch thuật ngữ đã duyệt | Các chỗ cùng nghĩa/định danh và phạm vi áp dụng bị ảnh hưởng; không thay chuỗi hàng loạt theo mặt chữ. |
| Đổi xưng hô | Đúng chiều người nói → người nghe, mốc và ngoại lệ đã chốt. |
| Tạo đề xuất chưa duyệt | Trạng thái đề xuất/chờ xử lý; chưa thay quy tắc chính thức. |
| Sửa skill đang dùng | Tự đồng bộ thay đổi hợp lệ theo 2B; phiên bản skill, quy tắc và kết quả thực sự phụ thuộc cần đánh giá lại. Lượt đang chạy giữ phiên bản đã gửi. |
| Thay ảnh trong `edit` | Bộ dữ liệu preview/export và các vị trí tham chiếu ảnh; thay ảnh thuần túy không bắt dịch lại toàn bộ chữ. |
| Đổi cách thể hiện ruby đã duyệt | Revision và Review của phạm vi thực sự chịu ảnh hưởng, quyết định liên quan và bộ dữ liệu preview/export. |
| Đổi bố cục thuần túy, không đổi nội dung/ngữ nghĩa | Bộ chuẩn bị và kiểm tra hiển thị/cấu trúc; không tự dịch lại phần chữ hoặc phủ nhận Review không bị ảnh hưởng. |
| Chỉ sửa thông tin sách dùng trong thư viện đọc EPUB | Lựa chọn metadata của tập và bộ preview/export; không tự thay tên trong nội dung hoặc mất hiệu lực Review phần chữ không đổi. |
| Thay EPUB nguồn | Snapshot, mapping và các kết quả liên quan trước khi tái sử dụng. |
| Đổi lựa chọn toàn bộ phần chữ/chỉ truyện | Phân loại phần cần dịch/giữ nguyên, hàng đợi, độ phủ và điều kiện xuất; giữ kết quả còn phù hợp, làm mới bộ preview/export liên quan. |

Nếu chưa xác định được ranh giới ảnh hưởng, giữ phần nghi liên quan ở trạng thái cần kiểm tra lại và ghi lý do. Không mặc định mọi tập đều sai, cũng không giả định mọi kết quả vẫn hợp lệ khi chưa kiểm tra.

Đối với thay đổi skill, phân biệt lựa chọn người dùng trực tiếp ghi/lưu trong file đã đăng ký với đề xuất do AI tạo. Cơ chế 2B trong [đặc tả Skills](SKILLS_GLOSSARY_SPEC.md) cho phép áp dụng lựa chọn rõ ràng của người dùng mà không xác nhận lại; không cho AI tự duyệt đề xuất bằng cách ghi vào file. Mẫu riêng, nhận diện nguồn sửa, ID quyết định và giao thức công bố/khôi phục đã chốt trong [đặc tả Skills](SKILLS_GLOSSARY_SPEC.md), mục 12.

## 8. Điều kiện hoàn tất chương và chuẩn bị xuất

Một chương có mọi đoạn đạt cục bộ vẫn cần kiểm tra nhất quán cấp chương theo đặc tả Review. Kết quả này gắn với chính tập hợp revision, nguồn và quy tắc được kiểm tra.

Sửa một đoạn làm kết quả kiểm tra mạch chương cũ cần được xem lại cho tập hợp revision mới; không đồng nghĩa bắt buộc dịch lại toàn chương. Phạm vi kiểm tra/sửa dựa trên ảnh hưởng và rubric.

Chương chỉ được hiển thị đạt hiện hành khi đủ độ phủ, các vấn đề chặn đã được xử lý, không còn quyết định dịch thuật bắt buộc đang chờ và các căn cứ còn phù hợp. Quyết định giữ ngoại lệ được hiển thị riêng.

Preview và EPUB đầu ra dùng cùng bộ dữ liệu chuẩn bị đã xác định revision và hash ảnh. Đổi chữ, ảnh hoặc cấu hình liên quan làm bộ preview cũ cần làm mới theo đặc tả ảnh; không trộn nội dung từ nhiều thời điểm vào cùng lần xuất.

Tiêu đề, nhãn mục lục, chú thích và cách thể hiện ruby có cùng yêu cầu truy vết nguồn/revision/quyết định như phần truyện. Không đánh dấu đã xử lý chỉ vì thân chương đạt; việc ẩn chú âm thuần phát âm ở bản Việt không xóa dữ liệu ruby nguồn. Nếu còn quyết định nghĩa đang chờ, bước chuẩn bị báo đúng phần vướng và trả về Dịch–Review thay vì tự chọn cách thể hiện.

Theo [điều kiện xuất](EPUB_OUTPUT_SPEC.md) bổ sung ngày **29/10/2026**, giai đoạn đầu chỉ xuất EPUB hoàn chỉnh của tập. Trạng thái sẵn sàng được suy ra từ độ phủ, Review hiện hành, quyết định còn hiệu lực và kết quả kiểm tra bộ chuẩn bị; không dùng trạng thái hoàn tất của tác vụ dịch một nhóm chương để thay cho điều kiện toàn tập.

Giao diện phân biệt Sẵn sàng xuất, Còn việc cần xử lý và Sẵn sàng xuất với ngoại lệ. Ngoại lệ gắn với đúng vấn đề/phạm vi/revision, không tự kế thừa khi nội dung hoặc căn cứ đổi; nó không bỏ qua việc thiếu nội dung/Review, quyết định nghĩa đang chờ hoặc lỗi kỹ thuật. S0 và việc chưa thay ảnh không tự chặn xuất.

Thông tin sách theo đặc tả đầu ra gồm tên sách/tập, tác giả, ngôn ngữ và người dịch/nhóm dịch tùy chọn. Lưu riêng lựa chọn của từng tập và căn cứ nguồn/quyết định để có thể dùng lại mà không ghi đè bằng mặc định. Bộ chuẩn bị xác định đúng phiên bản thông tin được dùng; thiếu tên sách/tác giả phải được bổ sung trước khi xuất. Sửa metadata không tự tạo quy ước đổi tên trong nội dung truyện.

Cập nhật ngày **30/10/2026** theo [đặc tả đầu vào](EPUB_INPUT_SPEC.md): độ phủ toàn tập căn cứ lựa chọn nội dung đã lưu. Phần giữ nguyên của chế độ chỉ truyện phải có trạng thái riêng, vẫn tồn tại trong sách và không được gán Đã dịch/Đạt Review. Giữ nguyên theo lựa chọn nội dung không phải ngoại lệ chất lượng; một chương truyện còn thiếu vẫn chặn xuất. Bộ chuẩn bị phải xác định được danh sách/phạm vi giữ nguyên cùng nguồn tương ứng.

Kết quả [Research](RESEARCH_STAGE_SPEC.md) được lưu với bằng chứng và bước cần tiếp tục. Chỉ kết quả còn phù hợp, đã được kiểm chứng theo đúng quy trình mới được dùng làm ngữ cảnh xác nhận; việc thu thập một URL hoặc đề xuất cách dịch chưa tạo một quyết định đã duyệt.

Cùng ngày **30/10/2026**, chốt đầu ra thống nhất EPUB 3 và mỗi lần xuất tạo một file phiên bản mới theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md). Phiên bản file xuất quản lý theo tập, phân biệt với revision/generation; hồ sơ giữ được nguồn, phạm vi, metadata, ảnh và kết quả kiểm tra của mỗi bản. Đổi dữ liệu hiện hành làm bản chuẩn bị cần cập nhật, không tự sửa file EPUB đã xuất. Lỗi xuất hoặc khôi phục sau gián đoạn không được ghi đè bản cũ hoặc công bố file dở dang là thành công.

## 9. Lưu tiến độ và tiếp tục

Ví dụ một chương có 60 block:

```text
20 block: đã đạt, còn phù hợp
10 block: đã lưu nháp, chờ Review
30 block: chưa có bản dịch
Tác vụ: đã tạm dừng
```

Khi tiếp tục, tool dùng lại 20 block đã đạt, Review 10 bản nháp đang chờ trước khi dịch phần tiếp theo theo bước đã ghi. Hết các lượt còn kiểm tra tổng thể chương. Không gửi lại toàn chương chỉ vì task bị dừng.

Nếu đã lưu phản hồi model nhưng chưa kiểm tra hoặc commit, tiếp tục từ bước còn thiếu khi dữ liệu vẫn phù hợp. Nếu đã commit generation nhưng chưa cập nhật bản thảo hiển thị, phục hồi từ dữ liệu đã lưu và bảo vệ sửa tay.

Lần gọi đang chờ lúc mất kết nối có thể đã được provider xử lý. Cần ghi nhận tình trạng chưa rõ và các lần thử tiếp theo; không cam kết mỗi request chỉ phát sinh chi phí đúng một lần.

Đề xuất thuật ngữ và quyết định người dùng đã lưu phải tồn tại qua lần mở lại. Chờ người dùng không bị biến thành tự chấp nhận sau một khoảng thời gian.

### 9.1. Bắt đầu bằng workspace mới, không chuyển workspace thử nghiệm cũ

Chốt ngày **30/10/2026** theo yêu cầu người dùng: workspace cũ chỉ là dữ liệu test, không cần giữ lại hoặc đưa sang bản Rust mới. Quyết định này thay thế yêu cầu giữ tương thích với workspace thử nghiệm cũ trong các tài liệu trước đó; không chọn phương án chuyển sang bản sao hoặc chuyển tại chỗ đã được đề xuất.

- Không xây bộ chuyển đổi, bước sao lưu bắt buộc hoặc cơ chế tiếp tục lịch sử/tiến độ từ workspace thử nghiệm cũ để đáp ứng lần chuyển sang Rust này.
- Cho phép thiết kế schema và bố trí dữ liệu nội bộ mới phù hợp sản phẩm. Các ranh giới nghiệp vụ đã chốt vẫn giữ: snapshot nguồn, revision bản dịch, quyết định có phiên bản, dependency, Review và tiến độ tác vụ.
- Bản Rust bắt đầu từ workspace mới và import EPUB theo luồng bình thường. Nếu người dùng đăng ký lại EPUB hoặc file skill có sẵn, đó là đầu vào cho workspace mới; không suy ra đã tiếp nhận lịch sử dịch/Review từ workspace cũ.
- Khi được trỏ tới workspace cũ/định dạng không hỗ trợ, báo cần tạo workspace mới, không diễn giải thử dữ liệu cũ thành dữ liệu hợp lệ. Mở/import không tự xóa hoặc ghi đè một thư mục đã tồn tại.

Đây là quyết định phạm vi thiết kế; lần cập nhật tài liệu này không thực hiện xóa thư mục hay dữ liệu trên máy. Việc không cần giữ workspace test không áp dụng cho EPUB/skill nguồn bên ngoài hoặc cho dữ liệu người dùng tạo trong workspace mới.

Workspace mới cần có phiên bản định dạng lưu trữ để nhận diện khi mở. Phiên bản này khác snapshot, revision, generation, số lần gọi và phiên bản EPUB xuất. Dữ liệu mới vẫn phải được lưu nhất quán, bảo vệ sửa tay/lịch sử và tiếp tục được sau gián đoạn theo các mục trên. Nâng cấp workspace Rust theo thứ tự phiên bản, khóa ghi, sao lưu dữ liệu và các file sẽ sửa, kiểm tra toàn vẹn rồi mở lại. Lỗi nâng cấp phục hồi mốc trước theo mục 11.

Chốt ngày **04/11/2026** dùng SQLite kết hợp file, schema khởi đầu 1 và cơ chế transaction/nâng cấp tại mục 11. PR triển khai cần mô tả rõ schema mới, điểm không tương thích và cách bắt đầu workspace mới, cập nhật tài liệu sử dụng khi có thay đổi thực tế. Tiếp tục giữ đường chạy Go hoạt động đến khi phần thay thế đạt yêu cầu; bỏ tương thích workspace test không đồng nghĩa xóa implementation Go ngay.

## 10. Tiêu chí nghiệm thu

Kiểm thử triển khai dùng dữ liệu tự tạo/model giả lập, offline và deterministic; không gọi provider trả phí trong CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Chia lại lượt dịch | Giữ ID, thứ tự, coverage nguồn và lịch sử các kết quả hợp lệ. |
| Người dùng sửa một bản đã đạt | Tạo revision mới; không tự kế thừa kết luận Review cũ. |
| Review cũ trả về sau khi đã sửa | Gắn kết quả đúng revision, không ghi đè hiện hành. |
| Quy tắc chỉ đổi cho một cặp/mốc | Chỉ đánh giá lại phạm vi liên quan, giữ các phần ngoài phạm vi. |
| Có đề xuất chưa duyệt | Lưu trạng thái chờ, không dùng phương án mới làm bản nháp hiện hành hoặc quy tắc chính thức. |
| Lỗi provider/tạm dừng | Không tự đổi chất lượng của các kết quả đã lưu còn phù hợp. |
| Đóng/mở lại | Phục hồi tiến độ, đề xuất, quyết định và bước tiếp theo. |
| Bắt đầu dùng bản Rust với EPUB nguồn | Tạo workspace mới và import; không yêu cầu chuyển/sao lưu workspace thử nghiệm cũ. |
| Mở workspace cũ hoặc định dạng không hỗ trợ | Báo rõ cần tạo workspace mới, không âm thầm sửa/xóa hoặc coi dữ liệu cũ là hợp lệ. |
| Workspace mới bị gián đoạn hoặc được mở lại | Giữ revision/sửa tay/quyết định/chi phí/bộ đếm, không dùng ngoại lệ workspace test để bỏ dữ liệu mới. |
| Chỉ thay ảnh | Làm mới preview/export, không tự kích hoạt dịch lại toàn bộ chữ. |
| Quyết định thể hiện ruby thay đổi | Xem lại phạm vi/revision phụ thuộc và làm mới bản chuẩn bị; giữ nguyên bằng chứng nguồn. |
| Chỉ thay bố cục thuần túy | Kiểm tra hiển thị/cấu trúc của bản chuẩn bị mới; không tự gọi lại model dịch. |
| Mọi đoạn đạt nhưng chưa kiểm tra mạch chương | Chương chưa được ghi nhận đạt đầy đủ. |
| Generation đã commit, bản thảo chưa đồng bộ | Có thể phục hồi hiển thị mà không phải gọi model lại. |
| Thay nguồn hoặc phát sinh sửa tay khi đang chạy | Phát hiện nền thay đổi và giữ lịch sử để xử lý xung đột. |

## 11. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [06](DESIGN_DECISIONS_36.md#decision-06), [07](DESIGN_DECISIONS_36.md#decision-07), [08](DESIGN_DECISIONS_36.md#decision-08), [09](DESIGN_DECISIONS_36.md#decision-09), [10](DESIGN_DECISIONS_36.md#decision-10) của bản 36 mục áp dụng cho đặc tả này.

### 11.1. Lựa chọn hệ thống lưu trữ workspace — mục 06

Dùng **một SQLite cho mỗi bộ truyện**, mọi tập của bộ cùng tham chiếu nhân vật, glossary và lịch sử. SQLite lưu mapping, revision, quyết định, dependency, Review, tiến độ, sổ lần gọi và hồ sơ xuất. EPUB nguồn, ảnh gốc/ảnh edit, file Skill, bản thảo Markdown và EPUB xuất là file trên đĩa. Dữ liệu bất biến có hash để đối chiếu. Hàng đợi dùng chung theo hồ sơ người dùng có SQLite riêng ở mục 28. Key thuộc kho thông tin xác thực của hệ điều hành hoặc nguồn môi trường đã chọn.

Workspace Rust mới bắt đầu ở schema 1. Nâng cấp sau này khóa ghi, tạo bản sao dữ liệu chính và những file sẽ bị sửa, chuyển từng phiên bản, kiểm tra toàn vẹn rồi mở lại. Lỗi nâng cấp phục hồi mốc trước. Workspace test Go cũ được xử lý bằng luồng tạo workspace mới và import EPUB theo quyết định đã chốt. [Đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md)

### 11.2. Cơ chế Generation và Atomic Commit — mục 07

Mỗi generation lưu ID cha, các tham chiếu thay đổi và dấu kiểm tra. Revision, tham chiếu Review, quyết định, dependency, tiến độ, metadata và con trỏ hiện hành được công bố trong **một giao dịch SQLite**. Lưu bảng trạng thái hiện hành để đọc nhanh và lịch sử thay đổi để truy ngược. Khi có file bất biến mới, ghi vào file tạm, hoàn tất/kiểm tra hash và đặt vào kho trước khi commit tham chiếu. File chưa được tham chiếu có thể thu gom sau. [Giao dịch của SQLite](https://www.sqlite.org/atomiccommit.html)

Markdown hiển thị được dựng sau commit và có bản ghi việc cần hoàn thành. Nếu đĩa đầy ở bước này, báo **Đã lưu bản dịch, chờ dựng Markdown**. Mở lại đọc generation đã commit, so hash bản thảo, bảo vệ sửa tay và tiếp tục dựng file. File Skill do người dùng chỉnh có quy trình công bố riêng ở mục 13–15.

### 11.3. Đồng bộ bản thảo Markdown và revision — mục 08

Khi dựng Markdown, lưu revision nền, hash và marker từng đơn vị. Theo dõi file ngoài và kiểm tra lại hash khi mở, trước khi ghi và trước khi tiếp tục tác vụ. Sau một lần lưu hoàn chỉnh, nếu marker hợp lệ, nhận diện được đơn vị và nền còn phù hợp, nhập phần thay đổi thành **revision sửa tay**, ghi người sửa và chuyển đúng phần sang chờ Review.

Khi thiếu/trùng marker, giữ nguyên file đã sửa, chỉ ra vùng mất liên kết và cho gắn lại đơn vị qua editor. Nếu AI cũng sửa cùng đoạn, mở so sánh **bản nền / bản người dùng / ứng viên AI**. Ghép tự động phần sửa khác đơn vị khi nền khớp. Phần sửa chồng nhau cần người dùng chọn hoặc tự kết hợp. Giữ lịch sử cả hai và Review revision được chọn.

### 11.4. Quản lý Dependency và Invalidation — mục 09

Mỗi revision dịch ghi các phạm vi nguồn, toàn bộ phiên bản quy tắc/ngữ cảnh được cung cấp trong request và cặp xưng hô/mốc cảnh. Lưu quan hệ ngược từ từng quyết định/sự kiện tới các đơn vị áp dụng, cùng chỉ mục lần xuất hiện của thuật ngữ và nhân vật. Phần AI khai báo đã sử dụng là thông tin bổ sung. Khi chưa chứng minh được quan hệ chỉ ảnh hưởng một đoạn, giữ phạm vi kiểm tra của cả lượt đã nhận ngữ cảnh ấy. Kết luận cấp chương tham chiếu tập hợp revision của chương.

Khi một căn cứ đổi, đánh dấu **Cần kiểm tra lại** ở những đơn vị phụ thuộc và đi tiếp theo chuỗi sử dụng cho đến khi hết ảnh hưởng. Quy tắc mới được đối chiếu thêm với chỉ mục lần xuất hiện và phạm vi hiệu lực để tìm các bản dịch trước khi quy tắc tồn tại. Review có thể xác nhận bản dịch hiện tại vẫn phù hợp. Nếu sai, dùng đúng phạm vi M1–M4. Nguồn thay đổi đến mức mất đối chiếu thì lập mapping mới cho phần đó. Ví dụ, bác một sự kiện ở chương 2 sẽ mở lại cảnh của chương 4 từng dùng sự kiện ấy làm căn cứ.

### 11.5. Quản lý trạng thái ở ba cấp — mục 10

Core quản lý ba trục độc lập. **Chất lượng** gắn với bản dịch và có các giá trị chưa dịch, chờ Review, cần sửa, cần xác minh, đạt, người dùng giữ ngoại lệ. **Hiệu lực** gắn với bộ dữ liệu đã dùng, gồm còn phù hợp hoặc cần kiểm tra lại. **Thực thi** gắn với tác vụ, gồm xếp hàng, chạy, đang tạm dừng, tạm dừng, chờ người dùng, lỗi, hoàn tất và hủy. TUI có thể hiện đồng thời “đã đạt ở r2”, “cần kiểm tra do đổi glossary” và “tác vụ tạm dừng”.

Mọi chuyển trạng thái đi qua bảng sự kiện của core. Lưu nháp hợp lệ dẫn đến chờ Review. Chỉ Review đủ bằng chứng và độ phủ mới xác nhận đạt. Đổi căn cứ hạ hiệu lực của kết luận cũ. Tiếp tục chỉ được gửi AI khi đủ quyền, ngân sách và dữ liệu. Chuyển trái điều kiện trả lý do và giữ lịch sử sự kiện trước đó. [Các trục trạng thái](DATA_MODEL_STATE_SPEC.md)

### 11.6. Liên kết và yêu cầu triển khai

Câu lệnh tạo bảng, kiểu Rust và mã migration được hiện thực theo các hợp đồng trên. Sổ giá/attempt theo [chiến lược model](MODEL_STRATEGY_SPEC.md), công bố nhóm quyết định theo [Skills](SKILLS_GLOSSARY_SPEC.md), hàng đợi chung theo [điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), bộ chuẩn bị theo [đầu ra EPUB](EPUB_OUTPUT_SPEC.md).
