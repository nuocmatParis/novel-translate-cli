# Quyết định 09 — Cấu hình model, hiển thị khi chạy, giới hạn thử lại và chi phí

Cập nhật ngày **04/11/2026** theo [36 quyết định đã chốt](DESIGN_DECISIONS_36.md). Hợp đồng thuộc tài liệu này được ghi đầy đủ tại mục 10.

Ngày ghi nhận: **28/10/2026**.

Trạng thái: **đã chốt các quy tắc dưới đây trong trao đổi**, bao gồm cấu hình model theo vai trò, luôn hiển thị model khi chạy, ngân sách chi phí và lựa chọn 9A về thời gian chờ/thử lại/tổng số lần gọi ngày **30/10/2026**. Đây là đặc tả sản phẩm cần triển khai, chưa phải xác nhận hành vi hiện có. Hợp đồng provider, giá, dự phòng và phản hồi muộn đã chốt tại mục 10. Giao diện theo [đặc tả TUI/CLI](TUI_CLI_SPEC.md).

Tài liệu bổ sung cho [Dịch](TRANSLATION_STAGE_SPEC.md), [Review](REVIEW_STAGE_SPEC.md), [mô hình dữ liệu và trạng thái](DATA_MODEL_STATE_SPEC.md) và [Skills/duyệt thuật ngữ](SKILLS_GLOSSARY_SPEC.md).

## 1. Không tự đổi model

- Tool dùng model người dùng đã chọn cho công việc tương ứng.
- Không có cơ chế tự chuyển model dự phòng khi gặp lỗi, kể cả đã có một danh sách model khác trong cấu hình.
- Chỉ đổi model khi người dùng chủ động chọn. Đổi model không tự làm mất hiệu lực các kết quả đã hoàn thành còn phù hợp.
- Khi đổi model, kiểm tra lại giới hạn dung lượng và chuẩn bị lại phần công việc chưa hoàn thành phù hợp model mới; ghi nhận model thực tế của từng lần gọi.
- Việc đổi model không tự xóa lịch sử, số vòng sửa chất lượng hoặc các lỗi đã được ghi nhận.

Đây là quyết định không tự chuyển model, thay cho đề xuất trước đó về một chế độ fallback tự động có thể bật.

## 2. Ba giới hạn riêng

| Loại xử lý | Giới hạn đã chốt | Khi hết giới hạn |
| --- | --- | --- |
| Thử lại lỗi dịch vụ tạm thời | Tối đa **2 lần thử lại**, ngoài lần gọi đầu, vẫn cùng model | Giữ tiến độ, báo lỗi và chuyển phần bị lỗi sang chờ người dùng xử lý. |
| Sửa phản hồi sai cấu trúc | Tối đa **2 lượt yêu cầu sửa cấu trúc** sau phản hồi đầu, vẫn cùng model | Giữ ứng viên/chẩn đoán, chưa công bố kết quả hợp lệ, chuyển người dùng xử lý. |
| Sửa chất lượng rồi Review lại | Tối đa **5 vòng sửa–Review thất bại**, theo đặc tả Review | Dừng tự sửa và đưa bằng chứng, lịch sử cho người dùng. |

Giới hạn mạng tính cho việc gửi lại cùng một request logic; lượt yêu cầu sửa cấu trúc là một request có nội dung mới để sửa lỗi đã xác định. Mỗi lần thực sự gửi provider đều có bản ghi lần thử và chịu điều kiện chạy/ngân sách đang áp dụng.

Ba bộ đếm không thay thế nhau. Lỗi mạng chưa tạo thành một vòng sửa chất lượng; phản hồi hợp lệ cấu trúc chưa đồng nghĩa bản dịch đúng nghĩa. Review đầu tiên trước khi sửa chưa được tính là một vòng sửa thất bại.

Không tự tạo tác vụ mới, đổi model hoặc chia lại phạm vi chỉ để làm mới bộ đếm. Khi người dùng chủ động xử lý và yêu cầu tiếp tục, ghi nhận quyết định cùng lịch sử và giới hạn cho phần chạy tiếp theo. Hai chế độ ngân sách tác vụ theo mục 6; giới hạn tổng số lần gọi model tùy chọn theo mục 3.3, độc lập với ba giới hạn xử lý lỗi ở trên.

## 3. Phân loại lỗi và hành động

| Tình huống | Hành động |
| --- | --- |
| Lỗi kết nối hoặc dịch vụ được xác định là tạm thời | Báo ngay, chờ một khoảng rồi thử lại cùng model trong giới hạn 2 lần. |
| Hết thời gian chờ một lần gọi | Ghi nhận timeout và tình trạng kết quả/usage chưa rõ; kiểm tra điều kiện thử lại theo mục 3.1, chưa coi là lỗi nghĩa hoặc kết quả hợp lệ. |
| Sai khóa API, không có quyền dùng model hoặc cấu hình không hợp lệ | Dừng phần bị lỗi, nêu điều cần sửa và chờ người dùng. |
| Hết hạn mức tài khoản hoặc điều kiện chạy chưa được đáp ứng | Dừng và báo nguyên nhân; không tự chuyển tài khoản/model hoặc thử liên tục. |
| Không xác định được nguyên nhân | Lưu chẩn đoán và chờ người dùng, không giả định lỗi sẽ tự hết. |
| Request vượt giới hạn dung lượng | Tính lại ngân sách token, giảm phạm vi/chia lại phần chưa hoàn thành theo đặc tả EPUB. |
| Phản hồi bị cắt vì giới hạn đầu ra | Điều chỉnh ngân sách đầu ra hoặc chia nhỏ phần chưa hoàn thành; không gửi nguyên request cũ lặp đi lặp lại. |
| JSON sai, thiếu/trùng ID, marker hoặc cấu trúc inline hỏng | Nêu đúng lỗi để model sửa trong tối đa 2 lượt sửa cấu trúc. |
| Sai nghĩa, thiếu ý, bịa nội dung | Kiểm chứng và sửa theo Review/rubric; dùng model đã chọn cho từng vai trò. |
| Thuật ngữ/quy ước đang chờ duyệt | Giữ trạng thái chờ người dùng; không gọi lại để model tự chọn phương án thay người dùng. |

Không coi mọi lỗi mạng là bằng chứng rằng đoạn nguồn quá dài. Thời gian chờ và thử lại tuân theo lựa chọn 9A dưới đây; việc chờ không được khóa giao diện hoặc ngăn người dùng dừng.

### 3.1. Thời gian tối đa cho một lần gọi — 9A

Chốt ngày **30/10/2026**: mặc định **10 phút cho mỗi lần gọi model**, tính từ lúc gửi đến khi nhận đủ phản hồi. Đây là mặc định của sản phẩm, không phải khẳng định giới hạn của mọi provider. Người dùng có thể tùy chỉnh, chẳng hạn tăng thời gian cho model xử lý chậm; giá trị phải rõ ràng, dương và hữu hạn.

Giới hạn áp dụng cho từng lần thực sự gửi, gồm Dịch, Review, Research và các lần thử lại/sửa. Thời gian chuẩn bị cục bộ, chờ người dùng và khoảng chờ trước retry không nằm trong thời gian của một lần gọi. Nếu phản hồi được nhận từng phần, nhận thêm một phần không tự đặt lại thời hạn tổng; phản hồi dở dang chưa được coi là kết quả hoàn chỉnh.

Khi hết thời gian, lưu lỗi timeout, dữ liệu trung gian nếu có và tình trạng usage/chi phí. Không suy ra provider đã ngừng xử lý hoặc chưa tính phí. Chỉ tự thử lại nếu lỗi được xác định thuộc nhóm tạm thời, còn lượt retry và đáp ứng ngân sách/giới hạn số lần gọi; nguyên nhân chưa rõ tiếp tục theo quy tắc chờ người dùng tại mục 3. Không gọi lại liên tục chỉ vì chưa thấy phản hồi.

Phản hồi đến muộn phải được đối chiếu với lần thử, request logic, revision và bước đang chờ. Giữ lịch sử/usage thực tế, không áp cùng kết quả hai lần hoặc ghi đè bản hiện hành đã thay đổi. Đổi thời gian chờ chỉ áp dụng cho lần chưa gửi; lần đang chạy giữ giá trị đã ghi nhận. Không khởi động lại thời hạn của cùng lần gọi chỉ vì đóng rồi mở ứng dụng.

### 3.2. Khoảng chờ trước khi thử lại — 9A

| Lần thử lại cùng request logic | Khoảng chờ mặc định sau lỗi tạm thời |
| --- | --- |
| Thử lại 1/2 | **10 giây** |
| Thử lại 2/2 | **30 giây** |

Cho phép người dùng tùy chỉnh khoảng chờ bằng giá trị hợp lệ. Nếu provider trả thời điểm/khoảng chờ hợp lệ dài hơn, không gửi sớm hơn yêu cầu đó; hiển thị thời gian thực tế đang chờ và lý do. Chi tiết đọc thông tin chờ của từng provider thuộc adapter, không thay giới hạn tối đa 2 lần thử lại hoặc model đang dùng.

Trước mỗi retry, kiểm tra lại ý định dừng, nguồn/quy tắc và phiên bản cần dùng, ngân sách tiền, trần số lần gọi nếu bật và số lượt retry còn lại. Lưu số lần đã thử cùng mốc chờ để tiếp tục đúng sau gián đoạn, không đặt lại chuỗi 10/30 giây như một lỗi mới. Lượt sửa cấu trúc/chất lượng là công việc riêng, không mặc định áp khoảng chờ retry dịch vụ khi không có lỗi dịch vụ.

Giao diện hiển thị tên model, bước hiện tại, thời gian đã chạy/thời hạn của lần gọi, lỗi và đếm ngược retry khi đang chờ. Người dùng có thể dừng trong thời gian này; không khóa giao diện hoặc tự chuyển model.

### 3.3. Giới hạn tổng số lần gọi model của tác vụ — 9A

Mặc định **không đặt trần tổng số lần gọi model**. Người dùng có thể bật và đặt một số nguyên dương cho tác vụ; đây là lựa chọn riêng với đặt hạn mức tiền hoặc chạy không giới hạn chi phí.

Một lần thực sự gửi yêu cầu đến model được tính một lần: bao gồm Dịch, đọc nguồn/Review, kiểm chứng, Research, retry, sửa cấu trúc và sửa chất lượng. Request đã gửi nhưng bị lỗi/timeout vẫn được tính; việc dựng ngữ cảnh hoặc kiểm tra cục bộ chưa gửi không được tính. Bộ đếm này đo lần gọi model, không phải số chương, lượt dịch logic hoặc số trang web Research đọc; giới hạn riêng của công cụ tra cứu được thiết kế trong đặc tả Research.

Nếu đặt trần N, chỉ gửi khi lần gửi tiếp theo không làm tổng vượt N. Lần thứ N đã gửi vẫn được tiếp nhận, kiểm tra và lưu kết quả; chỉ chặn khi cần gửi thêm lần N+1. Tác vụ có thể hoàn tất nếu kết quả lần N đã đủ và không cần thêm model. Nếu còn công việc cần model, lưu đúng bước đang chờ và báo **Chạm giới hạn số lần gọi**, không giả thành lỗi chất lượng hoặc thiếu tiền.

Lưu trạng thái bật/tắt trần, giá trị, tổng lần đã gửi, lần đang chờ/chưa rõ và lịch sử điều chỉnh cùng tác vụ. Ghi nhận việc gửi đủ để khôi phục không làm mất một lần gọi khi ứng dụng dừng; nếu chưa xác định đã gửi hay chưa, giữ thông tin để đối chiếu thay vì mặc định chưa sử dụng. Chuyển chương/vai trò, mở lại, đổi model hoặc chia lại phạm vi không làm mới bộ đếm.

Khi chạm trần, người dùng có thể chủ động nâng trần, tắt trần hoặc để sau; giữ tổng đã dùng và lịch sử. Tool không tự nâng/tắt trần hoặc tạo tác vụ khác để vượt giới hạn. Chạy không giới hạn chi phí không tự tắt trần số lần gọi đã đặt; không đặt trần số lần gọi cũng không bỏ ngân sách tiền hay giới hạn 2/2/5. Cấu hình có hiệu lực và số đã dùng/phần còn lại phải hiển thị trên màn tác vụ.

## 4. Thông báo và thao tác của người dùng

Khi phát hiện lỗi, màn tác vụ cần hiển thị ngay:

- Model đang dùng và bước đang chạy.
- Tập/chương/phạm vi bị ảnh hưởng.
- Nguyên nhân đã xác định hoặc trạng thái chưa rõ nguyên nhân.
- Số lần thử lại hoặc số lượt sửa cấu trúc đã dùng.
- Thời gian đã chờ/thời hạn của lần gọi, đếm ngược retry và tổng số lần gọi/trần nếu được bật.
- Tool đang chờ thử lại hay đã chuyển sang chờ người dùng.
- Tiến độ đã lưu và công việc sẽ tiếp tục nếu thử lại thành công.

Ví dụ lỗi tạm thời:

```text
Gọi lần đầu → lỗi tạm thời → thông báo
  → chờ 10 giây mặc định → thử lại 1/2 cùng model
  → vẫn lỗi → thông báo
  → chờ 30 giây mặc định → thử lại 2/2 cùng model
  → vẫn lỗi → chờ người dùng xử lý
```

Người dùng có thể dừng tự thử lại trong lúc chờ. Khi cần can thiệp, các thao tác gồm thử lại cùng model, sửa cấu hình, chủ động chọn model khác hoặc để sau. Phím điều hướng chung theo [đặc tả TUI/CLI](TUI_CLI_SPEC.md); dùng `p` tạm dừng và `r` tiếp tục ngoài ô nhập khi thao tác có hiệu lực, cùng các control bàn phím.

Yêu cầu dừng ngăn tool lên lịch các lần gọi tiếp theo và giữ dữ liệu đã lưu. Không mặc định provider đã hủy hoặc chưa tính phí một request trước đó chỉ vì người dùng dừng tại ứng dụng.

## 5. Lưu tiến độ, lịch sử và thông tin sử dụng

Mỗi lần gọi cần ghi tác vụ, phạm vi, mục đích, model thực tế, thời điểm, kết quả/lỗi và usage nếu có. Không đưa khóa API vào log hoặc hồ sơ.

Thử lại chỉ xử lý phần chưa hoàn thành. Giữ các bản nháp/kết quả đã lưu còn phù hợp, không chạy lại cả chương chỉ vì một request lỗi.

Khi mở lại ứng dụng, phục hồi cả số lần đã thử và trạng thái đang chờ. Không tự coi lần mở lại là quyền bắt đầu một chuỗi retry mới.

Nếu mất kết nối sau khi gửi, provider có thể đã xử lý request. Usage/chi phí chưa biết phải hiển thị chưa xác định; không ghi thành 0 và không cam kết request chỉ bị tính tiền một lần. Lần thử lại được ghi nhận riêng, kể cả khi nội dung giống lần trước.

Kết quả trả về sau khi nguồn, revision hoặc quyết định liên quan đã đổi phải được xử lý theo cơ chế kiểm tra nền/xung đột, không tự ghi đè hiện hành.

## 6. Hai lựa chọn ngân sách chi phí

### 6.1. Lựa chọn áp dụng cho tác vụ

Người dùng có hai lựa chọn:

| Lựa chọn | Hành vi |
| --- | --- |
| **Đặt hạn mức chi phí** | Người dùng đặt số tiền cho tác vụ; tool kiểm tra ngân sách trước mỗi lần gọi và tạm dừng khi không đủ ngân sách cho request tiếp theo. |
| **Chạy không giới hạn chi phí** | Không chặn request vì tổng chi phí của tác vụ; vẫn theo dõi usage/chi phí và tuân thủ các giới hạn xử lý lỗi, chất lượng, dung lượng và quyết định người dùng. |

Phạm vi tác vụ có thể là một chương, nhiều chương hoặc một tập được chọn. Hạn mức bao gồm mọi lần gọi trong tác vụ: Dịch, đọc nguồn, Review, kiểm chứng, hỗ trợ tra cứu/thuật ngữ, retry và sửa cấu trúc/chất lượng. Không chỉ tính lần tạo bản dịch đầu tiên.

Chế độ [Dịch tự động đến hết EPUB](WORKFLOW_ORCHESTRATION_SPEC.md) dùng cùng ngân sách tác vụ xuyên các chương, không cấp hạn mức mới mỗi lần chuyển chương. Người dùng có thể kết hợp chế độ tự động với một trong hai lựa chọn chi phí; tự chuyển chương không cho phép tự nâng hạn mức, đổi model hoặc bỏ các điểm chờ duyệt.

Màn chuẩn bị/chạy cần hiển thị rõ chế độ đang dùng và số tiền nếu có hạn mức. Mặc định và kế thừa cấu hình theo lựa chọn 8A tại mục 6.5; không tự hiểu một giá trị hạn mức bị thiếu hoặc không hợp lệ là người dùng đã chọn chạy không giới hạn.

### 6.2. Kiểm tra trước khi gửi request

Trong chế độ đặt hạn mức, xét tổng sau trước mỗi lần gửi, kể cả retry:

```text
Chi phí đã ghi nhận
  + khoản dự phòng cho request đang chạy/chưa rõ kết quả
  + khoản dự trù cho request tiếp theo
  <= hạn mức người dùng đã đặt
```

Không đủ ngân sách thì chưa gửi request mới; lưu tiến độ và báo **Chờ xử lý ngân sách**. Không tự nâng hạn mức, bỏ giới hạn hoặc chuyển sang model rẻ hơn. Người dùng có thể nâng hạn mức, chủ động chọn chạy không giới hạn chi phí hoặc để sau.

Khoản dự phòng phải được cập nhật khi nhận usage/kết quả để tránh tính trùng với chi phí đã ghi nhận. Request đang chạy không được tính như chưa dùng ngân sách; request mất kết nối không tự được bỏ khoản dự phòng chỉ vì chưa nhận được usage.

Nếu thiếu dữ liệu đến mức không thể kiểm tra ngân sách bằng một ước tính có căn cứ, chế độ đặt hạn mức tạm dừng và nêu phần thiếu. Việc chuyển sang không giới hạn phải do người dùng chủ động chọn; không tự đổi chế độ để tiếp tục.

### 6.3. Theo dõi và hiển thị chi phí

Trước khi chạy, hiển thị ước tính ban đầu khi đủ dữ liệu. Khi chạy, ghi nhận usage và tách các nhóm công việc để người dùng biết chi phí đến từ đâu. Ước tính ban đầu không phải giá cố định cho cả tác vụ, vì số lượt sửa/review có thể thay đổi.

Phân biệt usage provider trả về, chi phí ước tính từ usage/đơn giá, phần dự phòng và khoản chưa xác định. Thiếu usage hoặc đơn giá không được hiển thị thành 0. Trong chế độ không giới hạn, việc thiếu dữ liệu chi phí được báo rõ nhưng không tự tạo một hạn mức tiền để chặn tác vụ.

Hạn mức trong tool kiểm soát việc gửi request mới dựa trên thông tin hiện có. Request đã gửi có thể vẫn được xử lý hoặc tính phí sau khi tool tạm dừng; số tiền ước tính không phải bảo đảm tuyệt đối về hóa đơn của provider.

### 6.4. Lưu và tiếp tục

Lưu lựa chọn ngân sách, hạn mức nếu có, các khoản đã ghi nhận/dự phòng/chưa xác định và lịch sử thay đổi. Đóng/mở ứng dụng hoặc tiếp tục cùng tác vụ không làm số đã dùng về 0. Đổi model hay thay chế độ chi phí cũng không xóa lịch sử sử dụng.

Tạm dừng vì ngân sách vẫn giữ các kết quả đã lưu còn phù hợp. Nếu request đã gửi trả về sau đó, tiếp nhận kết quả/usage và kiểm tra nền phiên bản theo quy tắc hiện có; chưa tự gửi thêm request mới khi tác vụ đang chờ xử lý ngân sách.

**Cả hai chế độ đều giữ giới hạn 2 lần retry dịch vụ, 2 lượt sửa cấu trúc và 5 vòng sửa–Review**, không tự đổi model, không bỏ qua bước duyệt thuật ngữ và không vượt giới hạn của provider/model. Chạy không giới hạn chi phí không làm các vòng xử lý lỗi trở thành vô hạn.

### 6.5. Lưu ngân sách mặc định cho tác vụ mới — 8A

Chốt ngày **30/10/2026**: cho phép lưu ngân sách mặc định của tool và cấu hình riêng cho từng bộ. Khi tạo tác vụ mới, điền sẵn cấu hình có hiệu lực để người dùng xem và có thể sửa trước khi bắt đầu; không bắt chọn lại chế độ chi phí ở mỗi chương hoặc mỗi tác vụ nếu đã có mặc định hợp lệ.

Thứ tự xác định cấu hình cho tác vụ mới:

1. Lựa chọn người dùng đặt riêng cho tác vụ đó, nếu có.
2. Cấu hình ngân sách riêng của bộ, nếu đã thiết lập.
3. Cấu hình ngân sách mặc định của tool.

Một cấu hình ngân sách gồm chế độ **Đặt hạn mức chi phí** kèm số tiền hợp lệ hoặc chế độ **Chạy không giới hạn chi phí** được chọn rõ ràng. Kế thừa cả lựa chọn ngân sách; không ghép chế độ ở một cấp với số tiền không rõ nguồn ở cấp khác. Bộ chưa đặt cấu hình riêng thì kế thừa mặc định của tool; người dùng có thể chủ động bỏ cấu hình riêng để trở lại kế thừa.

Lần thiết lập đầu tiên phải có lựa chọn rõ ràng của người dùng giữa hai chế độ. Chưa có cấu hình hợp lệ thì yêu cầu hoàn tất lựa chọn trước khi gửi request. Cấu hình đã đặt nhưng thiếu/sai số tiền không được diễn giải thành không giới hạn, hoặc âm thầm bỏ qua để dùng một cấu hình khác; hiển thị chỗ cần sửa. Chính sách này không tự chọn một số tiền mặc định thay người dùng.

Màn chuẩn bị cho biết chế độ, hạn mức nếu có và nguồn cấu hình: mặc định của tool, riêng của bộ hay riêng tác vụ. Sửa cho tác vụ hiện tại chỉ thay tác vụ đó; chỉ cập nhật mặc định chung/riêng của bộ khi người dùng chủ động chọn lưu ở cấp tương ứng. Bắt đầu tác vụ dùng lựa chọn đang hiển thị, không thêm một bước bắt xác nhận lại ngân sách đã hợp lệ.

Lưu cấu hình ngân sách thực tế và nguồn kế thừa vào hồ sơ khi tạo tác vụ. Tác vụ đang chạy, đang tạm dừng hoặc được tiếp tục dùng cấu hình đã lưu; không đọc lại mặc định rồi tự đổi hạn mức/chế độ. Thay mặc định chỉ tác động đến các tác vụ mới. Muốn đổi ngân sách của một tác vụ đã có phải thao tác rõ trên tác vụ đó và giữ lịch sử thay đổi theo mục 6.4.

Trong chế độ tự động đến hết EPUB, cấu hình được giữ cho toàn tác vụ, không cấp lại hạn mức khi sang chương, chuyển Dịch/Review/Research hoặc khởi động lại tool. Số tiền đã dùng, dự phòng, khoản chưa xác định và các bộ đếm vẫn được giữ khi đổi chế độ hoặc nâng hạn mức. Lưu cấu hình đã chốt và sổ usage trong SQLite của bộ. Hợp đồng chi phí tại mục 10 và màn Tác vụ trong đặc tả TUI/CLI xác định cách trình bày.

## 7. Cấu hình model theo vai trò

### 7.1. Ba vai trò

Cập nhật ngày **30/10/2026**: gọi vai trò hỗ trợ thuật ngữ/tra cứu là **Research** theo [đặc tả Research](RESEARCH_STAGE_SPEC.md). Research được dùng Internet khi cần; Dịch/Review bàn giao yêu cầu tra cứu cho vai trò này. Đây vẫn là ba vai trò đã có, không thêm model bắt buộc thứ tư.

| Vai trò | Model dùng cho công việc |
| --- | --- |
| Dịch | Model người dùng chọn để tạo bản dịch và sửa theo yêu cầu Review. |
| Review | Model người dùng chọn để đọc nguyên tác, đối chiếu bản dịch và kiểm chứng nhận định. |
| Research — tra cứu và hỗ trợ thuật ngữ | Mặc định dùng cấu hình model Review của cùng tác vụ; người dùng có thể chọn model riêng. Research thực hiện tra cứu nội bộ/Internet và trả bằng chứng. |

Màn cấu hình ban đầu cần xác định model Dịch và Review; Research có lựa chọn **Dùng model Review** để không bắt người dùng cấu hình thêm một model. Các vai trò có thể dùng cùng model hoặc các model khác nhau. Quyền/công cụ tra cứu thuộc bước Research, không suy ra model Dịch hoặc Review đang tự dùng Internet vì trùng tên model.

Dù cùng model, từng vai trò và từng bước vẫn nhận đúng nguồn, ngữ cảnh và hợp đồng đầu ra của nó. Lượt đọc nguyên tác trong Review tiếp tục tách khỏi lượt đối chiếu bản dịch. Đề xuất thuật ngữ vẫn phải chờ người dùng chốt; chọn model cho vai trò hỗ trợ không thay thế quyền duyệt hoặc chính sách tra cứu nguồn.

Kết nối API được đăng ký một lần rồi chọn cho từng vai trò. Các vai trò có thể dùng chung hoặc khác kết nối; không bắt nhập lại khóa cho mỗi chương và không đưa khóa vào màn trạng thái hoặc lịch sử công khai.

Theo lựa chọn **11A** chốt ngày **30/10/2026** trong [đặc tả key và preview](SECURITY_SPEC.md), key được lưu qua kho thông tin xác thực của hệ điều hành; workspace chỉ tham chiếu kết nối. CLI có thể lấy key từ biến môi trường được chọn rõ ràng, không tự lưu giá trị đó vào workspace/`.env`. Không lấy được key thì báo đúng phần cấu hình cần xử lý; không tự đổi nguồn key, tài khoản hoặc model. Log, prompt, hồ sơ tác vụ, preview và EPUB xuất không chứa giá trị key.

### 7.2. Mặc định của tool và cấu hình riêng của bộ

- **Mặc định của tool:** cấu hình model dùng cho những bộ/tác vụ chưa có lựa chọn riêng.
- **Riêng từng bộ:** ghi đè những vai trò người dùng muốn đổi; các mục còn lại kế thừa mặc định.
- **Tác vụ:** trước khi chạy, hiển thị và lưu cấu hình có hiệu lực sau khi kết hợp hai cấp trên.

Tác vụ đang chạy hoặc tạm dừng giữ cấu hình đã lưu. Sửa mặc định của tool hoặc cấu hình của bộ cho công việc mới không âm thầm đổi model của tác vụ đã tồn tại.

### 7.3. Người dùng chủ động đổi model trong tác vụ

Khi người dùng chọn đổi model cho tác vụ hiện tại:

1. Ghi nhận vai trò, model cũ/mới và thời điểm thay đổi.
2. Áp dụng từ request chưa gửi tiếp theo; request đang chạy vẫn được ghi nhận với model đã dùng lúc gửi.
3. Kiểm tra lại giới hạn token, cách chia phần chưa hoàn thành và dự trù chi phí trước khi gửi tiếp.
4. Giữ các kết quả đã hoàn thành còn phù hợp, lịch sử lần thử, chi phí và số vòng sửa–Review.

Nếu Research đang chọn **Dùng model Review**, việc người dùng đổi model Review cho tác vụ cũng cập nhật lựa chọn kế thừa cho các request Research chưa gửi. Vai trò đã chọn model riêng giữ lựa chọn riêng đó. Đây là thay đổi do người dùng chủ động, không phải chuyển model dự phòng khi lỗi.

## 8. Luôn hiển thị tên model khi chạy

**Trong lúc có request model đang chạy, giao diện phải luôn hiển thị tên/ID model của request đó cùng vai trò.** Không chỉ hiển thị trong màn cấu hình, log hoặc sau khi có lỗi.

Màn tác vụ giữ thông tin này ở thanh trạng thái/vùng cố định, không mất khi cuộn nội dung. Nếu người dùng chuyển màn trong TUI khi tác vụ vẫn chạy, phần trạng thái tác vụ đang hoạt động vẫn cho biết vai trò và model.

Ví dụ tên model minh họa:

```text
Đang dịch           | Model: A | Chương 2 · Lượt 3
Đang đọc nguyên tác | Model: B | Chương 2 · Review
Đang Review         | Model: B | Chương 2 · Đối chiếu
Research            | Model: B | Chương 2 · Tra cứu thuật ngữ ở b042
```

Các dòng trên minh họa những bước khác nhau của tác vụ. Theo **13A** chốt ngày **30/10/2026** trong [đặc tả điều phối](WORKFLOW_ORCHESTRATION_SPEC.md), Dịch/Review/Research chạy tuần tự trong một tác vụ AI hoạt động; tác vụ khác xếp hàng. Thông tin hiển thị cập nhật theo request thực sự đang chạy, không lấy một tên model mặc định chung cho mọi vai trò.

Các trạng thái liên quan cũng phải rõ:

- Retry hoặc sửa cấu trúc: tiếp tục hiển thị model và số lần đang xử lý.
- Chờ người dùng, tạm dừng hoặc bước lưu/kiểm tra cục bộ: giữ thông tin model liên quan nhưng ghi đúng trạng thái, không báo AI đang chạy khi không có request hoạt động.
- Đã chọn model mới nhưng request cũ còn chạy: vẫn hiển thị model cũ cho request đó, phân biệt với model sẽ dùng cho các request tiếp theo.
- Có tác vụ khác trong hàng đợi hoặc đang mở hồ sơ khác: vẫn hiển thị model/vai trò của tác vụ thực sự hoạt động, phân biệt với cấu hình của tác vụ đang chờ.

Tên/ID phải lấy từ cấu hình có hiệu lực của lần gọi. Nếu provider trả thêm định danh model cụ thể, lưu và hiển thị phân biệt với tên/alias đã yêu cầu khi cần; không đoán một phiên bản model mà provider không cung cấp. Khi cần phân biệt các model trùng tên ở nhiều kết nối, hiển thị thêm tên provider/kết nối, không hiển thị khóa bí mật.

CLI cần ghi vai trò và model khi bắt đầu lượt gọi, đồng thời có thông tin tương ứng trong dữ liệu tiến độ/lịch sử; không phụ thuộc riêng vào giao diện TUI.

## 9. Tiêu chí nghiệm thu

Các kiểm thử triển khai dùng model/provider giả lập, offline và deterministic; không gọi provider trả phí trong CI.

| Tình huống | Kết quả cần chứng minh |
| --- | --- |
| Model đã chọn gặp lỗi | Giữ nguyên model và báo lỗi; không tự gọi model khác. |
| Lỗi tạm thời lặp lại | Tối đa lần gọi đầu cộng 2 retry cho request đó, sau đó chờ người dùng. |
| Dùng mặc định 9A | Mỗi lần gọi có thời hạn tổng 10 phút; retry tạm thời chờ 10 rồi 30 giây, vẫn cùng model. |
| Nhận phản hồi từng phần hoặc thay cấu hình trong khi gọi | Không tự đặt lại thời hạn tổng của lần đang chạy; cấu hình mới áp dụng cho lần chưa gửi. |
| Provider yêu cầu chờ lâu hơn | Tôn trọng thời gian hợp lệ dài hơn, hiển thị đếm ngược/lý do và cho phép dừng. |
| Timeout hoặc phản hồi đến muộn | Giữ lịch sử/usage chưa rõ, đối chiếu lần thử và phiên bản; không công bố phản hồi dở hoặc áp kết quả hai lần. |
| Không bật trần số lần gọi | Không chặn theo tổng số lần gọi; ngân sách tiền và các giới hạn 2/2/5 vẫn áp dụng. |
| Bật trần N, lần N vừa trả về | Kiểm tra/lưu kết quả; hoàn tất nếu đã đủ, nếu cần model tiếp thì chặn trước N+1 và lưu đúng bước. |
| Chuyển Dịch/Review/Research hoặc retry/sửa | Mọi lần thực sự gửi đều vào cùng bộ đếm tác vụ, kể cả lần lỗi/timeout. |
| Tiếp tục tác vụ hoặc chủ động nâng/tắt trần | Giữ tổng đã dùng/lần chưa rõ và lịch sử, không làm mới bộ đếm. |
| Sai xác thực, hết hạn mức hoặc lỗi chưa rõ | Dừng đúng nguyên nhân, không retry liên tục. |
| Phản hồi sai cấu trúc liên tiếp | Tối đa 2 lượt sửa cấu trúc; không đánh dấu kết quả lỗi là hợp lệ. |
| Phản hồi bị cắt | Kiểm tra dung lượng/phạm vi trước khi gửi tiếp, giữ coverage nguồn. |
| Người dùng dừng trong khoảng chờ | Không lên lịch retry mới; tiến độ đã lưu được giữ. |
| Đóng rồi mở lại giữa chuỗi retry | Phục hồi bộ đếm/lịch sử, không bắt đầu lại từ 0. |
| Mất kết nối, chưa có usage | Hiển thị chưa xác định, không tự tính chi phí bằng 0. |
| Đang chờ duyệt thuật ngữ | Không retry để tự quyết cách dịch. |
| Đổi model theo lựa chọn người dùng | Chuẩn bị lại phần chưa hoàn thành, lưu model mới, giữ lịch sử và số vòng chất lượng. |
| Chọn đặt hạn mức chi phí | Tính cả Dịch, Review, hỗ trợ thuật ngữ và mọi lần thử/sửa trong tác vụ. |
| Lần thiết lập đầu chưa chọn ngân sách | Yêu cầu chọn rõ hạn mức hoặc không giới hạn trước request; không suy từ giá trị thiếu thành không giới hạn. |
| Tạo tác vụ mới có mặc định chung/riêng | Điền cấu hình theo thứ tự riêng tác vụ → riêng bộ → mặc định tool; hiển thị chế độ, số tiền và nguồn cấu hình. |
| Cấu hình ngân sách được đặt nhưng không hợp lệ | Báo phần cần sửa, không tự bỏ qua cấu hình lỗi hoặc chuyển sang không giới hạn. |
| Sửa ngân sách chỉ cho tác vụ mới | Không thay mặc định của bộ/tool nếu người dùng chưa chọn lưu mặc định. |
| Đổi mặc định khi tác vụ đã tồn tại | Tác vụ giữ ngân sách đã lưu; tiếp tục không đặt lại số đã dùng/dự phòng/bộ đếm. |
| Request tiếp theo không còn đủ ngân sách | Chưa gửi, giữ tiến độ và báo chờ xử lý; không tự đổi model/chế độ hoặc nâng hạn mức. |
| Có request đang chạy hoặc chưa rõ kết quả | Giữ khoản dự phòng có căn cứ; không bỏ qua khoản này hoặc tính trùng khi usage đã nhận. |
| Không đủ dữ liệu kiểm tra hạn mức | Báo khoản chưa xác định và tạm dừng trong chế độ đặt hạn mức; không ghi chi phí thành 0. |
| Chọn chạy không giới hạn chi phí | Tiếp tục theo workflow mà không chặn theo tổng tiền; vẫn theo dõi usage và giữ các giới hạn 2/2/5. |
| Tiếp tục tác vụ hoặc chủ động đổi chế độ | Giữ chi phí, bộ đếm và lịch sử đã lưu, không bắt đầu lại từ 0. |
| Request trả về khi đang chờ ngân sách | Lưu/đối chiếu kết quả và usage, không tự gửi request kế tiếp. |
| Dùng cùng model cho các vai trò | Ngữ cảnh và yêu cầu mỗi vai trò/bước vẫn tách đúng theo đặc tả. |
| Hỗ trợ thuật ngữ dùng model Review | Kế thừa cấu hình Review của tác vụ, không bắt chọn model thứ ba. |
| Research tra Internet hoặc dùng cùng model với Review | Vai trò/request/lịch sử và quyền công cụ rõ ràng, hiển thị Research cùng model thực tế; kết quả không tự duyệt thuật ngữ. |
| Bộ có cấu hình ghi đè một vai trò | Hiển thị đúng cấu hình có hiệu lực trước khi chạy; các vai trò khác kế thừa mặc định. |
| Sửa cấu hình mặc định khi tác vụ đang chạy/tạm dừng | Tác vụ giữ cấu hình đã lưu cho đến khi người dùng chủ động đổi cho tác vụ đó. |
| Request đang chạy hoặc người dùng cuộn/chuyển màn | TUI vẫn hiển thị tên/ID model và vai trò của công việc đang hoạt động. |
| Chuyển từ Dịch sang Review hoặc hỗ trợ thuật ngữ | Tên model thay đổi đúng theo request và vai trò, không giữ nhãn model của bước trước. |
| Chọn model mới khi request cũ chưa trả về | Hiển thị đúng model của request cũ; model mới chỉ áp dụng cho request chưa gửi. |
| Tạm dừng, chờ duyệt hoặc xử lý cục bộ | Trạng thái phản ánh không có request đang chạy, thông tin model không làm hiểu nhầm AI vẫn hoạt động. |
| Xem lịch sử hoặc chạy bằng CLI | Truy được vai trò/model của từng lần gọi, không lộ khóa API. |
| Chuyển giữa các tác vụ theo 13A | Mỗi tác vụ giữ model/chi phí/dự phòng/bộ đếm riêng; chỉ tác vụ có quyền chạy được gửi AI, hiển thị đúng model thực tế. |

## 10. Hợp đồng đã chốt ngày 04/11/2026

Các quyết định [17](DESIGN_DECISIONS_36.md#decision-17), [18](DESIGN_DECISIONS_36.md#decision-18), [19](DESIGN_DECISIONS_36.md#decision-19) của bản 36 mục áp dụng cho đặc tả này.

### 10.1. Hợp đồng kết nối model/provider — mục 17

Dùng một giao diện cho ba vai trò. Adapter mô tả khả năng model, gửi request và trả các sự kiện bắt đầu, phần nội dung, hoàn tất hoặc lỗi. Phản hồi một lần và phản hồi dạng luồng đều kết thúc bằng cùng một hồ sơ, gồm ID lần gọi, alias đã yêu cầu, ID thực tế nếu provider trả, lý do kết thúc, nội dung, usage từng loại, thời gian chờ dịch vụ và lỗi đã phân loại. Phản hồi dở được lưu riêng để chẩn đoán.

Giới hạn context/output và bộ đếm token lấy từ khả năng provider hoặc hồ sơ model có nguồn/phiên bản. Thiếu giới hạn cần để gửi an toàn thì yêu cầu bổ sung. Trường usage, giá và model thực tế chưa biết có trạng thái riêng. TUI hiện model đã yêu cầu, bổ sung định danh thực tế khi biết. Adapter chỉ dùng kết nối đã chọn. Đổi key thay bí mật ở kho hệ điều hành cho lần gửi sau. Xóa key đánh dấu kết nối cần thiết lập lại, giữ lịch sử tác vụ. [Model](MODEL_STRATEGY_SPEC.md) · [API key](SECURITY_SPEC.md)

### 10.2. Cách tính và kiểm soát chi phí API — mục 18

Dùng bảng giá có phiên bản theo provider, model, đơn vị tiền, đơn vị tính, ngày hiệu lực và nguồn. Cho người dùng nhập/ghi đè đơn giá có ghi nguồn. Lúc gửi, chốt phiên bản giá của model đã chọn và dự phòng theo input cùng trần output/reasoning. Khi nhận kết quả, đối chiếu theo model và usage thực tế, lưu khoản điều chỉnh. Adapter tách các nhóm token theo cách provider tính phí để phần cached/reasoning chỉ bị tính một lần. Khoản phí công cụ Research có dòng riêng.

Mỗi tác vụ có một đơn vị tiền. Khi cần quy đổi, dùng tỷ giá do người dùng nhập, có ngày và được lưu cùng phép tính. Với hạn mức tiền, chỉ gửi khi tổng **đã ghi nhận + dự phòng cho các lần chưa rõ + dự trù lần tới** còn trong hạn mức và đủ dữ liệu để ước tính. Mất kết nối giữ khoản dự phòng cho tới khi đối chiếu. Ở chế độ không giới hạn tiền, phần thiếu giá hoặc usage hiển thị **Chưa xác định**. Màn chi phí phân biệt khoản provider báo và ước tính của tool.

### 10.3. Xử lý timeout và phản hồi đến muộn — mục 19

Một công việc logic có `request_id`. Mỗi lần thực sự gửi có `attempt_id`, thời hạn và bộ phiên bản đầu vào riêng. Trước khi gửi retry, nếu đã nhận đủ kết quả cũ còn hợp lệ thì dùng kết quả đó. Khi retry đã gửi cho cùng công việc, kết quả của lần cũ đến muộn được lưu ở lịch sử của lần cũ. Core dành quyền công bố cho lần gửi hiện hành, bằng kiểm tra phiên bản và một khóa duy nhất cho kết quả của công việc logic trên nền đó.

Mỗi lần gửi có sổ usage riêng, kể cả lần cũ đã timeout. Nếu provider hỗ trợ, đối chiếu bằng ID phía dịch vụ. Khoản chưa rõ vẫn được dự phòng. Dùng chính sách 9A với mặc định 10 phút/lần gọi, hai lần retry lỗi tạm thời sau 10 và 30 giây, tôn trọng thời gian chờ dài hơn do dịch vụ yêu cầu. Nguyên nhân chưa rõ hoặc hết lượt thì lưu bước và báo người dùng.

### 10.4. Liên kết và yêu cầu triển khai

Bộ lập lượt dùng tokenizer và dự trù dung lượng theo [đặc tả đầu vào](EPUB_INPUT_SPEC.md), mục 13. Chi phí công cụ tìm/đọc trang theo [Research](RESEARCH_STAGE_SPEC.md), mục 8. Cấu hình và bộ đếm thuộc hồ sơ tác vụ, được bảo toàn qua checkpoint.
