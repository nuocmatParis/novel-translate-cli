# Quyết định 03 — Xưng hô xuyên tập và đề xuất thay đổi có ngữ cảnh

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 12.

Ngày ghi nhận: **25/10/2026**.

Cập nhật ngày **27/10/2026** theo [Skills và duyệt thuật ngữ/quy ước](SKILLS_GLOSSARY_SPEC.md): mọi đề xuất xưng hô mới hoặc thay đổi phải được người dùng duyệt trước khi áp dụng, kể cả trong bản nháp hiện hành. Quyết định này thay cơ chế tự xác nhận được đề xuất trước đó.

Trạng thái: **ghi nhận thiết kế đã thống nhất trong trao đổi**. Tài liệu là căn cứ triển khai bản Rust; không khẳng định toàn bộ cơ chế đã hoạt động trong implementation hiện tại.

Tài liệu tiếp nối [đặc tả EPUB đầu vào](EPUB_INPUT_SPEC.md) và [đặc tả giai đoạn Dịch](TRANSLATION_STAGE_SPEC.md). Phạm vi gồm bộ nhớ xưng hô theo cặp nhân vật, mốc hiệu lực, bằng chứng cho đề xuất thay đổi, vị trí hiển thị trên giao diện và cách dùng lại ở các phần dịch sau.

Hợp đồng nhân vật, bí danh, cặp xưng hô có hướng và hai mốc thời gian đã chốt tại mục 12. Lưu bằng chứng, phiên bản và dependency trong SQLite của bộ theo [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md). Workspace Rust mới có cơ chế nâng cấp và phục hồi, bắt đầu từ schema 1. Quyết định ngày **30/10/2026** bỏ yêu cầu chuyển workspace test cũ tiếp tục có hiệu lực.

## 1. Mục tiêu và các quyết định chính

Tool cần giữ được những quyết định xưng hô đã được chấp nhận trong cùng bộ truyện và áp dụng đúng ở các chương/tập tiếp theo, kể cả sau khi đóng ứng dụng rồi mở lại.

Ví dụ: trước một mốc quan hệ, A và B dùng tôi–cậu; sau khi quan hệ được xác lập và quy ước dịch được chấp nhận, A dùng anh–em và B dùng em–anh. Quy ước mới tiếp tục có hiệu lực ở phần truyện sau, trong phạm vi đã xác định.

| Nội dung | Quyết định |
| --- | --- |
| Bộ nhớ | Lưu lâu dài trong dữ liệu workspace của bộ truyện, có lịch sử và phiên bản |
| Đơn vị áp dụng | Cặp có hướng: người nói → người nghe |
| Cách xưng hô | Tách cách xưng mình và cách gọi đối phương |
| Mốc thay đổi | Có vị trí nguồn và thời điểm cảnh tương ứng khi cần |
| Hiệu lực xuyên tập | Tiếp tục áp dụng cho các phần sau thuộc cùng bộ, đến khi hết phạm vi hoặc có thay đổi mới được xác nhận |
| Đề xuất | Phải nêu rõ sự kiện, diễn biến, bằng chứng, lý do lựa chọn và phạm vi hiệu lực |
| Xác nhận | Chỉ quy ước đã được người dùng duyệt, còn phù hợp nguồn/revision mới được áp dụng và trở thành bộ nhớ chính thức |
| Hiển thị | Màn Dịch cho biết đang dùng gì; màn Review giải thích và xử lý thay đổi; Ngữ cảnh → Xưng hô giữ lịch sử |
| Ngoại lệ | Hồi tưởng, một lần nổi giận hoặc cách nói trong một cảnh phải được xét đúng phạm vi |

Một từ khóa như “tỏ tình”, “chia tay” hoặc “ghét” là dấu hiệu cần xem ngữ cảnh, chưa đủ để tự kết luận cách xưng hô phải đổi lâu dài.

## 2. Bộ nhớ theo cặp nhân vật và mốc truyện

### 2.1. Nhận diện nhân vật

Mỗi nhân vật có ID ổn định. Tên Nhật, tên dịch và biệt danh đã xác nhận được liên kết với cùng ID. Việc đổi cách viết tên tiếng Việt không được tự tạo một nhân vật mới.

Nếu một biệt danh có thể chỉ nhiều người hoặc chưa rõ ai đang nói với ai, giữ trạng thái chưa xác định. Không tự ghép danh tính hoặc áp quy ước của một cặp khác để lấp thông tin thiếu.

### 2.2. Lưu hai chiều riêng

| Giai đoạn minh họa | A nói với B | B nói với A |
| --- | --- | --- |
| Trước mốc thay đổi | Xưng tôi, gọi cậu | Xưng tôi, gọi cậu |
| Từ mốc thay đổi | Xưng anh, gọi em | Xưng em, gọi anh |

Đây là một ví dụ về quy ước dịch, không phải cách xưng hô bắt buộc cho mọi cặp nhân vật. Hai chiều có thể thay đổi ở những thời điểm khác nhau và không được mặc định suy ra từ nhau.

Quy ước A → B không áp dụng khi A nói với C, khi nhân vật nói với một nhóm chưa phân giải được người nghe, hoặc cho mọi câu kể về B. Cần giữ ngữ cảnh phát ngôn để phân biệt hội thoại, lời kể và trích dẫn.

### 2.3. Thông tin của một quy ước đã xác nhận

Một bản ghi cần biểu diễn được:

```text
Bộ truyện: bộ hiện tại
Người nói: A
Người nghe: B
Xưng mình: anh
Gọi đối phương: em

Có hiệu lực từ: Tập 02 / Chương 04 / đoạn 046
Kết thúc: chưa có
Phạm vi: các cảnh phù hợp kể từ mốc trên
Trạng thái: đã xác nhận
Căn cứ: đề xuất, bằng chứng nguồn và quyết định đã duyệt
Phiên bản: revision của quy ước
```

“Kết thúc: chưa có” nghĩa là quy ước được dùng tiếp ở các phần sau trong bộ truyện, không phải áp ngược về quá khứ. Khi có thay đổi mới được xác nhận, ghi thêm mốc và giữ được lịch sử quy ước cũ.

Đối với một ngoại lệ theo cảnh, bản ghi phải có phạm vi kết thúc rõ. Hết ngoại lệ thì trở về quy ước có hiệu lực của cặp đó, trừ khi đã có thay đổi lâu dài được xác nhận.

### 2.4. Thời điểm cảnh và vị trí trong sách

Phải phân biệt vị trí đoạn trong tập/chương với thời điểm sự kiện diễn ra trong truyện.

Ví dụ, ở tập 3 có hồi tưởng về thời điểm trước khi A và B hẹn hò. Cảnh đó cần dùng quy ước tôi–cậu có hiệu lực trong quá khứ, dù phần hiện tại của tập 3 đang dùng anh–em.

Mốc có thể nằm giữa một lượt dịch, thậm chí giữa một block. Khi cần, lưu vị trí đến câu/phạm vi văn bản để phân biệt trước và sau mốc. Không áp một thay đổi mới cho toàn bộ lượt chỉ vì lượt đó chứa sự kiện chuyển quan hệ.

## 3. Phân biệt sự kiện nguồn và lựa chọn dịch

Mỗi đề xuất phải làm rõ hai lớp căn cứ:

1. **Nguồn xác nhận điều gì:** ai làm/nói gì, người kia phản ứng ra sao, quan hệ có thực sự thay đổi hay còn mơ hồ.
2. **Vì sao chọn cách xưng hô tiếng Việt này:** dấu hiệu ngôn ngữ trong nguồn, quy ước đã đặt cho cặp nhân vật hoặc lựa chọn dịch được nêu rõ.

Nguồn xác nhận bắt đầu hẹn hò chưa tự đồng nghĩa nguyên văn đã yêu cầu chuyển sang anh–em. Nếu anh–em là quy ước dịch đã chọn cho cặp này sau khi xác lập quan hệ, đề xuất phải nêu chính quy ước đó làm căn cứ.

| Tình huống | Nội dung cần làm rõ trước khi đề xuất thay đổi |
| --- | --- |
| Tỏ tình | Một người bày tỏ hay hai người đã xác nhận tình cảm? Người kia đồng ý, từ chối hay chưa trả lời? |
| Bắt đầu quan hệ | Có lời nói/hành động xác nhận hay mới là suy đoán? Mốc bắt đầu ở đâu? |
| Chia tay | Đã thực sự chấm dứt quan hệ, chỉ dọa chia tay, đang diễn hay là hiểu lầm? |
| Ghét bỏ, thù địch | Có căn cứ cho thay đổi lâu dài hay chỉ một câu giận dữ? Thay đổi ở một chiều hay cả hai chiều? |
| Cãi nhau | Cách nói trong cảnh có chủ ý tạo khoảng cách không? Sau cảnh có trở lại bình thường không? |
| Giữ khoảng cách trước người khác | Đây là quy ước theo hoàn cảnh hay thay đổi quan hệ thật? |

Lời tỏ tình bị từ chối hoặc tình cảm một phía không mặc định tạo một cặp xưng hô thân mật hai chiều. Một bên có thể thay đổi cách nói nếu nguồn hỗ trợ; phạm vi của đề xuất phải phản ánh đúng điều đó.

Nếu bằng chứng còn thiếu hoặc mâu thuẫn, giữ đề xuất ở trạng thái cần xác nhận. Không biến suy đoán về cảm xúc thành dữ kiện quan hệ đã chốt.

## 4. Nội dung bắt buộc của một đề xuất

### 4.1. Hồ sơ đề xuất

| Thông tin | Yêu cầu |
| --- | --- |
| Cặp nhân vật | ID và tên hiển thị của người nói/người nghe; thể hiện riêng từng chiều |
| Quy ước hiện tại | Cách xưng mình/gọi đối phương đang có hiệu lực, kèm phiên bản |
| Quy ước đề xuất | Cách xưng mình/gọi đối phương mới cho từng chiều liên quan |
| Sự kiện | Mô tả cụ thể tình tiết dẫn tới đề xuất: ai nói/làm gì, với ai |
| Ngữ cảnh diễn biến | Diễn biến trước sự kiện, phản ứng của các bên và phần sau liên quan nếu có |
| Bằng chứng nguồn | Trích nguyên văn Nhật, vị trí tập/chương/block/phạm vi và snapshot nguồn |
| Bản dịch để đối chiếu | Hiển thị bản dịch của đoạn bằng chứng, chỉ rõ revision và trạng thái nháp/đã duyệt |
| Lý do chọn xưng hô | Tách kết luận về sự kiện khỏi quyết định dịch tiếng Việt; dẫn quy ước liên quan nếu có |
| Mốc bắt đầu | Vị trí cụ thể từ đó cách xưng hô mới được đề xuất áp dụng |
| Phạm vi kết thúc | Một câu, một cảnh, một khoảng hoặc chưa có mốc kết thúc |
| Điều chưa chắc | Những gì nguồn chưa xác nhận, suy luận còn cần xem lại hoặc bằng chứng mâu thuẫn |
| Trạng thái và lịch sử | Đề xuất/chờ xử lý/đã xác nhận/bị từ chối hoặc cần đánh giá lại; lưu quyết định và lý do |

Một nhãn chung như “đổi do tỏ tình” hoặc “quan hệ thay đổi” chưa đủ đáp ứng yêu cầu. Hồ sơ phải cho phép người dùng hiểu được diễn biến và kiểm tra lại căn cứ ngay tại vị trí đó.

### 4.2. Bằng chứng và ngữ cảnh

Mỗi trích dẫn phải liên kết đến nguồn thực, có định vị đủ chính xác để mở lại đúng đoạn. Lưu snapshot/hash để biết bằng chứng thuộc phiên bản nguồn nào.

Ngữ cảnh cần đủ để thấy mối liên hệ giữa sự kiện và phản ứng. Có thể mở rộng các đoạn trước/sau trong cảnh liên quan, thay vì chỉ đưa một câu tách rời. Việc mở rộng vẫn phải giữ vị trí/thời điểm rõ ràng; không lấy tình tiết ở thời điểm sau để áp ngược quy ước vào cảnh trước.

Bản tóm tắt diễn biến phải gắn với các đoạn bằng chứng. Nếu có phần suy luận, cần nêu rõ đó là suy luận và chỉ ra điểm chưa chắc. Không dựng thêm lời thoại, phản ứng hoặc động cơ không có căn cứ để làm đề xuất có vẻ thuyết phục.

Trong giao diện, bản dịch tiếng Việt giúp đọc nhanh, còn nguyên văn Nhật là dữ liệu đối chiếu bắt buộc. Chỉ viện dẫn một bản dịch cũ do AI tạo ra chưa đủ để xác nhận sự kiện nguồn.

### 4.3. Hai lớp kiểm tra

**Kiểm tra bằng chương trình:** xác nhận ID, snapshot và vị trí bằng chứng tồn tại; quote khớp nguồn; phạm vi áp dụng hợp lệ; cặp nhân vật và quy ước được tham chiếu không bị thiếu hoặc stale.

**Kiểm tra nội dung trong Review:** đánh giá quote cùng ngữ cảnh có thực sự hỗ trợ sự kiện được mô tả không, quyết định xưng hô có phù hợp nguồn/quy ước không, mốc và phạm vi có đúng không, có bằng chứng mâu thuẫn không.

Quote tồn tại trong nguồn chỉ xác nhận tính hợp lệ của trích dẫn. Nó không tự chứng minh diễn giải của model là đúng.

## 5. Ví dụ đề xuất có ngữ cảnh rõ ràng

### 5.1. Tỏ tình được chấp nhận

Ví dụ dưới đây là **tình huống và lời thoại mẫu tự tạo để minh họa**, không phải trích từ một EPUB thật. Khi chạy trên sách, tool phải thay bằng các đoạn nguồn thực cùng định vị/hash tương ứng.

Giả sử quy ước của bộ truyện đã chọn anh–em cho cặp A–B sau khi hai người xác lập quan hệ tình cảm.

| Vị trí mẫu | Người nói | Nguyên văn mẫu | Diễn giải tiếng Việt |
| --- | --- | --- | --- |
| Đoạn 044 | A | 好きだ。僕と付き合ってくれないか。 | Tôi thích cậu. Cậu có muốn hẹn hò với tôi không? |
| Đoạn 045 | B | 私も好き。うん、付き合おう。 | Tôi cũng thích cậu. Ừ, chúng mình hẹn hò nhé. |

Hồ sơ đề xuất cần được trình bày tương đương:

```text
Đề xuất thay đổi xưng hô A–B

Hiện tại:
  A → B: xưng tôi, gọi cậu.
  B → A: xưng tôi, gọi cậu.

Sự kiện:
  A đề nghị hẹn hò; B xác nhận tình cảm và đồng ý.

Ngữ cảnh:
  Đoạn 044 là lời đề nghị của A.
  Đoạn 045 là lời đáp xác nhận của B.
  Phạm vi từ đoạn 046 là phần tiếp theo sau khi quan hệ được xác lập.

Bằng chứng:
  Hiển thị nguyên văn và bản dịch của 044–045.
  Cho mở rộng để đọc cả cảnh và kiểm tra chi tiết liên quan.

Căn cứ lựa chọn:
  Nguồn hỗ trợ việc hai người bắt đầu quan hệ.
  Quy ước bộ truyện đã chọn anh–em cho cặp này sau mốc đó.

Đề xuất:
  A → B: xưng anh, gọi em.
  B → A: xưng em, gọi anh.
  Bắt đầu: đoạn 046.
  Kết thúc: chưa có.

Trạng thái:
  Chờ đối chiếu và xác nhận.
```

Các câu nguồn mẫu chứng minh đề nghị và sự đồng ý. Lựa chọn anh–em trong ví dụ đến từ quy ước dịch đã nêu, không được trình bày như một yêu cầu xưng hô hiển nhiên của nguyên văn.

### 5.2. Chia tay hoặc ghét bỏ

Với cảnh chia tay, hồ sơ phải cho thấy lời xác lập việc chấm dứt quan hệ, phản ứng và dấu hiệu giữ khoảng cách nếu có. Cách xưng hô mới được đề xuất cho từng chiều, có mốc cụ thể. Nếu nguồn chỉ có lời dọa chia tay hoặc ngữ cảnh còn bỏ ngỏ, chưa được coi đó là thay đổi lâu dài đã xác nhận.

Với cảnh giận dữ/ghét bỏ, cần chỉ ra lời thoại và diễn biến hỗ trợ mức độ thay đổi. Nếu chỉ có căn cứ trong một lần cãi nhau, đề xuất có thể là ngoại lệ trong cảnh thay vì thay quy ước cho các tập còn lại. Khi chưa biết đây là thay đổi tạm thời hay lâu dài, ghi rõ điểm chưa chắc để xử lý.

Không đặt sẵn một bảng chuyển đổi cứng kiểu “chia tay = tôi–cô” hoặc “ghét = tao–mày” cho mọi trường hợp. Lựa chọn phải dựa trên nguồn, sắc thái và quy ước của đúng cặp nhân vật.

## 6. Vị trí hiển thị trong ứng dụng

### 6.1. Màn Dịch

Hiển thị gọn quy ước đang được dùng cho phần hiện tại và thông báo có đề xuất mới:

```text
Xưng hô đang áp dụng
  A → B: tôi / cậu
  B → A: tôi / cậu

Có đề xuất thay đổi tại đoạn 046 — chờ người dùng duyệt.
```

Tách rõ **đang áp dụng** với **đang đề xuất**. Người dùng có thể mở chi tiết đề xuất để xem bằng chứng và phương án. Cách xưng hô mới chưa được dùng trong bản nháp hiện hành khi người dùng chưa duyệt; phần phụ thuộc giữ trạng thái chờ. Câu minh họa trong hồ sơ đề xuất hoặc phản hồi thô lưu nội bộ phải được ghi nhãn rõ và chưa trở thành bản dịch đang dùng.

### 6.2. Màn Review

Đặt sự kiện, ngữ cảnh và bằng chứng ngay cạnh bảng trước/sau của đề xuất. Cần nhìn được:

- tên hai nhân vật và chiều phát ngôn;
- cách xưng/gọi hiện tại và cách đề xuất;
- diễn biến dẫn đến thay đổi;
- nguyên văn Nhật, bản dịch và vị trí của từng đoạn làm bằng chứng;
- lý do chọn cách xưng hô, quy ước dịch được viện dẫn;
- mốc bắt đầu, phạm vi kết thúc và các điểm chưa chắc.

Các thao tác người dùng cần có khi xử lý đề xuất:

| Thao tác | Kết quả |
| --- | --- |
| Chấp nhận | Xác nhận quy ước, mốc và phạm vi đã đối chiếu |
| Chỉnh cách xưng hô/mốc/phạm vi | Lưu quyết định đã chỉnh cùng lý do và bằng chứng |
| Giữ quy ước cũ | Ghi nhận từ chối đề xuất và lý do |
| Mở rộng ngữ cảnh | Đọc thêm đoạn liên quan trong nguồn để quyết định |

Review đối chiếu nguồn và chính sách đã đặt, giải thích các phương án và có thể khuyến nghị một lựa chọn. Người dùng quyết định cách xưng hô, mốc và phạm vi trước khi áp dụng. AI không tự xác nhận quy ước mới dù bằng chứng rõ hoặc nhiều model đồng ý; xác nhận sự kiện nguồn không đồng nghĩa duyệt lựa chọn tiếng Việt.

Thiếu bằng chứng hoặc chưa xác định được mốc thì đề xuất vẫn ở trạng thái cần xác nhận, chưa cập nhật bộ nhớ chính thức.

### 6.3. Ngữ cảnh → Xưng hô

Mục này hiển thị quy ước và lịch sử theo cặp nhân vật:

```text
Cặp A–B
├── Trước Tập 02 / Chương 04 / đoạn 046
│   ├── A → B: tôi / cậu
│   └── B → A: tôi / cậu
└── Từ Tập 02 / Chương 04 / đoạn 046
    ├── A → B: anh / em
    ├── B → A: em / anh
    ├── Lý do: quan hệ đã được xác lập + quy ước dịch đã chọn
    ├── Bằng chứng và quyết định đã duyệt
    └── Kết thúc: chưa có
```

Màn này giúp tra lại vì sao đã đổi và chỉnh quyết định khi cần. Các màn Dịch, Review và Ngữ cảnh cùng đọc một nguồn dữ liệu; không duy trì ba bản quy ước độc lập.

## 7. Gắn với năm bước của giai đoạn Dịch

| Bước | Vai trò của cơ chế xưng hô |
| --- | --- |
| 1. Nhận lệnh Dịch | Kiểm tra revision/dependency liên quan và nhận diện phần cần xử lý nếu quy ước đã thay đổi |
| 2. Chuẩn bị dịch và ngữ cảnh | Tra cặp nhân vật, thời điểm cảnh và quy ước đã xác nhận; đưa dữ liệu liên quan vào request |
| 3. Tạo bản dịch | Dịch theo quy ước hiện hành; khi có căn cứ mới, trả thêm đề xuất cùng bằng chứng và mốc |
| 4. Kiểm tra cấu trúc | Kiểm tra các ID, quote, vị trí và trường dữ liệu đề xuất; chưa tự kết luận quan hệ thực sự đã thay đổi |
| 5. Lưu bản nháp | Chỉ lưu bản nháp hiện hành của phần đã đủ điều kiện duyệt quy ước; phần phụ thuộc chưa duyệt giữ ứng viên và hồ sơ chờ riêng |

Sau khi người dùng duyệt, quy ước mới cùng căn cứ và mốc hiệu lực được lưu nhất quán vào quyết định chính thức và skill riêng của bộ. Chuẩn bị lại phần đang chờ để áp dụng lựa chọn, tạo/kiểm tra bản nháp rồi Review. Những lượt dịch sau lấy quy ước phù hợp trong bước chuẩn bị mà không hỏi lại cùng lựa chọn trong đúng phạm vi.

Một mốc thay đổi nằm giữa lượt dịch phải được giữ đến phạm vi câu/đoạn cần thiết. Kiểm tra sau đó cần đối chiếu phần trước và sau mốc, thay vì lấy một trạng thái quan hệ duy nhất áp cho toàn bộ lượt.

## 8. Áp dụng quy ước ở các phần sau

Trước một lượt dịch, core thực hiện:

1. Xác định các nhân vật và cặp người nói → người nghe liên quan, dùng alias đã xác nhận.
2. Xác định vị trí nguồn và thời điểm cảnh nếu có hồi tưởng hoặc trường hợp đặc biệt.
3. Lấy các quy ước đã xác nhận còn hợp lệ với source/revision và có hiệu lực trong phạm vi đó.
4. Xét ngoại lệ của cảnh nếu có; một ngoại lệ chỉ tác động trong phạm vi được xác nhận.
5. Phát hiện quy ước chồng lấn/mâu thuẫn; báo cần xử lý nếu không xác định được một lựa chọn hợp lệ.
6. Gửi quy ước liên quan cùng căn cứ cần thiết trong ngân sách ngữ cảnh.
7. Ghi dependency phiên bản quy ước vào request/bản nháp để truy vết về sau.

Không chọn trạng thái mới nhất của cả bộ rồi áp cho mọi cảnh. Một hồi tưởng dùng thời điểm cảnh; lời thoại trích lại cần xét người nói/người nghe trong ngữ cảnh được trích.

Nếu thiếu ngữ cảnh tập trước hoặc chưa rõ thời điểm, báo phần thiếu/chưa xác định. Không tự điền quan hệ bằng trạng thái của một tập sau. Khi thời điểm được làm rõ, dùng lại quy ước phù hợp và kiểm tra các đoạn bị ảnh hưởng.

Phần review chất lượng đối chiếu cách xưng hô trong bản dịch với quy ước đang có hiệu lực. Sự khác biệt có căn cứ theo cảnh cần được giải thích; không dùng thay thế từ hàng loạt để sửa mọi đại từ trong toàn chương.

## 9. Lưu lịch sử và xử lý thay đổi quyết định

Đề xuất, bằng chứng, quyết định và quy ước có phiên bản phải được lưu cùng liên kết đến nguồn và revision bản dịch liên quan. Thông tin cần đủ để trả lời: người dùng đã xác nhận điều gì, vì sao, tại mốc nào và dựa trên dữ liệu nào; khuyến nghị của AI được ghi riêng với quyết định duyệt.

Khi quy ước mới bắt đầu, lịch sử phải biểu diễn được thời gian hiệu lực của quy ước trước và quy ước sau. Không xóa mất căn cứ cũ để chỉ giữ một cặp từ hiện tại.

Nếu người dùng sửa mốc, cách xưng hô hoặc phạm vi của một quyết định đã duyệt:

- ghi revision và lý do chỉnh;
- xác định các bản dịch đã dùng phiên bản quy ước cũ;
- đánh dấu phần thực sự bị ảnh hưởng cần kiểm tra/dịch lại theo cơ chế dependency;
- giữ phần ngoài phạm vi không bị thay đổi;
- bảo toàn bản sửa tay và lịch sử, không âm thầm thay toàn bộ văn bản.

Nếu nguồn hoặc revision làm căn cứ bị thay đổi, kiểm tra lại tính hợp lệ của quyết định liên quan. Dữ liệu đã có hash/vị trí nguồn cũ không được tự coi là bằng chứng hợp lệ cho nguồn mới.

Việc lưu và xác nhận cần nhất quán với generation/revision hiện có để tránh trạng thái quy ước đã được công bố nhưng bản dịch/bằng chứng tương ứng chưa được lưu thành công.

## 10. Tiêu chí nghiệm thu

Kiểm thử tự động dùng dữ liệu mẫu tự tạo, offline và deterministic; không gọi provider trả phí.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| A → B đổi xưng hô | B → A và A → C không tự bị thay đổi nếu chưa có quyết định tương ứng |
| Tên thật/tên dịch/biệt danh cùng nhân vật | Tra được cùng quy ước qua alias đã xác nhận |
| Alias hoặc người nghe mơ hồ | Không áp nhầm quy ước; giữ điểm chưa xác định |
| Phát hiện đề xuất xưng hô mới | Lưu hồ sơ chờ; chưa áp dụng vào bản nháp hiện hành hoặc ngữ cảnh chính thức |
| AI đủ căn cứ hoặc tự tin về đề xuất | Vẫn chờ người dùng chốt cách xưng hô, mốc và phạm vi |
| Tỏ tình được chấp nhận và có chính sách dịch phù hợp | Có đề xuất với sự kiện, phản ứng, bằng chứng, lý do chọn từ và mốc rõ |
| Tỏ tình đơn phương hoặc bị từ chối | Không tự kết luận quan hệ hai chiều đã được xác lập |
| Đe dọa chia tay hoặc hiểu lầm | Không mặc định tạo thay đổi lâu dài |
| Một lần nổi giận | Phân biệt ngoại lệ theo cảnh với thay đổi quan hệ lâu dài |
| Quote không có trong nguồn | Kiểm tra cấu trúc/evidence thất bại |
| Quote có thật nhưng không hỗ trợ kết luận | Review đánh dấu chưa đủ căn cứ, chưa xác nhận quy ước |
| Đề xuất không nêu nguyên nhân hoặc phạm vi | Không đủ điều kiện công bố thành quy ước chính thức |
| Mốc nằm giữa lượt hoặc block | Chỉ áp đúng phần sau mốc, giữ phần trước theo quy ước cũ |
| Sang tập mới hoặc mở lại ứng dụng | Quy ước và lịch sử vẫn được dùng đúng phạm vi |
| Hồi tưởng trước mốc thay đổi | Dùng quy ước có hiệu lực ở thời điểm cảnh |
| Hết phạm vi ngoại lệ | Trở về quy ước phù hợp, không lan ngoại lệ sang cả bộ |
| Quy ước chồng lấn hoặc mâu thuẫn | Báo cần xử lý; không chọn ngẫu nhiên một bản ghi |
| Chỉnh một quyết định đã xác nhận | Truy ra phần phụ thuộc cần kiểm tra, giữ lịch sử và bản sửa tay |
| Ba màn Dịch/Review/Ngữ cảnh | Hiển thị nhất quán quy ước đang dùng, candidate và lịch sử |

## 11. Đối chiếu implementation và tài liệu liên quan

Go hiện có sự kiện theo người nói → người nghe, vị trí nguồn, thông tin thời điểm cảnh, bằng chứng và revision. Cơ chế context lọc sự kiện từ các bản dịch được chấp nhận; phần scene hỗ trợ phạm vi thời gian và dữ liệu chưa xác định.

Những thành phần đó là nền tảng tham khảo, chưa đồng nghĩa toàn bộ hồ sơ đề xuất, giao diện bằng chứng, cặp trường xưng/gọi và ngoại lệ trong tài liệu này đã được hiện thực đầy đủ. Khi port Rust cần đặc tả schema mới và cách resolve quy ước rõ ràng, giữ Go hoạt động và bảo toàn dữ liệu workspace mới. Không có yêu cầu migration từ workspace test cũ theo quyết định ngày **30/10/2026**.

- [Đặc tả EPUB đầu vào](EPUB_INPUT_SPEC.md).
- [Đặc tả giai đoạn Dịch](TRANSLATION_STAGE_SPEC.md).
- [Dữ liệu rule, event, evidence và entity](../../internal/translation/domain/types.go).
- [Ngữ cảnh từ revision đã chấp nhận](../../internal/translation/continuity/context.go).
- [ID và alias nhân vật](../../internal/translation/continuity/entities.go).
- [Phạm vi cảnh và thời điểm](../../internal/translation/engine/scene.go).
- [Tập hợp lịch sử và dùng ngữ cảnh khi dịch](../../internal/translation/engine/runner.go).
- [Hợp đồng candidate và review](../../internal/translation/model/contracts.go).

## 12. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [16](DESIGN_DECISIONS_36.md#decision-16) của bản 36 mục áp dụng cho đặc tả này.

### 12.1. Xưng hô theo nhân vật và mốc thời gian — mục 16

Mỗi nhân vật có ID, các tên/bí danh đã xác nhận cùng trỏ tới ID đó. Quy tắc có chiều A→B, cách A tự xưng, cách A gọi B, hoàn cảnh và khoảng hiệu lực. Ghi hai mốc riêng là vị trí cảnh xuất hiện trong sách và thời điểm sự kiện trong truyện. Trong hồi tưởng, tra quy tắc đã có hiệu lực vào thời điểm hồi tưởng, đồng thời lọc những gì độc giả được biết tại vị trí đang dịch.

Chưa rõ người nghe, thời điểm hoặc phản ứng sau lời tỏ tình thì lưu là **Chưa xác định**. Đề xuất đổi “tôi–cậu” sang “anh–em” phải cho thấy nguyên tác trước sự kiện, lời nói/phản ứng, bằng chứng dùng sau đó và phạm vi dự kiến. Người dùng có thể duyệt ngoại lệ chỉ cho một cảnh. Dời mốc hoặc nhập thêm bí danh sẽ xét lại đúng những câu thoại có liên quan thông qua chỉ mục ở mục 09. [Đặc tả xưng hô](ADDRESSING_CONTINUITY_SPEC.md)
