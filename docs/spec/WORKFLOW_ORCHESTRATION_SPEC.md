# Quyết định 10 — Điều phối Dịch–Review và dịch tự động đến hết EPUB

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 9.

Ngày ghi nhận: **28/10/2026**.

Trạng thái: **đã chốt hướng sản phẩm trong trao đổi**. Tài liệu xác định thứ tự điều phối cần triển khai; không khẳng định implementation hiện tại đã đáp ứng. Ba trục trạng thái theo [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md). Hợp đồng hàng đợi, quyền chạy và checkpoint đã chốt tại mục 9.

Quyết định chính: **review ngay sau từng lượt, kiểm tra tổng thể cuối chương, rồi mới chuyển sang chương kế tiếp**.

Bổ sung đã chốt: có lựa chọn **Dịch tự động đến hết EPUB**. Người dùng chọn tập và bắt đầu một lần; tool tự tiếp tục qua các chương đủ điều kiện cho đến hết nội dung cần dịch của EPUB, không yêu cầu chọn/bấm chuyển từng chương.

Cập nhật ngày **30/10/2026**, lựa chọn **13A**: mỗi thời điểm bộ điều phối của tool chỉ cho một tác vụ AI hoạt động; các tác vụ khác xếp hàng. Khi tác vụ tạm dừng hoặc chờ người dùng, có thể chủ động chọn chạy tác vụ độc lập khác theo mục 6.1–6.3. Giai đoạn này không cung cấp chế độ nhiều tác vụ AI chạy song song.

Tài liệu liên kết [đầu vào và chia lượt](EPUB_INPUT_SPEC.md), [năm bước Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [rubric](REVIEW_RUBRIC_SPEC.md), [dữ liệu/trạng thái](DATA_MODEL_STATE_SPEC.md), [duyệt thuật ngữ](SKILLS_GLOSSARY_SPEC.md), [model và ngân sách](MODEL_STRATEGY_SPEC.md), [ảnh và xuất EPUB](IMAGE_EDIT_SPEC.md).

## 1. Phạm vi và thứ tự thực hiện

- Xử lý các chương/phần được giao theo thứ tự nguồn: một/nhóm chương hoặc tự động đến hết EPUB. Lựa chọn nội dung toàn bộ phần chữ hay chỉ truyện theo đặc tả đầu vào quyết định những phần nào cần Dịch–Review và phần nào giữ nguyên; các phần cần dịch đều đi qua điều kiện tương ứng.
- Trong một chương, xử lý các lượt theo thứ tự nội dung. Lượt là phạm vi công việc, không phải một cấp chương/part mới trong sách.
- Sau khi lưu bản nháp hợp lệ của lượt hiện tại, chạy Review của lượt đó trước khi dịch lượt kế tiếp.
- Khi mọi lượt đạt điều kiện cục bộ, thực hiện kiểm tra tổng thể chương trên các revision hiện hành.
- Chỉ tự chuyển sang chương kế tiếp trong phạm vi đã chọn khi chương hiện tại hoàn tất các điều kiện Review và không còn việc chặn cần xử lý.
- Không tự thêm chương ngoài phạm vi được giao. Đọc thêm nguyên tác để xác minh ngữ cảnh không đồng nghĩa bắt đầu dịch một chương chưa được phép chạy.

Một lượt dịch hoặc một lần Review có thể cần nhiều request vì giới hạn dung lượng, retry hoặc các bước kiểm chứng. Số request không quyết định thứ tự chương và không thay cho trạng thái chất lượng của phạm vi nguồn.

### 1.1. Hai cách chọn phạm vi chạy

| Lựa chọn | Phạm vi và cách chạy |
| --- | --- |
| **Dịch phần đã chọn** | Người dùng chọn một hoặc nhiều chương/phần; tool chạy Dịch–Review theo thứ tự và dừng khi hoàn tất phạm vi đã chọn. |
| **Dịch tự động đến hết EPUB** | Người dùng chọn một tập EPUB một lần; tool lập hàng đợi toàn bộ chương/phần văn bản cần dịch của tập, giữ kết quả còn hợp lệ và tự xử lý phần còn thiếu đến hết tập. |

Chế độ tự động bao gồm mở đầu, các chương, phần kết, lời bạt và nội dung liên quan; nếu chọn toàn bộ phần chữ còn gồm bản quyền, giới thiệu tác giả, quảng cáo và các phần văn bản ngoài ảnh còn lại. Nếu chọn chỉ truyện, các trang phụ giữ nguyên theo phân loại đã xác định. Không yêu cầu đánh dấu từng chương. Tài nguyên và phần chưa phân loại vẫn phải được kiểm kê/xử lý, không tự bỏ qua rồi báo toàn EPUB đã hoàn tất.

Lựa chọn nội dung được hiển thị/lưu riêng với lựa chọn chạy một nhóm chương hoặc tự động. Hoàn tất tự động nghĩa là hoàn tất tất cả phần cần dịch theo lựa chọn nội dung; phần giữ nguyên có tiến độ/trạng thái riêng, không tính như đã dịch/Review đạt. Đổi lựa chọn cần xét lại hàng đợi, độ phủ và bản chuẩn bị xuất, giữ những kết quả còn phù hợp.

Trong cả hai lựa chọn, những chương đã nằm trong phạm vi được giao tự chuyển tiếp khi đủ điều kiện. Không tạo một hộp xác nhận hoặc yêu cầu bấm Dịch lần nữa chỉ vì vừa hoàn tất một chương.

### 1.2. Bắt đầu và tiếp tục chế độ tự động

1. Chọn tập EPUB, xem phạm vi và cấu hình model/ngân sách có hiệu lực, rồi bắt đầu tác vụ một lần.
2. Tool xác định thứ tự nội dung từ cây import, kiểm tra tiến độ và tìm phần chưa hoàn tất đầu tiên.
3. Giữ các kết quả đã hoàn tất còn phù hợp; nháp có sẵn được Review tiếp, không tự dịch lại từ đầu.
4. Chạy từng lượt Dịch–Review, kiểm tra toàn chương; đạt điều kiện rồi tự tiếp tục chương kế tiếp.
5. Lặp đến khi mọi chương/phần bắt buộc của tập đã đáp ứng điều kiện, hoặc tác vụ gặp điểm chặn/yêu cầu dừng.

Một tác vụ tự động giữ phạm vi của cùng tập/snapshot, cấu hình model và ngân sách xuyên các chương. Chuyển chương không tạo ngân sách mới, không xóa chi phí đã dùng hoặc lịch sử đợt sửa lỗi. Chế độ tự động không tự mở rộng sang EPUB/tập khác của bộ truyện.

### 1.3. Độc lập với lựa chọn chi phí

Người dùng có thể kết hợp **Dịch tự động đến hết EPUB** với **Đặt hạn mức chi phí** hoặc **Chạy không giới hạn chi phí**. Quyền tự chuyển chương không cấp quyền tự tăng tiền, đổi model, duyệt thuật ngữ hoặc bỏ Review.

Theo lựa chọn 8A ngày **30/10/2026** trong [chiến lược model](MODEL_STRATEGY_SPEC.md), tác vụ mới được điền ngân sách theo thứ tự riêng tác vụ → riêng bộ → mặc định tool và hiển thị trước khi bắt đầu. Ngân sách có hiệu lực được lưu cho toàn tác vụ; tiếp tục/chuyển chương giữ ngân sách và chi phí đã có, không lấy mặc định vừa sửa để tự thay cho tác vụ đang chạy.

## 2. Luồng của một chương

```text
Chương hiện tại
  → chuẩn bị lượt 1 với nguồn và quy ước hợp lệ
  → Dịch → kiểm tra cấu trúc → lưu nháp
  → Review lượt 1
      → có lỗi: sửa đúng phạm vi → kiểm tra/lưu bản mới → Review lại
      → đạt cục bộ: ghi kết quả và dependency
  → chuẩn bị lượt 2 bằng ngữ cảnh còn phù hợp
  → Dịch → kiểm tra cấu trúc → lưu nháp → Review → đạt cục bộ
  → tiếp tục cho đến hết các lượt
  → Review tổng thể chương
      → có lỗi: xử lý theo rubric → Review lại phần bị ảnh hưởng
      → xác lập kết luận cấp chương cho bộ revision hiện hành
  → hoàn tất chương
  → chương kế tiếp được chọn
```

Đây là luồng khi đủ điều kiện tiếp tục. Các nhánh chờ thuật ngữ, xác minh, ngân sách, lỗi model hoặc yêu cầu tạm dừng được mô tả ở mục 6.

Không tạo nháp hàng loạt cho các lượt sau trong cùng chương rồi mới quay lại Review lượt đầu. Không dịch xong cả nhóm hai chương mới bắt đầu Review.

## 3. Review ngay sau từng lượt

Sau khi giai đoạn Dịch lưu nháp, bộ điều phối giao đúng nguồn, revision và phạm vi đó cho Review. Lượt đọc nguyên tác tiếp tục tuân thủ ranh giới dữ liệu đã chốt: dùng nguồn và ngữ cảnh có căn cứ, không lấy bản dịch đang được kiểm tra làm sự thật tham chiếu.

Review gồm kiểm tra độ phủ và đối chiếu ý nghĩa theo hai chiều, cùng những liên hệ cần thiết với phần xung quanh. Nguồn nguyên văn vẫn phải có trong các lượt đối chiếu; ghi chú/tóm tắt không thay nguồn.

Nếu có lỗi đã xác nhận, giao sửa theo rubric M1–M4 và kiểm tra lại bản mới. Nếu chưa đủ căn cứ, mở rộng nguồn hoặc chờ người dùng; chưa đoán để sửa và chưa coi lượt đó đạt.

Kết quả **đạt cục bộ** cho phép sử dụng phần đã được chấp nhận, còn phù hợp làm ngữ cảnh cho lượt sau. Nó chưa chứng minh toàn chương đã đúng. Thuật ngữ, xưng hô và quy ước mới vẫn cần quyết định riêng của người dùng trước khi áp dụng.

Chỉ chuẩn bị lượt dịch kế tiếp sau khi điều kiện của lượt hiện tại đã được giải quyết, để request tiếp theo dùng các quyết định/ngữ cảnh đã lưu mới nhất. Không tự tiếp tục từ bản nháp bị đánh trượt.

## 4. Kiểm tra tổng thể cuối chương

Khi các lượt đã qua Review cục bộ, kiểm tra toàn chương để xác định:

- Mạch sự kiện, nguyên nhân/kết quả, thời gian và điểm nhìn có nhất quán với nguồn.
- Người nói, chủ thể, tên/thuật ngữ và xưng hô có đúng qua các đoạn và các lượt.
- Có nội dung bị bỏ sót, lặp hoặc mâu thuẫn tại điểm nối giữa các lượt hay không.
- Các quy tắc và mốc hiệu lực đã được áp dụng đúng phạm vi.
- Bộ revision đang xét có đủ độ phủ và còn phù hợp với nguồn, mapping, quy tắc/ngữ cảnh hiện hành.

Có thể dùng lại kết quả cục bộ và ghi nhận nguồn còn hợp lệ, nhưng phải kiểm tra các liên hệ cấp chương và dùng nguyên tác khi đối chiếu. Không chỉ cộng số lượt đạt hoặc đọc một bản tóm tắt rồi tự kết luận chương đạt.

Kiểm tra cấp chương có thể được chia thành nhiều request phù hợp hai giới hạn dung lượng. Không bắt model nhận nguyên chương trong một request quá dài.

Nếu phát hiện lỗi, sửa đúng phạm vi đủ để xử lý nguyên nhân và ảnh hưởng. Dịch lại cả chương chỉ theo điều kiện M4 và kiểm chứng bổ sung trong rubric, không dựa riêng vào số lỗi hoặc một điểm tổng.

Sửa nội dung tạo revision mới. Review lại phần sửa, những phần chịu ảnh hưởng và xác lập kết luận cấp chương cho tập hợp revision mới; kết luận của tập hợp cũ chỉ còn là lịch sử. Không bắt dịch lại các đoạn ngoài phạm vi ảnh hưởng.

Nếu kiểm tra tổng thể bác bỏ một kết luận cục bộ đã dùng làm ngữ cảnh, truy dependency để kiểm tra lại các lượt sau bị ảnh hưởng. Giữ nguyên lịch sử và bản sửa tay.

## 5. Điều kiện chuyển sang chương kế tiếp

Chương hiện tại cần có đủ nội dung đã xử lý, Review cục bộ đầy đủ, kiểm tra tổng thể hoàn tất, các vấn đề chặn đã được giải quyết và dữ liệu còn phù hợp. Sau khi lưu kết luận cấp chương, bộ điều phối tự bắt đầu chương tiếp theo trong phạm vi được giao: nhóm đã chọn hoặc hàng đợi đến hết EPUB. Không cần người dùng chọn lại chương tiếp theo.

Quyết định **Người dùng giữ — có ngoại lệ** tiếp tục theo đặc tả Review: ghi lý do/phạm vi và giữ nhãn riêng, không tự biến thành Reviewer đã xác nhận đúng. Ngoại lệ phải do người dùng chủ động xử lý; không bỏ kiểm tra tổng thể hoặc tự duyệt các quy tắc còn tranh chấp để chuyển chương.

“Hai chương mỗi đợt” là nhóm chương được chọn/theo dõi. Trong nhóm vẫn chạy:

```text
Chương 1: từng lượt Dịch–Review → kiểm tra toàn chương → hoàn tất
Chương 2: từng lượt Dịch–Review → kiểm tra toàn chương → hoàn tất
```

Chương 1 còn bị chặn thì chưa tự bắt đầu Dịch Chương 2 trong cùng chuỗi này. Kích thước nhóm không thay đổi các điều kiện Review và không gộp cả nhóm thành một request API.

Trong chế độ tự động đến hết EPUB, mốc hai chương không phải điểm dừng để xin chọn nhóm mới. Tool tiếp tục hàng đợi đến hết tập nếu không có điểm chặn.

## 6. Chờ người dùng, lỗi và công việc độc lập

| Tình huống | Hành vi điều phối |
| --- | --- |
| Thuật ngữ/quy ước mới chưa được duyệt | Giữ ứng viên/hồ sơ đề xuất, chờ quyết định; chưa áp dụng vào bản nháp hiện hành hoặc phần sau phụ thuộc. |
| Người dùng duyệt nhiều thuật ngữ theo 7A | Lưu quyết định riêng cho các mục đã chọn và xác nhận; chuẩn bị lại phần phụ thuộc đủ điều kiện, giữ các mục chưa duyệt ở trạng thái chờ. |
| Chưa đủ căn cứ để kết luận Review | Giữ trạng thái cần xác minh, chỉ rõ nguồn/ngữ cảnh còn thiếu; chưa vượt qua bước Review. |
| Dịch/Review cần tra cứu thêm | Ghi yêu cầu và bước đang chờ, chuyển Research tìm nguồn nội bộ/Internet; tiếp tục đúng bước sau khi căn cứ/quyết định cần thiết đã được xử lý. |
| Lỗi dịch vụ hoặc cấu trúc | Xử lý theo giới hạn 2 lần retry/2 lượt sửa cấu trúc; giữ model đã chọn và hiển thị lỗi. |
| Timeout hoặc đang chờ retry theo 9A | Mặc định thời hạn 10 phút/lần gọi, chờ retry 10/30 giây; hiển thị thời gian/model và cho dừng, kiểm tra lại điều kiện trước khi gửi. |
| Hết 5 vòng sửa–Review cho đợt lỗi | Chuyển người dùng xử lý, giữ bằng chứng/lịch sử; không dùng chuyển bước hoặc đổi phạm vi để xóa bộ đếm. |
| Không đủ ngân sách trong chế độ đặt hạn mức | Chưa gửi request mới của tác vụ, lưu tiến độ và chờ xử lý ngân sách. |
| Chạm trần tổng số lần gọi đã bật | Tiếp nhận/lưu phản hồi đang có; nếu cần thêm model thì chờ trước lần vượt trần. Không tự tăng trần, đổi model hoặc bỏ bước Review. |
| Người dùng tạm dừng/hủy | Giữ dữ liệu đã lưu, dừng lên lịch công việc tiếp theo theo trạng thái tác vụ. |
| Sửa tay hoặc thay nguồn/quy tắc khi model đang chạy | Kiểm tra nền phiên bản khi nhận kết quả, giữ lịch sử và giải quyết xung đột trước khi tiếp tục. |
| File skill đã bật được người dùng sửa rõ ràng, hợp lệ | Tự đồng bộ theo 2B, áp dụng từ lượt tiếp theo phù hợp; xét lại phần thực sự phụ thuộc, không bắt xác nhận mỗi lần lưu. |
| File skill thay đổi nhưng mơ hồ, không hợp lệ hoặc xung đột | Báo đúng phần vướng, chưa tiếp tục phần phụ thuộc; không tự chọn quy tắc thay người dùng. |

Công việc cục bộ độc lập có thể tiếp tục trong lúc tác vụ AI chạy, ví dụ xem nguồn, sửa ảnh bằng công cụ ngoài hoặc duyệt kết quả đã có. Công việc cần gọi AI phải theo giới hạn một tác vụ hoạt động của 13A dưới đây. Không dùng khái niệm độc lập để bỏ qua lượt đang chờ, tự bắt đầu chương sau trong cùng chuỗi Dịch–Review hoặc ghi cạnh tranh vào dữ liệu đang được xử lý.

Các điểm chặn vẫn áp dụng trong chế độ tự động. Sau khi người dùng xử lý và tiếp tục tác vụ, tool dùng lại hàng đợi đã lưu để chạy đến hết EPUB, không bắt chọn lại từng chương. Nếu người dùng đã chủ động tạm dừng/hủy, việc sửa một quy tắc hoặc xử lý lỗi riêng không tự hủy ý định dừng đó.

Theo cơ chế 2B chốt ngày **29/10/2026** trong [đặc tả Skills](SKILLS_GLOSSARY_SPEC.md), request đang chạy vẫn gắn với phiên bản skill đã gửi; kết quả được kiểm tra hiệu lực khi trả về. Trước request tiếp theo chưa gửi, dùng phiên bản hợp lệ mới và chuẩn bị lại phần liên quan nếu cần. Đồng bộ không tự đổi model, làm mới ngân sách/bộ đếm hoặc cho AI tự duyệt một đề xuất chưa được chốt.

Theo [đặc tả Research](RESEARCH_STAGE_SPEC.md) ngày **30/10/2026**, Research là vai trò tra cứu được dùng Internet. Bàn giao sang Research không tự hoàn tất lượt, vượt qua điểm chờ duyệt hoặc bắt đầu chương sau. Cùng tác vụ giữ ngân sách/bộ đếm và lịch sử, luôn hiển thị vai trò/model thực tế của bước đang chạy.

### 6.1. Một tác vụ AI hoạt động tại một thời điểm — 13A

Quyền chạy thuộc bộ điều phối chung, không cấp riêng theo chương, tập, bộ truyện, model hoặc vai trò. TUI và CLI cùng dùng kiểm tra này; việc mở thêm giao diện trong cùng phạm vi điều phối không tạo quyền chạy tác vụ thứ hai. Cơ chế phối hợp giữa các tiến trình/cửa sổ và quyền ghi sẽ được đặc tả kỹ thuật khi triển khai.

Dịch, đọc nguồn/Review, kiểm chứng, Research, retry và các lượt sửa là các bước của tác vụ đang hoạt động. Chuyển vai trò không tạo một tác vụ song song; không khởi chạy Research nền cho tác vụ B trong lúc tác vụ A đang Dịch. Nếu một thao tác tra cứu được giao thành tác vụ riêng, nó cũng phải chờ cùng hàng đợi AI.

Tác vụ giữ quyền chạy qua những bước tự động liên tiếp, các chương thuộc phạm vi đã chọn và khoảng chờ retry 10/30 giây theo 9A. Không luân phiên sang tác vụ khác sau mỗi request hoặc dùng thời gian chờ retry để chạy AI cho tác vụ khác. Người dùng vẫn có thể chủ động tạm dừng để chuyển việc.

Giới hạn này kiểm soát việc tool chủ động điều phối/gửi công việc AI. Request đã timeout hoặc mất kết nối có thể vẫn được provider xử lý; không diễn giải 13A thành bảo đảm provider đã hủy mọi request cũ. Giữ thông tin kết quả/usage chưa rõ và xử lý phản hồi đến muộn theo đặc tả model/dữ liệu.

### 6.2. Hàng đợi và chuyển tác vụ

Chỉ đưa vào hàng đợi tác vụ mà người dùng đã yêu cầu chạy/tiếp tục. Mở workspace, xem một bộ hoặc có EPUB khác trong thư mục không tự tạo tác vụ AI. Hàng đợi các tác vụ khác với hàng đợi chương bên trong một tác vụ tự động đến hết EPUB.

| Tình huống | Hành vi |
| --- | --- |
| Chưa có tác vụ AI hoạt động | Bắt đầu tác vụ đã được yêu cầu nếu đủ cấu hình, dependency, ngân sách và quyền ghi. |
| A đang chạy, người dùng yêu cầu chạy B | Ghi B vào hàng đợi, hiển thị Chờ chạy và tác vụ A đang chiếm quyền chạy; chưa gửi request của B. |
| Bấm chạy lại đúng tác vụ/phạm vi đã chạy hoặc đã xếp hàng | Mở hồ sơ hiện có, không tạo bản sao để chạy lặp hoặc làm mới bộ đếm. |
| A hoàn tất, còn tác vụ đã được yêu cầu trong hàng đợi | Kiểm tra và chạy tác vụ kế tiếp theo thứ tự hàng đợi; không bắt xác nhận lại một yêu cầu chạy đã được giao. |
| A tạm dừng/chờ duyệt/chờ xử lý lỗi hoặc giới hạn | Lưu trạng thái và bước của A. Người dùng có thể chọn chạy B độc lập khi A đã ngừng lên lịch và bàn giao quyền chạy an toàn; không tự chọn tác vụ khác để lách điểm chặn. |
| Người dùng yêu cầu tiếp tục A trong lúc B đang chạy | A chờ quyền chạy, chưa gửi request đồng thời hoặc tự ngắt B. Muốn chuyển ngay cần tạm dừng B theo luồng đã có. |
| B phụ thuộc phần chưa được xác nhận của A | Giữ B chờ đúng dependency, không lấy ứng viên/quy tắc chưa duyệt làm ngữ cảnh chỉ vì B có ID tác vụ khác. |

Trước khi một tác vụ trong hàng đợi thực sự chạy, kiểm tra lại nguồn, sửa tay, quy ước, cấu hình đã lưu và các giới hạn đang áp dụng. Dữ liệu có thể đã đổi trong thời gian chờ; không gửi lại ngữ cảnh chuẩn bị cũ mà chưa đối chiếu. Nếu tác vụ kế tiếp chưa đủ điều kiện, hiển thị lý do và cho người dùng xử lý hoặc chủ động chọn tác vụ độc lập khác.

### 6.3. Tạm dừng, tính độc lập và bảo toàn dữ liệu

Khi chuyển từ A sang B, trước hết lưu bước/phạm vi, ý định dừng/chờ, kết quả đã có, lần gọi đang xử lý hoặc chưa rõ, ngân sách và bộ đếm của A. A phải ngừng gửi công việc mới và lần gọi đang hoạt động phải được xử lý kết thúc tại tool theo luồng phản hồi/lỗi/timeout trước khi B nhận quyền chạy. Chỉ bấm Tạm dừng chưa có nghĩa request tại provider đã bị hủy.

Phản hồi A đến muộn được gắn với đúng A, request và revision, kiểm tra quyền ghi/nền dữ liệu trước khi tiếp nhận; không trộn vào B hoặc tự khởi động lại chuỗi AI của A. Việc nhận một kết quả cũ không cấp thêm quyền điều phối AI.

B độc lập khi công việc của nó không cần dữ liệu/quyết định còn chờ của A và không tranh chấp phạm vi ghi. Hai bộ riêng có thể đáp ứng điều kiện; hai tập của cùng bộ hoặc hai tác vụ cùng dùng quy ước đang tranh chấp không mặc nhiên độc lập. Trong cùng chuỗi truyện, thứ tự lượt/chương và yêu cầu Review vẫn áp dụng; không tách chương sau thành tác vụ mới để vượt điểm chặn.

Người dùng vẫn có thể xem nguồn, mở preview hợp lệ, sửa ảnh bên ngoài hoặc duyệt kết quả đã có. Thao tác thay dữ liệu tiếp tục qua cơ chế quyền ghi, revision và kiểm tra dependency; nếu cần gọi AI để đề xuất lại/kiểm chứng, phần gọi đó phải chờ quyền chạy theo 13A.

Mỗi tác vụ giữ model, ngân sách tiền, trần số lần gọi, usage/dự phòng và các bộ đếm riêng đã lưu. Không chuyển chi phí sang tác vụ khác, không đặt lại số đã dùng khi đổi tác vụ hay mở lại tool. Duyệt thuật ngữ/sửa skill có thể giải quyết một điểm chặn nhưng không tự giành quyền chạy từ B, và không tự tiếp tục A nếu người dùng đã chủ động tạm dừng/hủy. Khi A được tiếp tục, quay lại đúng bước còn thiếu.

## 7. Lưu tiến độ và hiển thị

Lưu chế độ chạy, lựa chọn nội dung toàn bộ phần chữ/chỉ truyện, phân loại phần dịch/giữ nguyên, phạm vi nguồn, hàng đợi chương/phần và vị trí đang xử lý, bước hiện tại, revision đã dịch/review, kết luận cục bộ, tình trạng kiểm tra tổng thể, yêu cầu Research/việc đang chờ và dependency liên quan. Chế độ tự động phải được khôi phục cùng tác vụ khi tiếp tục sau gián đoạn.

Lưu cấu hình thời gian chờ, mốc retry, trần số lần gọi nếu bật và tổng đã dùng theo 9A trong [chiến lược model](MODEL_STRATEGY_SPEC.md). Mặc định không đặt trần số lần gọi; mọi lần gửi Dịch/Review/Research, retry và sửa đều tính vào cùng tác vụ. Chuyển chương/vai trò hoặc tiếp tục không cấp lại trần, không bỏ các lần đã gửi nhưng kết quả/chi phí còn chưa rõ.

Theo **12A** chốt ngày **30/10/2026** trong [đặc tả TUI/CLI](TUI_CLI_SPEC.md), TUI có thao tác Tạm dừng rõ ràng bằng bàn phím và luôn hiển thị model/bước đang chạy. Điều hướng, resize hoặc mở trợ giúp không tự gọi thêm model, hủy hay tiếp tục tác vụ. CLI dùng cùng kiểm tra/quyền và bước điều phối; giới hạn một tác vụ AI hoạt động đã chốt theo **13A**.

Lưu danh sách/thứ tự tác vụ đã được yêu cầu, tác vụ đang giữ quyền chạy, trạng thái chờ và lý do, các ý định tạm dừng/tiếp tục cùng mốc bàn giao. Khi mở lại, đối chiếu trạng thái trước khi cấp quyền chạy; không biến mọi mục trong hàng đợi thành tác vụ đang chạy. Giao diện phân biệt một tác vụ đang chạy, những tác vụ Chờ chạy và những tác vụ chờ người dùng; luôn hiện model của tác vụ thực sự hoạt động, kể cả khi đang xem hồ sơ khác.

| Mốc gián đoạn | Việc cần tiếp tục khi dữ liệu còn phù hợp |
| --- | --- |
| Có ứng viên, chưa đủ điều kiện lưu nháp | Kiểm tra cấu trúc/duyệt quy tắc theo bước còn thiếu. |
| Nháp đã lưu, chưa Review | Review đúng revision đó, chưa dịch lượt kế tiếp. |
| Review cục bộ chưa hoàn tất | Tiếp tục phần Review/sửa còn thiếu, không coi phần không có kết luận là đạt. |
| Một lượt đạt cục bộ | Chuẩn bị lượt kế tiếp nếu còn nội dung. |
| Mọi lượt đạt, chưa kiểm tra tổng thể | Thực hiện kiểm tra tổng thể chương, chưa sang chương tiếp theo. |
| Kiểm tra tổng thể yêu cầu sửa | Tiếp tục phạm vi sửa/Review đã ghi, giữ số vòng và lịch sử. |
| Chương đã hoàn tất và còn phù hợp | Chuyển chương kế tiếp trong phạm vi đã chọn. |

Giao diện phân biệt **đạt cục bộ**, **chờ kiểm tra toàn chương** và **hoàn tất chương**, hiển thị lý do chờ khi có. Trong lúc gọi AI, luôn hiển thị vai trò và tên/ID model theo đặc tả model.

Màn chạy hiển thị chế độ **Tự động đến hết EPUB**, tập/phần hiện tại, tiến độ toàn tập và lý do nếu đang chờ. Không bắt mở màn chọn chương sau mỗi lần chuyển chương; người dùng vẫn có thể tạm dừng/hủy bằng các thao tác của tác vụ.

Không dùng số request hoàn tất làm phần trăm chất lượng chương. Tiến độ phải dựa trên phạm vi nội dung và các bước kiểm tra đã có kết quả hợp lệ; số lượt có thể đổi khi chia lại.

Khi chế độ tự động hoàn tất nội dung chữ cần dịch/review của tập, báo **Hoàn tất Dịch–Review EPUB** và sẵn sàng bước ảnh, preview và xuất theo đặc tả tương ứng. Không đồng nhất trạng thái này với đã chỉnh/dịch mọi chữ trong ảnh hoặc đã tạo file EPUB cuối. Nếu chỉ chạy một nhóm chương, hoàn tất nhóm chưa tự chứng minh toàn tập đủ điều kiện xuất đầy đủ.

Theo [đặc tả đầu ra](EPUB_OUTPUT_SPEC.md) chốt ngày **29/10/2026**, nội dung cần theo dõi còn gồm tiêu đề, nhãn mục lục, chú thích và cách thể hiện ruby; không tính toàn tập hoàn tất chỉ dựa trên các đoạn truyện chính. Nội dung phụ trợ dùng chung cần có mapping/tiến độ tương ứng, không buộc thành chương truyện mới. Những quyết định nghĩa còn vướng được xử lý trong Dịch–Review trước bước xuất, theo cùng quy tắc chờ người dùng và tiếp tục đã chốt.

Điều kiện xuất bổ sung cùng ngày chỉ cho xuất EPUB hoàn chỉnh trong giai đoạn đầu. Hoàn tất nhóm chương không mở quyền xuất bản dở dang; chế độ tự động đến hết EPUB cũng phải đáp ứng đủ điều kiện phần chữ, quyết định và kiểm tra kỹ thuật. Ngoại lệ người dùng đã giữ còn hiệu lực được hiển thị riêng, không tự bỏ qua việc chặn khác.

Quyết định ngày **30/10/2026** thống nhất bước xuất thành EPUB 3 và mỗi lần xuất mới tạo một file phiên bản mới trong thư mục đầu ra, giữ các bản trước. Thông báo xuất thành công phải chỉ đúng tên/số phiên bản và đường dẫn file đã tạo; trạng thái Hoàn tất Dịch–Review chưa đồng nghĩa đã chạy hoặc hoàn tất bước xuất này.

## 8. Tiêu chí nghiệm thu

Kiểm thử triển khai dùng fixture/model giả lập, offline và deterministic; không gọi provider trả phí trong CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Lượt 1 vừa lưu nháp | Bước tiếp theo là Review lượt 1, không gửi dịch lượt 2. |
| Review lượt 1 không đạt | Sửa/xác minh và Review lại trong giới hạn; lượt sau chưa tự chạy. |
| Lượt 1 đạt cục bộ | Lượt 2 được chuẩn bị bằng ngữ cảnh/quy tắc đã lưu còn phù hợp. |
| Mọi lượt đạt cục bộ | Chạy kiểm tra tổng thể, chưa tự báo chương hoàn tất. |
| Lỗi tại điểm nối hoặc mâu thuẫn cấp chương | Có bằng chứng, chọn đúng phạm vi sửa, kiểm tra lại ảnh hưởng và kết luận cho revision mới. |
| Kiểm tra tổng thể còn chặn | Không gửi dịch chương kế tiếp trong cùng chuỗi. |
| Chọn nhóm hai chương | Chương đầu đi đủ luồng trước khi tự bắt đầu chương thứ hai. |
| Chọn tự động đến hết EPUB | Lập phạm vi toàn tập một lần; tự chuyển qua mọi chương đủ điều kiện, không yêu cầu chọn/bấm từng chương. |
| Tự động đã hoàn tất hai chương nhưng tập còn nội dung | Tiếp tục chương kế tiếp, không dừng để yêu cầu chọn nhóm mới. |
| EPUB đã có phần hoàn tất và bản nháp còn chờ | Giữ kết quả còn phù hợp, tiếp tục đúng bước còn thiếu trong hàng đợi. |
| Tự động gặp thuật ngữ mới hoặc lỗi vượt giới hạn | Chờ người dùng đúng điều kiện; chưa tự duyệt, bỏ chương hoặc đổi model. |
| Tiếp tục tác vụ tự động sau khi xử lý điểm chặn | Giữ chế độ/hàng đợi đến hết EPUB, không bắt chọn lại phạm vi. |
| Tự chuyển sang chương mới | Giữ ngân sách toàn tác vụ và lịch sử chi phí/đợt lỗi, không đặt lại từ 0. |
| Chỉ chọn một/nhóm chương | Hoàn tất phạm vi đó thì dừng; không tự bật chế độ toàn EPUB. |
| Hoàn tất tự động Dịch–Review toàn tập | Báo sẵn sàng bước ảnh/xuất; không tự ghi là EPUB cuối đã được tạo. |
| Chờ duyệt thuật ngữ | Không tự dùng phương án tạm hoặc dùng công việc độc lập để vượt điều kiện chuyển lượt/chương. |
| Nháp có sẵn khi resume | Review phần đã lưu thay vì dịch lại hoặc bỏ qua kiểm tra. |
| Resume sau mọi lượt đã đạt | Tiếp tục kiểm tra tổng thể còn thiếu. |
| Sửa tay/thay dependency sau Review | Kết luận cũ không tự gắn với revision mới; phần phụ thuộc được đánh giá lại. |
| Công việc bị dừng vì model/ngân sách | Giữ model, tiến độ, bộ đếm và usage; không tạo chuỗi mới để vượt giới hạn. |
| Lần gọi cuối trong trần trả về nhưng còn cần Review | Lưu bản nháp, chờ đúng bước Review trước lần vượt trần; không báo hoàn tất chỉ vì đã dùng hết số lần cho phép. |
| A đang chạy, người dùng yêu cầu B qua TUI/CLI | B chờ trong hàng đợi, chỉ một tác vụ giữ quyền gửi AI. |
| Bấm chạy lại tác vụ đã xếp hàng | Dùng hồ sơ cũ, không tạo tác vụ/lần gọi trùng. |
| A chuyển Dịch → Review → Research hoặc chờ retry | Giữ cùng tác vụ/quyền chạy, không chạy AI cho B đồng thời. |
| A hoàn tất, B đã được giao chạy | Kiểm tra lại điều kiện rồi chạy B theo hàng đợi, không tự thêm tập/bộ chưa được yêu cầu. |
| A tạm dừng/chờ duyệt, người dùng chọn B độc lập | Lưu/bàn giao đúng trạng thái, B chạy được; A giữ bước chờ và chưa tự tiếp tục. |
| B cần kết quả/quy tắc chưa được xác nhận của A | B vẫn chờ, không vượt dependency bằng cách đổi tác vụ. |
| A còn request đang hoạt động lúc bấm tạm dừng | Chưa giao quyền chạy mới cho B đến khi xử lý kết thúc tại tool; không giả định provider đã hủy. |
| Phản hồi A đến muộn khi B đang chạy | Gắn đúng A/revision/usage, kiểm tra quyền ghi; không trộn dữ liệu hoặc gửi tiếp AI của A. |
| Tiếp tục A hoặc mở lại hàng đợi sau gián đoạn | Giữ thứ tự/ý định/model/chi phí/bộ đếm, không cấp nhiều quyền chạy hoặc làm mới tác vụ. |

## 9. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [28](DESIGN_DECISIONS_36.md#decision-28), [29](DESIGN_DECISIONS_36.md#decision-29) của bản 36 mục áp dụng cho đặc tả này.

### 9.1. Hàng đợi và quyền chạy tác vụ AI — mục 28

Dùng **một bộ điều phối theo hồ sơ người dùng trên máy**, chung cho TUI, CLI và các workspace. SQLite của bộ điều phối lưu thứ tự yêu cầu, ID tác vụ/workspace, chủ thể đang giữ quyền và số hiệu của lần cấp quyền. Một khóa của hệ điều hành bảo đảm chỉ một tiến trình làm bộ điều phối cho hồ sơ đó. Những giao diện còn lại gửi yêu cầu qua bộ điều phối. Ghi workspace có khóa ghi riêng và kiểm tra phiên bản trước commit.

Lấy tác vụ đã được giao theo thứ tự hàng đợi. A hoàn tất thì kiểm tra điều kiện của B rồi mới chạy. A tạm dừng/chờ người dùng thì lưu bước, ngừng lên lịch, xử lý xong lần gọi tại tool và cho người dùng chọn một B độc lập. Kiểm tra phụ thuộc ở mục 09 trước khi bàn giao. Sau crash, đối chiếu khóa hệ điều hành, số hiệu quyền, hàng đợi và sổ attempt trong workspace. Mỗi lần bắt đầu/kết thúc được đối chiếu bằng cùng ID ở hai kho. Phản hồi muộn thuộc A chỉ được ghi nhận theo mục 19. [Lựa chọn 13A](WORKFLOW_ORCHESTRATION_SPEC.md)

### 9.2. Tạm dừng, tiếp tục và khôi phục sau lỗi — mục 29

Ghi checkpoint khi đã chuẩn bị công việc, trước khi gửi, khi nhận đủ phản hồi, sau kiểm tra/commit và khi đổi sang Review, chờ duyệt, retry hoặc hoàn tất. Ghi trước ý định gửi và khoản dự phòng. Lần đã có ý định gửi nhưng bị tắt ứng dụng trước khi biết kết quả có trạng thái **Chưa rõ đã xử lý**. Đối chiếu với provider khi có khả năng tra cứu. Chỉ retry theo điều kiện lỗi tạm thời đã xác định hoặc yêu cầu của người dùng.

Có response đã lưu thì tiếp tục kiểm tra nó. Có nháp đã commit thì Review đúng nháp. Có kết quả Review cục bộ hợp lệ thì tiếp tục đúng lượt hoặc kiểm tra cuối chương. Lưu thời điểm retry sớm nhất. Khi tiếp tục, chờ phần thời gian còn lại và kiểm tra ý định dừng/quyền chạy trước khi gửi. Nếu nguồn hoặc Skill đã đổi, dựng lại request chưa gửi, còn kết quả đã có thì truy dependency để đánh giá phần cần Review lại. Chi phí, số lần thử và các khoản chưa rõ đi theo tác vụ.

### 9.3. Liên kết và yêu cầu triển khai

Chia lượt/đọc nguồn/kiểm tra toàn chương theo [đặc tả Review](REVIEW_STAGE_SPEC.md), mục 13. Bàn giao quyền chạy giữ các bộ đếm, giá và khoản dự phòng theo [chiến lược model](MODEL_STRATEGY_SPEC.md). Dữ liệu được công bố qua generation/transaction trong [đặc tả dữ liệu](DATA_MODEL_STATE_SPEC.md).
