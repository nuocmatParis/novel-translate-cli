# Quyết định 08 — Skills dùng chung, quy ước riêng và duyệt thuật ngữ

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 12.

Ngày ghi nhận: **27/10/2026**.

Cập nhật ngày **29/10/2026**, người dùng tự chuẩn bị Skills. Thư viện chung dùng Markdown linh hoạt. Mẫu riêng sáu nhóm và cách tiếp nhận file có sẵn đã được chốt bổ sung ngày **04/11/2026** tại mục 12. Giữ cơ chế đồng bộ **2B**, tự phát hiện và áp dụng thay đổi hợp lệ từ lượt tiếp theo, xử lý các điểm chưa rõ, không hợp lệ hoặc xung đột.

Trạng thái: **đã chốt hướng thiết kế trong trao đổi**. Tài liệu xác định hành vi cần triển khai; không khẳng định implementation hiện tại đã đáp ứng. Tên đường dẫn, đối tượng và trạng thái minh họa chưa phải schema hoặc command mới.

Tài liệu bổ sung cho [Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [xưng hô xuyên tập](ADDRESSING_CONTINUITY_SPEC.md) và [mô hình dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md).

Cập nhật ngày **30/10/2026**: [Research](RESEARCH_STAGE_SPEC.md) phụ trách tra cứu nội bộ/Internet để hỗ trợ câu hỏi và đề xuất thuật ngữ. Dịch/Review bàn giao nhu cầu tra cứu cho Research; bằng chứng tìm được không tự duyệt cách dịch hoặc thay đổi quy ước đã chốt.

Quyết định này cập nhật hai điểm trong đề xuất trước: **AI không tự duyệt thuật ngữ/quy ước mới; phương án chưa được người dùng duyệt chưa được áp dụng vào bản nháp hiện hành, kể cả chỉ dùng tạm**. Quy định này áp dụng cả khi model dịch và model review đều đồng ý với cùng một phương án.

Bổ sung ngày **29/10/2026** theo [đặc tả EPUB đầu ra](EPUB_OUTPUT_SPEC.md): ruby mang ý nghĩa riêng cần đề xuất cách thể hiện kèm chữ chính, chữ nhỏ, ngữ cảnh, các phương án và phạm vi để người dùng duyệt. Có thể dùng lại quyết định phù hợp đã có; không tự tạo chú thích người dịch hoặc dùng phương án mới lúc đóng gói EPUB. Ruby chỉ ghi phát âm áp dụng chính sách chung đã chốt sau khi được xác định/Review, không bắt duyệt từng occurrence; tên/thuật ngữ mới trong đó vẫn phải chờ chốt như bình thường.

## 1. Các quyết định chính

| Nội dung | Quyết định |
| --- | --- |
| Thư viện chung | Một thư mục chứa nhiều file Markdown hướng dẫn/tra cứu, có thể dùng cho nhiều bộ truyện. |
| Quy ước riêng | Mỗi bộ có file skill riêng quy định tên, thuật ngữ, văn phong, xưng hô và phạm vi áp dụng. |
| Cách tổ chức nội dung | Glossary, văn phong và xưng hô có thể cùng nằm trong file riêng của bộ; không bắt người dùng chia thành ba file. |
| Định dạng skill chung | Markdown linh hoạt do người dùng chuẩn bị; không bắt chuyển hướng dẫn, bảng tra cứu và ví dụ sang một mẫu cứng. |
| Định dạng skill riêng | Mẫu Markdown có phiên bản và sáu mục. File có sẵn được xem trước, rồi người dùng xác nhận những quy ước rõ ràng theo [Quyết định 11](DESIGN_DECISIONS_36.md#decision-11). |
| Sửa file bên ngoài | Với file đã đăng ký sử dụng, tự phát hiện và áp dụng thay đổi rõ ràng, hợp lệ từ lượt tiếp theo; không yêu cầu bấm Áp dụng cho mỗi lần sửa. |
| Kết hợp quy tắc | Skill riêng ưu tiên hơn hướng dẫn chung khi cùng điều chỉnh một vấn đề trong đúng phạm vi; ý nghĩa và diễn biến vẫn phải bám bản gốc. |
| Thuật ngữ/quy ước mới | AI đưa phương án và lý do, người dùng chọn hoặc nhập phương án riêng rồi xác nhận. |
| Trước khi duyệt | Giữ đề xuất và phần phụ thuộc ở trạng thái chờ; chưa dùng lựa chọn mới trong bản nháp hiện hành hoặc quy tắc chính thức. |
| Sau khi duyệt | Lưu quyết định và lịch sử, cập nhật quy ước riêng của bộ, tiếp tục phần phụ thuộc với đúng lựa chọn. |
| Lần xuất hiện sau | Dùng lại quyết định đã duyệt nếu cùng nghĩa/định danh và phạm vi; không hỏi lặp lại. |
| Muốn đổi quyết định cũ | Tạo đề xuất mới để người dùng duyệt; Reviewer không tự đổi chỉ vì thích cách diễn đạt khác. |
| Thư viện chung | AI không tự ghi thuật ngữ của một bộ vào thư viện chung; thư viện này chỉ đổi khi người dùng chủ động cập nhật. |

## 2. Hai tầng Skills

### 2.1. Thư viện dùng chung

Ví dụ người dùng cung cấp: `Library_COMMON_KANJI_SINO_VN.md`.

Những file loại này chứa nguyên tắc dịch, bảng tra cứu, cách xử lý kanji/từ ghép và ví dụ diễn đạt có thể dùng cho nhiều bộ. Một thư mục chung có thể có nhiều file, không cần chép toàn bộ thư viện vào từng workspace bộ truyện.

Tool cần biết file nào đang được bật và phiên bản nào đang được dùng. Những bảng gợi ý nhiều cách diễn đạt không tự tạo một quyết định glossary riêng cho mọi bộ truyện.

Người dùng được tự soạn và chuẩn bị các file này bằng Markdown linh hoạt, bao gồm đoạn giải thích, bảng và danh sách. Không bắt thư viện chung tuân theo mẫu của skill riêng. Việc đọc/chọn phần liên quan vẫn phải phân biệt hướng dẫn, ví dụ và phương án tham khảo với một quyết định dịch cụ thể; chỗ mơ hồ không được AI tự biến thành quy tắc đã chốt.

### 2.2. Skill riêng của bộ

Ví dụ người dùng cung cấp: `rezero_quy_uoc_dich_va_xung_ho.md`.

File riêng có thể chứa:

- Tên nhân vật, địa danh, tổ chức, vật phẩm, danh hiệu và biệt danh.
- Thuật ngữ thống nhất, biến thể của cùng một tên/khái niệm và các khái niệm cần phân biệt.
- Văn phong, hậu tố, dấu thoại, cách trình bày và các lựa chọn biên tập.
- Cặp xưng hô có hướng, mốc hiệu lực và ngoại lệ theo cảnh.
- Nguồn tham khảo, lý do lựa chọn và các lưu ý riêng của từng tập/chương.

Các mục thuộc phạm vi Vol. 26–27 trong file Re:Zero không tự áp ngược cho Vol. 1 chỉ vì cùng nằm trong một file của series. Tên hoặc danh tính ở phần sau chỉ được dùng đúng mức độ nguyên tác đã tiết lộ tại điểm đang dịch.

Mẫu Markdown có phiên bản gồm Văn phong, Nhân vật và bí danh, Thuật ngữ và danh hiệu, Địa danh, Xưng hô, Ngoại lệ. Mục 12 quy định trường bắt buộc, ID, phạm vi và quy trình xem trước/xác nhận khi đăng ký file riêng hiện có. Tool tạo bản chuyển đổi ở đường dẫn riêng và giữ file người dùng đã chuẩn bị.

### 2.3. Cây tổ chức minh họa

```text
Thư mục chung/
├── Skills/
│   ├── Library_COMMON_KANJI_SINO_VN.md
│   ├── Library_....md
│   └── ...
└── Series/
    ├── ReZero/
    │   ├── Skills/
    │   │   └── rezero_quy_uoc_dich_va_xung_ho.md
    │   ├── vol01_translation/
    │   └── vol02_translation/
    └── Bộ khác/
        ├── Skills/
        │   └── quy_uoc_dich_va_xung_ho.md
        └── vol01_translation/
```

Cây này thể hiện phạm vi dùng chung và riêng, không tự di chuyển file trên máy. Đường dẫn thư viện chung và tên file riêng cần được cấu hình/đăng ký phù hợp khi triển khai; `Skills/SKILL.md` là ví dụ bố trí file riêng. Theo quyết định ngày **30/10/2026** trong [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md), không cần chuyển hoặc giữ tương thích workspace test cũ. Người dùng vẫn có thể đăng ký file Markdown đã chuẩn bị làm đầu vào cho workspace mới; việc tiếp nhận file đi qua xem trước, xác nhận các quy ước rõ ràng và tạo bản theo mẫu riêng theo mục 12.

## 3. Chọn quy tắc cho một phần dịch hoặc Review

```text
Skill chung đang được bật
       +
Quy ước riêng của bộ, đúng phạm vi và phiên bản
       +
Nguyên tác và ngữ cảnh liên quan
       ↓
Ngữ cảnh dùng cho Dịch / Review
```

Trước mỗi lượt, xác định các quy tắc liên quan đến nội dung được giao, nhân vật, nghĩa của thuật ngữ, tập/chương và thời điểm cảnh. Quy tắc có mốc/ngoại lệ đã được người dùng duyệt chỉ áp dụng trong phạm vi đó.

Khi skill riêng quy định một cách dịch cụ thể, hướng dẫn chung không được tự thay nó bằng một phương án khác. Nếu các quy tắc cùng có hiệu lực mà mâu thuẫn, đưa xung đột ra để người dùng giải quyết; không lấy file đọc sau hoặc bản ghi mới nhất làm quyết định mặc định.

Nguồn quyết định ý nghĩa và diễn biến. Quy ước diễn đạt không cho phép thêm sự kiện, tiết lộ danh tính sớm hoặc thay đổi một khái niệm của nguyên tác. Nếu quy ước đã duyệt có dấu hiệu không phù hợp với đoạn mới, nêu vấn đề và đề xuất xử lý thay vì tự sửa quy ước.

Nội dung cần thiết được chọn theo hai giới hạn dung lượng đã chốt. Không mặc định nhồi toàn bộ thư viện vào mọi request hoặc âm thầm bỏ quy tắc bắt buộc để vừa giới hạn. Nếu quá lớn, giảm phạm vi xử lý/chọn lại phần tham khảo; ghi phiên bản và phần skill thực sự đã dùng để truy vết.

File skill là dữ liệu hướng dẫn dịch và tra cứu của sản phẩm. Các ghi chú trong file không thay quyền phê duyệt của người dùng, hợp đồng đầu ra, kiểm tra nguồn hoặc quy tắc lưu dữ liệu của ứng dụng.

## 4. Khi nào phải đưa ra đề xuất

Các mục cần thống nhất nhưng chưa có quyết định phù hợp phải được đề xuất, gồm tên riêng, thuật ngữ thế giới truyện, chức danh, tên kỹ năng/vật phẩm/tổ chức, biệt danh và quy ước xưng hô mới hoặc thay đổi.

Đề xuất cũng cần được tạo khi:

- Một thuật ngữ đã biết xuất hiện với nghĩa hoặc định danh khác.
- Có nhiều cách phiên âm/dịch và chưa có lựa chọn đã chốt cho trường hợp đó.
- Một quy tắc đã duyệt được đề nghị mở rộng sang phạm vi mới.
- Muốn thay cách dịch, cách gọi hoặc quy ước văn phong đã được chốt.
- Các quy tắc hiện hành mâu thuẫn hoặc thiếu căn cứ để chọn.

Từ vựng thông thường và việc diễn đạt câu theo ngữ cảnh không mặc định cần tạo một mục glossary cho mỗi từ. Nếu không chắc một từ đang là tên riêng/thuật ngữ hay từ thường và lựa chọn có ảnh hưởng, nêu rõ điều chưa chắc để xác minh.

Một thuật ngữ có nhiều lần xuất hiện cùng nghĩa/phạm vi nên được gom vào cùng đề xuất, có danh sách vị trí. Không gom các khái niệm khác nhau chỉ vì cùng mặt chữ.

## 5. Nội dung bắt buộc của một đề xuất

| Thông tin | Yêu cầu |
| --- | --- |
| Nguyên ngữ | Trích đúng từ/cụm từ trong EPUB đang dịch; không đoán một bản Nhật từ cách gọi tiếng Việt. |
| Vị trí | Tập, chương, block/phạm vi và snapshot nguồn để mở lại. |
| Ngữ cảnh | Đoạn xuất hiện và phần trước/sau cần thiết; chỉ rõ đang nói về người, chức danh, vật phẩm hay khái niệm nào. |
| Điều đã xác định | Nghĩa/định danh có căn cứ và những điều chưa rõ; không biến suy đoán thành sự thật. |
| Quy ước liên quan | Các mục trong skill riêng/chung đã được áp dụng, kèm phiên bản và phạm vi. |
| Các phương án | Cách dịch, sắc thái, ưu điểm và điểm có thể làm người đọc hiểu sai. |
| Khuyến nghị của AI | Có thể nêu phương án đề xuất và lý do; khuyến nghị chưa phải quyết định áp dụng. |
| Phạm vi đề xuất | Toàn series, một nghĩa/định danh cụ thể, một tập/chương hoặc từ mốc nào đến mốc nào. |
| Bằng chứng tham khảo | Phân biệt nguyên tác, quyết định dịch trước đó và nguồn tra cứu bên ngoài nếu có. |
| Trạng thái/lịch sử | Đang chờ, đã duyệt, đã từ chối, cần thêm ngữ cảnh hoặc bị thay thế bởi quyết định mới. |

Không bịa thêm phương án chỉ để đủ số lượng. Nếu chỉ có một phương án đáng tin cậy, vẫn nêu căn cứ, điểm chưa chắc và để người dùng quyết định.

Đối với xưng hô, bổ sung người nói → người nghe, cách xưng/gọi hiện tại và đề xuất, sự kiện trước/sau, mốc hiệu lực và bằng chứng theo đặc tả xưng hô. Một lời tỏ tình được xác nhận không tự duyệt lựa chọn anh–em.

### 5.1. Ví dụ màn chọn thuật ngữ

Ví dụ sau dựa trên cách gọi người dùng nêu, chỉ minh họa cách trình bày. Chưa có trích đoạn nguồn ở đây để kết luận phương án nào dịch đúng. Khi chạy thực tế phải lấy nguyên ngữ và ngữ cảnh từ EPUB.

| Phương án minh họa | Nội dung AI cần giải thích |
| --- | --- |
| Giám mục Đãi Đọa | Sắc thái Hán-Việt của cách gọi, nghĩa cần truyền đạt và mức phù hợp của chức danh với nguồn. |
| Giám mục Lười Biếng | Mức dễ hiểu của tên tội và việc chức danh có phù hợp quy ước của bộ hay không. |
| Đại Tội Tư Tế Lười Biếng | Mối liên hệ với các mục “Đại Tội Tư Tế” và “Lười Biếng” trong skill Re:Zero, nếu nguồn đúng là khái niệm tương ứng. |

Các mục thành phần đã có trong glossary là căn cứ đề xuất. Nếu danh hiệu/khái niệm đang xét vẫn cần một lựa chọn mới, không coi việc ghép các thành phần đó là người dùng đã duyệt lựa chọn mới.

```text
Thuật ngữ đang chờ duyệt
  Nguyên ngữ: lấy đúng từ EPUB
  Vị trí: tập / chương / đoạn
  Ngữ cảnh: mở được đoạn nguồn liên quan
  Quy ước hiện có: các mục liên quan
  Phương án: cách dịch + giải thích
  Phạm vi: nghĩa/định danh và mốc áp dụng

[Chọn phương án] [Nhập cách dịch riêng]
[Xem thêm ngữ cảnh] [Yêu cầu đề xuất lại] [Để sau]

Xác nhận: [Chốt và áp dụng]
```

Tên nút là minh họa. Việc một phương án được đánh dấu khuyến nghị hoặc đang được chọn trên màn hình chưa đủ để tự xác nhận.

### 5.2. Duyệt nhiều đề xuất trong cùng màn hình — 7A

Chốt ngày **30/10/2026**: cho phép xem và duyệt nhiều đề xuất thuật ngữ trong một màn hình. Người dùng vẫn có thể xử lý riêng một mục; không bắt mở và đóng từng thẻ để duyệt cả danh sách.

Mỗi dòng hiển thị nguyên ngữ, cách dịch đang chọn, phạm vi, trạng thái và vị trí xuất hiện; mở được đầy đủ ngữ cảnh, bằng chứng và các phương án ở mục 5. Người dùng chọn phương án hoặc nhập cách dịch riêng cho từng mục, chỉnh phạm vi khi cần, rồi đánh dấu những mục muốn xác nhận. Những mục chỉ đang hiển thị, được AI khuyến nghị hoặc chưa được chọn không tự nằm trong quyết định duyệt.

```text
Đề xuất thuật ngữ đang chờ
  [x] Thuật ngữ A · cách dịch đã chọn · phạm vi đã chọn
  [x] Thuật ngữ B · cách dịch người dùng nhập · phạm vi đã chọn
  [ ] Thuật ngữ C · cần thêm ngữ cảnh

[Xem ngữ cảnh] [Sửa lựa chọn] [Chốt 2 mục đã chọn]
```

Tên nút là minh họa. Nút xác nhận cần cho biết số mục và cho xem các lựa chọn/phạm vi sẽ được áp dụng. Một lần xác nhận ghi quyết định cho đúng các mục được chọn ở phiên bản đang xét; không đồng nghĩa duyệt mọi đề xuất trong danh sách, các trang khác hoặc những đề xuất mới xuất hiện sau đó.

Trước khi lưu, kiểm tra từng mục đã có lựa chọn/phạm vi rõ ràng, không xung đột với nhau hoặc với quy tắc đang có hiệu lực, và dữ liệu nền chưa bị thay đổi liên quan. Mục chưa đủ điều kiện phải được chỉ rõ để xử lý hoặc bỏ khỏi nhóm được chọn; không tự chọn phương án khác để hoàn tất nhóm. Nếu file skill đã bị sửa bên ngoài, áp dụng kiểm tra nền và giữ sửa tay theo mục 8.

Mỗi thuật ngữ có quyết định, bằng chứng, phạm vi và lịch sử riêng. Nhóm được chọn công bố cùng thành công sau khi kiểm tra toàn bộ phiên bản và xung đột. Chọn 10 mục mà mục 7 xung đột thì giữ lựa chọn, chỉ rõ mục 7, cho bỏ mục đó để xác nhận lại 9 mục. Ghi Skill chưa xong hiện **Đã nhận xác nhận, chờ đồng bộ Skill**. Phục hồi hoàn tất cùng thao tác rồi kích hoạt cả nhóm, theo mục 12.

Những mục chưa được xác nhận tiếp tục chờ và chặn đúng phần phụ thuộc. Duyệt một nhóm thuật ngữ chưa phải chấp nhận bản dịch hoặc kết luận Review đạt; phần dịch tiếp tục theo mục 7 và không nhảy qua chương đang bị chặn để gom thêm đề xuất.

## 6. Quyền quyết định của người dùng

| Thao tác | Kết quả |
| --- | --- |
| Chọn phương án và xác nhận | Ghi cách dịch, nghĩa/định danh, phạm vi và người duyệt. |
| Chọn nhiều mục rồi xác nhận một lần | Lưu quyết định riêng cho từng mục đã chọn theo mục 5.2; các mục còn lại tiếp tục chờ. |
| Nhập phương án riêng rồi xác nhận | Dùng lựa chọn của người dùng làm quyết định; vẫn Review bản dịch theo nguyên tác. |
| Yêu cầu đề xuất lại | Giữ trạng thái chờ và phản hồi của người dùng; tạo lại gợi ý có căn cứ. |
| Xem thêm ngữ cảnh | Mở nguồn liên quan để quyết định; chưa tự thay trạng thái duyệt. |
| Giữ quy tắc cũ | Ghi quyết định từ chối thay đổi và phạm vi mà quy tắc cũ tiếp tục phù hợp. |
| Để sau | Lưu đề xuất; giữ phần phụ thuộc chờ duyệt. |

Không tự duyệt vì model tự tin, nhiều model đồng ý, người dùng im lặng hoặc tác vụ đã chờ lâu. Bấm dịch cả chương hay chạy tự động không cấp quyền cho AI tự chọn thuật ngữ mới.

Với file riêng do người dùng chuẩn bị và xác nhận dùng làm quy ước của bộ, các lựa chọn có sẵn là cơ sở đã được người dùng cung cấp. Không bắt duyệt lại từng thuật ngữ ở mỗi lần xuất hiện. Người dùng trực tiếp tạo/sửa và xác nhận một quy tắc cũng là quyết định hợp lệ; không bắt AI tự tạo lại một đề xuất cho cùng lựa chọn.

Theo cơ chế 2B, khi người dùng trực tiếp sửa và lưu một lựa chọn rõ ràng trong file đã đăng ký sử dụng, thay đổi hợp lệ đó được xem là quyết định do người dùng cung cấp, không cần xác nhận thêm trong tool. Điều này không cho phép AI tự duyệt đề xuất của mình bằng cách ghi nó vào file; ví dụ, phương án đang cân nhắc hoặc câu mô tả chưa rõ trong file cũng không tự trở thành lựa chọn đã chốt.

## 7. Gắn vào luồng Dịch và Review

### 7.1. Phát hiện trước hoặc trong lúc tạo bản dịch

Khi chuẩn bị ngữ cảnh, nếu đã biết có mục chưa chốt, tạo/hiển thị đề xuất trước khi dùng cách dịch cho mục đó. Nếu chỉ phát hiện khi model trả lời, lưu phản hồi ứng viên và hồ sơ đề xuất nội bộ, đánh dấu phạm vi phụ thuộc `Chờ duyệt thuật ngữ`.

Phương án chưa duyệt có thể xuất hiện trong thẻ đề xuất hoặc câu minh họa được ghi nhãn rõ. Nó chưa được ghi thành bản nháp hiện hành, ngữ cảnh đã xác nhận, skill chính thức hoặc EPUB đầu ra. Nếu model đã tự đưa lựa chọn đó vào phản hồi, việc phản hồi hợp lệ JSON/cấu trúc không làm lựa chọn được phép áp dụng.

Tool có thể tiếp tục công việc cục bộ độc lập đã được giao và đủ điều kiện. Theo **13A** ngày **30/10/2026** trong [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), công việc cần AI phải chờ quyền chạy duy nhất; khi tác vụ đang chờ duyệt, người dùng có thể chọn chạy tác vụ khác thực sự độc lập sau khi lưu/bàn giao trạng thái. Nếu chia lại phạm vi, phải giữ coverage/ID, kiểm tra đầy đủ từng phạm vi mới và không công bố lượt còn thiếu như đã hoàn chỉnh. Điều này không cho phép bỏ qua lượt đang chờ hoặc bắt đầu chương sau trong cùng chuỗi Dịch–Review, kể cả bằng cách tạo tác vụ khác. Chính sách này không yêu cầu thêm một lượt AI đọc trước riêng cho mọi chương.

### 7.2. Sau khi người dùng duyệt

```text
Người dùng chốt phương án và phạm vi
  → lưu quyết định và cập nhật quy ước riêng nhất quán
  → chuẩn bị lại phần phụ thuộc bằng phiên bản quy tắc mới
  → tạo/hoàn thiện bản dịch theo lựa chọn đã duyệt
  → kiểm tra cấu trúc và lưu revision bản nháp
  → Review bản dịch với nguyên tác và quy tắc đó
```

Duyệt thuật ngữ cho phép áp dụng lựa chọn; chưa chứng minh mọi câu sử dụng lựa chọn đó đúng nghĩa. Bản dịch vẫn phải qua Review và điều kiện hoàn tất chương.

### 7.3. Phát hiện trong Review

Reviewer có thể phát hiện thuật ngữ mới chưa được nhận diện ở bước Dịch, xung đột hoặc nhu cầu đổi quy tắc. Khi đó, lưu đề xuất và chuyển phần phụ thuộc sang chờ người dùng. Giữ lịch sử bản đã tạo nhưng chưa cho phần này tiếp tục như kết quả đã được chấp nhận.

Reviewer sửa một chỗ dùng sai để trở về quy ước đã duyệt vẫn theo luồng sửa lỗi thông thường. Muốn đổi chính quy ước đó phải chờ người dùng quyết định. Các vòng sửa tự động không được vòng qua bước duyệt bằng cách tự chọn một phương án mới.

Trạng thái chờ duyệt không phải lỗi mạng, lỗi cấu trúc hay một vòng sửa thất bại. Không retry liên tục để mong model tự chốt và không tăng số vòng sửa chỉ vì đang chờ người dùng.

Nếu cần tìm thêm căn cứ để trình các phương án, Research nhận nguyên ngữ, vị trí/ngữ cảnh, quy ước và mốc đang xét; trả kết quả có nguồn cùng điểm chưa rõ. Thông tin từ Internet hoặc đề xuất Research chỉ là căn cứ để đối chiếu/duyệt, không được ghi vào skill chính thức bằng cơ chế đồng bộ 2B trước quyết định người dùng.

## 8. Lưu vào skill riêng và giữ lịch sử

Quyết định được duyệt cần lưu đủ: nội dung được chọn, nghĩa/định danh, phạm vi, bằng chứng, người xác nhận, phiên bản trước/sau và các vị trí liên quan. File skill riêng được cập nhật ở mục phù hợp; các phương án bị từ chối vẫn nằm trong lịch sử đề xuất, không trở thành quy tắc đang áp dụng.

Các màn Dịch, Review và Ngữ cảnh cùng sử dụng một tập quyết định. Không duy trì các bản glossary độc lập khiến màn này dùng một cách dịch, màn khác dùng cách khác.

File Markdown là nơi người dùng có thể xem/chỉnh quy ước. SQLite lưu trạng thái, lịch sử và liên kết phiên bản theo ID quyết định. Đối chiếu ID, hash nền/đích, phạm vi và ghi nhận thao tác công bố theo giao thức tại mục 12. Chỉ kích hoạt quyết định sau khi kiểm tra file đã ghi, giữ hồ sơ để phục hồi và hiện chờ đồng bộ khi việc ghi còn dở.

Nếu file được sửa bên ngoài trong lúc đang xử lý đề xuất, phát hiện thay đổi và giữ cả hai phía để đối chiếu. Không ghi đè phần người dùng vừa sửa bằng một bản file cũ. Khi nạp phiên bản mới, phải đánh giá dependency liên quan và giữ lịch sử.

Thư viện chung không tự nhận thay đổi từ một bộ riêng. Không chép tài liệu thực tế, EPUB, workspace hoặc nội dung dịch của người dùng vào Git.

### 8.1. Đồng bộ 2B — tự áp dụng thay đổi hợp lệ

Cơ chế áp dụng cho các file skill chung/riêng đã được đăng ký và bật. Chọn file để sử dụng là thao tác riêng; phát hiện một file Markdown mới ở nơi khác không tự cấp quyền dùng file đó.

1. Phát hiện file đang dùng thay đổi và đọc một phiên bản đầy đủ, ổn định; không công bố nội dung đọc dở trong lúc file đang lưu.
2. Đối chiếu với phiên bản trước, nhận diện quy tắc và phạm vi thay đổi; kiểm tra chỗ mơ hồ, không hợp lệ hoặc mâu thuẫn với quy tắc cùng có hiệu lực.
3. Với thay đổi rõ ràng, hợp lệ, lưu phiên bản/quyết định và lịch sử nhất quán, rồi tự đưa vào sử dụng từ lượt tiếp theo phù hợp. Hiển thị thông báo đã cập nhật và cho xem phần thay đổi; không tạo bước bắt buộc bấm Áp dụng.
4. Xét lại những bản dịch, Review hoặc ngữ cảnh thực sự phụ thuộc vào quy tắc đã đổi. Giữ lịch sử và sửa tay; không tự thay toàn bộ truyện hoặc yêu cầu dịch lại mọi chương.

Thay một cách dịch cũ bằng một cách dịch mới rõ ràng, đúng phạm vi trong file là cập nhật của người dùng, không tự bị coi là xung đột chỉ vì khác phiên bản trước. Xung đột cần xử lý là các lựa chọn cùng hiệu lực chưa xác định được ưu tiên, hoặc hai phía cùng thay đổi một quyết định trên các nền khác nhau.

Với skill chung, cập nhật hướng dẫn/tra cứu không biến mọi phương án gợi ý thành thuật ngữ chính thức của từng bộ. Quy ước riêng, quyền duyệt và ranh giới nội dung nguồn vẫn áp dụng như mục 3 và mục 6.

### 8.2. Lượt đang chạy, phần chưa rõ và xung đột

Lượt AI đang chạy giữ đúng phiên bản skill đã gửi. Khi nhận kết quả, kiểm tra các thay đổi liên quan trước khi dùng kết quả như bản hiện hành; không gắn ngược nhãn quy tắc mới vào một phản hồi đã tạo với quy tắc cũ. Lượt tiếp theo chưa gửi cần chuẩn bị ngữ cảnh bằng phiên bản hợp lệ mới; nếu đã chuẩn bị nhưng chưa gửi, kiểm tra lại phần bị ảnh hưởng.

Nếu chưa đọc ổn định, không hiểu rõ phạm vi, phát hiện mâu thuẫn hoặc không lưu đồng bộ được, hiển thị đúng file/vị trí và nguyên nhân, giữ phiên bản hợp lệ cùng dữ liệu thay đổi để xử lý. Phần phụ thuộc chưa được tiếp tục dựa trên lựa chọn chưa rõ hoặc âm thầm dùng quy tắc cũ như chưa có thay đổi. Công việc độc lập vẫn theo giới hạn của đặc tả điều phối, không nhảy qua lượt/chương đang bị chặn.

Nếu tool đang ghi một quyết định đã được duyệt mà file bên ngoài đã đổi, kiểm tra lại nền trước khi ghi. Không ghi đè file mới bằng bản cũ hoặc tự chọn bên thắng khi có xung đột; giữ cả hai phía và đưa chỗ cần giải quyết cho người dùng. Cách phát hiện, ghép thay đổi không xung đột và phục hồi cụ thể thuộc thiết kế kỹ thuật sau này.

Tự đồng bộ không tự khởi động lại tác vụ mà người dùng đã tạm dừng/hủy, không tự đổi model hoặc xóa bộ đếm/ngân sách. Đóng rồi mở tool vẫn phải nhận diện phiên bản đang có trước khi tiếp tục. Dùng watcher cùng kiểm tra hash trước request và trước khi tool ghi. Chỉ đọc bản file đã lưu ổn định, parse trọn nội dung và đối chiếu ID/phiên bản theo mục 12.

## 9. Phạm vi, xung đột và thay đổi quyết định

Một quyết định có thể áp dụng toàn series hoặc chỉ cho một nghĩa, định danh, tập/chương/giai đoạn cụ thể. Xưng hô còn có chiều phát ngôn và thời điểm cảnh, bao gồm hồi tưởng.

Khi thuật ngữ xuất hiện lại đúng phạm vi và cùng nghĩa, dùng quyết định đã duyệt. Nếu nghĩa khác, định danh chưa rõ hoặc phạm vi chồng lấn mâu thuẫn, tạo đề xuất/xung đột riêng; không thay thế theo mặt chữ.

Đổi quyết định đã duyệt phải tạo phiên bản mới sau thao tác người dùng. Tool truy các bản dịch chịu ảnh hưởng để kiểm tra/sửa lại, bảo toàn bản cũ và sửa tay. Không mặc định áp ngược mọi thay đổi cho các tập đã dịch hoặc đã xuất.

Các quy tắc trong file chung và riêng phải được đối chiếu trong phạm vi thực sự đang xét. Một ghi chú “ưu tiên cao” trong tài liệu chung không tự vượt quyết định riêng của người dùng cho bộ truyện.

## 10. Vị trí trong giao diện

- **Màn Dịch:** hiển thị quy tắc đang dùng, số mục đang chờ và phần công việc bị phụ thuộc; mở được chi tiết đề xuất.
- **Màn Review:** hiển thị thẻ đề xuất có nguyên tác, ngữ cảnh, các phương án và lý do; phân biệt với thẻ lỗi bản dịch thông thường.
- **Danh sách duyệt thuật ngữ:** mở được từ Dịch hoặc Review, chọn cách dịch/phạm vi từng mục rồi xác nhận nhiều mục cùng lúc theo 7A; vẫn mở được thẻ chi tiết và duyệt riêng một mục.
- **Ngữ cảnh của series:** xem skill đang dùng, glossary/văn phong/xưng hô đã duyệt, phạm vi và lịch sử quyết định.

Người dùng có thể xử lý đề xuất tại màn Dịch hoặc Review nhưng quyết định được lưu về cùng nguồn dữ liệu của bộ. Đóng ứng dụng rồi mở lại vẫn thấy đề xuất đang chờ và các lựa chọn đã chốt.

Theo **12A** chốt ngày **30/10/2026** trong [đặc tả TUI/CLI](TUI_CLI_SPEC.md), danh sách duyệt dùng `Space` để đánh dấu mục, `Tab` chuyển vùng và `Enter` kích hoạt nút xác nhận đang focus. Trong ô soạn, Space/Enter vẫn có nghĩa nhập liệu, không duyệt mục phía sau. Nút xác nhận hiển thị số mục được chọn; màn hẹp từ 80×24 vẫn mở đủ ngữ cảnh và lựa chọn, từ 120 cột có thể đặt danh sách cạnh chi tiết. Resize hoặc quay lại không tự mất lựa chọn/nội dung đang nhập hay tạo quyết định duyệt.

Khi file thay đổi, các màn liên quan cho biết phiên bản mới đã được áp dụng từ lượt nào hoặc đang vướng điều gì. Người dùng có thể xem quy tắc cũ/mới, đoạn file tương ứng và phạm vi cần kiểm tra lại; thay đổi hợp lệ theo 2B không tạo một hàng đợi duyệt lại bắt buộc.

## 11. Tiêu chí nghiệm thu

Kiểm thử triển khai dùng dữ liệu mẫu/model giả lập, offline và deterministic; không gọi provider trả phí trong CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Hai bộ dùng cùng thư viện | Dùng chung file được bật, mỗi bộ giữ quy ước riêng và không lẫn glossary. |
| Skill riêng khác hướng dẫn chung | Dùng quy tắc riêng đúng phạm vi, vẫn đối chiếu ý nghĩa với nguồn. |
| Mục xưng hô chỉ dành cho tập sau | Không áp ngược hoặc tiết lộ thông tin trước thời điểm nguyên tác. |
| Gặp thuật ngữ mới | Có nguyên ngữ, vị trí, ngữ cảnh, phương án, lý do và phạm vi chờ duyệt. |
| Model tự dùng thuật ngữ chưa duyệt | Phản hồi chỉ là ứng viên; phần phụ thuộc chưa thành bản nháp hiện hành. |
| AI khuyến nghị hoặc người dùng để sau | Không tự chọn/áp dụng phương án. |
| Chọn 2 trong 5 đề xuất rồi xác nhận | Chỉ 2 mục đã chọn được ghi quyết định riêng; 3 mục còn lại giữ trạng thái chờ và phần phụ thuộc tiếp tục bị chặn. |
| Mới chọn cách dịch/đánh dấu mục nhưng chưa xác nhận | Chưa công bố quy tắc hoặc dùng lựa chọn vào bản nháp hiện hành. |
| Danh sách có đề xuất mới sau lúc chọn | Không tự đưa mục mới vào nhóm được xác nhận. |
| Nhóm có lựa chọn xung đột hoặc nền skill đã đổi | Chỉ rõ mục bị ảnh hưởng, kiểm tra lại trước khi áp dụng; không ghi đè sửa tay hoặc tự chọn bên thắng. |
| Lưu nhóm bị lỗi/gián đoạn | Khôi phục đúng quyết định thực sự đã lưu và các mục còn chờ; không báo thành công toàn bộ hoặc làm mất lịch sử. |
| Người dùng nhập cách dịch riêng | Lưu đúng lựa chọn và phạm vi, tiếp tục dịch rồi Review. |
| Cùng thuật ngữ xuất hiện nhiều lần | Gom đề xuất khi cùng nghĩa/phạm vi; sau khi duyệt không hỏi lặp lại. |
| Cùng mặt chữ nhưng nghĩa khác | Xác minh/đề xuất riêng, không áp máy móc. |
| Reviewer muốn đổi thuật ngữ đã duyệt | Tạo đề xuất chờ người dùng, không tự sửa quy ước. |
| Sửa lỗi để tuân thủ quy tắc đã duyệt | Tiếp tục được theo luồng sửa–review thông thường. |
| Duyệt thuật ngữ nhưng câu còn sai nghĩa | Bản dịch vẫn bị Review theo rubric, chưa tự đạt. |
| Tác vụ bị dừng/mở lại | Giữ đề xuất, quyết định và phần đang chờ; không hỏi lại lựa chọn đã lưu còn hiệu lực. |
| Skill thay đổi khi đang xử lý | Phát hiện nền thay đổi, không ghi đè hoặc trộn phiên bản. |
| Người dùng chuẩn bị skill chung bằng Markdown linh hoạt | Dùng được hướng dẫn/tra cứu mà không bắt chuyển sang mẫu skill riêng; ví dụ chưa tự thành quyết định của bộ. |
| Người dùng lưu thay đổi rõ ràng trong file đã bật | Tự lưu phiên bản và áp dụng ở lượt tiếp theo, không yêu cầu xác nhận lặp lại. |
| Sửa quy tắc cũ thành lựa chọn mới rõ ràng | Ghi nhận quyết định cập nhật và phạm vi ảnh hưởng, không coi khác lịch sử là xung đột mặc định. |
| File thay đổi khi request đã gửi | Request giữ phiên bản cũ; kiểm tra hiệu lực kết quả trước khi dùng, lượt chưa gửi dùng phiên bản mới phù hợp. |
| File đang lưu dở hoặc có quy tắc mơ hồ/mâu thuẫn | Chỉ rõ phần vướng, không tự quyết hoặc cho phần phụ thuộc tiếp tục với dữ liệu chưa được xác định. |
| Tool cập nhật quy tắc đã duyệt trong lúc người dùng sửa file | Phát hiện nền mới, bảo toàn cả hai phía; xung đột cần người dùng giải quyết. |
| AI ghi một đề xuất chưa duyệt hoặc file chứa các phương án tham khảo | Không dùng cơ chế đồng bộ để tự duyệt thuật ngữ/quy ước. |
| File đổi lúc tác vụ đã được người dùng tạm dừng/hủy | Cập nhật dữ liệu phù hợp, không tự tiếp tục tác vụ. |
| Phần độc lập tiếp tục trong lúc chờ | Giữ coverage/ID và không đánh dấu phần còn chờ là hoàn tất. |

## 12. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [11](DESIGN_DECISIONS_36.md#decision-11), [12](DESIGN_DECISIONS_36.md#decision-12), [13](DESIGN_DECISIONS_36.md#decision-13), [14](DESIGN_DECISIONS_36.md#decision-14), [15](DESIGN_DECISIONS_36.md#decision-15) của bản 36 mục áp dụng cho đặc tả này.

### 12.1. Thiết kế mẫu Skill riêng của bộ truyện — mục 11

Dùng một mẫu Markdown có phiên bản với sáu mục **Văn phong, Nhân vật và bí danh, Thuật ngữ và danh hiệu, Địa danh, Xưng hô, Ngoại lệ**. Tool tạo sẵn khung, các mục có thể để trống. Dòng thuật ngữ cần nguyên ngữ, cách dịch và nghĩa/định danh. Phạm vi trống được hiển thị rõ là toàn bộ bộ truyện. Bí danh, cách đọc và ghi chú là tùy chọn. Quy tắc xưng hô cần người nói, người nghe, cách tự xưng, cách gọi và mốc hiệu lực. Cho khai báo nhiều lựa chọn khác phạm vi hoặc thời điểm. Tool cấp ID ổn định và lưu lịch sử, người duyệt, bằng chứng trong dữ liệu của bộ.

Khi đăng ký lần đầu, file riêng hiện có được xem trước theo ba nhóm **đọc được / cần làm rõ / giữ làm ghi chú**. Người dùng xác nhận một lần các quy ước rõ ràng dùng làm nền, rồi tool tạo bản theo mẫu ở đường dẫn riêng. Các mục đã được người dùng xác nhận và còn hiệu lực được dùng lại. Giữ nguyên file người dùng đã chuẩn bị. Đề xuất mới của AI về sau đi qua luồng duyệt từng mục hoặc nhóm. [Hai tầng Skills](SKILLS_GLOSSARY_SPEC.md)

### 12.2. Phân biệt hướng dẫn, ví dụ và quyết định chính thức — mục 12

Skill chung vẫn nhận Markdown tự do. Heading hoặc nhãn tùy chọn **Hướng dẫn bắt buộc**, **Ví dụ**, **Tham khảo** giúp phân loại đoạn. Văn xuôi thông thường dùng làm tham khảo. Ví dụ có nhiều phương án như “tớ–cậu / mình–bạn” được đưa vào đúng ngăn ví dụ. Một thuật ngữ chính thức của bộ phải truy được về quy ước người dùng cung cấp rõ ràng hoặc một quyết định đã duyệt.

Khi chọn ngữ cảnh, lọc file đang bật theo ngôn ngữ, loại nội dung, thực thể và phạm vi. Dùng quy tắc riêng đã duyệt đúng phạm vi trước hướng dẫn chung. Chọn ví dụ liên quan trong dung lượng còn lại. Hai chỉ dẫn bắt buộc cùng hiệu lực xung đột, hoặc một đoạn mơ hồ đang quyết định cách dịch, sẽ được đưa cho người dùng xem hai nguồn. Phần tham khảo đơn thuần có thể tiếp tục dùng với nhãn tham khảo.

### 12.3. Đồng bộ Skill khi người dùng chỉnh file ngoài — mục 13

Dùng cả thông báo thay đổi file lẫn kiểm tra hash trước request và trước khi tool ghi file. Sau thông báo, đọc lại khi file ổn định, parse trọn tài liệu và đối chiếu ID, phạm vi cùng phiên bản nền. Sửa một dòng có cùng ID tạo phiên bản mới của quy tắc đó. Dòng mới nhận ID mới. Khi ID bị mất và có nhiều cách đối chiếu, hiển thị diff để người dùng nhận diện. Thay đổi rõ ràng do người dùng lưu ngoài được áp dụng từ lượt chưa gửi tiếp theo theo 2B.

Khi tool ghi quyết định, lưu ý định cùng hash nền và hash đích trong SQLite, dựng file tạm, kiểm tra lại nền rồi thay file bằng thao tác bảo toàn phiên bản thực tế vừa bị thay. Đối chiếu cả bản được giữ lại lẫn hash đích trước khi kích hoạt quyết định trong giao dịch hoàn tất. Watcher nhận diện thao tác ghi của tool theo ID đang chờ để xử lý cùng giao dịch công bố. Mở lại có thể hoàn tất đúng thao tác đã ghi. Nếu file ngoài đổi trong lúc đó, ghép ba phía đối với các mục tách biệt. Phần chồng lấn chuyển sang **Chờ xử lý xung đột**. Trong khi việc công bố còn dở, phần phụ thuộc chờ. Request đã gửi giữ đúng phiên bản Skill thực sự đã dùng.

### 12.4. Phát hiện và gom thuật ngữ mới — mục 14

Bốn nơi được ghi nhận một lần xuất hiện là chuẩn bị ngữ cảnh, kết quả Dịch, Review và Research. Core gom tất cả về một danh sách của bộ. Khóa gom gồm dạng nguyên ngữ để tra, nghĩa/thực thể và phạm vi. Cùng một danh hiệu xuất hiện 20 lần tạo **một đề xuất có 20 vị trí**. Cùng chữ nhưng chỉ hai khái niệm khác nhau tạo hai đề xuất. Khi nghĩa chưa phân biệt được, giữ ví dụ nguồn của từng nơi để xác minh.

Đưa ra duyệt tên riêng, danh hiệu, kỹ năng, địa danh, từ tác giả đặt, hoặc cách gọi lặp lại có nhiều lựa chọn ảnh hưởng tính nhất quán. Từ thông dụng dùng theo ngữ cảnh khi đã rõ nghĩa. Mỗi đề xuất có nguyên ngữ, đoạn gốc, vị trí, cách dịch khả thi, lý do và phạm vi. Chặn đúng các đơn vị đang chờ lựa chọn. Ứng viên đã dùng thử từ mới nằm ở hồ sơ đề xuất cho tới khi người dùng chốt.

### 12.5. Duyệt nhiều thuật ngữ cùng lúc — mục 15

Chọn **cả nhóm cùng thành công** cho đúng danh sách và phiên bản người dùng đã đánh dấu. Nếu chọn 10 mục mà mục 7 xung đột, tool giữ nguyên lựa chọn, chỉ ra mục 7 và cho bỏ nó để bấm **Chốt 9 mục**. Kiểm tra mâu thuẫn giữa các mục trong nhóm, với quy tắc hiện hành và với file Skill trước khi công bố. Mỗi mục vẫn có bằng chứng, phạm vi và lịch sử riêng.

Nếu người dùng đã xác nhận nhưng ghi Skill thất bại, hiện **Đã nhận xác nhận, chờ đồng bộ Skill**. Phục hồi theo mục 13 rồi kích hoạt cả nhóm một lần. Sau đó tác vụ tự động vốn đang chờ đúng các mục này được tiếp tục khi đủ điều kiện và đến lượt chạy. Tác vụ người dùng đã tạm dừng cần lệnh tiếp tục. [Thao tác duyệt 7A](SKILLS_GLOSSARY_SPEC.md)

### 12.6. Liên kết và yêu cầu triển khai

Cú pháp biểu diễn mẫu, parser và giao thức đồng bộ phải thể hiện đủ trường, phạm vi và ID đã chốt. Cấu hình đường dẫn kho chung, đăng ký file riêng và danh sách bật/tắt được hiện thực theo mô hình hai tầng. Quy tắc xưng hô theo [đặc tả xưng hô](ADDRESSING_CONTINUITY_SPEC.md).
