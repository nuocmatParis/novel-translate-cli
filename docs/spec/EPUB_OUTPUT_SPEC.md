# Quyết định 11 — Bố cục, nội dung và điều kiện xuất EPUB tiếng Việt

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 12.

Ngày ghi nhận: **29/10/2026**.

Cập nhật ngày **30/10/2026**: điều kiện hoàn chỉnh căn cứ lựa chọn **Dịch toàn bộ phần chữ** hoặc **Chỉ dịch nội dung truyện và phần liên quan** theo [đặc tả đầu vào](EPUB_INPUT_SPEC.md). Phần được giữ nguyên ở lựa chọn thứ hai vẫn có mặt trong EPUB và được hiển thị rõ trên màn chuẩn bị.

Cùng ngày chốt: **đầu ra thống nhất EPUB 3** cho cả nguồn EPUB 2 và EPUB 3; **mỗi lần xuất tạo một file phiên bản mới (6A)** trong thư mục đầu ra, giữ các bản đã xuất trước đó. Chi tiết ở mục 9 và 10.

Trạng thái: **đã chốt hướng thiết kế trong trao đổi**. Tài liệu mô tả hành vi cần triển khai, không khẳng định bản Go hoặc Rust hiện đã đáp ứng. Command theo [đặc tả TUI/CLI](TUI_CLI_SPEC.md). Đích EPUB 3.3, bộ chuẩn bị và hồ sơ xuất đã chốt tại mục 12. Theo quyết định ngày **30/10/2026** trong [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), không cần chuyển workspace test cũ; dữ liệu, lịch sử và file EPUB xuất từ workspace mới vẫn phải được bảo toàn, với chính sách nâng cấp có sao lưu, kiểm tra và phục hồi đã chốt trong đặc tả dữ liệu.

Tài liệu liên kết [EPUB đầu vào](EPUB_INPUT_SPEC.md), [Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [rubric](REVIEW_RUBRIC_SPEC.md), [Skills và duyệt quy ước](SKILLS_GLOSSARY_SPEC.md), [dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md), [điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), [thay ảnh và preview](IMAGE_EDIT_SPEC.md).

Phạm vi quyết định này là cách trình bày bản tiếng Việt, tiêu đề/mục lục, định dạng có ý nghĩa, chú thích, ruby, ranh giới giữa Dịch–Review với ghép EPUB, điều kiện xuất toàn tập, thông tin sách, phiên bản EPUB và quản lý file xuất. Hợp đồng kỹ thuật đã chốt ngày **04/11/2026** được ghi tại mục 12.

## 1. Các quyết định đã chốt

| Thành phần | Quy tắc |
| --- | --- |
| Bản xuất mặc định | Bản tiếng Việt đọc ngang, từ trái sang phải. |
| Phiên bản EPUB đầu ra | Thống nhất EPUB 3 cho cả nguồn EPUB 2 và EPUB 3 trong phạm vi hỗ trợ. |
| Xuất lại | Mỗi lần xuất tạo file phiên bản mới, tăng số phiên bản và giữ bản trước; không ghi đè file đã có. |
| Chương, đoạn văn, ngắt cảnh | Giữ cấu trúc và thứ tự đã xác định từ nguồn. Các lượt dịch không trở thành phần mới của sách. |
| Tiêu đề và mục lục | Dùng nội dung đã dịch và được chấp nhận trong phạm vi dịch; nhãn thuộc trang phụ giữ nguyên theo lựa chọn nội dung. Mục lục dẫn đúng đến vị trí tương ứng. |
| Nhấn mạnh | Giữ ý nghĩa của in đậm, in nghiêng và các dấu hiệu nhấn mạnh trong nguồn. |
| Ảnh minh họa | Giữ vị trí; chọn ảnh source/edit theo đặc tả ảnh. |
| Chú thích | Dịch phần thuộc phạm vi dịch, giữ nguyên phần chỉ thuộc trang phụ được giữ nguyên; bảo toàn liên kết tới chú thích và chiều quay lại nếu nguồn có. |
| Ruby chỉ hướng dẫn phát âm | Mặc định không hiển thị lại phần chú âm trong bản Việt sau khi đã xác định không làm mất ý nghĩa. |
| Ruby mang thêm ý nghĩa | Đọc cả chữ chính và chữ nhỏ, đưa bằng chứng/phương án vào Dịch–Review để người dùng chốt cách thể hiện. |
| Quyết định liên quan đến nghĩa | Hoàn tất trong Dịch–Review trước khi xuất. Bước xuất không tự dịch thêm hoặc tự bỏ nội dung. |
| Phạm vi xuất giai đoạn đầu | Chỉ xuất EPUB hoàn chỉnh của tập theo lựa chọn nội dung: đủ phần phải dịch và đủ các phần giữ nguyên/tài nguyên. Xuất EPUB dở dang để giai đoạn sau. |
| Ngoại lệ chất lượng | Có thể xuất khi người dùng đã chủ động giữ vấn đề với lý do, phạm vi và revision; hiển thị rõ có ngoại lệ, các điều kiện còn lại vẫn phải đạt. |
| Thông tin sách | Điền sẵn tên sách/tập, tác giả và ngôn ngữ theo mục 8; người dịch/nhóm dịch là tùy chọn. Lưu lựa chọn theo từng tập để dùng lại. |

## 2. Bố cục đọc ngang và cấu trúc cần giữ

Phần chữ tiếng Việt được trình bày ngang và đọc từ trái sang phải. Với nguồn Nhật trình bày dọc, phần dựng EPUB phải điều chỉnh các quy tắc trình bày liên quan cho bản Việt, thay vì sao chép nguyên CSS khiến bản dịch tiếp tục bị dàn dọc.

Việc điều chỉnh bố cục không đảo thứ tự truyện hoặc thứ tự tài nguyên trong luồng đọc. Giữ các chương/part, ranh giới đoạn và ngắt cảnh, vị trí lời thoại, ảnh inline, trang minh họa và quan hệ giữa văn bản với tài nguyên theo mapping đã xác lập.

Bìa và trang tranh có bố cục cố định vẫn giữ vị trí và cách trình bày riêng phù hợp. Không xoay, lật, crop hoặc sửa bitmap chỉ để chuyển phần chữ sang đọc ngang. Ảnh người dùng thay vẫn theo [đặc tả ảnh](IMAGE_EDIT_SPEC.md), bao gồm trường hợp khác kích thước hoặc tỷ lệ.

Giữ ý nghĩa của định dạng nhấn mạnh khi dựng bản Việt. Bảo toàn tài nguyên nguồn và mapping không đồng nghĩa giữ nguyên từng byte CSS, XHTML hoặc nhãn mục lục trong bản xuất: các thành phần này có thể cần cập nhật để chứa bản dịch và bố cục đã chốt. Mọi tham chiếu sau cập nhật vẫn phải hợp lệ.

Preview dùng cùng cơ sở bố cục với EPUB đầu ra. Tiêu chí là chữ Việt hiển thị đầy đủ, dấu tiếng Việt và nhấn mạnh đọc được, nội dung và ảnh đúng vị trí; không cam kết số trang hoặc cách xuống dòng giống từng pixel giữa các ứng dụng đọc sách.

## 3. Tiêu đề, mục lục và chú thích

### 3.1. Tiêu đề chương/part và nhãn mục lục

Tiêu đề chương/part và nhãn mục lục gắn với nội dung được dịch phải được xử lý trong Dịch–Review. Nhãn chỉ thuộc trang phụ được giữ nguyên thì theo cùng lựa chọn của trang đó. Import phải kiểm kê và lưu mapping của tất cả các mục; bước xuất dùng bản đã được chấp nhận hoặc nguồn được giữ nguyên đúng phạm vi, không gọi model để dịch các nhãn còn thiếu.

Khi mục lục và tiêu đề trong chương cùng biểu diễn một tiêu đề nguồn, dùng cùng kết quả đã duyệt để tránh hai cách gọi khác nhau ngoài chủ ý. Nếu nguyên tác dùng nhãn mục lục rút gọn hoặc có nội dung khác tiêu đề trong chương, giữ quan hệ giữa chúng và xử lý phần khác biệt trong Dịch–Review; không tự đồng nhất đến mức mất thông tin.

Giữ cấp mục lục, thứ tự và đích liên kết. Một mục dẫn đến giữa chương hoặc tới tiêu đề part vẫn phải dẫn đến vị trí đó. Không sinh thêm mục vì một chương đã được chia thành nhiều lượt dịch.

Tiêu đề có tên riêng, thuật ngữ hoặc lựa chọn mới phải tuân theo quy trình duyệt quy ước. Việc tiêu đề ngắn hoặc nằm trong mục lục không cho phép tự chốt một tên mới.

### 3.2. Chú thích

Trích và dịch nội dung chú thích thuộc phạm vi được dịch, giữ liên kết với đúng chỗ gọi và liên kết quay lại nếu có. Chú thích chỉ thuộc trang phụ giữ nguyên thì giữ theo trang đó. Chú thích ở file riêng hoặc ngoài luồng đọc chính vẫn phải được theo dõi đầy đủ, không bị bỏ sót chỉ vì không mang nhãn chương.

Một chú thích được nhiều vị trí tham chiếu phải dùng đúng bản dịch đã lưu; không tự tạo nhiều bản dịch độc lập mâu thuẫn nhau. Những đoạn nguồn hoặc tên riêng được giữ nguyên theo quyết định đã duyệt vẫn tuân theo quy ước đó.

Chú thích gốc và chú thích do người dịch đề xuất là hai trường hợp khác nhau. Tạo chú thích mới để giải thích ruby hoặc chơi chữ phải nằm trong phương án người dùng đã duyệt; bước đóng gói không tự bổ sung lời giải thích.

## 4. Xử lý ruby trước khi xuất

### 4.1. Luôn giữ đủ nguồn để đối chiếu

Ruby là phần chữ nhỏ đi kèm chữ chính, có thể chỉ ghi cách đọc hoặc mang thêm ý nghĩa. Snapshot và mapping nguồn giữ đầy đủ chữ chính, phần chữ nhỏ và phạm vi liên kết giữa chúng. Dịch và Review nhận cả hai thành phần cùng ngữ cảnh liên quan.

Không xóa ruby ngay khi import hoặc lược bỏ khỏi ngữ cảnh chỉ vì bản xuất mặc định đọc ngang. Việc không hiển thị một phần chú âm ở bản Việt là quyết định thể hiện đầu ra, không phải xóa bằng chứng nguồn.

### 4.2. Ruby chỉ hướng dẫn phát âm

Nếu Dịch–Review xác định phần chữ nhỏ chỉ hướng dẫn cách đọc và không có ý nghĩa riêng cần truyền đạt, bản Việt mặc định dùng nội dung/tên đã dịch, không hiển thị lại lớp chú âm đó.

Ví dụ một tên nhân vật có chú âm chỉ cách đọc: dùng tên đã được người dùng duyệt, vẫn lưu chữ chính và cách đọc trong hồ sơ nguồn. Nếu tên hoặc cách đọc còn chưa rõ, phải xử lý theo quy trình thuật ngữ; không tự đoán tên để bỏ qua việc duyệt.

Quy tắc chung này không yêu cầu người dùng xác nhận từng ruby thuần phát âm. Tuy nhiên, phân loại cần có căn cứ trong nguồn/ngữ cảnh và được Review kiểm tra; không thể chỉ dựa vào việc có thẻ ruby để kết luận được phép ẩn toàn bộ.

### 4.3. Ruby có dụng ý hoặc ý nghĩa khác chữ chính

Ví dụ minh họa bằng tiếng Việt: chữ chính mang nghĩa “định mệnh”, nhưng phần chữ nhỏ gợi cách gọi “lời nguyền”. Đây là ví dụ thiết kế, không phải trích dẫn từ EPUB thực tế. Chỉ giữ một vế có thể làm mất dụng ý của tác giả.

Hồ sơ đề xuất phải hiển thị:

- Chữ chính, phần chữ nhỏ, vị trí và snapshot nguồn.
- Ngữ cảnh cần thiết để giải thích quan hệ giữa hai phần.
- Ý nghĩa có nguy cơ bị mất và căn cứ cho nhận định đó.
- Các cách thể hiện bằng tiếng Việt, ưu/nhược điểm, phương án đề xuất và phạm vi áp dụng.
- Quy ước hiện hành liên quan và những chỗ còn chưa chắc.

Các phương án có thể là diễn đạt trong câu hoặc bổ sung chú thích theo quyết định người dùng; tài liệu này không chọn sẵn một cách chung cho mọi trường hợp. Phương án chưa được duyệt chỉ nằm trong hồ sơ đề xuất/ứng viên nội bộ, chưa trở thành bản nháp hiện hành hay nội dung đem xuất.

Sau khi người dùng chốt, phần Dịch tạo/cập nhật bản dịch theo lựa chọn đó, kiểm tra cấu trúc, lưu revision và Review lại. Dùng lại quyết định đã có khi đúng nghĩa, ngữ cảnh và phạm vi; không hỏi lặp lại vô cớ.

### 4.4. Chưa xác định được loại ruby

Nếu chưa đủ căn cứ phân biệt thuần phát âm với nội dung có dụng ý, giữ trạng thái **Cần xác minh** hoặc **Chờ duyệt quy ước** theo đúng nguyên nhân, chỉ ra phạm vi đang vướng. Không mặc định ẩn, tự chọn một cách dịch hoặc đưa việc quyết định nghĩa sang lúc đóng gói EPUB.

Việc chờ xử lý tuân theo [điều phối từng lượt/chương](WORKFLOW_ORCHESTRATION_SPEC.md). Đây không phải lý do để tự nhảy qua chương đang bị chặn hoặc tự đổi model.

## 5. Trách nhiệm của từng giai đoạn

| Giai đoạn | Công việc |
| --- | --- |
| Import | Giữ nguồn và mapping; kiểm kê tiêu đề, mục lục, chú thích, ruby, định dạng và ảnh. |
| Dịch | Tạo nội dung Việt từ đầy đủ nguồn; áp dụng quyết định đã duyệt; trình đề xuất khi cần lựa chọn mới. |
| Review | Đối chiếu cả chữ chính và ruby, kiểm tra không mất ý, đủ nội dung phụ trợ và đúng quy ước. |
| Chuẩn bị preview/export | Chọn revision/quyết định còn hiệu lực, áp dụng bố cục đã chốt, chọn ảnh và dựng liên kết. |
| Đóng gói EPUB | Ghép bộ đã chuẩn bị, kiểm tra kỹ thuật và tạo file đầu ra. |

Bước xuất chạy trên các nội dung/quyết định đã lưu; không gọi model để dịch thêm, đoán nghĩa ruby hoặc sửa câu chữ âm thầm. Nếu phát hiện thiếu bản dịch hay quyết định liên quan đến nghĩa, trả chẩn đoán có vị trí để xử lý trong Dịch–Review. Kết quả kiểm tra kỹ thuật không thay thế kết luận Review.

## 6. Bộ dữ liệu dùng chung cho preview và xuất

Bộ chuẩn bị theo đặc tả ảnh phải xác định được nguồn/mapping, revision phần chữ gồm tiêu đề/chú thích/nhãn mục lục, kết quả thể hiện ruby và quyết định liên quan, cấu hình bố cục cùng các ảnh đã chọn. Preview và EPUB cuối dùng cùng bộ dữ liệu đó.

Bộ chuẩn bị còn gắn với lựa chọn nội dung, danh sách phần phải dịch và phần giữ nguyên từ nguồn. Đổi lựa chọn làm bộ cũ cần chuẩn bị lại; không trộn bản dịch chưa duyệt vào phần được giữ nguyên hoặc dùng nguyên tác để lấp phần bắt buộc còn chưa dịch.

Thông tin sách dùng cho lần xuất, gồm tên sách/tập, tác giả, ngôn ngữ và người dịch/nhóm dịch nếu có, cũng thuộc bộ dữ liệu này. Màn Thông tin sách phải hiển thị đúng lựa chọn sẽ được ghi vào EPUB; thay đổi sau khi chuẩn bị làm bộ cũ cần cập nhật như các cấu hình xuất liên quan khác.

Cấu hình đầu ra của bộ chuẩn bị là EPUB 3. Lần xuất liên kết bộ này với một file phiên bản mới và hồ sơ xuất tương ứng theo mục 10. Số phiên bản file xuất khác với revision nội dung hoặc generation của workspace.

Đổi bản dịch, quyết định thể hiện ruby, tiêu đề, chú thích hoặc cấu hình bố cục liên quan làm bản chuẩn bị cũ cần cập nhật. Nếu nội dung/quy ước đổi, xem lại Review của phần chịu ảnh hưởng; nếu chỉ đổi bố cục thuần túy mà không đổi nội dung và dependency ngữ nghĩa, kiểm tra lại hiển thị/cấu trúc và dựng lại preview/export, không tự dịch lại toàn bộ chữ.

Theo lựa chọn **11A** ngày **30/10/2026** trong [đặc tả key và preview](SECURITY_SPEC.md), bản chuẩn bị được kiểm tra XHTML/CSS/tài nguyên để không chạy nội dung chủ động từ nguồn hoặc tự tải tài nguyên ngoài khi preview. Giữ nguồn bất biến và cùng nội dung, ảnh, cơ sở bố cục đã kiểm tra cho bản xuất; phần cần thiết không thể chuẩn bị trong phạm vi hỗ trợ phải được báo để xử lý, không âm thầm loại bỏ. Key, mã truy cập phiên preview và các điều khiển riêng của tool không được đóng gói vào EPUB.

Giữ lịch sử quyết định và nguồn để người dùng truy lại vì sao một ruby được lược phần phát âm hoặc thể hiện bằng cách khác. Tên field, nơi lưu và hình thức ghi nhận chi tiết được chốt khi thiết kế schema.

## 7. Điều kiện xuất EPUB hoàn chỉnh

### 7.1. Phạm vi bản xuất

Giai đoạn đầu chỉ tạo EPUB hoàn chỉnh cho một tập theo lựa chọn nội dung đã lưu. Với **Dịch toàn bộ phần chữ**, phải xử lý cả truyện, mở đầu/kết/lời bạt, nội dung phụ trợ, trang bản quyền, giới thiệu tác giả và quảng cáo. Với **Chỉ dịch nội dung truyện và phần liên quan**, phải hoàn tất truyện và phần liên quan theo đặc tả đầu vào; các trang phụ giữ nguyên đầy đủ từ nguồn, đúng vị trí và liên kết. Chữ trong bitmap theo luồng ảnh trong cả hai trường hợp.

Một EPUB đủ phần dịch và phần giữ nguyên có chủ đích theo lựa chọn thứ hai là bản hoàn chỉnh trong chế độ đó. Thiếu bản dịch ở phần bắt buộc vẫn là dở dang; hoàn tất tác vụ dịch một nhóm chương chưa đủ để xuất toàn tập. Không dùng lựa chọn chỉ truyện để bỏ một chương truyện chưa dịch hoặc chưa Review.

Phần đang làm dở vẫn xem và xử lý trong tool. Chưa cung cấp chế độ xuất EPUB dở dang; không tự bù chương còn thiếu bằng nguyên tác hoặc bản nháp chưa được chấp nhận rồi báo bản xuất hoàn chỉnh. Quy tắc này áp dụng như nhau cho CLI và TUI.

### 7.2. Các điều kiện cần kiểm tra

| Tình trạng hiện tại | Quyết định xuất |
| --- | --- |
| Đủ bản dịch, Review từng lượt và kiểm tra tổng thể chương hoàn tất, kết quả còn hiệu lực | Đủ điều kiện phần chữ; cho phép xuất khi các kiểm tra kỹ thuật cũng đạt. |
| Còn nội dung bắt buộc trong lựa chọn hiện hành chưa dịch, chưa Review hoặc kết quả cần kiểm tra lại | Chặn xuất, chỉ rõ phạm vi và việc cần hoàn tất. |
| Trang phụ nằm trong danh sách giữ nguyên của chế độ chỉ truyện | Giữ đúng nguồn/vị trí/liên kết, không yêu cầu Review bản dịch của phần này và không báo đã dịch. |
| Chế độ toàn bộ phần chữ còn trang phụ chưa dịch hoặc chưa Review | Chặn xuất như phần truyện chưa hoàn tất, không âm thầm chuyển trang đó sang giữ nguyên. |
| Còn thuật ngữ, xưng hô, ruby hoặc quyết định nghĩa bắt buộc đang chờ | Chặn xuất đến khi được xử lý và phần phụ thuộc được kiểm tra lại. |
| Chỉ còn gợi ý S0 | Không chặn xuất chỉ vì gợi ý đó. |
| Còn lỗi S1–S3 đã xác nhận nhưng chưa giải quyết | Chặn xuất nếu chưa có quyết định giữ ngoại lệ hợp lệ cho đúng vấn đề/revision. |
| Người dùng đã chủ động giữ bản có vấn đề, kèm lý do/phạm vi/revision | Có thể xuất với nhãn có ngoại lệ nếu các điều kiện khác đều đạt. |
| Chưa thay ảnh có chữ Nhật, ảnh nguồn hợp lệ | Dùng ảnh nguồn; việc chưa thay ảnh không tự chặn xuất. |
| Ảnh edit khớp nhưng hỏng, nguồn/mapping thiếu hoặc cấu trúc/tài nguyên đầu ra không hợp lệ | Chặn xuất và chỉ rõ vị trí cần sửa; không âm thầm dùng phương án khác. |
| Bộ preview đã cũ hoặc dữ liệu chưa xác định nhất quán | Áp dụng quy tắc chuẩn bị/nạp lại theo đặc tả ảnh trước khi xuất bộ tương ứng. |
| Thiếu tên sách hoặc tác giả bắt buộc | Chỉ rõ trường còn thiếu để người dùng bổ sung trước khi xuất; không tự đoán hoặc điền nội dung giả. |

Kiểm tra lại hiệu lực trên đúng bộ dữ liệu được chọn để xuất: snapshot, mapping, revision, quyết định và các dependency liên quan. Một kết luận Review cũ không tự áp sang bản mới chỉ vì cùng chương hoặc cùng tên file.

### 7.3. Phạm vi của quyết định giữ ngoại lệ

Ngoại lệ là quyết định người dùng giữ một vấn đề nội dung cụ thể đã được xem xét, có lý do, phạm vi và revision theo đặc tả Review/rubric. Nó không biến kết quả thành Reviewer xác nhận đúng hoặc thành quy ước/ngữ cảnh chính thức.

Ngoại lệ không thay thế việc dịch đủ nội dung, thực hiện đủ Review và kiểm tra tổng thể chương; không duyệt hộ thuật ngữ/xưng hô/ruby còn chờ, cũng không bỏ qua lỗi cấu trúc, tài nguyên hay mapping. Nếu một trong các điều kiện này chưa đạt, bản xuất vẫn bị chặn dù đã có ngoại lệ khác.

Khi bản dịch hoặc căn cứ liên quan đổi, phải kiểm tra lại hiệu lực của quyết định giữ. Không áp ngoại lệ gắn với revision cũ cho một bản mới chưa được quyết định. Ngoại lệ còn hiệu lực không cần người dùng xác nhận lặp lại chỉ vì mở lại màn xuất.

### 7.4. Trạng thái và thao tác trên màn xuất

Màn tổng quan thể hiện ba trạng thái:

- **Sẵn sàng xuất:** các điều kiện đã đạt, không có ngoại lệ chất lượng đang áp dụng.
- **Còn việc cần xử lý:** liệt kê việc chặn, vị trí chương/đoạn/tài nguyên và thao tác mở đúng chỗ để xử lý.
- **Sẵn sàng xuất với ngoại lệ:** các điều kiện còn lại đã đạt, hiển thị danh sách ngoại lệ người dùng đã giữ và lý do tương ứng.

Nếu vừa có ngoại lệ vừa có việc chặn chưa giải quyết, dùng trạng thái Còn việc cần xử lý. Việc chưa thay ảnh hoặc còn S0 có thể hiển thị trong phần thông tin, không tự bị nâng thành lỗi chặn.

Hiển thị lựa chọn nội dung và danh sách/số phần giữ nguyên nếu có. Nhãn **Giữ nguyên theo chế độ chỉ truyện** phân biệt với Đã dịch, Đạt Review và Ngoại lệ chất lượng; việc giữ trang phụ theo chế độ không cần tạo một ngoại lệ lỗi dịch cho từng trang.

Ví dụ đã hoàn tất 5/6 chương nhưng chương 6 chưa Review: hiển thị **Chưa thể xuất: Chương 6 chưa hoàn tất Review**, kèm thao tác mở chương 6. Tool không yêu cầu người dùng tự tìm lại điểm đang vướng.

Trạng thái sẵn sàng không đồng nghĩa file cuối đã được tạo. Sau khi chọn Xuất EPUB, phải đóng gói đúng bộ dữ liệu đã chuẩn bị và kiểm tra bản xuất; chỉ báo hoàn tất khi thành công. Nếu thất bại, giữ tiến độ và bản xuất trước theo đặc tả ảnh.

## 8. Thông tin sách trong EPUB đầu ra

### 8.1. Các giá trị mặc định và lựa chọn của người dùng

Thông tin sách ở đây là metadata dùng để nhận diện cuốn sách trong thư viện của ứng dụng đọc EPUB. Quy tắc chốt ngày **29/10/2026**:

| Thông tin | Cách xử lý |
| --- | --- |
| Tên sách/tập | Dùng tên tiếng Việt đã được người dùng chốt nếu có; chưa có thì giữ tên từ EPUB nguồn. Cho phép người dùng sửa trước khi xuất. |
| Tác giả | Điền từ EPUB nguồn. Nếu người dùng muốn dùng tên phiên âm hoặc cách viết khác, dùng lựa chọn đã được họ xác nhận. |
| Ngôn ngữ bản xuất | Đặt là Tiếng Việt. |
| Người dịch/nhóm dịch | Không bắt buộc, do người dùng tự nhập nếu muốn ghi vào sách. |

Lựa chọn người dùng đã lưu riêng cho tập được dùng lại ở các lần xuất sau. Giá trị điền sẵn không được âm thầm ghi đè lựa chọn đó khi người dùng mở lại màn xuất. Metadata gốc vẫn nằm trong snapshot nguồn để đối chiếu; thông tin đầu ra được lưu riêng.

### 8.2. Màn Chuẩn bị xuất → Thông tin sách

Tool điền sẵn các giá trị có căn cứ và hiển thị để người dùng kiểm tra hoặc sửa khi cần. Không yêu cầu nhập lại hoặc xác nhận lặp lại các lựa chọn đã lưu còn phù hợp chỉ vì xuất thêm một lần.

Thiếu tên sách hoặc tác giả thì đánh dấu đúng trường cần bổ sung, theo điều kiện xuất ở mục 7. Người dùng điền và lưu trước khi xuất; tool không tự đoán tên, tự chọn một người làm tác giả hoặc dùng placeholder như thông tin chính thức. Để trống người dịch/nhóm dịch không chặn xuất.

Lưu thông tin theo từng tập, phục hồi khi mở lại workspace. Việc người dùng sửa và lưu một trường là quyết định dùng giá trị đó cho metadata của tập; không phải thao tác tự động đổi tên hoặc thuật ngữ trong nội dung truyện.

### 8.3. Khi người dùng muốn AI gợi ý tên tiếng Việt

Nếu người dùng yêu cầu AI đề xuất tên, tạo đề xuất có tên gốc, ngữ cảnh, các phương án và lý do theo quy trình duyệt quy ước. Phương án chỉ được dùng sau khi người dùng chọn hoặc sửa rồi xác nhận; AI không tự chốt tên vì đang ở màn chuẩn bị xuất.

Việc tạo đề xuất là tác vụ hỗ trợ riêng theo chiến lược model và ngân sách đã chốt. Bước đóng gói chỉ lấy tên đã lưu/được chấp nhận, không tự gọi model để dịch tên. Đề xuất chưa được duyệt không thay thế giá trị hiện hành.

### 8.4. Ảnh hưởng khi thay đổi thông tin sách

Thay đổi metadata làm bộ chuẩn bị cũ cần cập nhật; màn thông tin, preview và EPUB xuất phải dựa trên cùng lựa chọn đã lưu. Đổi thông tin hiển thị trong thư viện thuần túy không tự làm mất hiệu lực Review hoặc yêu cầu dịch lại nội dung chương.

Nếu người dùng muốn đổi cách viết một tên xuyên suốt bản dịch, xử lý như thay đổi thuật ngữ/quy ước: xác định phạm vi, lưu quyết định, sửa nội dung liên quan và Review lại. Không lấy việc sửa metadata làm lệnh thay chuỗi hàng loạt trong truyện, tiêu đề chương hoặc ảnh bìa.

## 9. Thống nhất đầu ra EPUB 3

Quyết định ngày **30/10/2026**: đầu vào tiếp tục hỗ trợ EPUB 2 và EPUB 3 theo phạm vi đã chốt, nhưng mọi EPUB được xuất bởi luồng này đều là **EPUB 3**. Người dùng không cần chọn lại phiên bản đầu ra cho từng tập/lần xuất.

| Nguồn | Hành vi xuất |
| --- | --- |
| EPUB 2 trong phạm vi hỗ trợ | Dựng bản EPUB 3 từ snapshot/mapping, nội dung dịch hoặc giữ nguyên và tài nguyên đã chọn. |
| EPUB 3 trong phạm vi hỗ trợ | Dựng bản EPUB 3 với nội dung, bố cục và metadata đầu ra đã chốt. |

Chuẩn hóa đầu ra phải xử lý cấu trúc gói, mục lục, metadata, liên kết và tài nguyên cần thiết để bản kết quả hợp lệ theo EPUB 3. Chỉ sửa nhãn phiên bản của gói nguồn chưa đáp ứng yêu cầu này. Giữ đúng thứ tự đọc, cấp mục lục, đích liên kết, vị trí ảnh, chú thích và ý nghĩa định dạng; kiểm tra cấu trúc/tài nguyên/readback/EPUBCheck trên bản thực sự tạo ra.

Trong chế độ chỉ dịch truyện, trang phụ giữ nguyên là giữ nội dung nguồn theo lựa chọn đó; phần cấu trúc bao chứa vẫn có thể cần chuẩn hóa để nằm trong EPUB 3 hợp lệ. Không dùng chuyển đổi phiên bản để tự dịch thêm, bỏ phần giữ nguyên hoặc thay đổi nghĩa.

Việc dựng EPUB 3 diễn ra trên bản chuẩn bị/đầu ra, không sửa snapshot nguồn hoặc yêu cầu dịch lại phần chữ còn hợp lệ chỉ vì nguồn là EPUB 2. Nếu gặp cấu trúc chưa chuyển được hoặc kiểm tra thất bại, báo rõ nguyên nhân/phạm vi và giữ dữ liệu để xử lý; không âm thầm xuất EPUB 2 thay cho lựa chọn đã chốt.

Chốt ngày **04/11/2026** dùng EPUB 3.3, crate `zip`, `quick-xml` và EPUBCheck 5.3.0. Quy tắc chuyển NCX, dựng NAV/package, bố cục Việt, ruby, kiểm tra và lưu báo cáo theo mục 12.

## 10. Mỗi lần xuất tạo một phiên bản file mới

### 10.1. Thư mục và cách đặt tên

Chọn **6A**: mỗi lần người dùng thực hiện một lần xuất mới thành công, tạo một file mới trong thư mục đầu ra của workspace, mặc định theo bố trí `Output/` đã có. Giữ các bản đã xuất trước đó để người dùng mở lại/đối chiếu; thao tác xuất không tự ghi đè hoặc dọn các bản cũ.

Mẫu tên dùng tên bộ/tập, ngôn ngữ và số phiên bản tăng dần. Ví dụ minh họa:

```text
Output/
├── Ten-bo_Tap-01_VI_r001.epub
├── Ten-bo_Tap-01_VI_r002.epub
└── Ten-bo_Tap-02_VI_r001.epub
```

Số phiên bản quản lý theo tập, không phải số lần sửa một block, generation hoặc số lần gọi model. Đổi nội dung, metadata, ảnh hay lựa chọn phạm vi rồi xuất lại vẫn tạo phiên bản file mới. Tên cụ thể được tạo phù hợp hệ thống file, không làm thay đổi tên sách đã lưu trong metadata.

Phải kiểm tra xung đột đường dẫn trước khi công bố file: tên đã tồn tại không được ghi đè, kể cả file do người dùng đặt vào thư mục. Cách cấp số và xử lý tên trùng cần bảo đảm mỗi lần xuất có đích mới; tên sách thay đổi không làm mất liên kết lịch sử với cùng tập.

### 10.2. Hoàn tất, lỗi và tiếp tục

Đóng gói và kiểm tra trên bộ dữ liệu đã chuẩn bị hợp lệ; chỉ công bố một phiên bản là hoàn tất khi file đầu ra đã tạo và kiểm tra thành công. Nếu thất bại hoặc gián đoạn, giữ các bản cũ và tiến độ; file dở dang không được hiện như một bản xuất thành công.

Tiếp tục một tác vụ xuất bị gián đoạn phải đối chiếu trạng thái đã lưu để khôi phục đúng lần xuất, không ghi đè file đã hoàn tất hoặc tự coi một file chưa kiểm tra là hợp lệ. Một lệnh xuất mới riêng biệt tạo bản mới; cấp số được đặt trước trong giao dịch của tập. Sau r003 thất bại, lệnh mới lấy r004. Tiếp tục đúng tác vụ r003 giữ số r003 và đối chiếu hồ sơ đã lưu theo mục 12.

### 10.3. Hồ sơ của bản xuất

Lưu đủ thông tin để truy được tập, số phiên bản file, đường dẫn, thời điểm, nguồn/mapping, bộ revision/quyết định, lựa chọn toàn bộ phần chữ/chỉ truyện, metadata, ảnh đã dùng và kết quả kiểm tra của lần xuất. Nội dung/hash file đầu ra được liên kết với đúng bộ dữ liệu đã chuẩn bị.

Khi xuất thành công, giao diện hiển thị **EPUB 3**, tên/số phiên bản và đường dẫn file vừa tạo. Hai bản có cùng tên sách nhưng khác phạm vi hoặc ảnh vẫn truy được cấu hình riêng qua hồ sơ xuất; không chỉ dựa vào tên file để suy đoán nội dung.

## 11. Tiêu chí nghiệm thu khi triển khai

Dùng dữ liệu tự tạo và model giả lập cho kiểm thử tự động offline, deterministic. Các tiêu chí dưới đây không có nghĩa mọi lỗi ngôn ngữ đều được AI phát hiện.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Nguồn Nhật dàn dọc, bản dịch tiếng Việt | Bản Việt đọc ngang từ trái sang phải, đúng thứ tự nội dung. |
| Chương có part và nhiều lượt dịch | Giữ cấu trúc nguồn, không sinh phần/mục lục từ lượt dịch. |
| Ảnh inline hoặc trang tranh xen giữa phần chữ | Giữ vị trí, không xoay/lật/sửa bitmap vì chuyển bố cục chữ. |
| In đậm, in nghiêng hoặc nhấn mạnh có ý nghĩa | Bản Việt giữ tác dụng định dạng, không mất nội dung hoặc mapping. |
| Mục lục và tiêu đề có cùng nội dung nguồn | Dùng tên đã duyệt nhất quán và dẫn đúng vị trí. |
| Nhãn mục lục khác tiêu đề nguồn | Bảo toàn khác biệt có nghĩa qua Dịch–Review, không tự ghi đè. |
| Chú thích ở file riêng, nhiều chỗ gọi | Đủ nội dung dịch, đúng liên kết và quay lại nếu có. |
| Ruby thuần phát âm | Không hiện lại chú âm theo mặc định; nguồn và căn cứ phân loại vẫn truy được. |
| Ruby mang thêm ý nghĩa hoặc chưa rõ | Có bằng chứng/đề xuất và trạng thái chờ; không tự ẩn hay áp phương án chưa duyệt. |
| Người dùng chốt cách thể hiện ruby | Tạo revision phù hợp, kiểm tra cấu trúc và Review trước khi dùng để xuất. |
| Thiếu tiêu đề, chú thích hoặc quyết định cần thiết lúc chuẩn bị | Báo vị trí cần xử lý trong Dịch–Review; không tự gọi model ở bước xuất. |
| Chỉ đổi bố cục thuần túy | Dựng lại và kiểm tra preview/export; không tự dịch lại phần chữ không đổi. |
| Nội dung/quyết định đổi sau preview | Phát hiện bản cũ; áp dụng quy tắc nạp lại trong đặc tả ảnh. |
| Xuất một bộ dữ liệu đã chuẩn bị | Chữ, ruby, ảnh và bố cục khớp bộ đã chọn; kiểm tra cấu trúc/tài nguyên/readback/EPUBCheck. |
| Chỉ hoàn tất một nhóm chương hoặc còn chương chờ Review | Chưa cho xuất toàn tập, chỉ rõ phần còn vướng; không tạo EPUB dở dang trong giai đoạn đầu. |
| Toàn bộ nội dung đạt, chỉ còn S0 hoặc ảnh chưa thay | Không chặn chỉ vì các trường hợp đó nếu những kiểm tra khác đạt. |
| Còn S1–S3 chưa giải quyết | Chặn xuất trừ đúng vấn đề có quyết định giữ ngoại lệ còn hiệu lực. |
| Ngoại lệ còn hiệu lực, mọi điều kiện khác đạt | Cho phép xuất với trạng thái và danh sách ngoại lệ rõ ràng. |
| Có ngoại lệ nhưng vẫn thiếu Review, quyết định bắt buộc hoặc tài nguyên hợp lệ | Tiếp tục chặn; ngoại lệ không bỏ qua điều kiện khác. |
| Revision hoặc dependency đổi sau quyết định giữ ngoại lệ | Xem lại hiệu lực, không tự dùng quyết định cũ cho bản mới. |
| CLI hoặc TUI yêu cầu xuất cùng một bộ dữ liệu | Áp dụng cùng điều kiện và nguyên nhân chặn. |
| Chỉ truyện đã hoàn tất, trang phụ giữ nguyên đầy đủ | Cho xuất theo chế độ đã chọn nếu các điều kiện khác đạt; hiển thị rõ phần giữ nguyên. |
| Toàn bộ phần chữ còn thiếu trang phụ | Chặn xuất, không tự đổi phạm vi hoặc dùng nguồn thay bản dịch còn thiếu. |
| Đổi lựa chọn nội dung sau preview | Tính lại điều kiện và làm mới bộ chuẩn bị; không coi preview cũ là bản xuất của phạm vi mới. |
| Có tên tiếng Việt đã chốt | Điền đúng tên cho metadata đầu ra; giữ metadata gốc để đối chiếu. |
| Chưa có tên tiếng Việt | Dùng tên nguồn còn hợp lệ, không tự dịch hoặc đặt tên mới. |
| Tác giả và ngôn ngữ | Tác giả theo nguồn/lựa chọn được xác nhận, ngôn ngữ bản xuất là Tiếng Việt. |
| Thiếu tên sách/tác giả, để trống người dịch | Chỉ yêu cầu bổ sung tên sách/tác giả; người dịch là tùy chọn. |
| Đã lưu thông tin riêng cho một tập | Mở lại hoặc xuất lại vẫn dùng lựa chọn đó, không bị mặc định nguồn ghi đè. |
| AI đề xuất tên chưa được duyệt | Không thay tên hiện hành; đóng gói không tự gọi model để quyết định. |
| Chỉ sửa metadata sau khi chuẩn bị | Làm mới bộ preview/export; không tự dịch lại hoặc phủ nhận Review phần chữ không đổi. |
| Muốn đổi tên xuyên suốt nội dung | Đi qua quy trình thuật ngữ và Review theo phạm vi, không dùng sửa metadata để thay hàng loạt. |
| Nguồn EPUB 2 hoặc EPUB 3 hợp lệ trong phạm vi | Đầu ra đều là EPUB 3 hợp lệ, bảo toàn nội dung/phạm vi và các quan hệ tài nguyên. |
| Trang phụ giữ nguyên từ nguồn EPUB 2 | Nội dung vẫn được giữ trong bản EPUB 3, không bị bỏ hoặc tự dịch trong bước chuyển đổi. |
| Không chuyển đổi/kiểm tra được | Báo lỗi và giữ dữ liệu/bản cũ; không tự hạ đầu ra xuống EPUB 2. |
| Xuất lại cùng tập | Tạo file phiên bản mới trong thư mục đầu ra, giữ các bản cũ. |
| Tên file dự kiến đã tồn tại | Chọn đích mới phù hợp, không ghi đè file có sẵn. |
| Đổi tên sách/ảnh/phạm vi rồi xuất lại | File mới có hồ sơ nguồn/cấu hình riêng, vẫn truy được về cùng tập. |
| Xuất thất bại hoặc bị gián đoạn | Không công bố bản dở dang là hoàn tất, không làm mất bản cũ. |

## 12. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [32](DESIGN_DECISIONS_36.md#decision-32), [33](DESIGN_DECISIONS_36.md#decision-33) của bản 36 mục áp dụng cho đặc tả này.

### 12.1. Chuẩn hóa và dựng EPUB 3 — mục 32

Chọn **EPUB 3.3** làm đích nghiệm thu của bản đầu. Core sở hữu quy tắc chuyển đổi, dùng crate Rust `zip` cho container và `quick-xml` cho đọc/ghi XML có namespace. Với EPUB 2, chuyển cấp, thứ tự và đích của mục lục NCX sang Navigation XHTML. Dựng lại package, manifest, spine, metadata tiếng Việt/định danh/bìa và kiểm tra mọi liên kết. Đặt `mimetype` đầu gói, ở dạng lưu không nén. [EPUB 3.3 của W3C](https://www.w3.org/TR/epub-33/) · [zip](https://github.com/zip-rs/zip2) · [quick-xml](https://github.com/tafia/quick-xml)

Chữ Việt đọc ngang trái sang phải. Giữ bố cục trang minh họa cố định trong phạm vi được hỗ trợ. Với ruby thuần phát âm, xuất chữ chính theo kết quả Dịch–Review. Với ruby mang nghĩa riêng, áp cách thể hiện đã được duyệt. Bảo toàn đoạn, chú thích, anchor, ảnh và nhấn mạnh. Mở đọc lại file đã dựng, kiểm tra XML/tài nguyên/độ phủ và chạy **EPUBCheck 5.3.0** làm mốc kiểm tra 3.3. Lưu phiên bản công cụ và báo cáo. Lỗi chặn xuất, cảnh báo phải được phân loại, trường hợp ảnh hưởng nội dung được xử lý trước khi công bố. Kiểm tra đọc thực tế trên fixture qua Calibre và một trình đọc mục tiêu. [EPUBCheck 5.3.0](https://github.com/w3c/epubcheck/releases/tag/v5.3.0) · [Đặc tả đầu ra](EPUB_OUTPUT_SPEC.md)

### 12.2. Bộ dữ liệu chuẩn bị và phiên bản xuất EPUB — mục 33

Lưu một manifest bất biến theo hash cho mỗi bản chuẩn bị. Ghi snapshot/mapping, chế độ nội dung và phần giữ nguyên, các revision/Review/ngoại lệ, quy tắc có hiệu lực, ruby, metadata, cấu hình bố cục, danh sách ảnh source/edit và hash đã chọn. Quét cả các đường dẫn có thể xuất hiện ảnh thay mới. Đổi ảnh hoặc metadata thì dựng lại các phần phụ thuộc và chạy lại kiểm tra đầu ra. Kết quả dịch chữ còn hợp lệ được dùng lại.

Khi có lệnh xuất mới, đặt trước số bản theo tập trong một giao dịch duy nhất. **Lệnh mới sau lần r003 thất bại lấy r004.** Tiếp tục chính tác vụ r003 thì đối chiếu hồ sơ r003. Dựng và kiểm tra file tạm, lưu hash, đích, ID lần xuất rồi mới công bố file bằng thao tác giữ mọi file đang tồn tại. Nếu file đã xuất nhưng hồ sơ chưa đánh dấu xong, khôi phục theo ID, đích, hash và báo cáo đã lưu. File cùng tên khác hash được báo xung đột. Mỗi lần thành công có đường dẫn và lịch sử riêng.

### 12.3. Liên kết và yêu cầu triển khai

Tên file hợp lệ với hệ đích, chi tiết CSS và cách dựng từng biểu diễn ruby được hiện thực theo hợp đồng này, phạm vi hỗ trợ đầu vào và quyết định nghĩa đã duyệt. Kiểm thử mất điện/gián đoạn tại lúc đặt số, ghi file và công bố hồ sơ theo [Quyết định 33 về phiên bản xuất](DESIGN_DECISIONS_36.md#decision-33) và [Quyết định 36 về nghiệm thu](DESIGN_DECISIONS_36.md#decision-36).
