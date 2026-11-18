# Quyết định 12 — Research, tra cứu nguồn và bàn giao cho Dịch–Review

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 8.

Ngày ghi nhận: **30/10/2026**.

Trạng thái: **đã chốt vai trò Research được tra cứu Internet**. Tài liệu cụ thể hóa ranh giới với Dịch, Review và duyệt thuật ngữ dựa trên các quyết định đã có; không khẳng định chức năng đã được triển khai. Công cụ tìm kiếm, giới hạn riêng, cache và hợp đồng bằng chứng đã chốt ngày **04/11/2026** tại mục 8.

Tài liệu liên kết [Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [Skills](SKILLS_GLOSSARY_SPEC.md), [model và ngân sách](MODEL_STRATEGY_SPEC.md), [điều phối](WORKFLOW_ORCHESTRATION_SPEC.md) và [dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md).

## 1. Vai trò và quyền tra cứu

**Research phụ trách tìm và đối chiếu thông tin tham khảo, được phép dùng Internet khi cần.** Đây là vai trò hỗ trợ thuật ngữ/tra cứu đã có trong thiết kế, trước đây còn được gọi là Search; không thêm một vai trò model thứ tư.

| Thành phần | Trách nhiệm |
| --- | --- |
| Dịch | Đọc nguồn/ngữ cảnh được giao, tạo bản dịch; nêu câu hỏi cần Research khi thiếu căn cứ. |
| Review | Đọc nguyên tác, đối chiếu và kiểm chứng; yêu cầu Research khi cần tra cứu bổ sung. |
| Research | Tra nguyên tác/tài liệu đã có và Internet khi cần; trả bằng chứng, phạm vi dùng được, điểm chưa rõ và đề xuất. |
| Người dùng | Chốt thuật ngữ/quy ước mới, thay đổi lựa chọn đã duyệt và xử lý các điểm chưa xác định cần quyết định. |
| Bộ điều phối | Gọi đúng vai trò, giữ bước đang chờ, kiểm tra phiên bản và bàn giao kết quả. |

Dịch và Review vẫn được đọc nguồn/ngữ cảnh để thực hiện công việc của mình. Nhu cầu tìm kiếm thông tin bên ngoài được bàn giao cho Research; không coi model Dịch/Review đang tự duyệt web như một bước tra cứu không được ghi nhận.

Research mặc định có thể dùng cùng model với Review theo cấu hình đã chốt, hoặc model riêng do người dùng chọn. Dùng chung model không gộp vai trò, quyền công cụ, request hoặc lịch sử. Chuyển công việc sang Research không phải tự đổi model dự phòng.

## 2. Khi nào cần Research

Research được gọi khi Dịch/Review thiếu thông tin cần thiết, ví dụ nghĩa/định danh thuật ngữ, cách đọc tên, quy ước đã dùng, tham chiếu văn hóa hoặc ngữ cảnh còn mâu thuẫn. Người dùng cũng có thể yêu cầu hỗ trợ đề xuất như cách dịch tên sách.

Không chạy Research cho mọi block hoặc gọi Internet chỉ vì đang import/xem nguồn. Khi dữ liệu đã lưu còn phù hợp và đủ trả lời, dùng lại; việc tra cứu không thay thế đọc nguyên tác trong Review.

Yêu cầu cần chỉ rõ câu hỏi, nguyên ngữ, vị trí và snapshot nguồn, ngữ cảnh liên quan, quy tắc đã có, mốc truyện và bước đang cần kết quả. Bản dịch đang bị đánh giá có thể là đối tượng cần kiểm chứng, không được xem như bằng chứng sự kiện nguyên tác. Nếu Research hỗ trợ lượt đọc nguồn của Review, tiếp tục giữ ranh giới nguồn với bản dịch đang bị đánh giá.

## 3. Thứ tự và nguồn tra cứu

1. Đối chiếu nguyên tác tại vị trí cần xử lý và phần ngữ cảnh nguồn liên quan.
2. Tra Skills/glossary, quyết định và các kết quả đã được chấp nhận còn hiệu lực trong workspace, đúng phạm vi và thời điểm.
3. Khi vẫn cần bổ sung, Research tìm trên Internet và các tài liệu tham khảo đã cấu hình, tuân thủ giới hạn nguồn người dùng đã thiết lập nếu có.

Chỉ gửi nội dung cần thiết cho câu hỏi tra cứu. Nguồn trên Internet phải được lưu với đường dẫn, phần bằng chứng thực sự đọc được và thông tin truy vết phù hợp. Không tạo URL hoặc trích dẫn tưởng tượng; nếu chỉ có thông tin chưa kiểm chứng hoặc không truy cập được nguồn, phải thể hiện đúng giới hạn đó.

Quyền tra cứu Internet đã được chốt cho vai trò Research; không yêu cầu người dùng xác nhận lại chỉ vì tác vụ hợp lệ cần một lần tra cứu. Tác vụ vẫn chịu cấu hình nguồn, ngân sách, quyền truy cập thực tế và giới hạn xử lý lỗi.

## 4. Bằng chứng, mâu thuẫn và thông tin ở tương lai

Nguyên tác của EPUB đang xử lý là căn cứ về nội dung cần dịch. Kết quả từ website, wiki hoặc bản chuyển thể chỉ là tham khảo; Research phải phân biệt nguồn, phiên bản tác phẩm và mốc liên quan nếu xác định được. Không dùng một trang ngoài để tự sửa diễn biến nguyên tác.

Giữ rõ ranh giới giữa điều nguồn thực sự nêu, suy luận của Research và lựa chọn diễn đạt tiếng Việt. Nhiều trang lặp lại một thông tin hoặc model tự tin không tự chứng minh thông tin đó đúng.

Thông tin từ tập/chương sau không tự trở thành dữ kiện mà cảnh đang dịch đã tiết lộ. Kết quả cần gắn mốc/phạm vi có căn cứ; dữ kiện tương lai hoặc chưa rõ mốc không được đưa vào bản dịch/ngữ cảnh hiện hành như đã xác nhận. Website có thể chứa spoiler; bộ lọc phạm vi không được xem là bảo đảm mọi nội dung ngoài đều không có spoiler.

Nếu nguồn tham khảo mâu thuẫn với nguyên tác, quy ước hoặc với nhau, ghi rõ bất đồng và bằng chứng. Không đủ căn cứ thì trả **Chưa xác định/Cần xác minh**, giữ việc phụ thuộc chờ xử lý; không bịa một kết luận để cho tác vụ chạy tiếp.

## 5. Kết quả bàn giao

| Thành phần | Nội dung cần có |
| --- | --- |
| Câu hỏi và phạm vi | Vấn đề đã tra, vị trí nguồn và bước yêu cầu. |
| Kết quả | Điều tìm được, điều chưa rõ và phần chỉ là suy luận. |
| Bằng chứng | Nguồn nội bộ có vị trí/phiên bản; nguồn ngoài có URL và đoạn thông tin hỗ trợ kết luận. |
| Mốc sử dụng | Tập/chương/cảnh hoặc phạm vi đã xác định; thông tin tương lai/chưa rõ được đánh dấu riêng. |
| Quy ước liên quan | Quyết định đang có hiệu lực, xung đột hoặc nhu cầu tạo đề xuất mới. |
| Phương án đề xuất | Các cách dịch/giải thích có căn cứ nếu cần, chưa phải lựa chọn đã được duyệt. |
| Lịch sử thực hiện | Vai trò Research, model thực tế, các lần gọi, kết quả/lỗi và usage/chi phí nếu xác định được. |

Kết quả Research chưa tự trở thành ngữ cảnh xác nhận, glossary chính thức hoặc kết luận Review đạt. Core kiểm tra tham chiếu/phiên bản, Dịch/Review đối chiếu căn cứ với nguồn, và lựa chọn thuật ngữ/quy ước mới vẫn chờ người dùng theo đặc tả Skills.

## 6. Trở lại luồng đang xử lý

```text
Dịch hoặc Review cần thêm căn cứ
  → lưu câu hỏi và bước đang chờ
  → Research tra nguồn đã có, Internet khi cần
  → trả bằng chứng và phần chưa rõ
  → kiểm tra phạm vi, phiên bản và căn cứ
  → nếu có lựa chọn mới: trình người dùng duyệt
  → tiếp tục Dịch hoặc Review tại bước đang chờ
```

Không bắt đầu dịch chương sau để bỏ qua vấn đề Research đang cần giải quyết. Research trả kết quả không tự hoàn tất lượt/chương; các điều kiện Review và kiểm tra tổng thể vẫn áp dụng.

Theo **13A** ngày **30/10/2026** trong [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), Research được bàn giao là bước của cùng tác vụ đang hoạt động, không phải một tác vụ AI nền được chạy song song. Một yêu cầu Research riêng cũng phải chờ quyền chạy chung; không dùng vai trò tra cứu để chạy AI cho tác vụ B trong lúc tác vụ A đang hoạt động. Các lần gọi và chi phí luôn gắn đúng tác vụ đã giao yêu cầu.

Các lần gọi model Research thuộc ngân sách của tác vụ gốc và tuân theo chiến lược model: không tự đổi model, báo lỗi, giữ tiến độ và dùng đúng giới hạn đã chốt. Theo 9A ngày **30/10/2026**, mỗi lần gọi model mặc định có thời hạn 10 phút, retry lỗi tạm thời chờ 10/30 giây và tính vào trần tổng số lần gọi của tác vụ nếu được bật. Bàn giao sang Research không tạo lại bộ đếm. Việc chưa tìm được căn cứ không phải quyền lặp tìm kiếm vô hạn. Giới hạn riêng về số lần tìm kiếm/thời gian tra cứu, gồm các lần dùng công cụ ngoài model, sẽ được cụ thể hóa; không tự coi mỗi lần tra cứu là một vòng sửa chất lượng.

Trong lúc chạy hiển thị **Research + tên/ID model + câu hỏi/phạm vi đang tra**; dữ liệu kết quả và lịch sử truy vết không chứa khóa API. Khi nguồn/quy tắc đã đổi, kiểm tra lại tính phù hợp trước khi tái sử dụng kết quả. Dừng, mở lại hoặc Research lỗi không tự làm mất các bản dịch đã lưu hợp lệ.

## 7. Tiêu chí nghiệm thu khi triển khai

Kiểm thử tự động dùng nguồn/model giả lập, offline và deterministic; không tìm web trực tiếp hoặc gọi model trả phí trong CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Dịch/Review cần tìm thông tin ngoài | Bàn giao câu hỏi cho Research, có bước đang chờ và lịch sử vai trò rõ ràng. |
| Research được phép tra Internet | Dùng đúng quyền/cấu hình nguồn của tác vụ, không hỏi lại từng lần tra cứu hợp lệ. |
| Dữ liệu đã có đủ và còn phù hợp | Tái sử dụng đúng phạm vi, không bắt buộc thêm tìm web. |
| Nguồn không truy cập được hoặc không có căn cứ | Báo đúng giới hạn/chưa xác định, không bịa URL, quote hoặc kết luận. |
| Website mâu thuẫn với EPUB | Giữ nguồn/bằng chứng và vấn đề xác minh, không tự sửa nguyên tác theo website. |
| Kết quả thuộc tương lai hoặc chưa rõ mốc | Không tự dùng như sự thật đã được cảnh hiện tại xác nhận. |
| Có phương án thuật ngữ mới | Chờ người dùng chốt, chưa đưa vào bản nháp hiện hành hoặc skill chính thức. |
| Research và Review dùng cùng model | Vai trò/request/lịch sử vẫn tách đúng, màn chạy hiển thị Research. |
| Research lỗi, hết ngân sách hoặc nguồn đã đổi | Giữ tiến độ, xử lý giới hạn/phiên bản theo chính sách hiện hành; không tự đổi model. |
| Kết quả Research đã trả | Tiếp tục bước Dịch/Review còn thiếu, không tự đánh dấu chương đạt. |

## 8. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [26](DESIGN_DECISIONS_36.md#decision-26), [27](DESIGN_DECISIONS_36.md#decision-27) của bản 36 mục áp dụng cho đặc tả này.

### 8.1. Công cụ tìm kiếm và giới hạn Research — mục 26

Tra nguyên tác, glossary và kết quả đã xác nhận trước. Khi cần web, dùng công cụ tìm URL và công cụ đọc trang riêng. Adapter web đầu tiên là **Brave Search API** do người dùng cấu hình. Cho đặt danh sách nguồn ưu tiên hoặc chặn theo bộ. Chỉ trích dẫn một trang khi đã đọc được đoạn có liên quan. [Brave Web Search](https://api-dashboard.search.brave.com/app/documentation/web-search/get-started) · [Đặc tả Research](RESEARCH_STAGE_SPEC.md)

*Mặc định thử nghiệm cho một câu hỏi* là **3 truy vấn web, 5 trang đọc, 15 phút tổng cộng**, cho phép chỉnh. Hết giới hạn thì trả bằng chứng đã có và điểm còn thiếu. Thời hạn có hiệu lực cho lần gọi model là phần nhỏ hơn giữa thời gian model được phép và thời gian Research còn lại. Phí dịch vụ tìm kiếm/đọc trang được dự trù và ghi vào cùng tác vụ ở dòng riêng. Mỗi lần gọi model Research vẫn tính vào trần model.

Cache theo câu hỏi, thực thể, mốc truyện, phạm vi nguồn, cấu hình nguồn và phiên bản những căn cứ đã dùng. Dữ liệu nội bộ dùng lại khi các căn cứ còn hiệu lực. Với nguồn web, kiểm tra lại sau 7 ngày và sớm hơn khi có mâu thuẫn hoặc nguồn đổi. Lưu URL, thời điểm, trích đoạn cần thiết và kết luận có nguồn. Việc lưu câu trả lời thô từ dịch vụ tìm kiếm phụ thuộc quyền lưu của kết nối.

### 8.2. Bằng chứng Research và giới hạn spoiler — mục 27

Mỗi dữ kiện có nguồn, trích đoạn, vị trí/URL, hash đoạn đọc, thời điểm lấy, thực thể, thời điểm sự kiện, **mốc sớm nhất độc giả được biết** và trạng thái xác minh. Đánh giá từng dữ kiện trong một trang. Một trang wiki có thể có cả chi tiết dùng được lẫn chi tiết chỉ xuất hiện ở tập sau. Khi thiếu căn cứ xác định mốc tiết lộ, giữ thông tin ở hồ sơ Research với trạng thái **Chưa rõ mốc** và loại khỏi gói ngữ cảnh dùng cho cảnh hiện tại.

Khi nguồn đổi hoặc biến mất, giữ trích đoạn đã ghi nhận kèm thời điểm, đánh dấu cần kiểm chứng lại và ưu tiên đối chiếu nguyên tác. Chỉ dùng xuyên tập khi cùng nghĩa/thực thể, đúng mốc tiết lộ, đúng phạm vi và nguồn còn đủ căn cứ. Kết quả tìm thấy một cách gọi mới đi qua bước duyệt thuật ngữ như các nguồn đề xuất khác.
