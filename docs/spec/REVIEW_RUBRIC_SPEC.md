# Quyết định 05 — Rubric Review, danh mục lỗi và bốn mức xử lý

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 12.

Ngày ghi nhận: **25/10/2026**.

Trạng thái: **ghi nhận thiết kế đã thống nhất đến thời điểm hiện tại**. Tài liệu chốt cách đánh giá lỗi và chọn phạm vi xử lý trong giai đoạn thiết kế; không khẳng định các hành vi đã được triển khai trong Go hoặc Rust.

Tài liệu cụ thể hóa phần rubric và lựa chọn phạm vi sửa của [đặc tả Review](REVIEW_STAGE_SPEC.md), đồng thời tuân theo [giai đoạn Dịch](TRANSLATION_STAGE_SPEC.md), [xử lý EPUB và hai giới hạn dung lượng](EPUB_INPUT_SPEC.md), [xưng hô xuyên tập](ADDRESSING_CONTINUITY_SPEC.md).

Cập nhật ngày **29/10/2026**: cách đánh giá ruby và định dạng đầu ra tuân theo [đặc tả EPUB tiếng Việt](EPUB_OUTPUT_SPEC.md). Giữ ý nghĩa nguồn và mapping không bắt buộc lặp lại lớp chú âm thuần phát âm trên bản Việt.

Phạm vi gồm: điều kiện xác nhận lỗi, mức độ ảnh hưởng 0–3, bốn mức xử lý 1–4, danh mục lỗi bản dịch, lỗi của chính Reviewer, hồ sơ quyết định và điều kiện duyệt sau khi sửa. Hợp đồng hồ sơ lỗi, kiểm chứng M1–M4 và vòng sửa/ngoại lệ được chốt tại mục 12 và [đặc tả Review](REVIEW_STAGE_SPEC.md), mục 13. Theo quyết định ngày **30/10/2026** trong [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), không cần chuyển workspace test cũ; schema workspace mới vẫn phải giữ bằng chứng, lịch sử và khả năng phục hồi/nâng cấp về sau.

## 1. Những nguyên tắc đã chốt

1. **Mức độ nghiêm trọng và mức xử lý là hai đánh giá riêng.** Một lỗi nghiêm trọng có thể chỉ cần sửa một vị trí; nhiều lỗi nhỏ không tự buộc dịch lại cả chương.
2. **Bằng chứng là điều kiện trước khi xác nhận lỗi.** Chưa rõ nguồn hoặc có căn cứ mâu thuẫn thì giữ trạng thái Cần xác minh.
3. **Mức xử lý dựa trên phạm vi ảnh hưởng và khả năng sửa cục bộ.** Không chọn chỉ bằng tên loại lỗi, số lỗi hoặc tổng điểm.
4. **Chọn phạm vi đủ để khắc phục nguyên nhân và ảnh hưởng đã xác nhận.** Không giữ các đoạn đã bị kéo lệch chỉ để giảm lượng dịch lại; cũng không thay các đoạn ngoài phạm vi khi chưa có lý do.
5. **Dịch lại toàn chương cần kiểm chứng bổ sung trên nguồn.** Chưa khoanh vùng được vì thiếu dữ liệu chưa đồng nghĩa đã chứng minh cả chương sai.
6. **Mọi thay đổi tạo phiên bản mới và phải được kiểm tra lại.** Áp câu đề xuất hoặc sửa tay không tự làm bản dịch đạt Review.
7. **Đánh giá theo nguyên tác và quy ước đang có hiệu lực.** Sở thích diễn đạt riêng của Reviewer không tự trở thành lỗi.

Trong tài liệu, ký hiệu **S0–S3** chỉ mức ảnh hưởng; **M1–M4** chỉ mức xử lý. Ký hiệu dùng để tránh nhầm hai thang, chưa phải tên enum hoặc trường dữ liệu bắt buộc.

## 2. Bốn tiêu chí đánh giá một vấn đề

| Tiêu chí | Câu hỏi phải trả lời | Dữ liệu cần ghi |
| --- | --- | --- |
| Bằng chứng | Nguồn có đủ để xác nhận lỗi không? Có ngữ cảnh phản bác không? | Quote, vị trí, snapshot/revision, phần đã đối chiếu, điều chưa rõ |
| Ảnh hưởng đến ý nghĩa | Lỗi câu chữ, sai một ý hay làm đổi sự kiện/quan hệ then chốt? | S0–S3 và giải thích gắn với nội dung cụ thể |
| Phạm vi ảnh hưởng | Một vị trí, một đoạn, một cảnh/nhóm đoạn hay cả chương? | Danh sách vị trí, quan hệ phụ thuộc, ranh giới đã kiểm tra |
| Khả năng sửa cục bộ | Đã biết cách sửa chính xác chưa? Phần giữ lại có còn đáng tin không? | Phần cần thay, phần có thể giữ và lý do; nhu cầu tạo lại văn bản |

### 2.1. Trạng thái bằng chứng

| Trạng thái | Ý nghĩa | Hệ quả |
| --- | --- | --- |
| Đã xác nhận lỗi | Bằng chứng hợp lệ cùng ngữ cảnh hỗ trợ nhận định và đã giải quyết các điểm mâu thuẫn ảnh hưởng đến kết luận | Có thể chấm mức ảnh hưởng và quyết định sửa |
| Cần xác minh | Thiếu dữ liệu, người nói chưa rõ, nguồn mơ hồ hoặc các căn cứ mâu thuẫn | Chỉ rõ phần cần xem thêm; chưa tự sửa theo giả thuyết |
| Đã bác nhận định | Đối chiếu cho thấy bản dịch phù hợp hoặc báo lỗi không có căn cứ | Không dùng nhận định đó để sửa hay tăng mức xử lý; giữ lịch sử |

Việc bằng chương trình xác nhận quote có thật không tự chứng minh diễn giải đúng. Với nghi vấn thêm nội dung, phải ghi phạm vi nguồn đã kiểm tra; chưa tìm thấy căn cứ trong một đoạn ngắn chưa chứng minh căn cứ không tồn tại trong ngữ cảnh liên quan.

Mức ảnh hưởng dự kiến có thể giúp ưu tiên xác minh, nhưng phải gắn nhãn dự kiến. Không dùng một nghi vấn chưa xác nhận như lỗi chắc chắn để tự sửa hoặc dịch lại chương.

### 2.2. Điều kiện dữ liệu

Review phải dùng đúng nguyên tác, bản dịch và quy ước của phạm vi đang xét. Nếu nguồn/mapping hỏng hoặc revision đã đổi, cần xử lý dữ liệu hay hiệu lực kết quả trước khi áp quyết định.

Core kiểm soát coverage, ID, quote, phiên bản và tính đầy đủ của kết quả. Reviewer đánh giá ý nghĩa và ngữ cảnh. Rubric không bảo đảm AI hiểu đúng 100%; nó buộc kết luận có căn cứ, có trạng thái chưa xác định và có đường kiểm tra lại.

## 3. Rubric mức ảnh hưởng S0–S3

| Mức | Tiêu chí | Ví dụ minh họa | Hệ quả đối với duyệt |
| --- | --- | --- | --- |
| S0 — Gợi ý | Đúng ý, rõ và phù hợp quy ước; phương án khác chủ yếu là lựa chọn diễn đạt | Đổi một câu vốn đúng sang cách nói khác vẫn tương đương | Không chặn; chưa phải lỗi bắt buộc sửa |
| S1 — Nhỏ | Có sai sót rõ về câu chữ hoặc quy ước, chưa làm đổi thông tin/cách hiểu | Chính tả, lặp từ ngoài ý muốn, cách viết tên không thống nhất nhưng đối tượng vẫn rõ | Giải quyết trước khi mang nhãn Đạt, hoặc người dùng giữ có ngoại lệ |
| S2 — Lớn | Sai/mất thông tin cần truyền đạt nhưng chưa làm thay đổi tiền đề hay sự kiện then chốt | Dịch sai một mô tả hành động, mất một ý giải thích có ý nghĩa | Chặn duyệt phạm vi liên quan đến khi xử lý |
| S3 — Nghiêm trọng | Làm thay đổi sự kiện, danh tính, quan hệ hoặc điều kiện then chốt, khiến hiểu sai diễn biến quan trọng | Bịa chia tay, đổi người gây án, biến chưa tiết lộ bí mật thành đã tiết lộ | Chặn duyệt, kiểm tra phạm vi lan truyền và chọn mức xử lý có căn cứ |

S1–S3 chỉ áp dụng như lỗi đã xác nhận khi có bằng chứng, thuộc phạm vi cần dịch và có tiêu chí bị vi phạm. Một câu chỉ khác sở thích của Reviewer thuộc S0 hoặc không tạo vấn đề.

### 3.1. Chấm theo ngữ cảnh

Tên loại lỗi không ấn định điểm. Sai một con số ở chi tiết phụ khác với sai giờ quyết định chứng cứ ngoại phạm. Một lỗi dấu câu có thể là S1, nhưng nếu làm đổi người nói hoặc nội dung lời thú nhận thì cần chấm theo ảnh hưởng thực tế.

Không hạ mức chỉ vì sửa được bằng một từ. Không nâng mức chỉ vì lỗi xuất hiện nhiều lần. Hồ sơ phải giải thích thông tin nào bị thay đổi và vì sao điều đó quan trọng trong cảnh/chương.

### 3.2. Không cộng điểm để quyết định chất lượng hoặc dịch lại chương

- Không lấy trung bình hay tổng điểm để bù một lỗi nghiêm trọng bằng nhiều đoạn đúng.
- Không đặt quy tắc tự động như “trên 10 lỗi” hoặc “tổng điểm trên X thì dịch lại chương”.
- Có thể thống kê số vấn đề theo mức và phạm vi để điều hướng, nhưng thống kê chưa thay thế kết luận có bằng chứng.
- Một vấn đề chính có nhiều nhãn liên quan không được nhân thành nhiều lỗi độc lập để phóng đại số lượng.

## 4. Bốn mức xử lý M1–M4

| Mức | Điều kiện chọn | Việc thực hiện | Phạm vi kiểm tra lại |
| --- | --- | --- | --- |
| M1 — Sửa trực tiếp | Biết chính xác phần cần thay; có thể giữ ý và cấu trúc xung quanh | Sửa vị trí đã xác nhận, có thể tại nhiều chỗ | Phần thay đổi, ngữ cảnh cần thiết và dependency chịu ảnh hưởng |
| M2 — Dịch lại đoạn | Cần tạo lại ý/cách diễn đạt của một đoạn, ảnh hưởng được khoanh trong đoạn đó | Dịch lại đoạn với nguyên tác, lỗi và ngữ cảnh trước/sau | Toàn đoạn và liên kết với các đoạn liên quan |
| M3 — Dịch lại cảnh/nhóm đoạn | Nhiều đoạn cùng chịu một cách hiểu sai hoặc phụ thuộc nhau, vẫn khoanh được phạm vi | Tạo lại danh sách đoạn bị ảnh hưởng, có thể liên tiếp hoặc rải rác | Từng đoạn, tính nhất quán của phạm vi và ranh giới với phần giữ lại |
| M4 — Dịch lại chương | Sai tiền đề hoặc sai lệch lan rộng đã xác nhận khiến sửa chọn lọc không đủ khôi phục chương | Kiểm chứng bổ sung, xử lý nguyên nhân, tạo lại chương từ nguồn hợp lệ | Đối chiếu đầy đủ bản mới và kiểm tra mạch chương |

### 4.1. M1 — Sửa trực tiếp tại vị trí lỗi

Phù hợp với lỗi chính tả, dấu câu, lặp từ, cách viết tên, thuật ngữ hoặc số liệu có phương án thay chính xác. Có thể áp dụng cho xưng hô khi đã chắc người nói/người nghe, quy ước và mốc hiệu lực.

M1 không bắt buộc gọi model dịch lại toàn đoạn. Áp đề xuất, sửa tay hoặc xử lý có kiểm soát đều phải lưu phiên bản mới, kiểm tra cấu trúc và review lại phần chịu ảnh hưởng.

Một thuật ngữ sai ở nhiều vị trí có thể vẫn là M1 tại danh sách vị trí đó. Không thay thế toàn văn theo chuỗi chữ khi chưa phân biệt nghĩa, nhân vật hoặc phạm vi quy tắc. Lỗi nghiêm trọng cũng có thể dùng M1 nếu cách sửa thực sự rõ và đã kiểm tra ảnh hưởng.

### 4.2. M2 — Dịch lại một đoạn

Phù hợp khi một đoạn đảo nghĩa, mất ý, nhầm chủ thể, tự thêm thông tin hoặc diễn đạt hỏng đến mức vá từng từ dễ sót lỗi. Điều kiện là ngữ cảnh xung quanh vẫn phù hợp và ảnh hưởng đã khoanh được.

Phần Dịch nhận đủ nguyên tác của đoạn, lỗi có căn cứ và các đoạn liên quan ở chế độ tham khảo. Review lại phải kiểm tra toàn đoạn mới, không chỉ xem câu từng bị báo lỗi đã đổi chưa.

Một lỗi phủ định đơn giản có thể dùng M1 nếu sửa được chính xác; nếu đoạn đã xây các ý tiếp theo theo nghĩa sai, có thể cần M2 hoặc mức rộng hơn. Không áp bảng loại lỗi → mức xử lý một cách cứng nhắc.

### 4.3. M3 — Dịch lại cảnh hoặc nhóm đoạn liên quan

Phù hợp với nhầm người nói trong cuộc hội thoại, hiểu sai sự kiện làm lệch phản ứng tiếp theo, áp sai mốc xưng hô, nhầm hồi tưởng hoặc đứt ngữ cảnh ở ranh giới các lượt dịch.

Hồ sơ cần ghi danh sách đoạn và lý do liên kết. Nếu phạm vi chỉ gồm các lời A nói với B, không tự sửa lời A nói với C. Nếu cảnh có các phần vẫn giữ được, xác định chúng rõ trước khi lập công việc.

M3 dùng khi cần tạo lại các đoạn/cảnh bị ảnh hưởng. Nếu chỉ cần thay một từ đã xác định tại nhiều vị trí, M1 vẫn phù hợp dù phạm vi tìm kiếm trải khắp chương.

### 4.4. M4 — Dịch lại toàn chương

Các tình huống có thể dẫn đến M4:

- Hiểu sai sự kiện chính hoặc tiền đề, kéo lệch diễn biến xuyên chương.
- Nhầm nhân vật/điểm nhìn khiến nhiều cảnh dựa trên thông tin sai.
- Bịa tình tiết rồi dùng tình tiết đó làm sự thật cho các phần sau.
- Phóng tác, tóm tắt hoặc bỏ sót trên diện rộng khiến không còn bản dịch đủ ý.
- Trộn thời gian, quan hệ hoặc sự kiện làm hỏng mạch chương.
- Sai lệch lan rộng đã xác nhận và không tách được phần đáng tin để sửa chọn lọc.

Trước M4, lượt kiểm chứng bổ sung phải xem nguyên tác và các bằng chứng liên quan, có khả năng xác nhận, bác hoặc thu hẹp kết luận. Nếu chưa đủ căn cứ hoặc còn mâu thuẫn, chuyển xác minh; không coi việc chưa khoanh vùng được là lý do tự động dịch lại chương.

Core kiểm tra nguyên nhân như thiếu nguồn, sai mapping, dùng nhầm quy tắc hoặc ngữ cảnh bị nhiễm. Xử lý nguyên nhân trước khi dịch lại, giữ bản cũ trong lịch sử, chặn dùng bản đã bị đánh trượt làm ngữ cảnh xác nhận.

M4 vẫn chia thành nhiều request theo hai giới hạn đã chốt. “Dịch lại chương” là phạm vi nghiệp vụ, không đồng nghĩa gửi cả chương trong một request.

## 5. Quy trình chọn hướng xử lý

```text
Kiểm tra đúng nguồn, revision và tính hợp lệ của kết quả Review
        ↓
Bằng chứng đủ để xác nhận lỗi?
        ├── Chưa → Mở ngữ cảnh / xử lý dữ liệu / xác minh
        └── Đủ
             ↓
        Chấm S0–S3, xác định phạm vi và khả năng sửa cục bộ
             ├── Chỉ là gợi ý S0 → Không bắt buộc sửa
             ├── Cách thay chính xác, giữ được phần còn lại → M1
             ├── Cần tạo lại một đoạn → M2
             ├── Cần tạo lại cảnh hoặc nhóm đoạn đã khoanh → M3
             └── Sai lệch toàn chương đã được kiểm chứng → M4
```

Trước khi thực hiện, ghi điều kiện kiểm tra lại: cần giữ thông tin nào, lỗi nào phải hết và những phần liên quan nào phải còn đúng. Core kiểm tra phiên bản hiện hành để tránh áp quyết định cũ vào bản người dùng vừa sửa.

### 5.1. Ba nhánh không tự chuyển sang dịch lại

| Tình trạng | Hành vi |
| --- | --- |
| Chưa rõ nguyên tác hoặc phạm vi | Xác minh, chỉ rõ dữ liệu còn thiếu; chưa tự chọn một giả thuyết để sửa |
| Nguồn, thứ tự, mapping hoặc cấu trúc hỏng | Sửa dữ liệu/cấu trúc trước; sau đó đánh giá lại bản dịch chịu ảnh hưởng |
| Reviewer báo sai, thiếu bằng chứng hoặc dùng nhầm revision | Bác/sửa nhận định hoặc review lại đúng phạm vi; chưa thay bản dịch theo báo lỗi đó |

Lỗi cấu trúc có thể khiến đầu ra chưa hợp lệ dù không có lỗi ngữ nghĩa. Nó vẫn chặn hoàn tất, nhưng không tự là lý do gọi model dịch lại chương nếu sửa dữ liệu/cấu trúc đã đủ.

## 6. Danh mục 14 nhóm lỗi bản dịch

Danh mục dùng để tìm và giải thích vấn đề. Mỗi mục vẫn phải qua bốn tiêu chí ở trên; không tự gán S hoặc M theo tên nhóm. Các ví dụ diễn đạt ý nguồn bằng tiếng Việt cho dễ đối chiếu, không phải trích dẫn đã định vị trong một EPUB thực.

### 6.1. Sai nghĩa so với nguyên tác

- Đảo khẳng định/phủ định: “chưa nói” thành “đã nói”.
- Sai phạm vi phủ định: “không phải tất cả đều biết” thành “không ai biết”.
- Sai mức độ chắc chắn: “có lẽ” thành “chắc chắn”.
- Biến suy nghĩ, tin đồn hoặc lời kể của nhân vật thành sự thật được xác nhận.
- Nhầm khả năng, ý định, nghĩa vụ hoặc sự cho phép.
- Biến giả định/điều kiện thành sự kiện đã xảy ra.
- Sai mức độ, so sánh, giới hạn hoặc tính độc quyền: “chỉ A biết” thành “A cũng biết”.
- Đổi vô tình/cố ý, miễn cưỡng/tự nguyện hoặc tính chất hành động.
- Hiểu sai từ nhiều nghĩa, thành ngữ hoặc cấu trúc câu trong ngữ cảnh.

### 6.2. Bỏ sót nội dung

- Thiếu đoạn, câu, vế câu hoặc lượt thoại.
- Có câu dịch nhưng mất một ý hoặc chi tiết cần truyền đạt.
- Bỏ lời dẫn khiến mất người nói hoặc hoàn cảnh phát ngôn.
- Bỏ suy nghĩ nội tâm, hành động, phản ứng của nhân vật.
- Mất dấu hiệu phủ định, do dự, điều kiện, thời gian hoặc mức độ.
- Bỏ tiêu đề, chú thích, lời đề từ hoặc chú thích ảnh thuộc phạm vi cần dịch.
- Tóm tắt nhiều câu thành một câu làm mất ý.
- Bỏ sự lặp lại có dụng ý về nhịp, cảm xúc hoặc nhấn mạnh.

Diễn đạt gọn hơn vẫn có thể đúng nếu giữ đủ ý và hiệu ứng cần thiết.

### 6.3. Thêm nội dung không được nguyên tác hỗ trợ

- Thêm sự kiện, hành động, lời thoại hoặc suy nghĩ nội tâm.
- Tự gán động cơ, cảm xúc hoặc lý do khi nguồn chưa hỗ trợ.
- Tự xác lập quan hệ, như bạn bè thành người yêu hoặc tỏ tình thành đã được chấp nhận.
- Thêm ngoại hình, tuổi, năng lực, địa điểm hoặc quá khứ.
- Thêm giải thích rồi trình bày như lời của tác giả.
- Lấy anime, wiki, bản chuyển thể hoặc phần truyện sau để bổ sung vào đoạn hiện tại.
- Biến điều nguồn cố ý để mơ hồ thành một kết luận chắc chắn.

Phải kiểm tra theo chiều bản dịch → nguồn. Làm rõ chủ thể đã được ngữ cảnh xác nhận, thay trật tự câu hoặc diễn đạt tự nhiên không mặc nhiên là bịa. Với nghi vấn thiếu căn cứ, ghi rõ câu dịch, phạm vi nguồn đã kiểm tra và phần ngữ cảnh cần mở rộng.

### 6.4. Nhầm nhân vật, chủ thể và điểm nhìn

- Gán lời thoại cho sai người; nhầm người nghe hoặc người được nói tới.
- Đổi người thực hiện và người chịu tác động: “A cứu B” thành “B cứu A”.
- Đại từ hoặc cách gọi trỏ sai nhân vật.
- Gộp hai nhân vật thành một hoặc tách một người thành nhiều người.
- Gán nhầm biệt danh, danh hiệu, tên giả hoặc alias.
- Biến suy nghĩ nội tâm thành lời nói thành tiếng.
- Đổi lời kể thành suy nghĩ của nhân vật hoặc ngược lại.
- Đổi điểm nhìn khiến nhân vật biết điều chưa được biết trong nguồn.
- Tự xác định giới tính/danh tính khi chưa đủ căn cứ.

Nguồn chưa rõ người nói có thể dẫn đến Cần xác minh; không bắt buộc model chọn một người để hoàn thành hồ sơ.

### 6.5. Sai thời gian, không gian, nguyên nhân và mạch truyện

- Nhầm quá khứ, hiện tại của cảnh, dự đoán hoặc hồi tưởng.
- Đổi trạng thái đang làm, đã làm, vừa làm, chưa làm.
- Đảo thứ tự sự kiện.
- Sai nơi chốn, hướng di chuyển hoặc vị trí nhân vật.
- Biến sự nối tiếp thành nguyên nhân–kết quả, hoặc đảo nguyên nhân và kết quả.
- Cho nhân vật biết bí mật trước mốc nguồn cho phép.
- Nhầm giấc mơ, giả tưởng, diễn kịch hoặc lời kể chưa chắc chắn với sự kiện thật.
- Tạo mâu thuẫn giữa cảnh/chương mà nguyên tác không có.

Nếu nguồn có điểm bất nhất, ghi nhận riêng. Không tự sửa cốt truyện rồi coi phần đã chỉnh là bản dịch trung thành.

### 6.6. Sai xưng hô và diễn biến quan hệ

- Sai cách tự xưng hoặc gọi đối phương.
- Đảo A → B với B → A, hoặc áp cách A nói với B sang A nói với C.
- Đổi xưng hô không có căn cứ; không đổi sau mốc đã xác nhận.
- Đổi quá sớm hoặc áp quan hệ hiện tại vào hồi tưởng.
- Biến giận nhất thời, đùa, đóng vai hoặc giữ khoảng cách thành thay đổi lâu dài.
- Bỏ ngoại lệ theo cảnh, như cách nói trước đám đông.
- Áp quy tắc hội thoại vào mọi lời kể về nhân vật.
- Dùng đề xuất chưa duyệt như quy tắc chính thức.
- Suy ra máy móc “tỏ tình = anh–em” hoặc “chia tay = tao–mày”.

Phải chỉ rõ cặp nhân vật, chiều nói, mốc, quy ước và bằng chứng. Sự kiện nguồn và lựa chọn đại từ tiếng Việt được đánh giá riêng theo đặc tả xưng hô.

### 6.7. Sai tên riêng, thuật ngữ và danh xưng

- Một nhân vật hoặc khái niệm có nhiều cách viết không theo quy ước.
- Nhầm tên người với địa điểm, tổ chức hoặc loại thực thể khác.
- Tự đoán cách đọc tên khi thiếu căn cứ.
- Sai thuật ngữ về năng lực, phép thuật, vật phẩm hoặc hệ thống thế giới.
- Gộp hai khái niệm vào một thuật ngữ, hoặc tách một khái niệm thành nhiều thuật ngữ gây hiểu sai.
- Sai chức vụ, cấp bậc hoặc danh hiệu.
- Thêm/bỏ kính ngữ làm đổi quan hệ khi yếu tố đó cần được giữ theo nguồn/quy ước.
- Làm lộ tên thật khi nguyên tác vẫn dùng bí danh.

Glossary mâu thuẫn với nguồn cần được xác minh; không ép nguồn theo một mục từ đã lưu sai.

### 6.8. Sai số liệu và chi tiết cụ thể

- Sai số lượng, tuổi, ngày tháng, thời lượng hoặc thứ tự.
- Sai thứ hạng, số tầng, số phòng hoặc thông tin định danh tương tự.
- Sai đơn vị đo, quy đổi, giá tiền, tỷ lệ, khoảng cách hoặc tốc độ.
- Biến số gần đúng thành số chính xác; đổi ít nhất, nhiều nhất, hơn hoặc kém.
- Sai màu sắc, bộ phận cơ thể, bên trái/phải hoặc đặc điểm đồ vật.

Mức ảnh hưởng phụ thuộc vai trò chi tiết trong truyện. Không mặc định mọi lỗi số liệu đều nhỏ hoặc đều nghiêm trọng.

### 6.9. Sai giọng nhân vật, cảm xúc và sắc thái

- Lịch sự thành thô lỗ hoặc ngược lại; do dự thành quả quyết.
- Mỉa mai thành khen thật; đùa thành đe dọa nghiêm túc.
- Lạnh nhạt thành thân mật hoặc đổi thái độ với người nghe.
- Làm mạnh/nhẹ cảm xúc, lời chửi, tình cảm hoặc bạo lực vượt nguồn.
- Làm mất khác biệt có chủ ý giữa giọng các nhân vật hoặc cách nói đặc trưng đã xác nhận.
- Đổi giọng người kể, như trang trọng thành suồng sã hoặc tiết chế thành lên gân.

Reviewer cần nêu sắc thái nguồn và phần bản dịch làm lệch sắc thái. Sở thích văn phong chưa đủ thành lỗi.

### 6.10. Làm mất dụng ý ngôn ngữ và cách kể

- Dịch nghĩa đen làm mất nghĩa của ẩn dụ/thành ngữ.
- Bỏ hoặc hiểu sai chơi chữ có vai trò trong cảnh.
- Làm mất lời nước đôi hoặc bí ẩn tác giả cố ý giữ; giải thích quá sớm làm lộ thông tin.
- Làm mất đối lập, nhấn mạnh hoặc nhịp có ý nghĩa.
- Sai tác dụng của tượng thanh, tượng hình hoặc tiếng cảm thán.
- Bỏ ý nghĩa của chú âm/ruby khi nó mang thông tin khác chữ chính.
- Xử lý sai tham chiếu văn hóa làm người đọc hiểu nhầm hành động hoặc quan hệ.

Có thể tồn tại nhiều phương án dịch hợp lệ. Đánh giá theo ý nghĩa/hiệu ứng cần giữ và chính sách dịch, không ép một câu chữ duy nhất.

Với ruby, phải đọc cả chữ chính và phần chữ nhỏ để xác định chức năng trong ngữ cảnh. Ruby có ý nghĩa riêng cần phương án người dùng duyệt trước khi áp dụng; khi chưa đủ căn cứ thì Cần xác minh, không tự chấm lỗi chỉ vì cách thể hiện khác hình thức nguồn. Nếu đã xác nhận mất ý, chọn S1–S3 và M1–M4 theo ảnh hưởng thực tế như các lỗi khác.

### 6.11. Lỗi tiếng Việt và khả năng đọc hiểu

- Sai chính tả, dấu thanh, ngữ pháp hoặc thiếu thành phần cần thiết.
- Dùng từ sai nghĩa, kết hợp từ không phù hợp.
- Bám cấu trúc nguồn khiến ý khó hiểu hoặc bị hiểu sai.
- Đại từ tiếng Việt gây nhầm người.
- Lặp/thừa từ hoặc diễn đạt vòng vèo ngoài dụng ý.
- Dấu câu làm sai ý hoặc ranh giới lời thoại; câu bị đứt/chưa hoàn thành.
- Lẫn ngôn ngữ ngoài ý muốn hoặc giọng văn thay đổi bất thường.

Từ nguồn được giữ theo chính sách, tên riêng hoặc ruby hợp lệ không tự là lỗi lẫn ngôn ngữ. Phương án khác vẫn đúng, rõ và phù hợp quy ước là S0 hoặc không cần báo.

### 6.12. Hỏng cấu trúc và liên kết nội dung

- Thiếu, trùng hoặc sai ID; bản dịch gắn nhầm đoạn nguồn.
- Đổi thứ tự, gộp/tách làm mất mapping cần thiết.
- Hỏng token, thẻ, ký hiệu hoặc thành phần phải bảo toàn.
- Mất tiêu đề, ranh giới cảnh hoặc định dạng có ý nghĩa.
- Mất/sai vị trí ảnh hoặc tham chiếu tài nguyên.
- Hỏng chú thích, liên kết đến chú thích hoặc liên kết quay lại.
- Hỏng ruby và thành phần định dạng được bảo vệ.
- Văn bản bị cắt cụt khi nhận, lưu hoặc ghép kết quả.

Phần lớn điều kiện cấu trúc có thể kiểm tra bằng chương trình và phải kiểm tra lại sau sửa tay. Với chữ trong ảnh hoặc thành phần được đánh dấu không dịch, chỉ báo thiếu dịch nếu thuộc phạm vi đã yêu cầu. Lỗi ảnh hưởng việc xuất/đọc có thể chặn hoàn tất mà chưa cần chấm như lỗi ngữ nghĩa S0–S3.

Theo lựa chọn nội dung chốt ngày **30/10/2026** trong [đặc tả đầu vào](EPUB_INPUT_SPEC.md), trang phụ giữ nguyên hợp lệ của chế độ chỉ truyện không bị báo thiếu dịch chỉ vì còn nguyên ngữ. Cùng trang đó trong chế độ toàn bộ phần chữ vẫn thuộc diện phải dịch/Review. Chưa phân loại hoặc bản dịch còn thiếu không tự được đổi thành giữ nguyên để tránh lỗi; tình trạng giữ nguyên không phải nhãn Đạt Review.

Chuyển chữ Việt sang đọc ngang, dùng nhãn mục lục đã dịch và không hiển thị chú âm thuần phát âm theo đặc tả đầu ra không tự là hỏng cấu trúc. Mapping nguồn, quan hệ liên kết và ý nghĩa cần giữ vẫn phải đầy đủ; không dùng quy tắc bảo toàn marker để buộc giữ nguyên hình thức dàn chữ Nhật.

### 6.13. Lỗi ở ranh giới các lượt dịch

- Dịch lặp đoạn chỉ dùng làm ngữ cảnh hoặc bỏ câu ở ranh giới.
- Ghép hai nửa câu sai thứ tự.
- Đổi tên, xưng hô, thuật ngữ, thời gian kể hoặc giọng văn khi sang lượt mới mà nguồn không hỗ trợ.
- Mất chủ thể do không nối được ngữ cảnh.
- Lặp lời giới thiệu hoặc thông tin nguồn chỉ nêu một lần.
- Lượt sau bỏ qua thay đổi quan hệ đã được xác nhận và đang có hiệu lực.
- Ghép các revision không tương thích.
- Dùng bản bị đánh trượt hoặc ngữ cảnh hết hiệu lực để dịch tiếp.

Cần đối chiếu hai phía ranh giới và dependency; một đoạn riêng lẻ có thể trông đúng nhưng kết quả ghép bị sai.

### 6.14. Vi phạm yêu cầu dịch đã cấu hình

- Không tuân thủ glossary, cách viết hoặc quy ước còn hiệu lực.
- Dùng sai phong cách đã được xác định rõ.
- Tự tóm tắt, phóng tác, biên tập lại cốt truyện hoặc bỏ nội dung.
- Thêm chú thích người dịch khi chưa được cho phép.
- Thay thành phần phải giữ nguyên.
- Đưa lời giải thích của AI, lời xin lỗi, hướng dẫn thao tác, prompt, nhận xét Review hoặc metadata vào truyện.

Quy ước phải rõ, đúng phiên bản và có hiệu lực trong phạm vi đang xét. Nếu các yêu cầu mâu thuẫn nhau hoặc mâu thuẫn với nguồn, báo xung đột thay vì âm thầm chọn rồi coi lựa chọn đó là bắt buộc.

## 7. Lỗi của chính kết quả Reviewer

Các vấn đề sau làm giảm hoặc mất hiệu lực của kết quả Review; chưa phải bằng chứng bản dịch sai:

- Bịa quote, trích sai nguyên văn hoặc gán sai vị trí.
- Quote có thật nhưng không hỗ trợ kết luận; bỏ ngữ cảnh làm đảo nghĩa bằng chứng.
- Đối chiếu nhầm chương, snapshot, revision hoặc quy tắc.
- Chỉ dựa vào tóm tắt/bản dịch cũ để kết luận nguyên tác nói gì.
- Kết luận “nguồn không có” khi phạm vi đã kiểm tra chưa đủ.
- Bỏ sót phạm vi hoặc chỉ kiểm tra một chiều nhưng báo hoàn tất.
- Dùng suy đoán về danh tính, cảm xúc hoặc quan hệ như sự thật.
- Bắt lỗi vì sở thích; ép một phương án diễn đạt duy nhất dù có phương án hợp lệ khác.
- Đề xuất sửa thêm ý, mất ý hoặc hỏng cấu trúc.
- Gán mức ảnh hưởng không có lý do hoặc yêu cầu M4 mà không chứng minh phạm vi.
- Bỏ qua bằng chứng phản bác hoặc phản hồi đã được xác minh.
- Trả kết quả rỗng, cắt cụt, thiếu dữ liệu hoặc sai hợp đồng đầu ra.

Core phải kiểm tra các điều kiện máy xác minh được trước khi dùng kết quả. Phần diễn giải cần được đối chiếu ngữ nghĩa theo cơ chế Review và kiểm chứng bổ sung đã chốt. Nếu báo lỗi bị bác, giữ lịch sử và chưa thay văn bản theo báo lỗi đó.

Không lấy lỗi của Reviewer cộng vào số lỗi bản dịch hoặc tự nâng M. Một kết quả Review không hợp lệ cũng không được chuyển thành kết luận bản dịch đã đạt.

## 8. Hồ sơ quyết định và cách hiển thị

### 8.1. Thông tin bắt buộc

| Thông tin | Yêu cầu |
| --- | --- |
| Vị trí và phiên bản | Tập, chương, block/phạm vi câu, nguồn và revision target |
| Loại lỗi | Một nhóm chính, có thể có nhãn liên quan |
| Trạng thái xác minh | Đã xác nhận lỗi, Cần xác minh hoặc Đã bác nhận định |
| Bằng chứng | Quote nguồn và target, định vị, ngữ cảnh liên quan, phạm vi đã kiểm tra |
| Giải thích | Thông tin nào lệch, bằng chứng hỗ trợ ra sao, có điều gì chưa rõ |
| Mức ảnh hưởng | S0–S3 và lý do; ghi dự kiến nếu chưa xác nhận |
| Phạm vi ảnh hưởng | Danh sách vị trí hoặc mối liên hệ cần xem; không chỉ một nhãn “toàn chương” |
| Khả năng sửa cục bộ | Phần cần thay, phần giữ lại, lý do có thể giữ hoặc cần tạo lại |
| Mức xử lý | M1–M4 và lý do, hoặc nhánh xác minh/sửa dữ liệu/review lại |
| Điều kiện hoàn tất | Những thông tin phải đúng, lỗi phải hết và các liên kết cần kiểm tra |
| Lịch sử | Quyết định, người/cơ chế thực hiện, revision, phản hồi và số vòng đã dùng |

Thiếu nội dung có thể không có câu target tương ứng: ghi khoảng nguồn bị thiếu và vị trí dự kiến, không tạo một quote target giả. Với lỗi cấu trúc, dùng bằng chứng cấu trúc/mapping liên quan thay vì ép tạo một lỗi ngữ nghĩa.

### 8.2. Ví dụ thẻ đánh giá

Ví dụ dưới đây là tình huống giả định. Khi chạy trên sách thực, các mô tả bằng chứng phải đi kèm trích dẫn và định vị thật trong snapshot/revision đang xét.

```text
Vấn đề: Nhầm người nói trong cảnh b021–b034
Trạng thái: Đã xác nhận lỗi

Bằng chứng:
Lời dẫn nguồn xác định A là người thú nhận.
Bản dịch gán lời thú nhận và các lượt thoại liên quan cho B.
Các phản ứng trong cảnh bị đảo theo cùng nhầm lẫn.

Mức ảnh hưởng: S3 — Nghiêm trọng
Lý do: Đổi người thú nhận và người phản ứng trong sự kiện quan trọng.

Phạm vi ảnh hưởng: b021–b034
Khả năng sửa cục bộ:
Chưa thể chỉ thay tên; đại từ, xưng hô và lời dẫn trong cảnh
đều bị tạo theo cách hiểu sai và cần được dựng lại.

Hướng xử lý: M3 — Dịch lại cảnh
Phần ngoài phạm vi: Giữ sau khi kiểm tra liên kết trước/sau cảnh.

Điều kiện hoàn tất:
- Đúng người nói và người nghe.
- Giữ đúng nội dung thú nhận, phản ứng và thứ tự sự kiện.
- Đúng quy ước xưng hô tại mốc đó.
- Nối được với cảnh trước và sau.
```

TUI hiển thị riêng mức ảnh hưởng và hướng xử lý, ví dụ **S3 — Nghiêm trọng / M3 — Dịch lại cảnh**. Người dùng mở được nguyên tác, target, lý do chọn phạm vi và lịch sử. Với vấn đề chưa xác nhận, ưu tiên nhãn Cần xác minh; chưa trình bày M dự kiến như lệnh đang được thực hiện.

## 9. Điều kiện hoàn tất và giới hạn sửa

### 9.1. Sau khi sửa

Mỗi thay đổi qua kiểm tra cấu trúc, lưu revision mới và review lại. Review phải xác nhận lỗi cũ đã được xử lý, không phát sinh lỗi mới và các liên kết/ngữ cảnh chịu ảnh hưởng vẫn phù hợp.

M1 và M2 có thể chỉ cần review lại phạm vi sửa cùng ngữ cảnh liên quan; không tự review lại những phần không có dependency bị đổi. Kết luận cấp chương vẫn phải được cập nhật hoặc kiểm tra lại theo những liên kết chịu ảnh hưởng. M4 cần review đầy đủ bản chương mới.

Một câu sửa do Reviewer đề xuất không được tự coi là đáp án đúng. Nếu sửa làm thay đổi nguồn/revision/dependency của bằng chứng cũ, kết quả dựa trên bằng chứng đó phải được kiểm tra hiệu lực trước khi dùng.

### 9.2. Điều kiện duyệt

Để một phạm vi mang nhãn **Đạt**, cần:

- Dữ liệu nguồn, bản dịch và cấu trúc hợp lệ.
- Đã đối chiếu đầy đủ phạm vi được giao theo hai chiều trên đúng phiên bản.
- Các lỗi đã xác nhận S1–S3 thuộc phạm vi/yêu cầu dịch đã được giải quyết.
- Không còn điểm chưa rõ có thể thay đổi kết luận duyệt.
- Những thay đổi và quyết định đã được lưu, chưa bị dependency làm mất hiệu lực.

Gợi ý S0 có thể còn lại. Chương chỉ mang nhãn Đạt sau khi mọi phần đáp ứng điều kiện và kiểm tra mạch chương hoàn tất. Không coi kết quả rỗng hoặc chưa có báo lỗi là chứng minh đã đạt.

Nếu người dùng giữ bản còn vấn đề, ghi **Người dùng giữ — có ngoại lệ**, lý do, phạm vi và revision. Không đổi nhãn thành Reviewer xác nhận đúng; không tự đưa các sự kiện/quy tắc đang tranh chấp vào bộ nhớ chính thức.

### 9.3. Khi lỗi ảnh hưởng ngữ cảnh và phần phía sau

Giữ bản bị đánh trượt trong lịch sử, chặn dùng phần sai làm ngữ cảnh đã xác nhận. Candidate lấy từ phần đó chưa được công bố. Nếu bộ nhớ đã xác nhận bị chứng minh sai, đánh dấu không còn hợp lệ và truy các phần thực sự phụ thuộc để review/sửa lại.

Không tự dịch lại toàn bộ bộ truyện chỉ vì một lỗi ở chương trước. Xác định phạm vi theo dependency, mốc hiệu lực và bằng chứng; bảo toàn sửa tay và các phiên bản cũ.

### 9.4. Năm vòng sửa–review

Giữ giới hạn **5 vòng sửa–review thất bại** theo đặc tả Review. Mở rộng từ M1/M2 lên M3/M4 trong cùng đợt lỗi không đặt lại bộ đếm. Một vòng dịch lại chương có nhiều request vẫn thuộc cùng kế hoạch sửa; số request không phải số vòng.

Retry mạng, lỗi hợp đồng và sửa cấu trúc có giới hạn riêng, không tạo quyền sửa chất lượng vô hạn. Khi hết vòng, hoặc khi cần người dùng xác minh sớm hơn, dừng tự sửa và hiển thị bằng chứng, lịch sử cùng điều còn chưa giải quyết.

## 10. Các ví dụ chọn mức

Tất cả tình huống trong bảng là ví dụ thiết kế, với điều kiện bằng chứng và phạm vi được kiểm tra như mô tả.

| Tình huống | Mức ảnh hưởng | Hướng xử lý và lý do |
| --- | --- | --- |
| Tên viết sai ở 20 chỗ, mọi chỗ đều xác định đúng cùng nhân vật | S1 | M1 tại danh sách vị trí; không dịch lại chương chỉ vì nhiều lần xuất hiện |
| Một từ phủ định then chốt bị thiếu; đã xác nhận thêm đúng từ khôi phục ý và các phần xung quanh vẫn đúng | S3 | M1 có kiểm tra lại; mức nghiêm trọng không buộc tạo lại cả đoạn |
| Một đoạn hiểu ngược sự kiện, nhiều vế trong đoạn được tạo theo nghĩa sai; phần ngoài vẫn đúng | S2 hoặc S3 theo vai trò sự kiện | M2 vì cần dựng lại đoạn và đã khoanh được ảnh hưởng |
| Nhầm người nói làm sai toàn cuộc hội thoại, cần viết lại đại từ/lời dẫn | S3 nếu làm đổi sự kiện then chốt | M3 trên cảnh đã xác định |
| Xưng hô sai tại một số lời A nói với B, cặp và mốc đã chắc chắn, câu thay đã rõ | S2 hoặc S3 theo ảnh hưởng | M1 tại các vị trí xác nhận; không đổi lời A nói với C |
| Bịa chia tay rồi đổi động cơ, lời thoại, quan hệ xuyên chương | S3 | M4 sau kiểm chứng và xử lý ngữ cảnh sai |
| Chưa rõ một đại từ nguồn chỉ A hay B | Chưa chốt | Cần xác minh ngữ cảnh; chưa dịch lại theo một giả thuyết |
| Lỗi mapping gắn bản dịch đúng sang block khác | Chặn do dữ liệu/cấu trúc | Sửa mapping và kiểm tra lại; chỉ dịch lại phần thực sự cần sau khi sửa dữ liệu |
| Reviewer dùng quote của revision cũ để bắt lỗi bản hiện tại | Kết quả Review chưa hợp lệ | Review lại đúng revision; không nâng M cho bản dịch chỉ vì Reviewer sai |
| Hai cách viết đều đúng ý, rõ và phù hợp chính sách | S0 hoặc không báo vấn đề | Không bắt buộc sửa, không chặn duyệt |

## 11. Tiêu chí nghiệm thu khi triển khai

Kiểm thử tự động dùng dữ liệu tự tạo và model giả lập, offline và deterministic. Các tiêu chí kiểm tra sự tuân thủ rubric và điều phối; không chứng minh model thực phát hiện đúng mọi lỗi ngôn ngữ.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Một vấn đề đã xác nhận | Có bằng chứng, S, phạm vi, khả năng sửa và lý do chọn M |
| Thiếu/mâu thuẫn bằng chứng | Giữ Cần xác minh, chưa sửa theo kết luận chưa chắc |
| Quote khớp nhưng không chứng minh giải thích | Có thể bác hoặc xác minh lại nhận định |
| Chỉ khác sở thích diễn đạt | Không biến thành S1–S3 hoặc lỗi chặn |
| Chú âm thuần phát âm được xử lý đúng chính sách đầu ra | Không tự báo bỏ sót; vẫn kiểm tra căn cứ phân loại và mapping nguồn |
| Mất ý nghĩa riêng của ruby | Nêu được hai lớp nguồn và ảnh hưởng; chọn S/M theo phạm vi, không tự buộc dịch lại chương |
| S3 nhưng cách sửa chính xác và cục bộ | Có thể chọn M1/M2, không buộc M4 |
| Nhiều lỗi S1 cùng loại | Không cộng điểm hoặc đếm lỗi để tự chọn M4 |
| Một lỗi có nhiều nhãn | Không đếm trùng như nhiều lỗi độc lập |
| Phạm vi rộng nhưng từng thay thế đã chắc chắn | Có thể dùng M1 tại danh sách vị trí |
| M3 | Có phạm vi cụ thể, ngữ cảnh và kiểm tra liên kết với phần giữ lại |
| M4 | Có kết luận cấp chương, kiểm chứng bổ sung và kiểm tra nguyên nhân |
| Chưa biết phạm vi vì thiếu nguồn | Xác minh/sửa nguồn; không tự coi cả chương sai |
| Lỗi của Reviewer hoặc dữ liệu | Đi đúng nhánh xử lý, không quy thành lỗi bản dịch chưa được chứng minh |
| Áp đề xuất hoặc sửa tay | Lưu revision mới, kiểm tra cấu trúc và review lại |
| Revision đổi khi tác vụ đang chạy | Không áp kết quả cũ lên nền mới hoặc ghi đè sửa tay |
| Sửa hết lỗi cũ nhưng phát sinh lỗi mới | Chưa duyệt chỉ vì lỗi cũ đã đổi |
| Hết năm vòng hoặc đổi mức xử lý | Dừng đúng giới hạn, không tự đặt lại bộ đếm |
| Người dùng giữ bản có lỗi | Nhãn ngoại lệ và lý do rõ, không tự duyệt candidate ngữ cảnh |
| Từng đoạn đạt nhưng kiểm tra mạch chương chưa xong | Chương chưa mang nhãn Đạt |

## 12. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [23](DESIGN_DECISIONS_36.md#decision-23), [25](DESIGN_DECISIONS_36.md#decision-25) của bản 36 mục áp dụng cho đặc tả này.

### 12.1. Thuật toán lựa chọn phạm vi sửa M1–M4 — mục 23

Reviewer đề xuất mức sửa, vị trí, nguyên nhân, bằng chứng và phần phụ thuộc. Core kiểm tra điều kiện và chọn phạm vi nhỏ nhất có thể sửa đúng. **M1** vá câu/cụm chính xác, kể cả ở nhiều vị trí đã định danh. **M2** dịch lại một đoạn. **M3** dịch lại cảnh hoặc nhóm đoạn, có danh sách vị trí và lý do chúng phụ thuộc nhau. **M4** dịch lại chương khi chứng minh được sai lệch xuyên chương, lý do sửa chọn lọc không đủ và một lượt kiểm chứng bổ sung trên nguyên tác xác nhận. Lượt kiểm chứng có quyền thu hẹp hoặc bác đề xuất.

Sửa nguyên nhân ở mapping, quy ước hoặc nguồn ngữ cảnh trước khi dịch lại. Lưu các revision trước và dùng lại phần nằm ngoài phạm vi vẫn hợp lệ. Sau M1/M2, kiểm tra phần vừa sửa, phần thật sự dùng nó làm ngữ cảnh và kết luận chương liên quan. Dù lỗi ít hay nhiều, quyết định M dựa trên phạm vi chứng minh được. [Rubric và điều kiện M4](REVIEW_RUBRIC_SPEC.md)

### 12.2. Vòng sửa chất lượng và ngoại lệ người dùng — mục 25

Theo dõi `repair_round_id` cho một kế hoạch sửa có phạm vi, nhóm lỗi, revision nền và revision mới. Một vòng có thể gồm nhiều request. Chỉ tính một vòng chất lượng thất bại khi đã có bản sửa hợp lệ và Review lại vẫn xác nhận lỗi của đợt đó. Review đầu tiên, lỗi mạng, lỗi cấu trúc và thời gian chờ duyệt có bộ đếm/trạng thái riêng.

Hết **năm vòng thất bại**, bàn giao bằng chứng và lịch sử. Người dùng sửa tay tạo revision mới và có thể yêu cầu Review bản đó. Giữ số vòng đã dùng. Nếu người dùng chọn giữ lỗi, lưu người duyệt, lý do, issue, mức, phạm vi, revision và căn cứ. Hiện **Người dùng giữ — có ngoại lệ**. Thay revision/căn cứ của phần được miễn thì cần xét lại ngoại lệ. Các đơn vị khác còn nguyên giữ được lịch sử riêng. Điều kiện độ phủ, Review chương, quyết định thuật ngữ và tính hợp lệ kỹ thuật vẫn phải đạt.

### 12.3. Liên kết và yêu cầu triển khai

Trường dữ liệu issue và kiểm tra bằng chứng theo [đặc tả Review](REVIEW_STAGE_SPEC.md), mục 13. Trạng thái đạt và hiệu lực của kết luận đi qua ba trục trong [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md).
