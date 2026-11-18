# Quyết định 04 — Review bản dịch, xử lý sai lệch và chuyển về phần Dịch

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 13.

Ngày ghi nhận: **25/10/2026**.

Cập nhật ngày **27/10/2026**: bổ sung ràng buộc [mô hình dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md) và [Skills, duyệt thuật ngữ/quy ước](SKILLS_GLOSSARY_SPEC.md). Các đề xuất lựa chọn dịch thuật mới hoặc thay đổi quy tắc đã chốt phải chờ người dùng duyệt.

Cập nhật ngày **28/10/2026**: các lần gọi Review tuân theo [chiến lược model](MODEL_STRATEGY_SPEC.md), không tự đổi model; giới hạn lỗi dịch vụ/cấu trúc tách riêng với giới hạn 5 vòng sửa chất lượng.

Cập nhật ngày **29/10/2026**: đối chiếu tiêu đề/mục lục/chú thích và cả hai lớp ruby theo [đặc tả EPUB đầu ra](EPUB_OUTPUT_SPEC.md); các lựa chọn liên quan đến nghĩa phải được giải quyết trước khi ghép EPUB.

Trạng thái: **ghi nhận thiết kế đã thống nhất đến thời điểm hiện tại**. Đây là đặc tả hành vi cần triển khai, không phải mô tả chức năng đã hoàn thành trong Go hoặc Rust. Tài liệu được viết ở giai đoạn thiết kế, không dựa trên việc kiểm tra mã nguồn trong lần ghi nhận này.

Tài liệu tiếp nối:

- [Xử lý EPUB đầu vào, cấu trúc nội dung và hai giới hạn dung lượng](EPUB_INPUT_SPEC.md).
- [Giai đoạn Dịch: từ nhận lệnh đến lưu bản nháp](TRANSLATION_STAGE_SPEC.md).
- [Xưng hô xuyên tập và đề xuất thay đổi có ngữ cảnh](ADDRESSING_CONTINUITY_SPEC.md).

Phạm vi gồm: nhận bản nháp; đọc nguyên tác; đối chiếu bản dịch; trình bày bằng chứng trên màn Review; chọn phạm vi sửa; chuyển việc sửa về phần Dịch; review lại; chấp nhận hoặc chuyển cho người dùng xử lý. Bao gồm cả lỗi cục bộ, sai mạch truyện cả chương và thêm nội dung không được nguyên tác hỗ trợ.

Hợp đồng đọc nguồn, issue, bằng chứng và Review chương đã chốt tại mục 13. Lưu đúng revision, phạm vi và căn cứ trong SQLite theo [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md). Workspace Rust mới có nâng cấp, giữ sửa tay và lịch sử, bắt đầu schema 1. Quyết định bỏ chuyển workspace test cũ ngày **30/10/2026** tiếp tục có hiệu lực.

## 1. Mục tiêu và trách nhiệm

Review phải giúp người dùng biết bản dịch lệch nguyên tác ở đâu, căn cứ nào hỗ trợ kết luận, phạm vi nào bị ảnh hưởng và tool sẽ xử lý như thế nào. Việc sửa cần được điều phối trong cùng tác vụ; người dùng không phải bấm Dịch lại sau mỗi lỗi đã xác nhận.

Luồng sửa tự động được dùng trong phạm vi nguồn và quy ước đã duyệt. Nếu phát sinh lựa chọn thuật ngữ/xưng hô/quy ước mới cần người dùng chốt, phần phụ thuộc chuyển sang chờ duyệt theo đặc tả Skills; Reviewer không tự thay quy tắc để tiếp tục vòng sửa.

| Thành phần | Trách nhiệm |
| --- | --- |
| Core | Chọn đúng nguồn/revision, lập phạm vi, kiểm tra bằng chứng và kết quả, điều phối sửa, quản lý lịch sử và trạng thái |
| Reviewer | Đọc nguồn, đối chiếu ý nghĩa, nêu vấn đề có căn cứ và hướng khắc phục |
| Phần Dịch | Nhận yêu cầu sửa hoặc dịch lại, tạo bản mới từ nguồn và ngữ cảnh phù hợp |
| TUI | Hiển thị tiến độ, nguyên tác, bản dịch, bằng chứng, quyết định và thao tác của người dùng |
| Người dùng | Duyệt lựa chọn thuật ngữ/quy ước mới, giải quyết xung đột, xử lý trường hợp chưa đủ căn cứ, sửa trực tiếp hoặc giữ bản dịch có ngoại lệ |

Reviewer và phần Dịch là hai vai trò trong workflow. Thiết kế không bắt buộc phải dùng hai provider hoặc hai model khác nhau. Lượt kiểm chứng bổ sung cũng không tự bảo đảm kết luận đúng chỉ vì do một model khác thực hiện.

### 1.1. Hiểu đúng yêu cầu 100%

Không thể cam kết AI hiểu đúng nguyên tác 100%. Việc gửi đủ văn bản, nhận được một bản tóm tắt hoặc chạy thêm một lượt review không chứng minh khả năng hiểu tuyệt đối.

Các điều kiện hệ thống cần kiểm soát được là:

- Mọi phạm vi nội dung cần dịch đều có nguồn tương ứng và trạng thái kiểm tra rõ ràng.
- Request kiểm tra một phạm vi phải chứa nguyên văn nguồn và bản dịch của phạm vi đó, cùng ngữ cảnh bắt buộc.
- Kết quả phải gắn với đúng phiên bản nguồn, bản dịch và quy tắc liên quan.
- Kết luận về lỗi phải có bằng chứng và giải thích; bằng chứng phải mở lại được.
- Không đánh dấu chương đã review đầy đủ nếu còn phần chưa đối chiếu hoặc còn kết quả không hợp lệ.
- Khi chưa đủ căn cứ, giữ trạng thái **Cần xác minh**, chưa tự sửa theo một cách hiểu phỏng đoán.

Kiểm tra bằng chương trình chứng minh được tính đầy đủ của phạm vi và tính hợp lệ của trích dẫn theo hợp đồng dữ liệu. Kiểm tra đó không chứng minh Reviewer đã hiểu đúng mọi câu.

### 1.2. Bảy phần của giai đoạn Review

| Phần | Đầu ra chính |
| --- | --- |
| 1. Nhận nguyên tác và bản nháp | Phạm vi review và phiên bản dữ liệu đã xác định |
| 2. Đọc nguyên tác | Ghi nhận sự kiện, chủ thể, thời gian, quan hệ và điểm chưa rõ, có dẫn nguồn |
| 3. Đối chiếu hai chiều và mạch chương | Kết quả kiểm tra, vấn đề và phạm vi ảnh hưởng |
| 4. Hiển thị kết quả có bằng chứng | Thẻ lỗi, kết luận cấp chương và thao tác xử lý |
| 5. Chọn phạm vi sửa | Sửa đoạn, dịch lại cảnh, dịch lại chương hoặc xác minh thêm |
| 6. Chuyển về phần Dịch | Bản nháp mới đã qua kiểm tra cấu trúc, chờ review lại |
| 7. Chấp nhận hoặc chuyển cho người dùng | Quyết định được lưu, tiến độ và ngữ cảnh được cập nhật đúng phạm vi |

## 2. Phần 1 — Nhận nguyên tác và bản nháp

Đầu vào là bản nháp đã lưu và hợp lệ về cấu trúc theo đặc tả giai đoạn Dịch. Hợp lệ về cấu trúc chưa đồng nghĩa đã đúng nghĩa hoặc đủ ý.

Core cần xác định:

- Bộ truyện, tập, chương và danh sách block/phạm vi con được giao review.
- Snapshot nguồn, mapping và hash nguồn do chương trình quản lý.
- Revision/hash của bản dịch cần review; nếu chương gồm nhiều revision thành phần, lưu tập phiên bản tương ứng.
- Ngữ cảnh nguồn liên quan, glossary và quy tắc xưng hô đã xác nhận, còn hiệu lực tại vị trí truyện.
- Các điểm chưa rõ, đề xuất ngữ cảnh và phản hồi người dùng cần xem xét; giữ rõ trạng thái chưa xác nhận.
- Những phần đã kiểm tra còn hợp lệ và những phần chưa kiểm tra.

Phạm vi Review theo lựa chọn nội dung ở [đặc tả đầu vào](EPUB_INPUT_SPEC.md), cập nhật ngày **30/10/2026**. Chế độ toàn bộ phần chữ bao gồm cả các trang phụ; chế độ chỉ truyện chỉ bỏ yêu cầu Review bản dịch đối với những phần được phân loại giữ nguyên theo nguồn. Phần giữ nguyên vẫn cần đúng mapping/tài nguyên, không mang nhãn Đạt Review. Chưa rõ loại nội dung không phải lý do tự loại khỏi độ phủ.

Ví dụ: kết quả review của đoạn `b018`, bản nháp `r1`, chỉ có hiệu lực với nguồn, bản dịch và dependency đã ghi nhận. Nếu người dùng sửa thành `r2` trong lúc Reviewer đang chạy, kết quả cho `r1` được giữ trong lịch sử nhưng chưa được áp dụng như kết quả của `r2`.

Nếu thiếu nguồn, mapping hỏng, revision không còn phù hợp hoặc có xung đột sửa tay, core báo đúng nguyên nhân. Không yêu cầu model tự tái tạo nguyên tác còn thiếu từ bản dịch.

### 2.1. Chương dài và phạm vi kiểm tra

Chương vẫn là đơn vị người dùng chọn. Core có thể chia việc đọc nguồn và đối chiếu thành các phần theo hai giới hạn trong đặc tả EPUB:

1. Giới hạn kỹ thuật của request hoàn chỉnh, gồm nguồn, bản dịch, hướng dẫn, ngữ cảnh, bằng chứng và dung lượng đầu ra dự kiến.
2. Giới hạn vận hành về lượng nội dung xử lý trong một lượt, được cấu hình và hiệu chỉnh khi triển khai.

Mốc khoảng 3.000 token nguồn/lượt trong đặc tả EPUB là mức thử nghiệm. Không coi đó là bảo đảm một request Review sẽ vừa, vì Review còn chứa bản dịch và phần giải thích. Phải tính lại request thực tế; nếu thiếu chỗ, thu nhỏ phạm vi thay vì cắt bỏ nguồn bắt buộc.

Mỗi lượt phân biệt phạm vi cần kiểm tra với phần ngữ cảnh chỉ đọc. Ngữ cảnh có thể xuất hiện ở nhiều lượt, nhưng không vì vậy mà được tính là đã hoàn tất đối chiếu.

Core theo dõi riêng phạm vi đã đọc nguồn, đã đối chiếu nguồn → dịch, đã đối chiếu dịch → nguồn và phần cần kiểm tra liên kết giữa các cảnh. Kết quả từng phần có thể xuất hiện sớm trên TUI; trạng thái đạt của từng phần chưa phải trạng thái đạt của cả chương.

## 3. Phần 2 — Đọc nguyên tác trước khi đối chiếu

### 3.1. Căn cứ phải hình thành từ nguồn

Lượt đọc nguồn dùng nguyên tác và ngữ cảnh đã xác nhận có liên quan. Bản dịch đang bị đánh giá không được dùng làm căn cứ để xác định sự kiện nguồn trong lượt này. Nếu triển khai bằng các request model, phải giữ ranh giới dữ liệu này; chỉ đưa cả nguồn và bản dịch vào cùng request rồi yêu cầu “đọc nguồn trước” chưa thể hiện đúng sự tách biệt đã thiết kế.

Nếu cần tra cứu ngoài dữ liệu đang có, Reviewer gửi câu hỏi cho [Research](RESEARCH_STAGE_SPEC.md). Research phụ trách tìm nội bộ/Internet và trả nguồn/bằng chứng; kết quả phải được đối chiếu, không thay nguyên tác hoặc tự chứng minh bản dịch đạt. Khi hỗ trợ lượt đọc nguồn, Research vẫn không được lấy bản dịch đang đánh giá làm sự thật của nguồn. Thông tin chưa rõ mốc/từ tương lai và đề xuất thuật ngữ chưa duyệt tiếp tục giữ trạng thái riêng.

Reviewer ghi nhận những điểm cần giữ đúng:

| Nội dung | Ví dụ điều cần phân biệt |
| --- | --- |
| Chủ thể và đối tượng | Ai làm gì, làm với ai; danh tính nào chưa rõ |
| Hội thoại | Người nói, người nghe, lời kể, lời trích lại |
| Trạng thái sự kiện | Đã xảy ra, chưa xảy ra, mong muốn, giả định, phủ định |
| Mức độ chắc chắn | Sự kiện được xác nhận, suy nghĩ của nhân vật, tin đồn, phỏng đoán |
| Thời gian | Trước/sau, hồi tưởng, hiện tại của cảnh, lời kể về tương lai |
| Quan hệ và sắc thái | Quan hệ được xác lập đến đâu, lời nói đùa, diễn kịch, giận nhất thời |
| Ruby và dụng ý trình bày | Chữ chính và phần chữ nhỏ có cùng chức năng phát âm hay mang ý nghĩa/cách gọi khác nhau |
| Điểm chưa rõ | Thiếu chủ thể, người nghe mơ hồ hoặc cần đọc thêm cảnh liên quan |

Mỗi nhận định phải gắn với vị trí và bằng chứng nguồn. Không điền một nhân vật, động cơ hoặc quan hệ chỉ để hoàn thành biểu mẫu. Khi ngữ cảnh đã lưu mâu thuẫn với nguyên tác, phải mở vấn đề xác minh; không ép nguồn theo bộ nhớ cũ.

### 3.2. Ví dụ ghi nhận từ nguồn

Ví dụ dùng câu trong mẫu giao diện người dùng cung cấp. ID và vị trí dưới đây chỉ để minh họa, không phải trích dẫn đã định vị trong một EPUB thực.

```text
Vị trí minh họa: Tập 01 / Chương 2 / b018

Nguyên tác:
彼はまだ彼女に真実を話していなかった。

Thông tin cần giữ:
Tại thời điểm được kể, anh ấy vẫn chưa nói sự thật cho cô ấy.

Bằng chứng nguyên văn, ở hai vị trí riêng:
- まだ
- 話していなかった

Chưa xác định chỉ từ câu này:
- Danh tính cụ thể của hai nhân vật.
- Lý do chưa nói sự thật.
- Quan hệ giữa họ và cách xưng hô trong hội thoại.
```

Không lưu chuỗi minh họa `まだ ... ていなかった` như một trích dẫn liên tục có thật trong nguồn. Nếu bằng chứng gồm nhiều đoạn, lưu từng đoạn và vị trí riêng.

### 3.3. Ghi chú không thay thế nguyên tác

Ghi nhận từ nguồn cũng có thể bị hiểu sai. Nó hỗ trợ điều hướng và kiểm tra tính nhất quán, không trở thành một bản “sự thật tuyệt đối” để Reviewer chỉ đối chiếu bản dịch với ghi chú.

Mọi lượt đối chiếu vẫn phải có nguyên văn nguồn tương ứng. Những nhận định ảnh hưởng lớn phải được mở lại cùng ngữ cảnh khi cần kiểm chứng.

Với chương dài, đọc nguồn theo các phần có thứ tự và giữ danh sách điểm chưa giải quyết. Có thể bắt đầu review phần đã đủ căn cứ mà chưa cần nháp toàn chương. Kết luận cuối chương vẫn phải chờ đủ phạm vi và kiểm tra các mối liên hệ có ảnh hưởng đến kết luận.

## 4. Phần 3 — Đối chiếu hai chiều và kiểm tra mạch chương

### 4.1. Nguyên tác → bản dịch

Kiểm tra thông tin trong nguồn có được truyền đạt đầy đủ và đúng hay không:

- Bỏ sót câu, ý, lời thoại, chú thích hoặc nội dung cần dịch.
- Đảo nghĩa phủ định, thay đổi khả năng, mức độ chắc chắn hoặc điều kiện.
- Đổi người nói, người nghe, người thực hiện hoặc đối tượng hành động.
- Đổi trình tự, thời điểm hoặc quan hệ nguyên nhân/kết quả.
- Sai tên riêng, thuật ngữ, xưng hô hoặc sắc thái có ý nghĩa.

### 4.2. Bản dịch → nguyên tác

Kiểm tra thông tin được khẳng định trong bản dịch có được nguồn và ngữ cảnh liên quan hỗ trợ hay không. Đặc biệt chú ý sự kiện, lời thoại, động cơ, cảm xúc, quan hệ và kết luận do bản dịch tự thêm.

Việc kiểm tra dựa trên ý nghĩa, không đòi mỗi từ tiếng Việt phải có một từ nguồn tương ứng. Chuyển trật tự câu, diễn đạt tự nhiên hoặc làm rõ chủ thể đã được ngữ cảnh xác nhận không mặc nhiên là bịa nội dung.

Đối với nghi vấn thêm nội dung, hồ sơ phải ghi câu dịch bị nghi, phần nguồn/ngữ cảnh đã đối chiếu và lý do chưa tìm thấy căn cứ. Chưa được kết luận một sự kiện không tồn tại trong cả chương nếu mới kiểm tra một đoạn ngắn có thể thiếu ngữ cảnh liên quan.

Nếu cần mở rộng ngữ cảnh để kết luận, chuyển sang **Cần xác minh** và chỉ rõ phần cần xem. Không biến việc chưa tìm thấy căn cứ trong một lượt thành bằng chứng chắc chắn rằng nguyên tác không có thông tin đó.

### 4.3. Kiểm tra mạch chương

Sau kiểm tra các phần, cần đánh giá các quan hệ xuyên đoạn/cảnh:

- Sự kiện chính có bị hiểu sai và kéo lệch các cảnh sau không.
- Danh tính, người nói và điểm nhìn có nhất quán với nguồn không.
- Trình tự thời gian và hồi tưởng có bị trộn không.
- Quan hệ và xưng hô có thay đổi đúng mốc, đúng cặp và đúng phạm vi không.
- Có tình tiết tự thêm ở một đoạn được dùng như sự thật ở các đoạn sau không.
- Có ý bị bỏ sót hoặc lặp khi ghép các phần dịch không.

Kiểm tra cấp chương có thể gồm nhiều request theo các giới hạn đã chốt. Ghi chú giúp tìm các đoạn liên quan; kết luận vẫn phải truy về nguyên tác. Không bắt buộc gửi toàn chương vào một request vượt dung lượng.

### 4.4. Đề xuất về xưng hô và quan hệ

Áp dụng [đặc tả xưng hô](ADDRESSING_CONTINUITY_SPEC.md): phân biệt sự kiện nguồn với lựa chọn dịch; chỉ rõ người nói → người nghe, quy ước hiện hành, bằng chứng trước/sau sự kiện và mốc hiệu lực.

Một lời tỏ tình chưa tự chứng minh quan hệ đã được chấp nhận. Một quan hệ yêu đương đã xác lập cũng chưa tự bắt buộc tiếng Việt dùng anh–em nếu chưa có căn cứ ngôn ngữ hoặc quy ước dịch phù hợp.

Review bản dịch đạt không tự duyệt các đề xuất xưng hô đi kèm. Mỗi lựa chọn mới hoặc thay đổi phải có quyết định riêng của người dùng trước khi áp dụng, kể cả trong bản nháp hiện hành. Nếu đề xuất là điều kiện để tiếp tục phần đang xét, giữ phần đó chờ duyệt; phương án trong thẻ đề xuất chưa phải bản dịch được phép dùng.

### 4.5. Ruby, tiêu đề, mục lục và chú thích

Review phải đối chiếu cả chữ chính và phần chữ nhỏ của ruby từ nguồn. Nếu ruby chỉ hướng dẫn phát âm, việc không hiện lại chú âm trong bản Việt theo [chính sách đầu ra](EPUB_OUTPUT_SPEC.md) không tự là lỗi thiếu dịch. Nếu phần chữ nhỏ có ý nghĩa riêng hoặc dụng ý ngôn ngữ, phải kiểm tra bản dịch giữ được nội dung đó theo phương án người dùng đã duyệt. Chưa phân biệt được thì Cần xác minh; chưa chốt cách thể hiện thì Chờ duyệt quy ước.

Thẻ đề xuất hiển thị hai lớp nguồn, vị trí, ngữ cảnh, lý do cần giữ ý nghĩa, các cách thể hiện và phạm vi. Reviewer không tự chọn cách dịch, tạo chú thích mới hoặc xóa phần chữ nhỏ để kết luận bản dịch đã đạt. Cách thể hiện đã duyệt vẫn phải được kiểm tra trên revision thực tế.

Tiêu đề chương/part, nhãn mục lục và chú thích thuộc phạm vi được dịch phải nằm trong độ phủ Review; không giao phần còn lại cho bước xuất tự dịch. Các mục chỉ thuộc trang phụ được giữ nguyên thì kiểm tra đúng quan hệ nguồn và trạng thái đó. Kiểm tra tên nhất quán khi cùng nội dung nguồn, bảo toàn khác biệt có nghĩa của nhãn mục lục, đủ chú thích và mapping liên kết. Kiểm tra kỹ thuật các liên kết trong EPUB cuối thuộc bước ghép/xuất.

## 5. Phần 4 — Kết quả có bằng chứng và màn Review

### 5.1. Hồ sơ một vấn đề

| Thông tin | Yêu cầu |
| --- | --- |
| Vị trí | Tập, chương, block hoặc phạm vi câu/đoạn liên quan |
| Phiên bản | Snapshot nguồn, revision/hash bản dịch và dependency |
| Loại vấn đề | Sai nghĩa, thiếu ý, thêm nội dung, sai nhân vật, thời gian, xưng hô, diễn đạt… |
| Mức độ | Ảnh hưởng đến ý nghĩa và tiêu chí duyệt; tách khỏi độ chắc chắn của kết luận |
| Phạm vi ảnh hưởng | Một đoạn, một cảnh, nhiều vị trí hoặc cả chương, kèm lý do |
| Căn cứ nguồn | Trích dẫn có vị trí thật, cùng ngữ cảnh cần thiết |
| Căn cứ bản dịch | Câu/phạm vi bản dịch bị đánh giá |
| Giải thích | Vì sao hai bên lệch nhau hoặc vì sao chưa đủ căn cứ |
| Hướng xử lý | Điều cần giữ/sửa, câu đề xuất nếu có, hoặc thông tin cần xác minh |
| Quyết định | Đang chờ, đã xác nhận lỗi, cần xác minh, đã sửa, đã bác nhận định hoặc người dùng giữ có lý do |
| Lịch sử | Các phiên bản, phản hồi và kết quả kiểm tra tiếp theo |

Không bắt Reviewer tạo một câu sửa chắc chắn cho vấn đề đang mơ hồ. Phát hiện thiếu nguồn có thể gắn với khoảng nguồn và vị trí chèn dự kiến, không cần bịa ra một câu target để điền hồ sơ.

### 5.2. Kiểm tra kết quả của Reviewer

Core kiểm tra bằng chương trình: ID và phạm vi hợp lệ, quote khớp snapshot/revision, dữ liệu bắt buộc đầy đủ, kết quả phủ đúng phần được giao, không gán kết luận cho một phiên bản khác.

Kết quả bị cắt cụt, thiếu phạm vi hoặc trích dẫn không tồn tại chưa được tính là review hoàn tất. Phải xử lý lỗi kết quả hoặc tạo lại lượt còn thiếu trong giới hạn thích hợp, không tự coi phần không có báo lỗi là đạt.

Kiểm tra ngữ nghĩa đánh giá bằng chứng có thực sự hỗ trợ giải thích không, có ngữ cảnh phản bác không và đề xuất có sửa đúng lỗi không. Quote khớp nguồn chỉ là điều kiện cần.

Trước một quyết định tự động dịch lại toàn chương, bắt buộc có lượt kiểm chứng bổ sung trên nguyên tác, bản dịch và các kết luận liên quan. Lượt này cần có khả năng xác nhận, bác bỏ hoặc thu hẹp nhận định trước đó. Nếu các kết luận mâu thuẫn hoặc chưa đủ căn cứ, chưa thực hiện dịch lại chương theo giả thuyết đó; chuyển sang xác minh.

### 5.3. Bố cục TUI

Theo lựa chọn **12A** ngày **30/10/2026** trong [đặc tả TUI/CLI](TUI_CLI_SPEC.md), hỗ trợ tối thiểu 80 cột × 24 dòng. Từ 120 cột, màn Review có thể đặt danh sách chương/vấn đề cạnh vùng thẻ chi tiết; ở màn hẹp, chọn mục rồi xem thẻ nguyên tác/bản dịch/bằng chứng theo chiều dọc. Resize giữ mục chọn, vị trí xem và nội dung đang nhập. Mũi tên/Tab/Enter/Esc/F1 dùng theo ngữ cảnh; ngoài ô nhập dùng `/` tìm, `f` lọc và `e` mở editor đúng đoạn, `Ctrl+S` lưu trong editor theo đặc tả TUI/CLI. Toàn bộ thao tác dùng được bằng bàn phím, chuột chỉ hỗ trợ thêm.

Thẻ vấn đề theo mẫu người dùng cung cấp:

```text
TẬP 01 / CHƯƠNG 2 / b018 · BẢN NHÁP r1
Trạng thái: Đã xác nhận lỗi · Phạm vi: Đoạn b018

NGUYÊN TÁC
彼はまだ彼女に真実を話していなかった。

BẢN DỊCH
Anh đã nói cho cô ấy biết sự thật.

REVIEWER PHÁT HIỆN                         [Sai nghĩa]
- Căn cứ nguồn: まだ / 話していなかった
- Câu dịch liên quan: “đã nói”
- Giải thích: nguồn nói hành động chưa xảy ra tại thời điểm được kể;
  bản dịch lại khẳng định hành động đã xảy ra.
- Đề xuất: “Anh vẫn chưa nói sự thật cho cô ấy biết.”

[Xem ngữ cảnh] [Áp dụng đề xuất] [Tự sửa]
[Gửi phản hồi] [Giữ bản dịch, ghi lý do] [Lịch sử]
```

Khi chọn một vấn đề, làm nổi bật đúng phần nguồn và target liên quan. Một đoạn có nhiều vấn đề phải cho phép xem từng vấn đề và căn cứ tương ứng. Các phần ngữ cảnh mở rộng phải có nhãn nguồn, vị trí và trạng thái bản dịch nếu hiển thị bản dịch tham khảo.

Với lỗi xưng hô, thẻ bổ sung cặp nhân vật, chiều nói, quy ước hiện hành, sự kiện được viện dẫn, bằng chứng và phạm vi thay đổi. Người dùng có thể chỉnh đề xuất hoặc giữ quy ước cũ kèm lý do.

Theo lựa chọn 7A chốt ngày **30/10/2026** trong [đặc tả Skills](SKILLS_GLOSSARY_SPEC.md), màn Review mở được danh sách nhiều đề xuất thuật ngữ: xem ngữ cảnh từng mục, chọn cách dịch/phạm vi, rồi xác nhận các mục đã chọn trong một thao tác. Mỗi mục giữ quyết định riêng; mục chưa chọn tiếp tục chờ và chặn phần phụ thuộc. Duyệt nhóm thuật ngữ không tự chấp nhận các thẻ lỗi hoặc kết luận bản dịch/Review đạt.

### 5.4. Kết luận cấp chương và tiến độ

Trong lúc đọc nguồn, đối chiếu hoặc kiểm chứng bằng model, màn Review luôn hiển thị tên/ID model và bước đang thực hiện theo [chiến lược model](MODEL_STRATEGY_SPEC.md). Thông tin nằm ở vùng trạng thái luôn nhìn thấy, cập nhật theo request thực tế và không chỉ xuất hiện trong log lỗi.

Phía trên các thẻ có phần tổng quan, ví dụ:

```text
CHƯƠNG 2 · PHIÊN BẢN ĐANG XÉT r3
Đối chiếu từng phần: Hoàn tất
Kiểm tra mạch chương: Hoàn tất

Kết luận: Không đạt
Phạm vi ảnh hưởng: Toàn chương
Nguyên nhân: Hiểu sai sự kiện chính, kéo theo nội dung thêm
Kiểm chứng quyết định dịch lại: Đã hoàn tất
Hướng xử lý: Dịch lại chương từ nguyên tác
Trạng thái: Đang chuẩn bị bản dịch mới

[Các đoạn làm căn cứ] [Xem nguyên tác] [Lịch sử xử lý]
```

Trong lúc chạy, hiển thị bước hiện tại: đang đọc nguồn, đang đối chiếu, cần xác minh, đang sửa, đã lưu bản mới, đang review lại. Công bố thẻ sau khi kết quả của phần đó đã được kiểm tra và lưu. Không trình bày suy luận chưa hoàn thành như kết luận chính thức.

## 6. Phần 5 — Chọn phạm vi xử lý

### 6.1. Mức độ và phạm vi là hai trục riêng

Một lỗi đảo nghĩa trong một câu có thể nghiêm trọng nhưng chỉ ảnh hưởng một đoạn. Một lỗi xác định sai nhân vật có thể lan sang nhiều cảnh. Không quyết định dịch lại chương chỉ bằng số lượng báo lỗi hoặc vì một lỗi có nhãn nghiêm trọng.

| Tình trạng đã xác nhận | Cách xử lý |
| --- | --- |
| Sai một câu/đoạn, ảnh hưởng giới hạn và xác định được | Sửa đoạn đó, kiểm tra lại cùng ngữ cảnh |
| Sai người nói hoặc một sự kiện khiến cả cảnh lệch | Dịch lại cảnh bị ảnh hưởng |
| Sai quy ước xưng hô ở nhiều vị trí | Sửa phạm vi thực sự chịu ảnh hưởng theo cặp, mốc và quy ước đã xác nhận |
| Hiểu sai tiền đề, quan hệ hoặc mạch truyện xuyên chương | Dịch lại toàn chương sau kiểm chứng |
| Thêm nội dung/bỏ sót lan rộng, không khoanh vùng đáng tin cậy | Dịch lại toàn chương sau kiểm chứng |
| Thiếu dữ liệu hoặc chưa xác định cách hiểu đúng | Chặn duyệt, xác minh trước khi sửa |
| Gợi ý diễn đạt không làm sai nghĩa và không vi phạm tiêu chí duyệt | Có thể giữ như gợi ý không chặn |

Mức độ có thể được trình bày theo các nhóm nghiêm trọng, lớn, nhỏ và gợi ý. Những nhãn này phục vụ rubric; quy tắc ánh xạ chi tiết sang trạng thái chặn sẽ được hoàn thiện khi triển khai. Một lỗi nội dung đã xác nhận cần được giải quyết hoặc có quyết định ngoại lệ rõ ràng trước khi duyệt phạm vi tương ứng.

Quyết định M1–M4 dựa trên phạm vi và bằng chứng, không dùng ngưỡng cứng kiểu “trên 10 lỗi thì dịch lại chương”. Các chỉ số số lỗi/tỷ lệ block chỉ hỗ trợ nhận định; quyết định phải nêu được phạm vi và nguyên nhân. Không dùng lỗi nhỏ đơn lẻ làm lý do dịch lại cả chương.

### 6.2. Kiểm tra nguyên nhân trước khi dịch lại

Trước khi tạo bản mới, cần xác định lỗi đến từ đâu trong phạm vi có bằng chứng:

- Nguồn nhập thiếu, sai thứ tự hoặc mapping không đầy đủ.
- Chọn sai ngữ cảnh, sai nhân vật hoặc sai quy ước có hiệu lực.
- Áp một sự kiện ở tương lai ngược về cảnh trước.
- Model hiểu sai nguồn, bỏ ý hoặc thêm thông tin khi tạo bản dịch.
- Chia lượt làm thiếu ngữ cảnh cần thiết ở ranh giới cảnh.

Nếu đầu vào có lỗi, xử lý đầu vào trước rồi cập nhật những kết quả chịu ảnh hưởng. Nếu ngữ cảnh đã lưu sai, vô hiệu hóa việc sử dụng phần sai trước khi dịch lại. Không gửi lại cùng bộ dữ liệu sai với một yêu cầu chung “hãy dịch tốt hơn”.

Nếu chưa xác định được nguyên nhân, ghi rõ điều chưa biết. Việc tạo bản dịch mới không thay thế bước xác minh cách hiểu nguồn cần thiết.

## 7. Phần 6 — Chuyển về phần Dịch và review lại

### 7.1. Hồ sơ yêu cầu sửa

Core tạo công việc sửa trong tác vụ hiện tại, gồm:

- Phạm vi block/câu/cảnh/chương cần tạo bản mới.
- Snapshot nguồn và revision dự kiến sẽ được thay thế.
- Nguyên tác đầy đủ của phạm vi được giao.
- Ngữ cảnh cần thiết và các quy tắc đã xác nhận, còn hợp lệ.
- Danh sách lỗi đã được xác nhận, bằng chứng và yêu cầu khắc phục cụ thể.
- Câu đề xuất nếu có, cùng phần chưa được phép suy đoán.
- Những đoạn chỉ đọc để tham khảo, không thuộc phạm vi được ghi đè.
- Liên kết đến lịch sử sửa và số vòng sửa–review đã dùng.

Ví dụ sửa cục bộ có thể kèm bản dịch lỗi để chỉ đúng câu cần sửa. Với dịch lại toàn chương, nội dung được tạo lại từ nguyên tác và ngữ cảnh hợp lệ; bản bị đánh trượt chỉ có vai trò lịch sử hoặc ví dụ lỗi có nhãn rõ, không làm sự thật tham chiếu hay văn bản mặc định để tiếp tục viết.

### 7.2. Tái sử dụng giai đoạn Dịch

Việc sửa đi qua ranh giới đã chốt trong [đặc tả Dịch](TRANSLATION_STAGE_SPEC.md):

1. Core ghi nhận công việc sửa, phạm vi, nền revision và tiến độ trong tác vụ.
2. Chuẩn bị nguồn, ngữ cảnh và yêu cầu khắc phục theo hai giới hạn.
3. Phần Dịch tạo bản mới cho đúng phạm vi được giao.
4. Kiểm tra cấu trúc, ID, mapping và các thành phần được bảo vệ.
5. Lưu revision mới ở trạng thái chờ Review; giữ lịch sử bản cũ.

Lưu thành công `r2` chưa có nghĩa lỗi trong `r1` đã được giải quyết. Reviewer cần đối chiếu `r2` với nguyên tác, kiểm tra lỗi cũ và tìm lỗi mới, đồng thời xem lại những liên kết/ngữ cảnh chịu ảnh hưởng.

Nếu nền revision đã đổi do người dùng sửa tay, kết quả sửa dựa trên nền cũ chưa được ghi đè lên bản hiện hành. Core giữ kết quả để xử lý xung đột hoặc chuẩn bị lại trên nền phù hợp.

### 7.3. Luồng tổng thể

```text
Bản nháp đã lưu, hợp lệ về cấu trúc
        ↓
Đọc nguồn → Đối chiếu hai chiều → Kiểm tra mạch chương
        ↓
Kiểm tra kết quả Review và bằng chứng
        ├── Đủ điều kiện duyệt → Chấp nhận đúng phạm vi
        │
        ├── Có lỗi đã xác nhận
        │       ↓
        │   Chọn phạm vi sửa và kiểm tra nguyên nhân
        │       ↓
        │   Nếu dịch lại cả chương: kiểm chứng quyết định trước
        │       ↓
        │   Phần Dịch tạo bản mới
        │       ↓
        │   Kiểm tra cấu trúc → Lưu nháp mới → Review lại
        │
        └── Chưa đủ căn cứ / mâu thuẫn / hết vòng tự sửa
                ↓
            Cần xác minh hoặc người dùng xử lý
```

Sơ đồ mô tả các trách nhiệm tổng thể. Review từng phần có thể chạy ngay khi phần đó có nháp; kiểm tra mạch chương và kết luận cả chương được hoàn thành khi đã đủ dữ liệu cần thiết.

### 7.4. Sửa trực tiếp từ màn Review

Áp dụng một câu đề xuất hoặc tự sửa văn bản có thể bỏ qua bước gọi model tạo bản dịch. Kết quả vẫn phải kiểm tra cấu trúc, lưu revision mới và review lại trước khi được coi là đã đạt.

Phản hồi của người dùng được lưu với vấn đề/revision liên quan. Reviewer dùng phản hồi để làm rõ cách hiểu và yêu cầu sửa, rồi chuyển yêu cầu có căn cứ cho phần Dịch. Phản hồi về cách hiểu nguyên tác không tự trở thành bằng chứng nguồn; nếu còn mâu thuẫn, phải thể hiện mâu thuẫn đó.

## 8. Phần 7 — Chấp nhận, tiếp tục hoặc chuyển cho người dùng

### 8.1. Điều kiện chấp nhận

Một phạm vi chỉ được chấp nhận khi kết quả cấu trúc và review còn hợp lệ với phiên bản hiện hành, đã kiểm tra đủ phần được giao và không còn vấn đề chặn chưa giải quyết theo rubric.

Một chương chỉ mang trạng thái **Đạt** khi:

- Mọi phần nội dung cần dịch đã có kết quả hợp lệ.
- Các phần đã được đối chiếu đủ hai chiều trên đúng nguồn/revision.
- Kiểm tra mạch chương đã hoàn thành.
- Các vấn đề chặn và điểm chưa rõ ảnh hưởng đến kết luận đã được giải quyết.
- Quyết định được lưu và chưa bị một thay đổi dependency làm mất hiệu lực.

Không có báo lỗi, kết quả model rỗng hoặc chỉ có một số phần đạt chưa đủ để duyệt cả chương.

Gợi ý không chặn có thể còn lại. Nếu người dùng quyết định giữ một bản còn vấn đề, trạng thái phải thể hiện **Người dùng giữ — có ngoại lệ**, ghi lý do và phạm vi. Không hiển thị trường hợp đó như Reviewer đã xác nhận hoàn toàn đúng; nó cũng không tự duyệt các sự kiện/quy tắc ngữ cảnh còn tranh chấp.

### 8.2. Điều phối sang phần dịch tiếp theo

Chỉ nội dung đã được chấp nhận và còn hợp lệ mới được dùng như ngữ cảnh bản dịch đã xác nhận. Nguồn nguyên văn vẫn là căn cứ khi cần kiểm tra lại; không lấy câu tiếng Việt đã duyệt thay cho bằng chứng nguồn.

Theo [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md) chốt ngày **28/10/2026**, mỗi lượt dịch được Review ngay sau khi lưu nháp; đạt điều kiện cục bộ rồi mới dịch lượt tiếp theo. Khi mọi lượt đã được xử lý, thực hiện kiểm tra tổng thể và lưu kết luận chương trước khi tự chuyển sang chương tiếp theo. Nhóm/batch hai chương chỉ là phạm vi lựa chọn, không trì hoãn Review đến sau khi dịch xong cả nhóm.

Kết quả đạt cục bộ còn phù hợp có thể làm ngữ cảnh cho lượt sau, nhưng chưa phải kết luận hoàn tất chương. Nếu kiểm tra tổng thể phát hiện vấn đề, xử lý theo rubric và truy lại phần phụ thuộc; kết luận cấp chương phải gắn với tập hợp revision sau sửa. Các nhánh chờ người dùng hoặc ngoại lệ tiếp tục giữ điều kiện và nhãn riêng đã chốt, không tự vượt bước Review.

Đề xuất xưng hô, glossary và quy ước dịch chỉ được áp dụng sau quyết định riêng của người dùng, có căn cứ và đúng phạm vi/mốc. Dữ kiện về sự kiện hoặc quan hệ trong nguồn cũng cần được xác minh trước khi thành ngữ cảnh chính thức; xác minh dữ kiện đó không tự duyệt cách diễn đạt tiếng Việt. Việc chấp nhận văn bản không tự chấp nhận mọi candidate đi kèm.

### 8.3. Giới hạn năm vòng sửa–review

Sau **5 vòng sửa–review vẫn không đạt**, core dừng tự sửa và chuyển cho người dùng. Đây là giới hạn vòng sửa chất lượng đã thống nhất; không phải số lần gọi API hoặc số lần retry mạng.

Một vòng gồm việc tạo bản sửa cho phạm vi đang xử lý rồi review bản đó. Một lần dịch lại chương có thể cần nhiều request vì dung lượng, nhưng vẫn thuộc cùng một vòng sửa theo kế hoạch đó. Review đầu tiên trước khi sửa không tự tính là một vòng sửa thất bại.

Mở rộng từ sửa đoạn sang dịch lại chương cho cùng đợt lỗi không được dùng để tự xóa số vòng đã thất bại. Lỗi cấu trúc, phản hồi không hợp lệ hoặc retry dịch vụ có giới hạn riêng; chúng không tạo thêm quyền sửa chất lượng vô hạn.

Có thể chuyển cho người dùng sớm hơn khi thiếu căn cứ, có kết luận mâu thuẫn hoặc không xác định được cách hiểu nguồn. Khi người dùng xử lý và chủ động tiếp tục, lưu quyết định cùng một đợt xử lý có giới hạn mới; bảo toàn lịch sử và không tự bắt đầu lại chỉ vì đã chờ một khoảng thời gian.

Màn Review lúc cần người dùng phải có nguyên tác, bản dịch hiện tại, lỗi còn tồn tại, bằng chứng, những lần sửa đã thử và điều cần người dùng làm rõ.

## 9. Ngăn bản lỗi làm sai ngữ cảnh phía sau

Khi một phần hoặc cả chương bị đánh trượt:

1. Lưu kết luận và phạm vi ảnh hưởng cùng revision đang bị đánh giá.
2. Giữ nguyên lịch sử bản dịch và bằng chứng; không xóa các phiên bản cũ.
3. Chặn sử dụng phần bị đánh trượt như ngữ cảnh đã xác nhận cho công việc tiếp theo.
4. Giữ các candidate lấy từ bản đó ở trạng thái chưa xác nhận hoặc cần kiểm tra lại.
5. Nếu một quyết định ngữ cảnh đã công bố bị chứng minh sai, đánh dấu quyết định không còn hợp lệ và truy các bản dịch thực sự đã phụ thuộc vào nó.
6. Đánh dấu phần phụ thuộc cần review/sửa lại; không tự ghi đè bản sửa tay.

Phần đã được duyệt cục bộ có thể cần kiểm tra lại sau khi review toàn chương phát hiện sai tiền đề. Lịch sử phải thể hiện việc thay đổi quyết định đó. Không tự coi mọi chương về sau đều hỏng; xác định phần bị ảnh hưởng theo dependency và phạm vi hiệu lực.

Việc lưu bản dịch, quyết định Review và công bố ngữ cảnh cần nhất quán với cơ chế lưu trữ có phiên bản. Không cho thấy một quy tắc đã được chấp nhận khi bằng chứng hoặc revision làm căn cứ chưa lưu thành công.

## 10. Lưu lịch sử và tiếp tục khi bị gián đoạn

Cần lưu phạm vi, phiên bản đầu vào, ghi chú nguồn, kết quả đối chiếu, bằng chứng, quyết định kiểm chứng, yêu cầu sửa, các revision và số vòng đã dùng. Ghi nhận model/lần gọi và usage khi có; không đưa API key vào hồ sơ.

| Trạng thái lúc gián đoạn | Cách tiếp tục |
| --- | --- |
| Một phần nguồn đã được đọc, snapshot còn phù hợp | Dùng lại ghi nhận hợp lệ và xử lý phần còn thiếu |
| Một phần đã review và lưu, dependency còn hợp lệ | Giữ kết quả, không tính phần còn thiếu là đạt |
| Kết quả model chưa qua kiểm tra hợp đồng | Kiểm tra kết quả đã lưu trước khi cân nhắc gọi lại |
| Quyết định sửa đã lưu nhưng chưa tạo bản mới | Tiếp tục công việc sửa theo phạm vi và số vòng hiện tại |
| Bản sửa đã lưu nhưng chưa review | Review đúng revision đã lưu |
| Nguồn, bản dịch hoặc quy tắc đã thay đổi | Kiểm tra lại hiệu lực, chuẩn bị lại phần bị ảnh hưởng |
| Đã hết vòng tự sửa hoặc đang cần người dùng | Giữ trạng thái chờ xử lý, không tự chạy lại vô hạn |

## 11. Ví dụ xử lý xuyên suốt

### 11.1. Một lỗi nghiêm trọng nhưng cục bộ

Nguồn: `彼はまだ彼女に真実を話していなかった。`

Bản nháp `r1`: “Anh đã nói cho cô ấy biết sự thật.”

Reviewer đối chiếu và xác nhận đảo nghĩa phủ định. Nếu ngữ cảnh cho thấy ảnh hưởng chỉ nằm ở đoạn này, core giao sửa đoạn cùng phần tham khảo cần thiết. Phần Dịch tạo `r2`: “Anh vẫn chưa nói sự thật cho cô ấy biết.”

`r2` được kiểm tra cấu trúc, lưu và review lại. Chỉ sau đó đoạn mới được chấp nhận. Tính nghiêm trọng của lỗi không tự buộc dịch lại toàn chương.

### 11.2. Nội dung tự thêm kéo lệch cả chương

Tình huống dưới đây là giả định để minh họa, không phải trích từ một EPUB thực.

Nguyên tác của cảnh chỉ cho biết A chưa nói sự thật với B. Bản dịch lại viết A đã thú nhận, B tức giận và chia tay A. Các cảnh dịch tiếp theo dựa vào cuộc chia tay tự thêm để đổi lời thoại, quan hệ và xưng hô.

Reviewer cần đối chiếu cảnh nguồn và các cảnh liên quan, chỉ ra những câu đảo nghĩa/thêm nội dung và đường ảnh hưởng sang phần sau. Sau khi lượt kiểm chứng bổ sung xác nhận sai lệch ở cấp chương, core đánh trượt phiên bản chương, chặn ngữ cảnh sai và giao dịch lại chương từ nguồn hợp lệ.

Bản mới được chia thành các lượt theo ngân sách, kiểm tra cấu trúc, lưu nháp và review đầy đủ. Không chỉ xóa câu “chia tay” rồi coi các cảnh đã bị kéo lệch là đúng.

Nếu việc mở rộng ngữ cảnh cho thấy cuộc chia tay thực sự có trong nguồn ở vị trí khác, phải sửa hoặc bác kết luận “tự thêm” cho phù hợp. Reviewer cũng phải chịu kiểm tra bằng chứng của chính mình.

### 11.3. Chưa rõ người nói hoặc mốc thay đổi quan hệ

Nguồn chưa đủ để xác định người nói, hoặc một lời dọa chia tay chưa rõ có trở thành sự kiện thật. Reviewer báo **Cần xác minh**, chỉ ra phần còn mơ hồ và mở ngữ cảnh liên quan. Chưa được tự đổi cách xưng hô của cả chương theo một giả thuyết.

## 12. Tiêu chí nghiệm thu khi triển khai

Các tình huống dưới đây dùng dữ liệu tự tạo và model giả lập cho kiểm thử tự động offline, deterministic. Chúng xác nhận hành vi điều phối, dữ liệu và giao diện; không chứng minh chất lượng hiểu ngôn ngữ của model thực đạt 100%.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Review một bản nháp | Có nguồn, mapping và revision tương ứng |
| Lượt đọc nguồn trước đối chiếu | Không dùng bản dịch đang bị đánh giá làm căn cứ sự kiện nguồn |
| Một phần chưa được đối chiếu | Chương không được đánh dấu review hoàn tất |
| Request dài hoặc thiếu dung lượng đầu ra | Chia lại phạm vi, giữ nguồn bắt buộc, kiểm tra đủ hai giới hạn |
| Reviewer trả quote không có thật | Kết quả chưa hợp lệ, không tự sửa theo báo lỗi đó |
| Reviewer trả kết quả rỗng/cắt cụt/thiếu ID | Không coi phần thiếu là đạt |
| Quote có thật nhưng kết luận không được ngữ cảnh hỗ trợ | Có thể bác hoặc chuyển xác minh, không lấy quote khớp làm đủ bằng chứng |
| Bỏ sót ý nguồn | Được phát hiện qua chiều nguồn → dịch và có vị trí cụ thể |
| Thêm tình tiết không được nguồn hỗ trợ | Có câu dịch, phạm vi nguồn đã kiểm tra và giải thích |
| Diễn đạt tiếng Việt tự nhiên nhưng giữ đúng ý | Không bị coi là bịa chỉ vì khác số từ hoặc trật tự câu |
| Một lỗi nghiêm trọng ở một đoạn | Chọn phạm vi theo ảnh hưởng, không tự dịch lại chương |
| Quyết định dịch lại toàn chương | Có kiểm chứng bổ sung và nguyên nhân/phạm vi rõ ràng |
| Các lượt kiểm chứng mâu thuẫn | Chưa tự dịch lại theo giả thuyết; chuyển xác minh |
| Áp dụng đề xuất hoặc sửa tay | Tạo revision mới, kiểm tra cấu trúc và review lại |
| Bản dịch đổi trong lúc review/sửa | Kết quả cũ chưa được áp vào revision mới hoặc ghi đè sửa tay |
| Bản sửa hết lỗi cũ nhưng thêm lỗi mới | Chưa được chấp nhận chỉ vì lỗi cũ đã biến mất |
| Sai tiền đề được phát hiện ở cấp chương | Phần đã duyệt cục bộ chịu ảnh hưởng được đánh dấu lại |
| Candidate xưng hô từ bản bị đánh trượt | Chưa được dùng như quy tắc chính thức |
| Bộ nhớ đã xác nhận sau đó bị chứng minh sai | Vô hiệu hóa phần sai, truy dependency, giữ lịch sử và sửa tay |
| Năm vòng sửa–review vẫn không đạt | Dừng tự sửa, TUI hiển thị bằng chứng và lịch sử cho người dùng |
| Mở rộng phạm vi hoặc khởi động lại ứng dụng | Không tự xóa số vòng thất bại |
| Người dùng giữ bản còn vấn đề | Hiển thị quyết định có ngoại lệ, không nhầm với Reviewer xác nhận đúng |
| Reviewer muốn tạo/đổi thuật ngữ hoặc quy ước | Có đề xuất với căn cứ/phương án, chờ người dùng duyệt trước khi áp dụng |
| Sửa lỗi về đúng quy ước đã duyệt | Tiếp tục luồng sửa–review, không tạo một lựa chọn dịch thuật mới trái quyết định hiện hành |
| Ruby thuần phát âm không hiện chú âm trong bản Việt | Không tự báo thiếu dịch khi phân loại có căn cứ, nguồn/mapping còn đầy đủ và chính sách được áp dụng đúng |
| Ruby có ý nghĩa riêng bị bỏ hoặc dùng phương án chưa duyệt | Chỉ rõ hai lớp nguồn và ý nghĩa bị ảnh hưởng; yêu cầu xử lý trong Dịch–Review |
| Tiêu đề, mục lục hoặc chú thích thuộc phạm vi dịch chưa được đối chiếu | Phần phụ trợ vẫn chưa hoàn tất; không coi thân chương đạt là đã Review đủ toàn bộ nội dung |
| Mọi phần đã kiểm tra nhưng mạch chương chưa kiểm tra | Chương vẫn chưa mang trạng thái Đạt |
| Chương hiện tại chưa hoàn tất kiểm tra tổng thể hoặc còn bị chặn | Chưa gửi dịch chương tiếp theo trong cùng chuỗi tác vụ |
| Kết quả đã chấp nhận và còn hợp lệ | Bộ điều phối tiếp tục theo workflow, chỉ công bố ngữ cảnh có quyết định phù hợp |

## 13. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [21](DESIGN_DECISIONS_36.md#decision-21), [22](DESIGN_DECISIONS_36.md#decision-22), [24](DESIGN_DECISIONS_36.md#decision-24) của bản 36 mục áp dụng cho đặc tả này.

### 13.1. Cách thực hiện lượt đọc nguyên tác của Reviewer — mục 21

Mỗi lượt Review bắt đầu bằng một request **chỉ đọc nguyên tác và ngữ cảnh nguồn đã kiểm chứng**. Tách thành nhiều request nguồn liên tiếp nếu vượt dung lượng. Trường hợp đã có kết quả đọc nguồn đúng phạm vi và còn hiệu lực, có thể dùng lại. Bản Việt đang xét chỉ xuất hiện ở request đối chiếu sau đó.

Mỗi ghi nhận chứa phạm vi, trích dẫn, nhân vật/người nói/người nghe, sự kiện, phủ định/giả định, thời điểm, điểm chưa rõ. Gom nhận định theo vị trí nguồn. Gặp mâu thuẫn thì mở lại đoạn gốc và cảnh quanh đó. Ghi nhận ban đầu là bằng chứng làm việc của Review. Chỉ nhận định đã kiểm chứng với nguyên tác mới đủ điều kiện thành ngữ cảnh dùng về sau. Một sự kiện thiếu căn cứ được bác hoặc giữ **Cần xác minh**. Độ phủ và vị trí trích dẫn có thể kiểm tra tự động. Việc hiểu đúng toàn bộ ý nghĩa vẫn có thể sai, cần kiểm chứng và đo bằng bộ đối chứng ở mục 36. [Lượt đọc nguồn](REVIEW_STAGE_SPEC.md)

### 13.2. Kiểm tra bằng chứng và kết luận Review — mục 22

Mỗi lỗi có ID, loại, mức S0–S3, snapshot/phạm vi nguồn, revision/phạm vi bản Việt, trích dẫn hiện có, lời giải thích, trạng thái bằng chứng, phạm vi M đề xuất và cách sửa. Lỗi bỏ sót dùng câu gốc và vị trí thiếu ở bản dịch. Lỗi thêm nội dung dùng chính câu tiếng Việt và ghi rõ vùng nguồn đã đối chiếu để kiểm tra sự vắng mặt.

Code kiểm tra ID, phiên bản, vị trí, sự tồn tại của trích dẫn và độ phủ. Reviewer kiểm tra liệu ngữ cảnh có thực sự chứng minh kết luận, gồm cả cách diễn đạt khác mà vẫn đúng nghĩa. Khi kết quả kiểm chứng trái nhau, lưu cả hai, mở rộng nguyên tác và đưa về **Cần xác minh** đến khi có đủ căn cứ. Nếu chỉ một nhận định của Reviewer sai, sửa/bác nhận định đó. Nếu nền nguồn hoặc độ phủ sai, mở lại toàn bộ phạm vi Review bị ảnh hưởng.

### 13.3. Kiểm tra tổng thể cuối chương — mục 24

Tạo hồ sơ Review chương có tập hợp revision hiện tại, độ phủ nguồn→dịch và dịch→nguồn, kết quả từng lượt, các điểm nối, vấn đề còn mở và kết luận toàn chương. Dùng lại kết quả cục bộ còn hiệu lực. Mỗi điểm nối, quan hệ xuyên cảnh hoặc nghi vấn được kiểm tra với nguyên tác và bản Việt thực tế. Khi độ phủ cũ không đủ, mở lại phần thiếu, kể cả toàn chương khi cần.

Chương dài kiểm tra lần lượt theo cảnh với ít nhất một đoạn hai bên điểm nối nếu có, rồi mở rộng theo sự kiện, người nói, mốc thời gian, xưng hô, thuật ngữ, thêm/bớt ý và phủ định. Phần chồng nhau là ngữ cảnh đối chiếu. Nếu lượt đầu sai, lần theo mục 09 để xét lại các lượt sau phụ thuộc. Chỉ chuyển sang chương kế khi dữ liệu còn hiệu lực, kiểm tra mạch chương đã xong và mọi điểm chặn đã được giải quyết theo quy tắc ngoại lệ.

### 13.4. Liên kết và yêu cầu triển khai

M1–M4, hồ sơ vòng sửa và ngoại lệ theo [rubric](REVIEW_RUBRIC_SPEC.md), mục 12. Màn bằng chứng mở editor đúng đoạn theo [TUI/CLI](TUI_CLI_SPEC.md), mục 8. Triển khai kiểm chứng dùng bộ đối chứng Nhật–Việt theo [Quyết định 36 về kiểm thử và nghiệm thu](DESIGN_DECISIONS_36.md#decision-36).
