# 36 quyết định thiết kế đã chốt

**Đã chốt ngày 04/11/2026.** Người dùng xác nhận “chốt 36 mục này” cho toàn bộ mục 01–36. Đây là hợp đồng thiết kế làm căn cứ triển khai Rust. Các giá trị mang nhãn *mặc định thử nghiệm* đã được chấp nhận làm cấu hình ban đầu và được hiệu chỉnh qua bộ nghiệm thu ở mục 36.

Gói này dựa trên các quyết định đã ghi nhận về nhập EPUB 2/3, hai phạm vi dịch, duyệt thuật ngữ, Review sau từng lượt và cuối chương, một tác vụ AI hoạt động, ngân sách, bảo mật và xuất EPUB 3. Research được gọi khi Dịch, Review hoặc người dùng có câu hỏi cần tra cứu. [Điều phối](WORKFLOW_ORCHESTRATION_SPEC.md) · [Chiến lược model](MODEL_STRATEGY_SPEC.md) · [Quyền duyệt](SKILLS_GLOSSARY_SPEC.md)

## Đối chiếu với các đặc tả

| Mục | Tài liệu áp dụng |
| --- | --- |
| 01, 02, 03, 04, 05 | [EPUB đầu vào](EPUB_INPUT_SPEC.md) |
| 06, 07, 08, 09, 10 | [Workspace và trạng thái](DATA_MODEL_STATE_SPEC.md) |
| 11, 12, 13, 14, 15 | [Skills và glossary](SKILLS_GLOSSARY_SPEC.md) |
| 16 | [Xưng hô](ADDRESSING_CONTINUITY_SPEC.md) |
| 17, 18, 19 | [Model, chi phí và timeout](MODEL_STRATEGY_SPEC.md) |
| 20 | [Dịch](TRANSLATION_STAGE_SPEC.md) |
| 21, 22, 24 | [Đọc nguồn, bằng chứng và Review chương](REVIEW_STAGE_SPEC.md) |
| 23, 25 | [Phạm vi sửa và ngoại lệ](REVIEW_RUBRIC_SPEC.md) |
| 26, 27 | [Research](RESEARCH_STAGE_SPEC.md) |
| 28, 29 | [Điều phối và phục hồi](WORKFLOW_ORCHESTRATION_SPEC.md) |
| 30 | [Thay ảnh](IMAGE_EDIT_SPEC.md) |
| 31 | [Preview cục bộ](SECURITY_SPEC.md) |
| 32, 33 | [Dựng và xuất EPUB](EPUB_OUTPUT_SPEC.md) |
| 34, 35 | [TUI và CLI](TUI_CLI_SPEC.md) |
| 36 | [Kiểm thử và nghiệm thu](#decision-36) |

Các đặc tả đã được cập nhật cùng lần chốt này. Câu lệnh tạo bảng, mã Rust, cấu hình bảo vệ trình duyệt và kết quả kiểm thử được thực hiện trong giai đoạn triển khai theo các hợp đồng đã duyệt.

## EPUB đầu vào

<a id="decision-01"></a>

### 01. Lựa chọn nội dung dịch mặc định · P1

Đã chốt **01A**. Bộ mới dùng **Dịch toàn bộ phần chữ**. Màn chuẩn bị cho đổi sang **Chỉ dịch nội dung truyện và phần liên quan**. Một thay đổi áp dụng cho tập đang mở. Nút **Lưu làm mặc định của bộ** mới đổi giá trị cho các tập tạo sau. Tập 2 lấy mặc định của bộ dù tập 1 đã được chỉnh riêng. Lưu lựa chọn thực tế vào tập/tác vụ và hiển thị trước lần dịch đầu. Chữ trong bitmap đi qua luồng ảnh.

<a id="decision-02"></a>

### 02. Phân loại nội dung trong EPUB · P0

Đã chốt **02A**. Import cục bộ lập cây từ cấu trúc sách, mục lục, tiêu đề, vị trí và liên kết. Lưu riêng loại phần và cách xử lý theo chế độ. Ví dụ, đoạn chữ đọc được ở cuối sách vẫn cần dịch ở chế độ toàn bộ chữ dù chưa rõ đó là ngoại truyện hay quảng cáo. Ở chế độ chỉ truyện, nếu hai cách phân loại dẫn đến hai kết quả khác nhau, trang vào **Cần phân loại**.

Trên cây, người dùng xem đoạn nguồn, các dấu hiệu và các nơi tham chiếu, rồi chọn loại trang. Nút **Nhờ Research đề xuất** tạo yêu cầu có chủ đích sau import, có căn cứ và chịu ngân sách. Người dùng chốt kết quả. Lưu quyết định cùng hash EPUB, phiên bản mapping và vị trí. Dùng lại khi cùng nguồn/mapping, đối chiếu lại khi một trong hai đổi. Chú thích dùng chung với phần truyện được dịch một lần, giữ các liên kết từ cả hai nơi. Nhãn chỉ thuộc trang phụ theo chế độ của trang đó.

<a id="decision-03"></a>

### 03. Nhận diện chương và chỉnh sửa mapping · P0

Spine xác định thứ tự đọc. Trong thứ tự đó, ưu tiên ranh giới người dùng đã xác nhận cho đúng nguồn, tiếp đến là đích mục lục Nav/NCX hợp lệ, rồi heading và dấu ngắt trong XHTML. Ranh giới file chỉ làm gợi ý cuối. Đích mục lục hỏng hoặc mâu thuẫn có ý nghĩa với heading tạo phạm vi tạm kèm hai dấu hiệu để người dùng đối chiếu. Part vẫn là tiêu đề trong chương.

Cho **gộp, tách, dời ranh giới và đổi tên hiển thị** trên cây. Trước khi lưu, kiểm tra phạm vi có đúng thứ tự và phủ đủ nguồn. Ghi phiên bản mapping mới. Giữ bản dịch của đơn vị nguồn khớp chắc chắn và quy tắc còn hiệu lực. Xét lại kết luận toàn chương và ngữ cảnh ở những điểm nối đã đổi.

<a id="decision-04"></a>

### 04. Định danh block và phạm vi con · P0

ID block là SHA-256 của một bản ghi có phiên bản gồm hash EPUB, phiên bản parser, đường dẫn XHTML đã chuẩn hóa và vị trí node ổn định trong tài liệu. Ưu tiên ID XHTML duy nhất, còn lại dùng đường đi node và thứ tự xuất hiện. Hai đoạn giống hệt chữ vẫn có ID riêng. Tên chương và số thứ tự lượt dịch được lưu riêng.

Đánh số **các cụm ký tự hiển thị và phần tử inline được bảo vệ** trong block. Một phạm vi con mang block ID và khoảng nửa mở `[đầu, cuối)`, kèm bảng ánh xạ về node/cụm chữ gốc. Ruby, ảnh và cặp marker có quy tắc ranh giới riêng. Lưu độ phủ Dịch và Review theo các đơn vị thực sự đã đối chiếu. Khi chia lại, tái dùng nguyên đơn vị còn hợp lệ. Nếu thứ tự từ trong bản Việt làm một phạm vi con khó tách chính xác, giữ cả đơn vị cũ hoặc dịch lại vùng đó. EPUB đổi tạo snapshot mới. Chỉ tái dùng qua snapshot sau khi khớp duy nhất vị trí, nội dung, tài nguyên được bảo vệ và kiểm tra các phụ thuộc. [Mapping nguồn](EPUB_INPUT_SPEC.md)

<a id="decision-05"></a>

### 05. Chia lượt dịch theo token · P1

Dùng một bộ lập lượt cho mỗi hồ sơ model. Ưu tiên tokenizer đúng phiên bản từ provider hoặc tokenizer chạy cục bộ với bộ từ vựng đúng model. Khi chỉ có bộ ước lượng, hiện rõ trạng thái ước tính, dùng biên lớn hơn và tạm dừng nếu thiếu giới hạn đáng tin. Đối với model cung cấp `tokenizer.json`, dùng thư viện Rust của Hugging Face. [Tokenizers](https://github.com/huggingface/tokenizers)

Đếm toàn bộ request đã dựng gồm hướng dẫn, Skills/glossary, ngữ cảnh, nguồn, khuôn JSON và phần bao do adapter thêm. *Mặc định thử nghiệm* giữ mốc 3.000 token nguồn/lượt, dự trù đầu ra Dịch bằng khoảng hai lần token nguồn cộng phần bao JSON. Chừa thêm 5% cửa sổ khi có tokenizer phù hợp, 10% khi phải ước lượng. Hồ sơ từng vai trò có thể điều chỉnh dự trù. Dự trù reasoning theo cách tính của model, tính một lần khi reasoning đã nằm trong trần đầu ra. Kiểm tra riêng trần input, output và tổng context. Cắt ngữ cảnh tùy chọn trước, giữ quy ước bắt buộc.

Gom các block liên tiếp vừa ngân sách. Nếu có điểm ngắt part/cảnh trong 20% cuối phần vừa chứa, ưu tiên điểm gần cuối đó, sau đó đến cuối đoạn rồi cuối câu. Với một block không có chỗ ngắt câu an toàn, hiển thị vị trí để chỉnh điểm chia hoặc giới hạn. Đổi model theo yêu cầu người dùng sẽ tính lại phần chưa gửi và bảo toàn các đơn vị đã hoàn thành còn hiệu lực.

## Workspace và lưu trạng thái

<a id="decision-06"></a>

### 06. Lựa chọn hệ thống lưu trữ workspace · P0

Dùng **một SQLite cho mỗi bộ truyện**, mọi tập của bộ cùng tham chiếu nhân vật, glossary và lịch sử. SQLite lưu mapping, revision, quyết định, dependency, Review, tiến độ, sổ lần gọi và hồ sơ xuất. EPUB nguồn, ảnh gốc/ảnh edit, file Skill, bản thảo Markdown và EPUB xuất là file trên đĩa. Dữ liệu bất biến có hash để đối chiếu. Hàng đợi dùng chung theo hồ sơ người dùng có SQLite riêng ở mục 28. Key thuộc kho thông tin xác thực của hệ điều hành hoặc nguồn môi trường đã chọn.

Workspace Rust mới bắt đầu ở schema 1. Nâng cấp sau này khóa ghi, tạo bản sao dữ liệu chính và những file sẽ bị sửa, chuyển từng phiên bản, kiểm tra toàn vẹn rồi mở lại. Lỗi nâng cấp phục hồi mốc trước. Workspace test Go cũ được xử lý bằng luồng tạo workspace mới và import EPUB theo quyết định đã chốt. [Đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md)

<a id="decision-07"></a>

### 07. Cơ chế Generation và Atomic Commit · P0

Mỗi generation lưu ID cha, các tham chiếu thay đổi và dấu kiểm tra. Revision, tham chiếu Review, quyết định, dependency, tiến độ, metadata và con trỏ hiện hành được công bố trong **một giao dịch SQLite**. Lưu bảng trạng thái hiện hành để đọc nhanh và lịch sử thay đổi để truy ngược. Khi có file bất biến mới, ghi vào file tạm, hoàn tất/kiểm tra hash và đặt vào kho trước khi commit tham chiếu. File chưa được tham chiếu có thể thu gom sau. [Giao dịch của SQLite](https://www.sqlite.org/atomiccommit.html)

Markdown hiển thị được dựng sau commit và có bản ghi việc cần hoàn thành. Nếu đĩa đầy ở bước này, báo **Đã lưu bản dịch, chờ dựng Markdown**. Mở lại đọc generation đã commit, so hash bản thảo, bảo vệ sửa tay và tiếp tục dựng file. File Skill do người dùng chỉnh có quy trình công bố riêng ở mục 13–15.

<a id="decision-08"></a>

### 08. Đồng bộ bản thảo Markdown và revision · P0

Khi dựng Markdown, lưu revision nền, hash và marker từng đơn vị. Theo dõi file ngoài và kiểm tra lại hash khi mở, trước khi ghi và trước khi tiếp tục tác vụ. Sau một lần lưu hoàn chỉnh, nếu marker hợp lệ, nhận diện được đơn vị và nền còn phù hợp, nhập phần thay đổi thành **revision sửa tay**, ghi người sửa và chuyển đúng phần sang chờ Review.

Khi thiếu/trùng marker, giữ nguyên file đã sửa, chỉ ra vùng mất liên kết và cho gắn lại đơn vị qua editor. Nếu AI cũng sửa cùng đoạn, mở so sánh **bản nền / bản người dùng / ứng viên AI**. Ghép tự động phần sửa khác đơn vị khi nền khớp. Phần sửa chồng nhau cần người dùng chọn hoặc tự kết hợp. Giữ lịch sử cả hai và Review revision được chọn.

<a id="decision-09"></a>

### 09. Quản lý Dependency và Invalidation · P0

Mỗi revision dịch ghi các phạm vi nguồn, toàn bộ phiên bản quy tắc/ngữ cảnh được cung cấp trong request và cặp xưng hô/mốc cảnh. Lưu quan hệ ngược từ từng quyết định/sự kiện tới các đơn vị áp dụng, cùng chỉ mục lần xuất hiện của thuật ngữ và nhân vật. Phần AI khai báo đã sử dụng là thông tin bổ sung. Khi chưa chứng minh được quan hệ chỉ ảnh hưởng một đoạn, giữ phạm vi kiểm tra của cả lượt đã nhận ngữ cảnh ấy. Kết luận cấp chương tham chiếu tập hợp revision của chương.

Khi một căn cứ đổi, đánh dấu **Cần kiểm tra lại** ở những đơn vị phụ thuộc và đi tiếp theo chuỗi sử dụng cho đến khi hết ảnh hưởng. Quy tắc mới được đối chiếu thêm với chỉ mục lần xuất hiện và phạm vi hiệu lực để tìm các bản dịch trước khi quy tắc tồn tại. Review có thể xác nhận bản dịch hiện tại vẫn phù hợp. Nếu sai, dùng đúng phạm vi M1–M4. Nguồn thay đổi đến mức mất đối chiếu thì lập mapping mới cho phần đó. Ví dụ, bác một sự kiện ở chương 2 sẽ mở lại cảnh của chương 4 từng dùng sự kiện ấy làm căn cứ.

<a id="decision-10"></a>

### 10. Quản lý trạng thái ở ba cấp · P1

Core quản lý ba trục độc lập. **Chất lượng** gắn với bản dịch và có các giá trị chưa dịch, chờ Review, cần sửa, cần xác minh, đạt, người dùng giữ ngoại lệ. **Hiệu lực** gắn với bộ dữ liệu đã dùng, gồm còn phù hợp hoặc cần kiểm tra lại. **Thực thi** gắn với tác vụ, gồm xếp hàng, chạy, đang tạm dừng, tạm dừng, chờ người dùng, lỗi, hoàn tất và hủy. TUI có thể hiện đồng thời “đã đạt ở r2”, “cần kiểm tra do đổi glossary” và “tác vụ tạm dừng”.

Mọi chuyển trạng thái đi qua bảng sự kiện của core. Lưu nháp hợp lệ dẫn đến chờ Review. Chỉ Review đủ bằng chứng và độ phủ mới xác nhận đạt. Đổi căn cứ hạ hiệu lực của kết luận cũ. Tiếp tục chỉ được gửi AI khi đủ quyền, ngân sách và dữ liệu. Chuyển trái điều kiện trả lý do và giữ lịch sử sự kiện trước đó. [Các trục trạng thái](DATA_MODEL_STATE_SPEC.md)

## Skills, glossary và xưng hô

<a id="decision-11"></a>

### 11. Thiết kế mẫu Skill riêng của bộ truyện · P0

Dùng một mẫu Markdown có phiên bản với sáu mục **Văn phong, Nhân vật và bí danh, Thuật ngữ và danh hiệu, Địa danh, Xưng hô, Ngoại lệ**. Tool tạo sẵn khung, các mục có thể để trống. Dòng thuật ngữ cần nguyên ngữ, cách dịch và nghĩa/định danh. Phạm vi trống được hiển thị rõ là toàn bộ bộ truyện. Bí danh, cách đọc và ghi chú là tùy chọn. Quy tắc xưng hô cần người nói, người nghe, cách tự xưng, cách gọi và mốc hiệu lực. Cho khai báo nhiều lựa chọn khác phạm vi hoặc thời điểm. Tool cấp ID ổn định và lưu lịch sử, người duyệt, bằng chứng trong dữ liệu của bộ.

Khi đăng ký lần đầu, file riêng hiện có được xem trước theo ba nhóm **đọc được / cần làm rõ / giữ làm ghi chú**. Người dùng xác nhận một lần các quy ước rõ ràng dùng làm nền, rồi tool tạo bản theo mẫu ở đường dẫn riêng. Các mục đã được người dùng xác nhận và còn hiệu lực được dùng lại. Giữ nguyên file người dùng đã chuẩn bị. Đề xuất mới của AI về sau đi qua luồng duyệt từng mục hoặc nhóm. [Hai tầng Skills](SKILLS_GLOSSARY_SPEC.md)

<a id="decision-12"></a>

### 12. Phân biệt hướng dẫn, ví dụ và quyết định chính thức · P0

Skill chung vẫn nhận Markdown tự do. Heading hoặc nhãn tùy chọn **Hướng dẫn bắt buộc**, **Ví dụ**, **Tham khảo** giúp phân loại đoạn. Văn xuôi thông thường dùng làm tham khảo. Ví dụ có nhiều phương án như “tớ–cậu / mình–bạn” được đưa vào đúng ngăn ví dụ. Một thuật ngữ chính thức của bộ phải truy được về quy ước người dùng cung cấp rõ ràng hoặc một quyết định đã duyệt.

Khi chọn ngữ cảnh, lọc file đang bật theo ngôn ngữ, loại nội dung, thực thể và phạm vi. Dùng quy tắc riêng đã duyệt đúng phạm vi trước hướng dẫn chung. Chọn ví dụ liên quan trong dung lượng còn lại. Hai chỉ dẫn bắt buộc cùng hiệu lực xung đột, hoặc một đoạn mơ hồ đang quyết định cách dịch, sẽ được đưa cho người dùng xem hai nguồn. Phần tham khảo đơn thuần có thể tiếp tục dùng với nhãn tham khảo.

<a id="decision-13"></a>

### 13. Đồng bộ Skill khi người dùng chỉnh file ngoài · P0

Dùng cả thông báo thay đổi file lẫn kiểm tra hash trước request và trước khi tool ghi file. Sau thông báo, đọc lại khi file ổn định, parse trọn tài liệu và đối chiếu ID, phạm vi cùng phiên bản nền. Sửa một dòng có cùng ID tạo phiên bản mới của quy tắc đó. Dòng mới nhận ID mới. Khi ID bị mất và có nhiều cách đối chiếu, hiển thị diff để người dùng nhận diện. Thay đổi rõ ràng do người dùng lưu ngoài được áp dụng từ lượt chưa gửi tiếp theo theo 2B.

Khi tool ghi quyết định, lưu ý định cùng hash nền và hash đích trong SQLite, dựng file tạm, kiểm tra lại nền rồi thay file bằng thao tác bảo toàn phiên bản thực tế vừa bị thay. Đối chiếu cả bản được giữ lại lẫn hash đích trước khi kích hoạt quyết định trong giao dịch hoàn tất. Watcher nhận diện thao tác ghi của tool theo ID đang chờ để xử lý cùng giao dịch công bố. Mở lại có thể hoàn tất đúng thao tác đã ghi. Nếu file ngoài đổi trong lúc đó, ghép ba phía đối với các mục tách biệt. Phần chồng lấn chuyển sang **Chờ xử lý xung đột**. Trong khi việc công bố còn dở, phần phụ thuộc chờ. Request đã gửi giữ đúng phiên bản Skill thực sự đã dùng.

<a id="decision-14"></a>

### 14. Phát hiện và gom thuật ngữ mới · P1

Bốn nơi được ghi nhận một lần xuất hiện là chuẩn bị ngữ cảnh, kết quả Dịch, Review và Research. Core gom tất cả về một danh sách của bộ. Khóa gom gồm dạng nguyên ngữ để tra, nghĩa/thực thể và phạm vi. Cùng một danh hiệu xuất hiện 20 lần tạo **một đề xuất có 20 vị trí**. Cùng chữ nhưng chỉ hai khái niệm khác nhau tạo hai đề xuất. Khi nghĩa chưa phân biệt được, giữ ví dụ nguồn của từng nơi để xác minh.

Đưa ra duyệt tên riêng, danh hiệu, kỹ năng, địa danh, từ tác giả đặt, hoặc cách gọi lặp lại có nhiều lựa chọn ảnh hưởng tính nhất quán. Từ thông dụng dùng theo ngữ cảnh khi đã rõ nghĩa. Mỗi đề xuất có nguyên ngữ, đoạn gốc, vị trí, cách dịch khả thi, lý do và phạm vi. Chặn đúng các đơn vị đang chờ lựa chọn. Ứng viên đã dùng thử từ mới nằm ở hồ sơ đề xuất cho tới khi người dùng chốt.

<a id="decision-15"></a>

### 15. Duyệt nhiều thuật ngữ cùng lúc · P1

Chọn **cả nhóm cùng thành công** cho đúng danh sách và phiên bản người dùng đã đánh dấu. Nếu chọn 10 mục mà mục 7 xung đột, tool giữ nguyên lựa chọn, chỉ ra mục 7 và cho bỏ nó để bấm **Chốt 9 mục**. Kiểm tra mâu thuẫn giữa các mục trong nhóm, với quy tắc hiện hành và với file Skill trước khi công bố. Mỗi mục vẫn có bằng chứng, phạm vi và lịch sử riêng.

Nếu người dùng đã xác nhận nhưng ghi Skill thất bại, hiện **Đã nhận xác nhận, chờ đồng bộ Skill**. Phục hồi theo mục 13 rồi kích hoạt cả nhóm một lần. Sau đó tác vụ tự động vốn đang chờ đúng các mục này được tiếp tục khi đủ điều kiện và đến lượt chạy. Tác vụ người dùng đã tạm dừng cần lệnh tiếp tục. [Thao tác duyệt 7A](SKILLS_GLOSSARY_SPEC.md)

<a id="decision-16"></a>

### 16. Xưng hô theo nhân vật và mốc thời gian · P1

Mỗi nhân vật có ID, các tên/bí danh đã xác nhận cùng trỏ tới ID đó. Quy tắc có chiều A→B, cách A tự xưng, cách A gọi B, hoàn cảnh và khoảng hiệu lực. Ghi hai mốc riêng là vị trí cảnh xuất hiện trong sách và thời điểm sự kiện trong truyện. Trong hồi tưởng, tra quy tắc đã có hiệu lực vào thời điểm hồi tưởng, đồng thời lọc những gì độc giả được biết tại vị trí đang dịch.

Chưa rõ người nghe, thời điểm hoặc phản ứng sau lời tỏ tình thì lưu là **Chưa xác định**. Đề xuất đổi “tôi–cậu” sang “anh–em” phải cho thấy nguyên tác trước sự kiện, lời nói/phản ứng, bằng chứng dùng sau đó và phạm vi dự kiến. Người dùng có thể duyệt ngoại lệ chỉ cho một cảnh. Dời mốc hoặc nhập thêm bí danh sẽ xét lại đúng những câu thoại có liên quan thông qua chỉ mục ở mục 09. [Đặc tả xưng hô](ADDRESSING_CONTINUITY_SPEC.md)

## Model và chi phí

<a id="decision-17"></a>

### 17. Hợp đồng kết nối model/provider · P0

Dùng một giao diện cho ba vai trò. Adapter mô tả khả năng model, gửi request và trả các sự kiện bắt đầu, phần nội dung, hoàn tất hoặc lỗi. Phản hồi một lần và phản hồi dạng luồng đều kết thúc bằng cùng một hồ sơ, gồm ID lần gọi, alias đã yêu cầu, ID thực tế nếu provider trả, lý do kết thúc, nội dung, usage từng loại, thời gian chờ dịch vụ và lỗi đã phân loại. Phản hồi dở được lưu riêng để chẩn đoán.

Giới hạn context/output và bộ đếm token lấy từ khả năng provider hoặc hồ sơ model có nguồn/phiên bản. Thiếu giới hạn cần để gửi an toàn thì yêu cầu bổ sung. Trường usage, giá và model thực tế chưa biết có trạng thái riêng. TUI hiện model đã yêu cầu, bổ sung định danh thực tế khi biết. Adapter chỉ dùng kết nối đã chọn. Đổi key thay bí mật ở kho hệ điều hành cho lần gửi sau. Xóa key đánh dấu kết nối cần thiết lập lại, giữ lịch sử tác vụ. [Model](MODEL_STRATEGY_SPEC.md) · [API key](SECURITY_SPEC.md)

<a id="decision-18"></a>

### 18. Cách tính và kiểm soát chi phí API · P1

Dùng bảng giá có phiên bản theo provider, model, đơn vị tiền, đơn vị tính, ngày hiệu lực và nguồn. Cho người dùng nhập/ghi đè đơn giá có ghi nguồn. Lúc gửi, chốt phiên bản giá của model đã chọn và dự phòng theo input cùng trần output/reasoning. Khi nhận kết quả, đối chiếu theo model và usage thực tế, lưu khoản điều chỉnh. Adapter tách các nhóm token theo cách provider tính phí để phần cached/reasoning chỉ bị tính một lần. Khoản phí công cụ Research có dòng riêng.

Mỗi tác vụ có một đơn vị tiền. Khi cần quy đổi, dùng tỷ giá do người dùng nhập, có ngày và được lưu cùng phép tính. Với hạn mức tiền, chỉ gửi khi tổng **đã ghi nhận + dự phòng cho các lần chưa rõ + dự trù lần tới** còn trong hạn mức và đủ dữ liệu để ước tính. Mất kết nối giữ khoản dự phòng cho tới khi đối chiếu. Ở chế độ không giới hạn tiền, phần thiếu giá hoặc usage hiển thị **Chưa xác định**. Màn chi phí phân biệt khoản provider báo và ước tính của tool.

<a id="decision-19"></a>

### 19. Xử lý timeout và phản hồi đến muộn · P1

Một công việc logic có `request_id`. Mỗi lần thực sự gửi có `attempt_id`, thời hạn và bộ phiên bản đầu vào riêng. Trước khi gửi retry, nếu đã nhận đủ kết quả cũ còn hợp lệ thì dùng kết quả đó. Khi retry đã gửi cho cùng công việc, kết quả của lần cũ đến muộn được lưu ở lịch sử của lần cũ. Core dành quyền công bố cho lần gửi hiện hành, bằng kiểm tra phiên bản và một khóa duy nhất cho kết quả của công việc logic trên nền đó.

Mỗi lần gửi có sổ usage riêng, kể cả lần cũ đã timeout. Nếu provider hỗ trợ, đối chiếu bằng ID phía dịch vụ. Khoản chưa rõ vẫn được dự phòng. Dùng chính sách 9A với mặc định 10 phút/lần gọi, hai lần retry lỗi tạm thời sau 10 và 30 giây, tôn trọng thời gian chờ dài hơn do dịch vụ yêu cầu. Nguyên nhân chưa rõ hoặc hết lượt thì lưu bước và báo người dùng.

## Dịch và Review

<a id="decision-20"></a>

### 20. Cấu trúc request và response của phần Dịch · P0

Chọn hợp đồng JSON có `schema_version`. Request gồm danh sách đơn vị được giao theo thứ tự, ID/phạm vi và nguyên văn từng đơn vị, marker được bảo vệ, ngữ cảnh chỉ đọc, Skills/glossary/xưng hô đã duyệt, hướng dẫn văn phong và hợp đồng đầu ra. Chương trình giữ snapshot, hash, phiên bản và model của lần gửi. Model trả `units[{id, target}]`, `uncertainties[]`, `continuity_candidates[]` và `rule_candidates[]`. Mỗi đề xuất/điểm chưa rõ có vị trí và căn cứ nguồn.

Core kiểm tra đủ ID, đúng thứ tự, mỗi ID xuất hiện một lần, bản dịch có chữ và các marker, ruby, liên kết, ảnh, chú thích còn đối chiếu được. Lỗi cấu trúc tạo chẩn đoán theo ID và tối đa hai yêu cầu sửa. Kết quả bị cắt cần lập lại ngân sách/phạm vi. Đơn vị dùng thuật ngữ mới chờ quyết định ở vùng ứng viên. Chỉ bản đã qua các điều kiện mới thành nháp hiện hành. [Hợp đồng Dịch](TRANSLATION_STAGE_SPEC.md)

<a id="decision-21"></a>

### 21. Cách thực hiện lượt đọc nguyên tác của Reviewer · P0

Mỗi lượt Review bắt đầu bằng một request **chỉ đọc nguyên tác và ngữ cảnh nguồn đã kiểm chứng**. Tách thành nhiều request nguồn liên tiếp nếu vượt dung lượng. Trường hợp đã có kết quả đọc nguồn đúng phạm vi và còn hiệu lực, có thể dùng lại. Bản Việt đang xét chỉ xuất hiện ở request đối chiếu sau đó.

Mỗi ghi nhận chứa phạm vi, trích dẫn, nhân vật/người nói/người nghe, sự kiện, phủ định/giả định, thời điểm, điểm chưa rõ. Gom nhận định theo vị trí nguồn. Gặp mâu thuẫn thì mở lại đoạn gốc và cảnh quanh đó. Ghi nhận ban đầu là bằng chứng làm việc của Review. Chỉ nhận định đã kiểm chứng với nguyên tác mới đủ điều kiện thành ngữ cảnh dùng về sau. Một sự kiện thiếu căn cứ được bác hoặc giữ **Cần xác minh**. Độ phủ và vị trí trích dẫn có thể kiểm tra tự động. Việc hiểu đúng toàn bộ ý nghĩa vẫn có thể sai, cần kiểm chứng và đo bằng bộ đối chứng ở mục 36. [Lượt đọc nguồn](REVIEW_STAGE_SPEC.md)

<a id="decision-22"></a>

### 22. Kiểm tra bằng chứng và kết luận Review · P0

Mỗi lỗi có ID, loại, mức S0–S3, snapshot/phạm vi nguồn, revision/phạm vi bản Việt, trích dẫn hiện có, lời giải thích, trạng thái bằng chứng, phạm vi M đề xuất và cách sửa. Lỗi bỏ sót dùng câu gốc và vị trí thiếu ở bản dịch. Lỗi thêm nội dung dùng chính câu tiếng Việt và ghi rõ vùng nguồn đã đối chiếu để kiểm tra sự vắng mặt.

Code kiểm tra ID, phiên bản, vị trí, sự tồn tại của trích dẫn và độ phủ. Reviewer kiểm tra liệu ngữ cảnh có thực sự chứng minh kết luận, gồm cả cách diễn đạt khác mà vẫn đúng nghĩa. Khi kết quả kiểm chứng trái nhau, lưu cả hai, mở rộng nguyên tác và đưa về **Cần xác minh** đến khi có đủ căn cứ. Nếu chỉ một nhận định của Reviewer sai, sửa/bác nhận định đó. Nếu nền nguồn hoặc độ phủ sai, mở lại toàn bộ phạm vi Review bị ảnh hưởng.

<a id="decision-23"></a>

### 23. Thuật toán lựa chọn phạm vi sửa M1–M4 · P0

Reviewer đề xuất mức sửa, vị trí, nguyên nhân, bằng chứng và phần phụ thuộc. Core kiểm tra điều kiện và chọn phạm vi nhỏ nhất có thể sửa đúng. **M1** vá câu/cụm chính xác, kể cả ở nhiều vị trí đã định danh. **M2** dịch lại một đoạn. **M3** dịch lại cảnh hoặc nhóm đoạn, có danh sách vị trí và lý do chúng phụ thuộc nhau. **M4** dịch lại chương khi chứng minh được sai lệch xuyên chương, lý do sửa chọn lọc không đủ và một lượt kiểm chứng bổ sung trên nguyên tác xác nhận. Lượt kiểm chứng có quyền thu hẹp hoặc bác đề xuất.

Sửa nguyên nhân ở mapping, quy ước hoặc nguồn ngữ cảnh trước khi dịch lại. Lưu các revision trước và dùng lại phần nằm ngoài phạm vi vẫn hợp lệ. Sau M1/M2, kiểm tra phần vừa sửa, phần thật sự dùng nó làm ngữ cảnh và kết luận chương liên quan. Dù lỗi ít hay nhiều, quyết định M dựa trên phạm vi chứng minh được. [Rubric và điều kiện M4](REVIEW_RUBRIC_SPEC.md)

<a id="decision-24"></a>

### 24. Kiểm tra tổng thể cuối chương · P0

Tạo hồ sơ Review chương có tập hợp revision hiện tại, độ phủ nguồn→dịch và dịch→nguồn, kết quả từng lượt, các điểm nối, vấn đề còn mở và kết luận toàn chương. Dùng lại kết quả cục bộ còn hiệu lực. Mỗi điểm nối, quan hệ xuyên cảnh hoặc nghi vấn được kiểm tra với nguyên tác và bản Việt thực tế. Khi độ phủ cũ không đủ, mở lại phần thiếu, kể cả toàn chương khi cần.

Chương dài kiểm tra lần lượt theo cảnh với ít nhất một đoạn hai bên điểm nối nếu có, rồi mở rộng theo sự kiện, người nói, mốc thời gian, xưng hô, thuật ngữ, thêm/bớt ý và phủ định. Phần chồng nhau là ngữ cảnh đối chiếu. Nếu lượt đầu sai, lần theo mục 09 để xét lại các lượt sau phụ thuộc. Chỉ chuyển sang chương kế khi dữ liệu còn hiệu lực, kiểm tra mạch chương đã xong và mọi điểm chặn đã được giải quyết theo quy tắc ngoại lệ.

<a id="decision-25"></a>

### 25. Vòng sửa chất lượng và ngoại lệ người dùng · P1

Theo dõi `repair_round_id` cho một kế hoạch sửa có phạm vi, nhóm lỗi, revision nền và revision mới. Một vòng có thể gồm nhiều request. Chỉ tính một vòng chất lượng thất bại khi đã có bản sửa hợp lệ và Review lại vẫn xác nhận lỗi của đợt đó. Review đầu tiên, lỗi mạng, lỗi cấu trúc và thời gian chờ duyệt có bộ đếm/trạng thái riêng.

Hết **năm vòng thất bại**, bàn giao bằng chứng và lịch sử. Người dùng sửa tay tạo revision mới và có thể yêu cầu Review bản đó. Giữ số vòng đã dùng. Nếu người dùng chọn giữ lỗi, lưu người duyệt, lý do, issue, mức, phạm vi, revision và căn cứ. Hiện **Người dùng giữ — có ngoại lệ**. Thay revision/căn cứ của phần được miễn thì cần xét lại ngoại lệ. Các đơn vị khác còn nguyên giữ được lịch sử riêng. Điều kiện độ phủ, Review chương, quyết định thuật ngữ và tính hợp lệ kỹ thuật vẫn phải đạt.

## Research và điều phối

<a id="decision-26"></a>

### 26. Công cụ tìm kiếm và giới hạn Research · P1

Tra nguyên tác, glossary và kết quả đã xác nhận trước. Khi cần web, dùng công cụ tìm URL và công cụ đọc trang riêng. Adapter web đầu tiên là **Brave Search API** do người dùng cấu hình. Cho đặt danh sách nguồn ưu tiên hoặc chặn theo bộ. Chỉ trích dẫn một trang khi đã đọc được đoạn có liên quan. [Brave Web Search](https://api-dashboard.search.brave.com/app/documentation/web-search/get-started) · [Đặc tả Research](RESEARCH_STAGE_SPEC.md)

*Mặc định thử nghiệm cho một câu hỏi* là **3 truy vấn web, 5 trang đọc, 15 phút tổng cộng**, cho phép chỉnh. Hết giới hạn thì trả bằng chứng đã có và điểm còn thiếu. Thời hạn có hiệu lực cho lần gọi model là phần nhỏ hơn giữa thời gian model được phép và thời gian Research còn lại. Phí dịch vụ tìm kiếm/đọc trang được dự trù và ghi vào cùng tác vụ ở dòng riêng. Mỗi lần gọi model Research vẫn tính vào trần model.

Cache theo câu hỏi, thực thể, mốc truyện, phạm vi nguồn, cấu hình nguồn và phiên bản những căn cứ đã dùng. Dữ liệu nội bộ dùng lại khi các căn cứ còn hiệu lực. Với nguồn web, kiểm tra lại sau 7 ngày và sớm hơn khi có mâu thuẫn hoặc nguồn đổi. Lưu URL, thời điểm, trích đoạn cần thiết và kết luận có nguồn. Việc lưu câu trả lời thô từ dịch vụ tìm kiếm phụ thuộc quyền lưu của kết nối.

<a id="decision-27"></a>

### 27. Bằng chứng Research và giới hạn spoiler · P1

Mỗi dữ kiện có nguồn, trích đoạn, vị trí/URL, hash đoạn đọc, thời điểm lấy, thực thể, thời điểm sự kiện, **mốc sớm nhất độc giả được biết** và trạng thái xác minh. Đánh giá từng dữ kiện trong một trang. Một trang wiki có thể có cả chi tiết dùng được lẫn chi tiết chỉ xuất hiện ở tập sau. Khi thiếu căn cứ xác định mốc tiết lộ, giữ thông tin ở hồ sơ Research với trạng thái **Chưa rõ mốc** và loại khỏi gói ngữ cảnh dùng cho cảnh hiện tại.

Khi nguồn đổi hoặc biến mất, giữ trích đoạn đã ghi nhận kèm thời điểm, đánh dấu cần kiểm chứng lại và ưu tiên đối chiếu nguyên tác. Chỉ dùng xuyên tập khi cùng nghĩa/thực thể, đúng mốc tiết lộ, đúng phạm vi và nguồn còn đủ căn cứ. Kết quả tìm thấy một cách gọi mới đi qua bước duyệt thuật ngữ như các nguồn đề xuất khác.

<a id="decision-28"></a>

### 28. Hàng đợi và quyền chạy tác vụ AI · P0

Dùng **một bộ điều phối theo hồ sơ người dùng trên máy**, chung cho TUI, CLI và các workspace. SQLite của bộ điều phối lưu thứ tự yêu cầu, ID tác vụ/workspace, chủ thể đang giữ quyền và số hiệu của lần cấp quyền. Một khóa của hệ điều hành bảo đảm chỉ một tiến trình làm bộ điều phối cho hồ sơ đó. Những giao diện còn lại gửi yêu cầu qua bộ điều phối. Ghi workspace có khóa ghi riêng và kiểm tra phiên bản trước commit.

Lấy tác vụ đã được giao theo thứ tự hàng đợi. A hoàn tất thì kiểm tra điều kiện của B rồi mới chạy. A tạm dừng/chờ người dùng thì lưu bước, ngừng lên lịch, xử lý xong lần gọi tại tool và cho người dùng chọn một B độc lập. Kiểm tra phụ thuộc ở mục 09 trước khi bàn giao. Sau crash, đối chiếu khóa hệ điều hành, số hiệu quyền, hàng đợi và sổ attempt trong workspace. Mỗi lần bắt đầu/kết thúc được đối chiếu bằng cùng ID ở hai kho. Phản hồi muộn thuộc A chỉ được ghi nhận theo mục 19. [Lựa chọn 13A](WORKFLOW_ORCHESTRATION_SPEC.md)

<a id="decision-29"></a>

### 29. Tạm dừng, tiếp tục và khôi phục sau lỗi · P0

Ghi checkpoint khi đã chuẩn bị công việc, trước khi gửi, khi nhận đủ phản hồi, sau kiểm tra/commit và khi đổi sang Review, chờ duyệt, retry hoặc hoàn tất. Ghi trước ý định gửi và khoản dự phòng. Lần đã có ý định gửi nhưng bị tắt ứng dụng trước khi biết kết quả có trạng thái **Chưa rõ đã xử lý**. Đối chiếu với provider khi có khả năng tra cứu. Chỉ retry theo điều kiện lỗi tạm thời đã xác định hoặc yêu cầu của người dùng.

Có response đã lưu thì tiếp tục kiểm tra nó. Có nháp đã commit thì Review đúng nháp. Có kết quả Review cục bộ hợp lệ thì tiếp tục đúng lượt hoặc kiểm tra cuối chương. Lưu thời điểm retry sớm nhất. Khi tiếp tục, chờ phần thời gian còn lại và kiểm tra ý định dừng/quyền chạy trước khi gửi. Nếu nguồn hoặc Skill đã đổi, dựng lại request chưa gửi, còn kết quả đã có thì truy dependency để đánh giá phần cần Review lại. Chi phí, số lần thử và các khoản chưa rõ đi theo tác vụ.

## Ảnh, preview và EPUB đầu ra

<a id="decision-30"></a>

### 30. Kiểm tra và lựa chọn ảnh thay thế · P1

Nhận thay ảnh **JPEG, PNG, GIF, WebP và SVG tĩnh an toàn**. Khớp chính xác đường dẫn tương đối, chữ hoa/thường và đuôi file trong `images/edit/`. Kiểm tra loại nội dung thật, đọc được cấu trúc, kích thước, bộ nhớ giải mã và tham chiếu. Với SVG, chỉ nhận nội dung tĩnh thuộc tập cho phép, chặn script và tài nguyên ngoài. Ảnh thay khớp nhưng lỗi chặn xuất và chỉ đúng file cần sửa. Hiện chênh lệch kích thước/tỷ lệ trong preview. File hợp lệ giữ nguyên byte người dùng đã chuẩn bị.

*Mặc định thử nghiệm* cho mỗi ảnh là 64 MiB, mỗi chiều bitmap tối đa 16.384 pixel, tối đa 40 triệu pixel/khung và 100 triệu pixel cộng dồn khi kiểm tra ảnh động. Các giới hạn này điều chỉnh được theo cấu hình. Trước preview/export, đọc phiên file ổn định vào kho theo hash, kiểm tra lại và ghim cả ảnh được chọn lẫn ảnh gốc cần cho so sánh. File đổi trong lúc đọc thì chuẩn bị lại tài nguyên đó. [Đặc tả ảnh](IMAGE_EDIT_SPEC.md)

<a id="decision-31"></a>

### 31. Bảo mật và vận hành Preview cục bộ · P0

Mở phiên trên loopback, cổng tạm và danh sách tài nguyên theo bản chuẩn bị. Sinh bí mật ngẫu nhiên 256 bit dùng một lần, truyền ở phần fragment của URL mở trình duyệt. Trang điều khiển tin cậy đổi mã đó qua POST cùng origin, xóa fragment khỏi thanh địa chỉ và giữ quyền phiên trong bộ nhớ của trang. Tài nguyên sách chỉ được trang điều khiển lấy qua yêu cầu có quyền, theo ID nội bộ. Kiểm tra Host/origin hợp lệ, tránh ghi thông tin phiên vào log/referrer. Phiên kết thúc khi bấm đóng, tiến trình phục vụ đóng, hoặc sau **30 phút không hoạt động** theo mặc định thử nghiệm.

Dựng tài liệu xem cô lập với giao diện tin cậy. Parser chỉ cho phép các phần tử/thuộc tính/CSS cần cho sách, chặn script, sự kiện, form/iframe chủ động, tham chiếu mạng, đường dẫn file tùy ý và nội dung lồng có khả năng thực thi. Tài nguyên theo ID mở từ bản bất biến đã kiểm tra. Khi ảnh/chữ/bố cục đổi, hiện **Bản xem trước đã cũ** và nút nạp lại. Bản xuất lấy cùng bộ dữ liệu đã kiểm tra. [Đặc tả bảo mật](SECURITY_SPEC.md)

<a id="decision-32"></a>

### 32. Chuẩn hóa và dựng EPUB 3 · P0

Chọn **EPUB 3.3** làm đích nghiệm thu của bản đầu. Core sở hữu quy tắc chuyển đổi, dùng crate Rust `zip` cho container và `quick-xml` cho đọc/ghi XML có namespace. Với EPUB 2, chuyển cấp, thứ tự và đích của mục lục NCX sang Navigation XHTML. Dựng lại package, manifest, spine, metadata tiếng Việt/định danh/bìa và kiểm tra mọi liên kết. Đặt `mimetype` đầu gói, ở dạng lưu không nén. [EPUB 3.3 của W3C](https://www.w3.org/TR/epub-33/) · [zip](https://github.com/zip-rs/zip2) · [quick-xml](https://github.com/tafia/quick-xml)

Chữ Việt đọc ngang trái sang phải. Giữ bố cục trang minh họa cố định trong phạm vi được hỗ trợ. Với ruby thuần phát âm, xuất chữ chính theo kết quả Dịch–Review. Với ruby mang nghĩa riêng, áp cách thể hiện đã được duyệt. Bảo toàn đoạn, chú thích, anchor, ảnh và nhấn mạnh. Mở đọc lại file đã dựng, kiểm tra XML/tài nguyên/độ phủ và chạy **EPUBCheck 5.3.0** làm mốc kiểm tra 3.3. Lưu phiên bản công cụ và báo cáo. Lỗi chặn xuất, cảnh báo phải được phân loại, trường hợp ảnh hưởng nội dung được xử lý trước khi công bố. Kiểm tra đọc thực tế trên fixture qua Calibre và một trình đọc mục tiêu. [EPUBCheck 5.3.0](https://github.com/w3c/epubcheck/releases/tag/v5.3.0) · [Đặc tả đầu ra](EPUB_OUTPUT_SPEC.md)

<a id="decision-33"></a>

### 33. Bộ dữ liệu chuẩn bị và phiên bản xuất EPUB · P1

Lưu một manifest bất biến theo hash cho mỗi bản chuẩn bị. Ghi snapshot/mapping, chế độ nội dung và phần giữ nguyên, các revision/Review/ngoại lệ, quy tắc có hiệu lực, ruby, metadata, cấu hình bố cục, danh sách ảnh source/edit và hash đã chọn. Quét cả các đường dẫn có thể xuất hiện ảnh thay mới. Đổi ảnh hoặc metadata thì dựng lại các phần phụ thuộc và chạy lại kiểm tra đầu ra. Kết quả dịch chữ còn hợp lệ được dùng lại.

Khi có lệnh xuất mới, đặt trước số bản theo tập trong một giao dịch duy nhất. **Lệnh mới sau lần r003 thất bại lấy r004.** Tiếp tục chính tác vụ r003 thì đối chiếu hồ sơ r003. Dựng và kiểm tra file tạm, lưu hash, đích, ID lần xuất rồi mới công bố file bằng thao tác giữ mọi file đang tồn tại. Nếu file đã xuất nhưng hồ sơ chưa đánh dấu xong, khôi phục theo ID, đích, hash và báo cáo đã lưu. File cùng tên khác hash được báo xung đột. Mỗi lần thành công có đường dẫn và lịch sử riêng.

## Giao diện và nghiệm thu

<a id="decision-34"></a>

### 34. Hợp đồng lệnh CLI · P1

Dùng tên thực thi **`novel-translate-cli`**. Mở không có tham số vào TUI. Các lệnh nhận `--workspace <đường-dẫn>` khi cần workspace, `--json` cho script và `--non-interactive` khi cần trả kết quả mà không đợi nhập. Bảng dưới là cú pháp đã chốt để triển khai.

| Lệnh hoặc nhóm lệnh | Đích và tham số chính |
| --- | --- |
| `workspace create <path>`, `workspace open <path>` | Tạo bộ mới hoặc kiểm tra/mở workspace đang hỗ trợ. |
| `import <epub> --volume <id>`, `scope set --volume <id> --content all-text\|story` | Nhập nguồn, xem cây và lưu phạm vi nội dung của tập. |
| `translate --volume <id>` | Chọn `--chapter <id>` lặp được hoặc `--to-end`. Cho ghi đè lựa chọn nội dung và ngân sách của tác vụ bằng `--cost-limit <số> --currency <mã>` hoặc `--unlimited-cost`. |
| `review --volume <id> --chapter <id>`, `issues --task <id>` | Review đúng dữ liệu hiệu lực hoặc liệt kê lỗi/bằng chứng của tác vụ. |
| `terms list`, `terms confirm --file <json>` | File xác nhận liệt kê ID, phiên bản, cách dịch và phạm vi những mục đã chọn. Áp dụng quy tắc nhóm theo [Quyết định 15](#decision-15). |
| `status [--task <id>]`, `pause <task-id>`, `resume <task-id>` | Đọc trạng thái, tạm dừng hoặc tiếp tục đúng bước. |
| `check --volume <id>`, `preview --volume <id>`, `export --volume <id>` | Kiểm tra sẵn sàng, mở phiên preview hoặc xuất EPUB phiên bản mới. |
| `connection set <name> --key-env <tên-biến>`, `connection list`, `connection remove <name>` | Quản lý tham chiếu kết nối. Dùng kho khóa hệ điều hành khi nhập key trong TUI. |

JSON có `schema_version, command, state, task_id, scope, result, issues, next_action`. Dữ liệu máy đọc ở stdout, chẩn đoán đã che thông tin ở stderr. Mã thoát cho lệnh chạy công việc gồm **0** hoàn tất, **2** tham số/cấu hình sai, **3** lỗi vận hành, **4** chờ duyệt/xử lý xung đột, **5** chạm ngân sách/trần gọi, **6** đã xếp hàng/đang có tác vụ đó, **7** tạm dừng hoặc chưa hoàn tất bởi điều kiện khác. `status` trả 0 khi đọc trạng thái thành công, còn trạng thái công việc nằm trong kết quả. CLI không tương tác lưu checkpoint, trả ID và hành động tiếp theo, ví dụ xác nhận thuật ngữ rồi `resume <id>`. [Quy ước CLI/TUI](TUI_CLI_SPEC.md)

<a id="decision-35"></a>

### 35. Chi tiết giao diện TUI · P2

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

<a id="decision-36"></a>

### 36. Bộ dữ liệu kiểm thử EPUB và chất lượng dịch · P1

Tạo EPUB thử nghiệm trong thư mục tạm từ nguồn và trình sinh mẫu tự viết. Bao phủ EPUB 2/3 cơ bản, ruby có/không mang nghĩa, ảnh xen trang kể cả fixed-layout, chú thích dùng chung, nhiều file/một chương và ngược lại, ngoại truyện dễ nhầm quảng cáo, tài nguyên thiếu/hỏng và đường dẫn vượt phạm vi. Git chỉ chứa trình sinh, nguồn mẫu và kỳ vọng. *Mặc định thử nghiệm cho bộ nhập* là tối đa 10.000 mục ZIP, 128 MiB/mục và 1 GiB tổng dữ liệu sau giải nén. Kiểm tra cả số byte thực tế khi giải nén, đường dẫn và xung đột tên trên hệ đích. Vượt giới hạn phải báo chẩn đoán trước khi sử dụng dữ liệu.

Dùng bộ chuẩn ban đầu **30 tình huống Nhật–Việt và 3 chương ngắn tự soạn** có đối chứng. Bao phủ phủ định, chủ thể ẩn, thêm/bớt ý, hồi tưởng, đổi xưng hô sau sự kiện, nhiều nghĩa của một thuật ngữ, ruby, điểm nối, trích dẫn giả, ngoại lệ và trường hợp dịch đúng nhưng diễn đạt khác. Người đọc được nguyên tác Nhật và biên tập được tiếng Việt xác nhận ý nghĩa, trích dẫn, cách dịch chấp nhận được và lỗi dự kiến. Ca thiếu đồng thuận mang nhãn chờ duyệt.

CI chạy offline bằng model/công cụ giả lập để kiểm tra mapping, cấu trúc, điều phối, lỗi, phục hồi, tiền, trần gọi và xuất sách. Đánh giá model thật là một đợt thủ công có ngân sách do người dùng chọn. Báo riêng lỗi nghĩa, độ phủ, tính nhất quán, lỗi bịa, khả năng phát hiện của Reviewer và số lần báo oan. Điều kiện chuyển module là các ca nghiệm thu thuộc phạm vi đều đạt, dữ liệu phục hồi đúng sau các điểm ngắt đã thử, và bộ kiểm tra Rust của dự án đạt. Với module chất lượng, bộ đối chứng phải phát hiện đủ các lỗi S3 cố ý cài, dẫn nguồn đúng và các trường hợp báo oan phải được đối chiếu trước khi chấp nhận. Với module xuất, EPUBCheck đạt và mẫu được mở kiểm tra trong trình đọc. [Bộ lệnh kiểm tra Rust](../../AGENTS.md#required-checks)
