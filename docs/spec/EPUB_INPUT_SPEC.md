# Quyết định 01 — Xử lý EPUB đầu vào và chia lượt dịch theo chương

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 13.

Ngày ghi nhận: **25/10/2026**.

Cập nhật phạm vi hỗ trợ đầu vào: **27/10/2026**.

Cập nhật ngày **29/10/2026**: liên kết [bố cục EPUB tiếng Việt đầu ra](EPUB_OUTPUT_SPEC.md). Nguồn và mapping giữ đủ thông tin; bản Việt mặc định đọc ngang, xử lý ý nghĩa ruby trong Dịch–Review trước khi xuất.

Cập nhật ngày **30/10/2026**: có hai lựa chọn nội dung, **Dịch toàn bộ phần chữ** hoặc **Chỉ dịch nội dung truyện và phần liên quan**. Cả hai tách chữ trong ảnh sang luồng thay ảnh. Phạm vi cụ thể ở mục 1.2; lựa chọn này độc lập với chạy một nhóm chương hay tự động đến hết EPUB.

Trạng thái: **đã chốt hướng sản phẩm trong trao đổi**. Tài liệu này ghi chi tiết hành vi cần có để làm căn cứ triển khai bản Rust. Các mô tả đích không có nghĩa chức năng đã được triển khai. Mức **3.000 token nguồn/lượt** là tham số thử nghiệm ban đầu, cần điều chỉnh qua nghiệm thu JP–VI với model thực tế.

Phạm vi quyết định: nhận EPUB, nhận diện cấu trúc sách, tổ chức cây nội dung và dữ liệu workspace, xác định chương dài, chia lượt dịch và tiếp tục khi gián đoạn.

## 1. Các điểm đã chốt

| Nội dung | Quyết định |
| --- | --- |
| Đầu vào | Một file EPUB được đăng ký làm nguồn của một tập trong workspace bộ truyện. |
| Phạm vi hỗ trợ ban đầu | **EPUB 2 và EPUB 3** có phần truyện là văn bản XHTML trích xuất được, bố cục reflowable; bao gồm sách xen bìa và trang minh họa fixed-layout. Các dạng đặc biệt ngoài phạm vi này xử lý sau. |
| Phiên bản đầu ra | Thống nhất **EPUB 3** cho cả hai loại nguồn theo đặc tả xuất; mỗi lần xuất tạo file phiên bản mới. |
| Nội dung cần dịch | Người dùng chọn dịch toàn bộ phần chữ hoặc chỉ nội dung truyện và phần liên quan; phần ngoài phạm vi ở lựa chọn thứ hai được giữ nguyên trong EPUB theo mục 1.2. |
| Đơn vị người dùng thao tác | **Chương**. Người dùng có thể chọn một/nhóm chương hoặc bật **Dịch tự động đến hết EPUB** để tool xử lý toàn bộ nội dung cần dịch của tập mà không phải chọn từng chương. |
| Cây quản lý chính | **Bộ truyện → Tập → Chương → Nội dung trong chương**. Mở đầu, lời bạt và các phần tương đương được hiển thị riêng nếu sách có. |
| Part trong sách | Giữ tiêu đề, vị trí và thứ tự part trong nội dung. Giai đoạn này chưa cần một cấp quản lý Part riêng. |
| Chương không có part | Giữ các đoạn trực tiếp trong chương. Tool vẫn chia được công việc dịch. |
| Một chương và một request | Một chương ngắn có thể dùng một lượt; một chương dài dùng nhiều lượt. |
| Ranh giới chia | Ưu tiên ranh giới part hoặc cảnh phù hợp; sau đó đến ranh giới đoạn, cuối cùng mới đến ranh giới câu khi cần. |
| Hai giới hạn | Kiểm tra đồng thời **giới hạn kỹ thuật của request** và **giới hạn vận hành của phần nguồn cần dịch**. |
| Ngữ cảnh giữa các lượt | Gửi ngữ cảnh liên quan, glossary và phần nguồn/bản dịch đã duyệt trước đó trong phạm vi dung lượng cho phép. |
| Nguồn và bản dịch | Giữ snapshot nguồn; bản dịch, review và revision được lưu riêng, gắn với vị trí nguồn. |
| Import và gọi model | Import, xem cây nội dung và kiểm tra cấu trúc chạy cục bộ. Model chỉ chạy khi người dùng bắt đầu tác vụ cần model. |

Theo [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md) chốt ngày **28/10/2026**, “hai chương mỗi đợt” chỉ là nhóm chương được chọn/theo dõi. Trong từng chương, review ngay sau mỗi lượt, kiểm tra tổng thể cuối chương rồi mới chuyển sang chương tiếp theo. Không đợi dịch xong cả nhóm mới Review; số chương trong nhóm không quyết định số request.

Chế độ **Dịch tự động đến hết EPUB** tự lập phạm vi toàn tập một lần, giữ kết quả đã hoàn tất còn phù hợp và tiếp tục phần còn thiếu theo thứ tự. Không yêu cầu chọn lại chương/nhóm chương khi chuyển tiếp; các điểm chờ người dùng và giới hạn model/ngân sách vẫn áp dụng.

### 1.1. Phạm vi EPUB 2 và EPUB 3 trong giai đoạn đầu

Quyết định ngày **27/10/2026**: ưu tiên hỗ trợ EPUB 2 và EPUB 3 phục vụ dịch light novel dạng chữ, theo phạm vi dưới đây. Đây là yêu cầu sản phẩm cần triển khai và nghiệm thu, không phải xác nhận implementation hiện tại đã hỗ trợ đầy đủ.

| Trường hợp trong phạm vi | Hành vi cần có |
| --- | --- |
| EPUB 2 có nội dung XHTML | Đọc package, manifest, spine và mục lục NCX để dựng cây nội dung. |
| EPUB 3 có nội dung XHTML | Đọc package, manifest, spine và navigation; đối chiếu NCX nếu có, ghi chẩn đoán khi các dấu mốc mâu thuẫn. |
| Phần truyện reflowable | Bóc văn bản trực tiếp từ XHTML để dịch và review; giữ mapping phục vụ ghép lại. |
| Phần truyện reflowable xen bìa hoặc trang ảnh fixed-layout | Nhận diện bố cục theo từng mục trong spine; bảo toàn trang ảnh và đúng vị trí xen giữa các đoạn/chương. |
| Văn bản Nhật trình bày dọc, có ruby/furigana | Giữ thứ tự nội dung, chữ gốc, phần chú thích và quan hệ giữa chúng. Bản Việt đọc ngang; cách thể hiện ruby theo đặc tả đầu ra, không loại bỏ thông tin khi import. |
| Một chương nhiều XHTML, một XHTML nhiều chương, chương có/không có part | Áp dụng cơ chế nhận diện chương và chia lượt trong tài liệu này; tên file `part...xhtml` không tự xác định part trong truyện. |
| Tranh có chữ, bìa và ảnh minh họa | Tách vào `images/source/`, thay bằng ảnh người dùng chuẩn bị trong `images/edit/` theo đặc tả ảnh. |

Sách có phần chữ reflowable kèm các trang ảnh fixed-layout, như cấu trúc tập Re:Zero đã kiểm tra trong trao đổi, thuộc phạm vi hỗ trợ ban đầu. Có trang ảnh cố định không phải lý do từ chối cả sách. Việc nhận ảnh không có nghĩa tool đã nhận dạng hoặc dịch chữ nằm trong ảnh.

Các trường hợp để giai đoạn sau:

- EPUB toàn trang scan hoặc truyện tranh cần OCR, xác định thứ tự đọc trong ảnh và kiểm tra văn bản nhận dạng.
- Phần truyện có chữ dàn cố định theo trang hoặc bố cục đặc biệt cần dựng lại sau dịch; ngoại lệ trang bìa/tranh nêu trên vẫn được hỗ trợ.
- Nội dung bị khóa/mã hóa mà tool không đọc được để tạo nguồn đối chiếu; chưa triển khai cơ chế mở khóa nội dung.
- Nội dung phụ thuộc script/tương tác hoặc yêu cầu dịch, dựng lại và đồng bộ âm thanh/video với bản dịch.
- Các dạng nội dung khác chưa có quy tắc trích xuất và ghép lại được nghiệm thu.

Import phải kiểm tra cả cấu trúc package lẫn tài nguyên thực tế trước khi xác nhận nguồn nằm trong phạm vi. Phát hiện nội dung chưa hỗ trợ phải báo vị trí và lý do, giữ phần đó chưa hoàn tất; không âm thầm bỏ nội dung rồi báo cả tập đã xử lý xong. Có thể tiếp tục phần độc lập đã xác định đầy đủ, nhưng chưa xuất bản đầy đủ của tập khi nội dung bắt buộc còn bị thiếu.

Nguồn hỏng hoặc thiếu file bắt buộc được báo là lỗi nguồn cần sửa. Trường hợp đọc được nội dung nhưng chưa chắc ranh giới chương tiếp tục dùng cơ chế xem/chỉnh mapping ở mục 3.3. Bước kiểm tra đầu vào chạy cục bộ, không tự gọi model.

### 1.2. Hai lựa chọn nội dung cần dịch

Quyết định ngày **30/10/2026**: người dùng có hai lựa chọn dưới đây. Đây là lựa chọn nội dung cho cả tập, khác với thao tác chọn chương để xử lý trong một tác vụ.

| Nội dung | Dịch toàn bộ phần chữ | Chỉ dịch nội dung truyện và phần liên quan |
| --- | --- | --- |
| Mở đầu, chương chính, ngoại truyện của tác phẩm, phần kết và lời bạt | Dịch và Review. | Dịch và Review. |
| Tiêu đề, mục lục, chú thích, ruby, caption và alt text liên quan | Dịch và Review toàn bộ nội dung văn bản tương ứng. | Dịch và Review phần gắn với nội dung được dịch; nhãn/thông tin chỉ thuộc trang phụ giữ nguyên cùng trang đó. |
| Trang bản quyền, giới thiệu tác giả, quảng cáo, giới thiệu/trích đọc quảng bá tập khác | Dịch và Review, không tự bỏ qua vì là trang phụ. | Giữ nguyên nội dung nguồn trong EPUB, không xóa tài nguyên hoặc liên kết. |
| Chữ nằm trong bitmap ảnh | Theo luồng ảnh source/edit, chưa có OCR/dịch ảnh tự động. | Theo cùng luồng ảnh source/edit. |

Toàn bộ văn bản phục vụ đọc thuộc lựa chọn đầu tiên, kể cả ở ngoài chương truyện hoặc luồng đọc chính. URL, mã định danh, số liệu và tên riêng vẫn được bảo toàn/xử lý đúng loại và quy ước; không ép thay đổi các giá trị đó chỉ để mọi chuỗi ký tự khác nguồn. Metadata nhận diện sách theo đặc tả đầu ra, không thay thế việc xử lý văn bản trên trang sách.

Import vẫn kiểm kê đầy đủ trong cả hai lựa chọn. Lưu riêng loại nội dung và cách xử lý theo phạm vi dịch. Phần chưa rõ loại được hiển thị cùng bằng chứng. Khi các cách phân loại dẫn đến lựa chọn dịch/giữ nguyên khác nhau, phần đó chờ người dùng xác định. Ở chế độ toàn bộ chữ, nội dung đọc được vẫn được dịch khi mọi cách phân loại đang xét đều yêu cầu dịch. Tên file hoặc vị trí cuối sách không đủ để coi một ngoại truyện là quảng cáo. Cơ chế cục bộ và nút Nhờ Research đề xuất theo mục 13.

Giữ nguyên có nghĩa nội dung vẫn tồn tại ở đúng vị trí và còn liên kết hợp lệ trong EPUB; không mang nhãn Đã dịch/Đạt Review. Nhãn mục lục và tiêu đề cùng phạm vi phải nhất quán. Một tài nguyên phụ trợ dùng chung cho phần truyện và trang phụ phải được nhận diện quan hệ để không bỏ sót phần cần dịch hoặc tạo bản dịch mâu thuẫn.

Giao diện hiển thị và lưu lựa chọn nội dung cho tập/tác vụ, cùng phiên bản mapping dùng để phân loại. Chế độ tự động xử lý đến hết nội dung cần dịch theo lựa chọn đó. Chọn một nhóm chương chỉ giới hạn lượt làm việc hiện tại, không biến các chương còn lại thành phần được phép giữ nguyên khi xuất.

Khi người dùng đổi lựa chọn nội dung, tính lại phần cần xử lý và điều kiện xuất; giữ các bản dịch/Review còn phù hợp, không xóa lịch sử hay sửa tay. Mở rộng từ chỉ truyện sang toàn bộ phần chữ phải bổ sung các trang phụ còn thiếu trước khi xuất theo phạm vi mới. Bộ preview/export cũ cần được chuẩn bị lại nếu lựa chọn thay đổi; đổi phạm vi không tự làm mới ngân sách hoặc bộ đếm lỗi của tác vụ đang tiếp tục.

Chốt ngày **04/11/2026**, bộ mới mặc định **Dịch toàn bộ phần chữ**. Người dùng đổi riêng cho tập hoặc bấm **Lưu làm mặc định của bộ** để áp dụng cho các tập tạo sau. Tập mới kế thừa mặc định của bộ. Tool lưu và hiển thị lựa chọn thực tế trước khi chạy. Cơ chế phân loại, xử lý phần chưa rõ và tài nguyên dùng chung theo mục 13.

## 2. Ba loại cấu trúc cần biểu diễn rõ

### 2.1. Cấu trúc file bên trong EPUB

EPUB là một gói ZIP có cấu trúc. Ví dụ minh họa:

```text
tap-01.epub
├── mimetype
├── META-INF/
│   └── container.xml
└── EPUB/
    ├── package.opf
    ├── nav.xhtml
    ├── text/
    │   ├── opening.xhtml
    │   ├── ch01.xhtml
    │   ├── ch02-a.xhtml
    │   ├── illustration.xhtml
    │   ├── ch02-b.xhtml
    │   └── afterword.xhtml
    ├── images/
    │   └── illustration-01.jpg
    ├── styles/
    │   └── book.css
    └── fonts/
        └── book-font.otf
```

Tên thư mục và file trong ví dụ chỉ để minh họa. Tool tìm package qua `META-INF/container.xml`, đọc danh sách tài nguyên trong OPF và lấy thứ tự đọc từ `spine`.

Một chương có thể trải qua nhiều file XHTML, có thể xen một file chứa ảnh. Một file XHTML cũng có thể chứa nhiều chương hoặc nhiều part. Số file, tên file và thứ tự file trong ZIP không đủ để xác định chương.

### 2.2. Cây nội dung người dùng thấy

Cây chính dùng chương để chọn phạm vi và theo dõi trạng thái:

```text
Bộ truyện
├── Tập 01
│   ├── Mở đầu        — Đã dịch
│   ├── Chương 1      — Hoàn tất
│   ├── Chương 2      — Đang dịch
│   ├── Chương 3      — Chưa dịch
│   └── Lời bạt       — Chưa dịch
└── Tập 02
    ├── Mở đầu
    ├── Chương 1
    └── …
```

Tên và các mục mở đầu/lời bạt lấy từ sách; không tự thêm những phần không có trong nguồn. Cấu trúc này là cây nội dung trong ứng dụng, không phải yêu cầu tạo một thư mục trên đĩa cho từng nút.

Khi mở chi tiết một chương **có part**, các tiêu đề part được giữ trong dòng nội dung:

```text
Chương 1
├── Tiêu đề: Part 1
├── Đoạn văn 001
├── Đoạn văn 002
├── Tiêu đề: Part 2
├── Đoạn văn 003
├── Ảnh minh họa
├── Đoạn văn 004
├── Tiêu đề: Part 3
└── Đoạn văn 005
```

Khi chương **không có part**:

```text
Chương 1
├── Đoạn văn 001
├── Đoạn văn 002
├── Ngắt cảnh
├── Đoạn văn 003
├── Ảnh minh họa
├── Đoạn văn 004
└── Đoạn văn 005
```

Điểm ngắt cảnh như dấu `＊＊＊` hoặc thẻ phân cách được giữ đúng vị trí. Tool có thể dùng nó làm điểm chia lượt dịch. Dấu ngắt cảnh tự nó chưa đủ để khẳng định sách có một part mới.

Mục lục lồng nhau trong EPUB được giữ nguyên thông tin. Những mục con được xác định là part sẽ trở thành dấu mốc trong chương, không bị nâng thành các chương độc lập chỉ vì có liên kết trong mục lục.

### 2.3. Các lượt xử lý bên trong chương

Kế hoạch dịch là một lớp dữ liệu riêng để chạy tác vụ:

```text
Tác vụ: Dịch Chương 1
├── Lượt 1 → một nhóm đoạn liên tiếp
├── Lượt 2 → nhóm đoạn tiếp theo
└── Lượt 3 → phần còn lại của chương
```

Số lượt phụ thuộc độ dài, model và ngữ cảnh cần gửi. Thay đổi cách chia lượt không làm thay đổi mục lục, tiêu đề part hay ID của các block nguồn đã được xác lập trong snapshot.

## 3. Luồng xử lý EPUB đầu vào

### 3.1. Nhận file và tạo snapshot nguồn

**Cập nhật được duyệt 14/11/2026:** TUI trong workspace có nút **＋ Nhập tập truyện** ở Tổng quan hoặc Tập và chương; chooser gốc Windows cho duyệt thư mục, chọn một `.epub`, rồi Mở/OK. Ứng dụng tự xác định/đăng ký Volume ID và đưa đường dẫn cùng ID vào core; người dùng không gõ path hay ID nội bộ trong luồng thông thường. CLI `import <epub> --volume <id>` và kiểm tra core giữ nguyên. Hủy/lỗi không công bố nguồn dở dang; không áp kết quả nhầm workspace khi đổi workspace hoặc resize. Adapter do P02-D06/T12 quyết định.

1. Nhận đường dẫn EPUB và tập đích trong workspace.
2. Kiểm tra đọc được gói ZIP, `mimetype`, container và package.
3. Kiểm tra đường dẫn tài nguyên, file trùng, tài nguyên thiếu và giới hạn giải nén trước khi sử dụng nội dung.
4. Tính hash nguồn và lưu snapshot cùng thông tin phiên bản parser.
5. Lưu kết quả kiểm kê và bản đồ nội dung để những bước sau dùng cùng một nguồn.

File EPUB người dùng đưa vào được giữ nguyên. Import lại cùng nguồn phải cho kết quả ổn định với cùng phiên bản parser. Nếu nguồn hoặc cách phân tích đã thay đổi, cần nhận diện revision nguồn và xử lý mapping trước khi tái sử dụng bản dịch cũ.

Trường hợp không đọc được package, nguồn mã hóa hoặc dạng nội dung chưa hỗ trợ phải có chẩn đoán rõ theo phạm vi ở mục 1.1 và được nghiệm thu theo mục 10. Sự có mặt của `encryption.xml` không tự chứng minh toàn bộ sách có DRM, vì tệp này còn có thể mô tả font obfuscation; phải xác định tài nguyên thực tế bị ảnh hưởng.

### 3.2. Đọc bản đồ tài nguyên và thứ tự đọc

Tool cần đọc:

| Thành phần | Dữ liệu cần lấy |
| --- | --- |
| Metadata | Tên sách, tác giả, ngôn ngữ và các thông tin nguồn cần bảo toàn. |
| Manifest trong OPF | Tài nguyên, đường dẫn, loại nội dung và thuộc tính liên quan. |
| Spine | Thứ tự đọc mặc định và thông tin nội dung linear/nonlinear. |
| Navigation/NCX | Tên mục, cấp mục lục, file đích và anchor. |
| XHTML | Tiêu đề, đoạn văn, dấu mốc nội dung, ảnh, chú thích và liên kết. |

Nội dung phụ trợ như mục lục, chú thích và tài liệu ngoài luồng đọc chính vẫn phải được kiểm kê. Những nội dung này cần được phân loại để xử lý đúng phạm vi, thay vì cộng toàn bộ thành chương truyện.

Kiểm kê cả trang bản quyền, giới thiệu tác giả và quảng cáo để áp dụng lựa chọn nội dung ở mục 1.2. Phân loại là trang phụ không tự có nghĩa được bỏ qua; ở chế độ toàn bộ phần chữ, những trang này vẫn phải có bản dịch và Review.

Metadata nguồn được giữ làm căn cứ điền sẵn thông tin sách theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md). Giá trị thiếu được ghi nhận để người dùng bổ sung, không suy đoán khi import. Tên sách/tác giả và các lựa chọn cho bản xuất được lưu riêng theo tập, không sửa metadata trong snapshot nguồn.

### 3.3. Nhận diện chương

Kết hợp `spine`, mục lục, anchor và cấu trúc tiêu đề để xác định phạm vi từng chương. Những file tiếp nối một chương được gom vào cùng chương theo thứ tự đọc.

Ví dụ:

```text
Các file theo spine                  Cấu trúc nội dung
ch02-a.xhtml      ─┐
illustration.xhtml ├───────────────→ Chương 2
ch02-b.xhtml      ─┘
```

Nếu nguồn thiếu mục lục hoặc các dấu mốc mâu thuẫn, tool hiển thị cấu trúc tạm cùng bằng chứng nhận diện để người dùng có thể chỉnh phạm vi. Phần chưa xác định rõ được đánh dấu cần kiểm tra trước khi dịch phần đó. Một lần điều chỉnh mapping phải được ghi nhận để không áp bản dịch đã lưu vào vị trí sai.

### 3.4. Bóc nội dung trong chương

| Loại nội dung | Cách xử lý |
| --- | --- |
| Tiêu đề chương/part | Trích văn bản, giữ cấp tiêu đề và vị trí. Tiêu đề part có thể là dấu mốc chia lượt. |
| Đoạn kể, lời thoại | Tạo block theo cấu trúc nguồn, giữ thứ tự và ID để dịch/review. |
| Nhấn mạnh, xuống dòng, liên kết | Giữ thông tin inline bằng mapping/marker có kiểm tra. |
| Ruby/furigana | Giữ chữ gốc, phần cách đọc và phạm vi liên quan để model/reviewer sử dụng. |
| Chú thích | Trích nội dung, giữ liên kết từ chỗ gọi đến chú thích và chiều quay lại nếu có. |
| Ảnh | Tách bản gốc vào `images/source/` ngay khi import, giữ nội dung tài nguyên và từng vị trí xuất hiện. Ảnh nằm giữa đoạn vẫn phải giữ mapping inline. |
| Chữ trong ảnh | Ghi nhận để xử lý trong workflow ảnh riêng; import văn bản không đồng nghĩa đã đọc được chữ trong ảnh. |
| CSS, font | Giữ tài nguyên nguồn và quan hệ tham chiếu; bản xuất được điều chỉnh bố cục cho tiếng Việt theo đặc tả đầu ra. |
| Nhãn mục lục, mô tả ảnh, metadata | Kiểm kê riêng, xác định phạm vi xử lý để không bỏ sót hoặc dịch lặp ngoài chủ ý. |
| Nội dung chưa phân loại | Ghi chẩn đoán và vị trí nguồn; không âm thầm loại bỏ. |

Một block dịch có thể là tiêu đề, đoạn văn, chú thích hoặc thuộc tính văn bản. Block không mặc nhiên tương ứng với một câu, một file XHTML hay một request.

Theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md), tiêu đề chương/part, nhãn mục lục và nội dung chú thích phải được theo dõi để Dịch–Review hoặc giữ nguyên đúng lựa chọn ở mục 1.2. Cùng một nội dung dùng nhiều nơi cần có quan hệ tham chiếu để tránh dịch lặp hoặc dùng tên khác nhau ngoài chủ ý; nhãn khác nhau có nghĩa riêng vẫn được giữ để đối chiếu. Import giữ cả chữ chính và phần chữ nhỏ của ruby, chưa tự quyết phần nào chỉ là phát âm hoặc được bỏ khỏi bản Việt.

### 3.5. Kết quả import

Sau import, ứng dụng có:

- snapshot EPUB và hash nguồn;
- kiểm kê tài nguyên, thứ tự đọc và các quan hệ liên kết;
- thư mục ảnh gốc `images/source/` cùng mapping về tài nguyên và các vị trí sử dụng;
- cây chương và danh sách nội dung trong từng chương;
- bản đồ từ block về vị trí trong nguồn;
- chẩn đoán nội dung thiếu, chưa hỗ trợ hoặc cần người dùng kiểm tra;
- dữ liệu để lập kế hoạch dịch theo model đã chọn.

Có thể hiển thị số lượt dịch dự kiến khi đã có thông tin model và cấu hình dung lượng. Số này là ước lượng: ngữ cảnh, review, retry hoặc đổi model có thể làm kế hoạch thay đổi. Nếu chưa đủ thông tin, hiển thị chưa xác định thay vì đưa một con số chắc chắn.

## 4. Mapping nguồn và cách lưu dữ liệu

### 4.1. Thông tin gắn với một block

| Thông tin | Mục đích |
| --- | --- |
| ID block | Gắn bản dịch, review và sửa tay với đúng nội dung. |
| Tập/chương, thứ tự | Dựng cây, chọn phạm vi và giữ thứ tự truyện. |
| Đường dẫn XHTML và locator/anchor | Tìm lại vị trí trong EPUB để ghép bản dịch. |
| Nguyên văn và hash nguồn | Đối chiếu, phát hiện thay đổi và kiểm tra dữ liệu cũ. |
| Loại nội dung và trạng thái xử lý | Phân biệt dịch, bảo toàn tài nguyên, dấu mốc và phần chưa giải quyết. |
| Inline token, ruby, liên kết | Giữ quan hệ định dạng và tham chiếu. |
| Mapping con khi chia một block | Ghi phạm vi nguồn của từng phần và thứ tự ghép lại. |

ID phải ổn định khi import cùng snapshot bằng cùng parser. EPUB thay đổi tạo snapshot mới. Chỉ tái sử dụng qua snapshot sau khi khớp duy nhất vị trí, nội dung, tài nguyên được bảo vệ và kiểm tra dependency theo mục 13.

### 4.2. Cây thư mục workspace

Cây dưới đây minh họa các nhóm dữ liệu của workspace và thư mục ảnh chỉnh theo [đặc tả xử lý ảnh](IMAGE_EDIT_SPEC.md). Việc tách ảnh gốc được chốt thực hiện ngay khi import; đây là yêu cầu thiết kế, không khẳng định hành vi đã có trong mọi phiên bản. Tên chương trong ví dụ dùng ID, còn tên hiển thị được lưu trong dữ liệu sách. Theo quyết định ngày **30/10/2026** tại [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), bản Rust bắt đầu bằng workspace mới, không cần giữ/chuyển workspace test cũ; workspace dùng một SQLite cho bộ và các file độc lập theo mục 13 và đặc tả dữ liệu. Các tên file nội bộ trong cây là minh họa bố trí, còn vai trò lưu trữ đã chốt.

```text
workspace/bo-truyen/
├── workspace.yaml                  # Cấu hình không bí mật
├── Skills/
│   └── SKILL.md                     # Quy ước riêng theo mẫu
├── vol01_translation/
│   ├── chapters/
│   │   ├── <chapter-id-1>.md         # Bản thảo có unit markers
│   │   └── <chapter-id-2>.md
│   └── images/
│       ├── source/                  # Ảnh gốc tách ngay khi import
│       └── edit/                    # Ảnh người dùng chỉnh
├── vol02_translation/
│   └── …
├── Output/                          # EPUB đã xuất, có phiên bản
└── .translation_work/
    ├── workspace.sqlite             # Một database cho cả bộ
    ├── writer.lock
    ├── source/
    │   └── <source-hash>/
    │       ├── source.epub
    │       └── manifest-<digest>.json
    ├── objects/                     # Nội dung bất biến theo hash
    └── prepared/
        └── <publication-hash>/      # Manifest/tài nguyên preview–export
```

Đây là cây rút gọn sau nhiều bước làm việc. SQLite lưu mapping, revision, Review, generation, con trỏ hiện hành, tác vụ và usage. Nguồn, nội dung bất biến, bộ chuẩn bị, ảnh và Markdown là file độc lập có tham chiếu/hash. Hàng đợi chung thuộc hồ sơ người dùng ngoài từng workspace. Kiểm kê snapshot do tool tạo khác với manifest trong OPF.

Ảnh trong `source/` giữ nguyên để đối chiếu. Khi chuẩn bị xem trước hoặc xuất EPUB, file hợp lệ trong `edit/` có cùng đường dẫn tương đối và tên file sẽ tự thay ảnh nguồn tương ứng; không có file thay thì dùng ảnh gốc. Không cần tách EPUB thêm lần nữa ở bước xử lý ảnh. Quy tắc kiểm tra, xem trước và đồng bộ với bản xuất theo [IMAGE_EDIT_SPEC.md](IMAGE_EDIT_SPEC.md).

Nguồn của một tập có thể nằm ngoài workspace; snapshot được lưu trong vùng dữ liệu nội bộ, minh họa bằng `.translation_work`. Cấu hình cho phép thay đổi tên thư mục bản thảo và thư mục xuất; cấu hình có hiệu lực được lưu theo bộ/tập/tác vụ và tham chiếu trong SQLite.

Part và lượt dịch được biểu diễn trong cây nội dung và dữ liệu tác vụ. Bản thảo vẫn tổ chức theo chương. Hợp đồng request nhiều block theo mục 20 của [36 quyết định đã chốt](DESIGN_DECISIONS_36.md#decision-20), dùng chung ID/phạm vi nguồn với Review và Export.

EPUB, workspace, `.env`, ảnh nguồn và đầu ra sinh ra tiếp tục nằm ngoài Git. Tài liệu này không chứa nội dung sách hay khóa thật.

Theo lựa chọn **11A** ngày **30/10/2026** trong [đặc tả key và preview](SECURITY_SPEC.md), workspace mới chỉ giữ tham chiếu kết nối, không lưu giá trị key hoặc mặc định tạo `.env`. Key nằm trong kho thông tin xác thực của hệ điều hành hoặc biến môi trường được người dùng chọn cho CLI. Quy tắc bỏ `.env` khỏi Git vẫn áp dụng nếu người dùng có file riêng.

## 5. Xác định chương dài bằng hai giới hạn

### 5.1. Giới hạn kỹ thuật của một request

Một request phải đáp ứng giới hạn của đúng provider/model được chọn. Cần tính toàn bộ đầu vào thực gửi:

- hướng dẫn dịch và định dạng trả về;
- các block nguồn được giao dịch, ID và thông tin inline/ruby;
- glossary, quy ước, thông tin cảnh và ngữ cảnh liên quan;
- đoạn nguồn/bản dịch trước đó dùng để tham khảo;
- message/schema/tool overhead mà API tính vào token.

Đồng thời phải dành đủ dung lượng dự kiến cho bản dịch tiếng Việt, cấu trúc trả về và phần suy luận nếu model tính chúng vào cùng hạn mức. Không được mặc định số token tiếng Việt bằng số token tiếng Nhật.

Ký hiệu để diễn đạt quy tắc:

| Ký hiệu | Ý nghĩa |
| --- | --- |
| `T_in` | Token của toàn bộ đầu vào request đã lắp ráp. |
| `I_max` | Giới hạn đầu vào riêng, nếu provider/model có quy định. |
| `O_reserved` | Ngân sách đầu ra dành cho request theo cách tính của provider. |
| `O_max` | Giới hạn đầu ra hiệu lực sau khi xét model và cấu hình người dùng. |
| `C` | Context chung, nếu provider/model dùng ngân sách đầu vào + đầu ra chung. |
| `M` | Khoảng dự phòng cho sai số ước lượng và phần overhead liên quan. |

Các điều kiện phải được áp dụng theo đặc tả của model:

```text
T_in <= I_max                         nếu có giới hạn đầu vào riêng
O_reserved <= O_max
T_in + O_reserved + M <= C            nếu dùng context chung
```

Nếu reasoning được tính trong ngân sách đầu ra/context, adapter phải dành chỗ đúng cách và tránh cộng trùng. Nếu provider có cách tính khác, adapter áp quy tắc tương ứng. Cấu hình giới hạn do người dùng đặt thấp hơn giới hạn model vẫn có hiệu lực.

Trước khi gửi, cần đếm hoặc ước lượng token phù hợp với model; khi có bộ đếm phù hợp thì dùng kết quả đó. Nếu chỉ ước lượng được, phải ghi rõ và dùng dự phòng thận trọng. Byte UTF-8, số ký tự và token là các đại lượng khác nhau.

Nếu thiếu thông tin giới hạn đáng tin cậy, trạng thái “vừa dung lượng” chưa được xác nhận. Cần cấu hình giới hạn phù hợp hoặc áp một profile bảo thủ đã được xác lập trước khi chạy. Khi người dùng chủ động đổi model, phải kiểm tra lại phần công việc chưa hoàn thành. Theo [chiến lược model](MODEL_STRATEGY_SPEC.md) chốt ngày 28/10/2026, tool không tự chuyển model dự phòng khi gặp lỗi.

### 5.2. Giới hạn vận hành của phần nguồn cần dịch

Ngoài sức chứa kỹ thuật, đặt mức nguồn tối đa cho một lượt để dễ review, lưu tiến độ và xử lý lỗi.

**Mốc thử nghiệm ban đầu: khoảng 3.000 token nguồn/lượt.**

Mốc này:

- chỉ tính phần nguồn được giao dịch trong lượt, tách khỏi ngữ cảnh tham khảo;
- là tham số có thể điều chỉnh, không phải con số đóng cứng trong thuật toán;
- chưa phải kết quả nghiệm thu chất lượng hay ngưỡng tối ưu cho mọi model;
- vẫn chịu giới hạn kỹ thuật, nên có trường hợp phải chia nhỏ hơn 3.000 token;
- có thể kết thúc sớm hơn mức mục tiêu để giữ ranh giới cảnh/đoạn có nghĩa.

Khi nghiệm thu, so sánh độ đầy đủ, nghĩa, giọng kể, xưng hô, thuật ngữ, số lần sửa, thời gian và token sử dụng. Việc tăng kích thước cần dựa trên kết quả này.

Cấu hình Rust cho giới hạn nguồn phải thể hiện rõ đơn vị token và được kiểm tra theo hợp đồng bộ lập lượt tại mục 13. `max_input_bytes` của bản Go vẫn là đơn vị byte. Tên khóa trong mã/cấu hình được hiện thực theo hợp đồng mới, không bắt buộc tiếp nhận khóa hoặc workspace test cũ theo quyết định ngày **30/10/2026**.

### 5.3. Khi nào một chương được chia nhỏ

Thử lập request chứa toàn bộ nội dung cần dịch của chương cùng ngữ cảnh liên quan. Chương cần chia khi:

1. Request đó vượt một điều kiện kỹ thuật; **hoặc**
2. Phần nguồn được giao dịch vượt giới hạn vận hành.

Nếu cả hai đều đạt, chương có thể được dịch trong một lượt. Quy tắc cũng áp dụng cho mở đầu, lời bạt hoặc phần nội dung tương đương được chọn dịch.

Chương nhiều token là một nguyên nhân phải chia; ngữ cảnh quá lớn hoặc dung lượng đầu ra hạn chế cũng có thể làm một chương ngắn cần nhiều lượt. Mỗi request sau khi chia đều phải được kiểm tra lại cả hai giới hạn.

## 6. Thuật toán chia lượt theo thứ tự nội dung

### 6.1. Chuẩn bị danh sách nguồn

Từ chương đã nhận diện, lấy các block theo thứ tự đọc. Đính kèm dấu mốc part/cảnh và mapping tài nguyên, chú thích, ruby, inline token.

Nội dung cần dịch có phạm vi sở hữu rõ ràng trong từng lượt. Đoạn đưa vào làm ngữ cảnh được đánh dấu chỉ để tham khảo và không thuộc phần phải trả bản dịch.

### 6.2. Gom block và chọn ranh giới

1. Bắt đầu ở block chưa hoàn thành đầu tiên của chương.
2. Chuẩn bị hướng dẫn, glossary và ngữ cảnh liên quan trong ngân sách cho phép.
3. Thêm lần lượt các block liên tiếp vào request ứng viên.
4. Sau mỗi lần thêm, tính lại dung lượng đầu vào, đầu ra dự trù và số token nguồn được giao.
5. Khi thêm block tiếp theo sẽ vượt giới hạn, chọn điểm kết thúc phù hợp trong phần đã vừa dung lượng.
6. Ưu tiên ranh giới part hoặc ngắt cảnh gần cuối phần đang xét; nếu không có điểm phù hợp, dùng cuối đoạn văn.
7. Chốt phạm vi lượt, kiểm tra lại request hoàn chỉnh và lưu mapping cần thiết.
8. Tiếp tục từ đúng vị trí kế tiếp cho đến hết chương.

Điểm chia cần giữ mạch kể và hội thoại khi có thể. Không chia đều theo số ký tự hoặc số trang. Quy tắc chọn một dấu mốc “gần cuối” cần được hiệu chỉnh khi triển khai để tránh tạo các lượt quá nhỏ chỉ vì có một ngắt cảnh ở rất xa giới hạn.

### 6.3. Một block riêng lẻ đã quá dài

Nếu một đoạn không vừa giới hạn ngay cả khi đứng riêng, chia tiếp ở cuối câu và lưu các phạm vi con của block đó.

Điểm chia phải giữ hợp lệ:

- ký tự Unicode và ranh giới văn bản;
- marker/inline token cùng cặp cần bảo toàn;
- ruby và phần chữ được ruby chú thích;
- vị trí ảnh, chú thích và liên kết liên quan.

Nếu một thành phần không thể tách vẫn vượt dung lượng, báo rõ vị trí và nguyên nhân để điều chỉnh cấu hình hoặc xử lý riêng. Không cắt hỏng thành phần để ép request vừa giới hạn.

Các phần con phải phủ đủ nguồn của block, không trùng hoặc bỏ đoạn. Khi ghép bản dịch, giữ đúng thứ tự các phần con và gắn kết quả với block gốc.

### 6.4. Giữ ngữ cảnh giữa các lượt

Mỗi lượt có thể nhận:

- glossary và cách gọi tên liên quan đến nội dung đang dịch;
- quy ước xưng hô có hiệu lực tại thời điểm tương ứng trong truyện;
- thông tin người nói/người nghe, cảnh và các sự kiện trước đó cần thiết;
- một đoạn nguồn/bản dịch đã duyệt ngay trước phần đang xử lý;
- bằng chứng nguồn liên quan khi cần giải nghĩa câu hoặc thuật ngữ.

Lượng ngữ cảnh phải có giới hạn và được tính vào `T_in`. Gửi lại toàn bộ lịch sử sẽ làm request tăng dần, nên chỉ chọn thông tin cần thiết. Tóm tắt ngữ cảnh không thay thế nguyên văn của phần được giao dịch hoặc bằng chứng cần đối chiếu.

Model chỉ trả bản dịch cho ID/phạm vi đang được giao. Phần tham khảo không được ghi thêm vào bản dịch đầu ra. Thông tin xưng hô hoặc sự kiện ở thời điểm sau trong truyện không được tự áp ngược vào cảnh trước.

## 7. Ví dụ chia chương

### 7.1. Chương khoảng 8.000 token nguồn

Giả sử giới hạn vận hành đang đặt ở 3.000 token nguồn/lượt và các request dưới đây đều đã qua kiểm tra kỹ thuật:

```text
Chương 1 — khoảng 8.000 token nguồn
├── Lượt 1: đoạn 001–020 — khoảng 2.800 token
│   └── Kết thúc ở chỗ ngắt cảnh
├── Lượt 2: đoạn 021–041 — khoảng 2.900 token
│   └── Kết thúc ở cuối đoạn văn
└── Lượt 3: đoạn 042–060 — khoảng 2.300 token
    └── Kết thúc chương
```

Số đoạn và số token là minh họa. Kích thước thực tế được đo trên nguồn và request cụ thể. Ngữ cảnh tham khảo của lượt 2 hoặc 3 được gửi thêm trong ngân sách kỹ thuật, không được cộng thành nội dung cần dịch lại.

### 7.2. Cả hai giới hạn cùng tác động

Ví dụ giả định, không đại diện cho thông số một model cụ thể:

| Đại lượng | Giá trị minh họa |
| --- | --- |
| Context chung `C` | 16.000 token |
| Ngân sách đầu ra `O_reserved` | 6.000 token, nằm trong giới hạn đầu ra hiệu lực |
| Dự phòng `M` | 1.000 token |
| Hướng dẫn và ngữ cảnh | 2.000 token cho request đang xét |
| Nguồn của chương | 8.000 token |
| Giới hạn vận hành | 3.000 token nguồn/lượt |

Gửi cả chương cần khoảng `2.000 + 8.000 + 6.000 + 1.000 = 17.000` token theo cách tính giả định này, vượt context. Phần nguồn 8.000 token cũng vượt giới hạn vận hành.

Một lượt 3.000 token nguồn cần khoảng `2.000 + 3.000 + 6.000 + 1.000 = 12.000` token, nằm trong context giả định. Cần kiểm tra giới hạn đầu vào riêng nếu có và tính lại ngữ cảnh cho từng lượt. Ngân sách đầu ra 6.000 token chỉ là giá trị minh họa đã dự trù; không phải quy luật bản dịch tiếng Việt dài gấp đôi nguồn.

### 7.3. Chương có part và chương không có part

| Tình huống | Hành vi |
| --- | --- |
| Part vừa dung lượng | Có thể dùng điểm kết thúc part làm ranh giới lượt. |
| Part dài | Chia bên trong part ở ngắt cảnh, cuối đoạn hoặc cuối câu theo quy tắc. |
| Nhiều part rất ngắn | Có thể gom nội dung liên tiếp nếu hợp ngữ cảnh và vừa cả hai giới hạn. |
| Chương không có part | Gom đoạn liên tiếp, tận dụng ngắt cảnh nếu có. |
| Chương rất ngắn | Có thể dùng một lượt cho toàn chương. |

## 8. Lưu tiến độ, review và xử lý gián đoạn

### 8.1. Sau mỗi lượt

1. Kiểm tra trạng thái kết thúc request và cấu trúc kết quả.
2. Đối chiếu ID/phạm vi nguồn được giao, marker và các thành phần cần bảo toàn.
3. Lưu kết quả hợp lệ cùng dependency, model và revision liên quan.
4. Review phần đã dịch; nếu chưa đạt, xử lý đúng phạm vi bị lỗi theo chính sách review.
5. Cập nhật tiến độ chương từ dữ liệu đã lưu.

Chỉ chuyển sang lượt dịch kế tiếp khi lượt hiện tại đã đáp ứng điều kiện Review. Khi hết các lượt, kiểm tra tổng thể chương trước khi chuyển chương theo [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md).

Đủ ID là kiểm tra độ phủ cấu trúc, chưa chứng minh bản dịch đủ nghĩa. Review vẫn cần đối chiếu nội dung Nhật–Việt. Một chương chỉ hoàn tất khi các nội dung bắt buộc đã được xử lý và đạt điều kiện review, bao gồm kiểm tra nhất quán giữa các lượt.

Request review cũng phải nằm trong ngân sách của model review. Chương dài có thể được review theo phạm vi và kiểm tra các điểm nối; không mặc định gửi nguyên chương dài vào một request review.

### 8.2. Khi bị cắt đầu ra hoặc không vừa request

Nếu provider báo dừng vì giới hạn token, kết quả thiếu ID, thiếu phần con hoặc không khép được cấu trúc, lượt đó chưa được đánh dấu hoàn thành.

Cần lưu trạng thái lỗi để có thể kiểm tra, rồi tính lại phạm vi chưa hoàn thành. Có thể tăng phần đầu ra trong giới hạn hiệu lực hoặc giảm nguồn mỗi lượt theo chính sách cấu hình. Nếu chia lại, phải kiểm tra coverage và mapping để không ghi lặp phần đã hoàn tất.

Giới hạn token là lý do thay đổi cách chia. Lỗi mạng tạm thời cần đi theo retry policy có giới hạn; không phải mọi lỗi mạng đều là bằng chứng đoạn nguồn quá dài.

### 8.3. Pause, dừng và resume

Pause bình thường có hiệu lực tại ranh giới công việc đã được lưu an toàn. Nếu bị dừng đột ngột, kết quả đã commit vẫn được giữ; phần đang chạy chưa có kết quả hợp lệ không được coi là hoàn thành.

Khi resume:

- đọc trạng thái đã lưu và xác định phần còn thiếu;
- kiểm tra nguồn, mapping và dependency vẫn phù hợp;
- dùng lại kết quả còn hợp lệ, xử lý phần chưa hoàn thành hoặc đã stale;
- kiểm tra lại ngân sách nếu model/cấu hình đã đổi;
- giữ bản sửa tay và lịch sử revision theo cơ chế hiện có.

Mapping một request chứa nhiều block, cách lưu kế hoạch và checkpoint phải được xác định rõ trong schema workspace mới. Không cần migration từ workspace test cũ; dữ liệu của workspace mới vẫn phải được lưu/phục hồi nhất quán và có chính sách nâng cấp về sau theo [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md). Việc cập nhật tài liệu không tự chuyển hoặc xóa workspace trên máy.

### 8.4. Trạng thái trên giao diện

Người dùng chọn chương và thấy các trạng thái như chưa dịch, đang dịch, đang review, cần xử lý hoặc hoàn tất. Có thể mở chi tiết để xem lượt đang chạy và đoạn lỗi.

Tiến độ dịch và tiến độ review cần phân biệt. Chỉ ghi phần trăm khi có cơ sở tính ổn định, chẳng hạn phạm vi nguồn đã được xử lý so với toàn bộ phạm vi cần xử lý. Số lượt có thể thay đổi khi chia lại, nên số request hoàn tất không tự tương đương với phần trăm chương đã hoàn tất.

## 9. Dựng EPUB sau khi dịch

Bản dịch được ghép về đúng vị trí nhờ mapping block, file và locator trong snapshot nguồn. Giữ thứ tự chương, dấu mốc part, vị trí ảnh, chú thích và liên kết.

Quyết định ngày **30/10/2026** trong [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md): nguồn EPUB 2 và EPUB 3 đều xuất thành EPUB 3. Core dựng/chuyển các cấu trúc cần thiết trên bản chuẩn bị, kiểm tra đầu ra thực tế; không chỉ đổi nhãn phiên bản hoặc sửa snapshot nguồn. Nội dung trang phụ được giữ nguyên theo chế độ chỉ truyện vẫn phải nằm trong cấu trúc EPUB 3 hợp lệ.

Áp dụng [đặc tả EPUB đầu ra](EPUB_OUTPUT_SPEC.md): phần chữ Việt đọc ngang từ trái sang phải; giữ ý nghĩa định dạng nhấn mạnh, dùng tiêu đề/mục lục/chú thích đã qua Dịch–Review trong phạm vi được dịch, giữ các phần ngoài phạm vi đúng lựa chọn ở mục 1.2. Với phần dịch, ruby thuần phát âm mặc định không hiện lại chú âm sau khi đã được xác định; ruby có ý nghĩa riêng phải có cách thể hiện do người dùng chốt trước bước xuất. CSS/XHTML liên quan được cập nhật cho bố cục và bản dịch, không hiểu bảo toàn nguồn là giữ nguyên từng byte ở đầu ra.

Ảnh được chọn tự động từ `images/source/` và `images/edit/` theo [đặc tả xử lý ảnh](IMAGE_EDIT_SPEC.md). Bản xem trước tùy chọn hiển thị chữ đã dịch cùng ảnh đã thay trong chương; preview và EPUB đầu ra dùng cùng bộ dữ liệu chuẩn bị hợp lệ. Ảnh thay bị hỏng phải được báo và xử lý, không âm thầm quay về ảnh gốc.

Các lượt dịch chỉ là cách thực hiện công việc. Chúng không tạo thêm tiêu đề, mục lục hoặc chương trong EPUB đầu ra. Bản thảo Markdown là một dạng xem/chỉnh sửa; không dùng riêng Markdown để thay thế toàn bộ package và tài nguyên gốc.

Chuẩn bị và đóng gói chỉ dùng nội dung/quyết định đã lưu, không tự dịch thêm hoặc bỏ nội dung còn vướng. Sau ghép chữ, áp dụng bố cục và chọn ảnh, kiểm tra cấu trúc, tài nguyên, readback và EPUBCheck. Mở trình sửa XML không phải bước bắt buộc của luồng thay ảnh/xuất đã chốt.

Mỗi lần xuất mới tạo file có số phiên bản tăng dần theo tập trong thư mục đầu ra của workspace (`Output/` theo bố trí hiện có). Giữ file cũ, không ghi đè đường dẫn đã tồn tại; lưu hồ sơ để truy nguồn/revision/ảnh/phạm vi của từng bản xuất. Chi tiết tên và quản lý phiên bản theo đặc tả đầu ra.

## 10. Tiêu chí nghiệm thu

Các fixture và kiểm thử tự động phải offline, deterministic, không gọi provider trả phí. Nội dung mẫu cần tự tạo hoặc có quyền sử dụng phù hợp.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| EPUB 2 dạng chữ, có NCX | Đọc đúng thứ tự và ranh giới nội dung từ package, spine và mục lục. |
| EPUB 3 dạng chữ, có navigation và có thể kèm NCX | Đọc đúng cây nội dung; phát hiện các dấu mốc mâu thuẫn thay vì tự bỏ qua. |
| EPUB reflowable xen bìa/trang tranh fixed-layout | Chấp nhận trong phạm vi ban đầu; giữ vị trí và bố cục trang ảnh khi ghép bản dịch. |
| Văn bản Nhật trình bày dọc, có ruby/furigana | Trích đủ chữ gốc và chú thích theo đúng quan hệ; không coi cả sách là ảnh scan vì chữ hiển thị dọc. |
| Nội dung ngoài phạm vi hỗ trợ ban đầu | Chỉ ra tài nguyên/phạm vi bị vướng; không tính nội dung bị bỏ sót là đã hoàn tất và không xuất cả tập như bản đầy đủ. |
| Có `encryption.xml` chỉ mô tả font obfuscation | Không tự kết luận toàn bộ nội dung bị khóa; kiểm tra đúng tài nguyên bị ảnh hưởng. |
| EPUB có chapter và part | Chương được nhận diện đúng; tiêu đề part và thứ tự nội dung được giữ. |
| EPUB không có part | Import và chia lượt vẫn hoạt động; không sinh part giả. |
| Một chương nhiều XHTML, có trang ảnh xen giữa | Nội dung và ảnh được gắn đúng chương, đúng thứ tự. |
| Một XHTML chứa nhiều chương | Các chương được tách theo dấu mốc hợp lệ. |
| Mục lục lồng nhau | Mục con không bị mặc định nâng thành chương. |
| Mục lục thiếu hoặc mâu thuẫn | Có chẩn đoán và khả năng xem/chỉnh mapping. |
| Chương vừa cả hai giới hạn | Có thể lập một request dịch toàn chương. |
| Chương vượt giới hạn vận hành nhưng vẫn vừa model | Vẫn được chia theo giới hạn vận hành. |
| Nguồn ngắn nhưng ngữ cảnh/đầu ra làm vượt giới hạn kỹ thuật | Request được lập lại hoặc báo thiếu ngân sách; không gửi vượt giới hạn đã biết. |
| Một block quá dài | Chia ở ranh giới hợp lệ, bảo toàn Unicode, ruby và inline mapping. |
| Một thành phần không thể tách quá lớn | Báo đúng vị trí và lý do, không làm hỏng nguồn. |
| Nguồn được chia thành nhiều lượt | Các phạm vi sở hữu phủ nguồn đúng một lần và giữ thứ tự. |
| Có ngữ cảnh chồng giữa các lượt | Nội dung tham khảo không bị ghi lặp vào bản dịch. |
| Đầu ra bị cắt hoặc thiếu đoạn | Lượt/chương chưa được đánh dấu hoàn tất. |
| Gián đoạn rồi resume | Dùng lại kết quả hợp lệ đã lưu; tiếp tục phần còn thiếu. |
| Đổi model, cấu hình hoặc nguồn | Kiểm tra lại ngân sách và dependency/mapping phù hợp. |
| Ghép và xuất EPUB | Chunks không xuất hiện như cấu trúc mới của sách; ảnh và liên kết giữ đúng vị trí. |
| Tiêu đề, nhãn mục lục, chú thích hoặc ruby ngoài đoạn truyện chính | Kiểm kê đầy đủ và giữ mapping để Dịch–Review; không đợi đến lúc xuất mới tự dịch hoặc loại bỏ. |
| Chọn dịch toàn bộ phần chữ | Trang bản quyền, giới thiệu tác giả, quảng cáo và văn bản ngoài ảnh đều được đưa vào phạm vi cần xử lý. |
| Chọn chỉ dịch nội dung truyện | Nội dung truyện và phần liên quan được dịch; trang phụ giữ nguyên vẫn có mặt, không mang nhãn đã dịch hoặc bị bỏ khỏi sách. |
| Phần chưa phân loại, ngoại truyện và quảng cáo dễ nhầm | Giữ chẩn đoán và xác minh loại; không tự giảm phạm vi để báo hoàn tất. |
| Đổi lựa chọn nội dung | Tính lại độ phủ/điều kiện xuất, giữ kết quả hợp lệ và làm mới bộ chuẩn bị liên quan. |
| Xuất từ nguồn EPUB 2 hoặc EPUB 3 | Cùng tạo EPUB 3 hợp lệ và file phiên bản mới, giữ snapshot nguồn và các bản đã xuất trước đó. |

Chất lượng dịch và mức token vận hành phù hợp cần được nghiệm thu riêng bằng model thực tế trên mẫu đại diện. Kiểm thử cấu trúc không thay thế kiểm tra chất lượng Nhật–Việt.

## 11. Quan hệ với implementation hiện tại

Go hiện có reader EPUB, snapshot nguồn, chương/block, generation, bản thảo có unit markers và cơ chế xử lý block dài. Runner đang dịch theo block với cơ chế chia nhỏ block dài; tài liệu này chốt hướng lập request theo ngân sách token cho nội dung trong chương ở bản Rust, không khẳng định cơ chế gom nhiều block đã có sẵn.

Trong quá trình port:

- giữ đường chạy Go hoạt động đến khi phần thay thế đã được kiểm thử;
- mô tả rõ schema workspace mới và việc không hỗ trợ chuyển dữ liệu test cũ trong PR liên quan; giữ dữ liệu/sửa tay/lịch sử và thiết kế nâng cấp cho workspace mới;
- chưa đổi ngầm `max_input_bytes` thành token;
- tiếp tục giữ `Cargo.lock` vì sản phẩm là application binary;
- cập nhật README khi command, bố trí file thực tế hoặc nền tảng được hỗ trợ thay đổi.

Bộ đếm/ước lượng token, dự trù đầu ra/reasoning, ưu tiên ranh giới và định danh kế hoạch đã chốt tại mục 13. Hợp đồng request nhiều block theo [đặc tả Dịch](TRANSLATION_STAGE_SPEC.md), checkpoint theo [điều phối](WORKFLOW_ORCHESTRATION_SPEC.md). Plan triển khai hiện thực các hợp đồng này và đo các mặc định thử nghiệm bằng bộ nghiệm thu đã chốt.

## 12. Nguồn đối chiếu

- [OPF 2.0.1 — IDPF](https://idpf.org/epub/20/spec/OPF_2.0_latest.htm): package, manifest, spine và mục lục NCX của EPUB 2.
- [EPUB 3.3 — W3C](https://www.w3.org/TR/epub-33/): cấu trúc container, package, manifest, spine và navigation.
- [Stop reasons and fallback — Claude API](https://platform.claude.com/docs/en/build-with-claude/handling-stop-reasons): ví dụ provider phân biệt dừng vì giới hạn token/context với kết thúc thông thường.
- [README](../../README.md): luồng sử dụng và bố trí workspace hiện có.
- [EPUB reader](../../internal/translation/epub/reader.go), [domain types](../../internal/translation/domain/types.go): cấu trúc nguồn và mapping.
- [Import nguồn](../../internal/translation/engine/source.go), [snapshot/revision nguồn](../../internal/translation/engine/source_revision.go): snapshot, hash và con trỏ nguồn.
- [State store](../../internal/translation/state/store.go), [bản thảo chương](../../internal/translation/revision/edit.go): generation và đường dẫn dữ liệu.
- [Runner](../../internal/translation/engine/runner.go), [chia block](../../internal/translation/engine/split.go), [cấu hình workspace](../../internal/translation/workspace/config.go): cơ chế hiện có cần đối chiếu khi triển khai.

## 13. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [01](DESIGN_DECISIONS_36.md#decision-01), [02](DESIGN_DECISIONS_36.md#decision-02), [03](DESIGN_DECISIONS_36.md#decision-03), [04](DESIGN_DECISIONS_36.md#decision-04), [05](DESIGN_DECISIONS_36.md#decision-05) của bản 36 mục áp dụng cho đặc tả này.

### 13.1. Lựa chọn nội dung dịch mặc định — mục 01

Đã chốt **01A**. Bộ mới dùng **Dịch toàn bộ phần chữ**. Màn chuẩn bị cho đổi sang **Chỉ dịch nội dung truyện và phần liên quan**. Một thay đổi áp dụng cho tập đang mở. Nút **Lưu làm mặc định của bộ** mới đổi giá trị cho các tập tạo sau. Tập 2 lấy mặc định của bộ dù tập 1 đã được chỉnh riêng. Lưu lựa chọn thực tế vào tập/tác vụ và hiển thị trước lần dịch đầu. Chữ trong bitmap đi qua luồng ảnh.

### 13.2. Phân loại nội dung trong EPUB — mục 02

Đã chốt **02A**. Import cục bộ lập cây từ cấu trúc sách, mục lục, tiêu đề, vị trí và liên kết. Lưu riêng loại phần và cách xử lý theo chế độ. Ví dụ, đoạn chữ đọc được ở cuối sách vẫn cần dịch ở chế độ toàn bộ chữ dù chưa rõ đó là ngoại truyện hay quảng cáo. Ở chế độ chỉ truyện, nếu hai cách phân loại dẫn đến hai kết quả khác nhau, trang vào **Cần phân loại**.

Trên cây, người dùng xem đoạn nguồn, các dấu hiệu và các nơi tham chiếu, rồi chọn loại trang. Nút **Nhờ Research đề xuất** tạo yêu cầu có chủ đích sau import, có căn cứ và chịu ngân sách. Người dùng chốt kết quả. Lưu quyết định cùng hash EPUB, phiên bản mapping và vị trí. Dùng lại khi cùng nguồn/mapping, đối chiếu lại khi một trong hai đổi. Chú thích dùng chung với phần truyện được dịch một lần, giữ các liên kết từ cả hai nơi. Nhãn chỉ thuộc trang phụ theo chế độ của trang đó.

### 13.3. Nhận diện chương và chỉnh sửa mapping — mục 03

Spine xác định thứ tự đọc. Trong thứ tự đó, ưu tiên ranh giới người dùng đã xác nhận cho đúng nguồn, tiếp đến là đích mục lục Nav/NCX hợp lệ, rồi heading và dấu ngắt trong XHTML. Ranh giới file chỉ làm gợi ý cuối. Đích mục lục hỏng hoặc mâu thuẫn có ý nghĩa với heading tạo phạm vi tạm kèm hai dấu hiệu để người dùng đối chiếu. Part vẫn là tiêu đề trong chương.

Cho **gộp, tách, dời ranh giới và đổi tên hiển thị** trên cây. Trước khi lưu, kiểm tra phạm vi có đúng thứ tự và phủ đủ nguồn. Ghi phiên bản mapping mới. Giữ bản dịch của đơn vị nguồn khớp chắc chắn và quy tắc còn hiệu lực. Xét lại kết luận toàn chương và ngữ cảnh ở những điểm nối đã đổi.

### 13.4. Định danh block và phạm vi con — mục 04

ID block là SHA-256 của một bản ghi có phiên bản gồm hash EPUB, phiên bản parser, đường dẫn XHTML đã chuẩn hóa và vị trí node ổn định trong tài liệu. Ưu tiên ID XHTML duy nhất, còn lại dùng đường đi node và thứ tự xuất hiện. Hai đoạn giống hệt chữ vẫn có ID riêng. Tên chương và số thứ tự lượt dịch được lưu riêng.

Đánh số **các cụm ký tự hiển thị và phần tử inline được bảo vệ** trong block. Một phạm vi con mang block ID và khoảng nửa mở `[đầu, cuối)`, kèm bảng ánh xạ về node/cụm chữ gốc. Ruby, ảnh và cặp marker có quy tắc ranh giới riêng. Lưu độ phủ Dịch và Review theo các đơn vị thực sự đã đối chiếu. Khi chia lại, tái dùng nguyên đơn vị còn hợp lệ. Nếu thứ tự từ trong bản Việt làm một phạm vi con khó tách chính xác, giữ cả đơn vị cũ hoặc dịch lại vùng đó. EPUB đổi tạo snapshot mới. Chỉ tái dùng qua snapshot sau khi khớp duy nhất vị trí, nội dung, tài nguyên được bảo vệ và kiểm tra các phụ thuộc. [Mapping nguồn](EPUB_INPUT_SPEC.md)

### 13.5. Chia lượt dịch theo token — mục 05

Dùng một bộ lập lượt cho mỗi hồ sơ model. Ưu tiên tokenizer đúng phiên bản từ provider hoặc tokenizer chạy cục bộ với bộ từ vựng đúng model. Khi chỉ có bộ ước lượng, hiện rõ trạng thái ước tính, dùng biên lớn hơn và tạm dừng nếu thiếu giới hạn đáng tin. Đối với model cung cấp `tokenizer.json`, dùng thư viện Rust của Hugging Face. [Tokenizers](https://github.com/huggingface/tokenizers)

Đếm toàn bộ request đã dựng gồm hướng dẫn, Skills/glossary, ngữ cảnh, nguồn, khuôn JSON và phần bao do adapter thêm. *Mặc định thử nghiệm* giữ mốc 3.000 token nguồn/lượt, dự trù đầu ra Dịch bằng khoảng hai lần token nguồn cộng phần bao JSON. Chừa thêm 5% cửa sổ khi có tokenizer phù hợp, 10% khi phải ước lượng. Hồ sơ từng vai trò có thể điều chỉnh dự trù. Dự trù reasoning theo cách tính của model, tính một lần khi reasoning đã nằm trong trần đầu ra. Kiểm tra riêng trần input, output và tổng context. Cắt ngữ cảnh tùy chọn trước, giữ quy ước bắt buộc.

Gom các block liên tiếp vừa ngân sách. Nếu có điểm ngắt part/cảnh trong 20% cuối phần vừa chứa, ưu tiên điểm gần cuối đó, sau đó đến cuối đoạn rồi cuối câu. Với một block không có chỗ ngắt câu an toàn, hiển thị vị trí để chỉnh điểm chia hoặc giới hạn. Đổi model theo yêu cầu người dùng sẽ tính lại phần chưa gửi và bảo toàn các đơn vị đã hoàn thành còn hiệu lực.

### 13.6. Liên kết và yêu cầu triển khai

Kiểm tra giới hạn nhập 10.000 mục ZIP, 128 MiB/mục và 1 GiB dữ liệu thực sau giải nén theo [Quyết định 36 về kiểm thử và nghiệm thu](DESIGN_DECISIONS_36.md#decision-36). Đây là mặc định thử nghiệm có thể cấu hình.
