# Quy trình hai model — triển khai và review độc lập

**Phiên bản:** 2.0 · **Ngày:** 12/11/2026  
**Trạng thái:** Phase 00 và M1 đã có review độc lập. Bản 2.0 thêm chat phase, reviewer được gọi tự động và checkpoint công bố sớm; trạng thái duyệt bản kế hoạch ở PLAN.  
**Điểm nhận việc:** [PLAN.md](../../PLAN.md) · **Quy tắc repo:** [AGENTS.md](../../AGENTS.md).

## 1. Phạm vi

Áp dụng cho các task được giao trong [Phase 00–09](ROADMAP.md), gồm task tài liệu/ADR, thử nghiệm kỹ thuật, code, test và bàn giao. Model A triển khai và tự review; một model khác, B, kiểm tra độc lập trong phiên riêng. Cuối mỗi phase hoặc cổng bàn giao sớm, B kiểm tra thêm tính tích hợp của đầu ra.

Việc soạn bộ quy trình này chỉ chuẩn bị cách làm. Không biến trạng thái chưa bắt đầu của các task thành đã thực hiện hoặc tạo bằng chứng review giả. Người dùng đã giao mỗi chat phase tự gọi gpt-6.1-sol / max làm B trong phiên độc lập. A dùng model khác B; truyền requirements/candidate và đường báo cáo, không kế thừa kết luận triển khai làm chỉ thị review. Không có đúng công cụ/model thì giữ Chờ review, không tự duyệt. Chi tiết ở [quy tắc song song](PARALLEL_EXECUTION.md#review).

## 2. Vai trò và quyền cập nhật

| Vai trò | Công việc | File được cập nhật trong lượt đó |
|---|---|---|
| A — triển khai | Đọc yêu cầu, làm task, viết/chạy test, tự rà soát, sửa lỗi và chuẩn bị bản bàn giao | Source/test/tài liệu trong phạm vi; báo cáo bàn giao và sổ riêng của phase; gửi điều phối cập nhật PLAN trung tâm |
| B — review độc lập | Tự đối chiếu yêu cầu, kiểm code/test, chạy kiểm chứng, ghi phát hiện và kết luận | Báo cáo review riêng; log/fixture thử nghiệm tạm trong vùng review |
| Người dùng | Chọn model/vai, giao phạm vi, giải quyết yêu cầu sản phẩm còn mơ hồ hoặc thay đổi phạm vi | Quyết định được ghi lại trong tài liệu tương ứng |

A và B ghi tên model theo thông tin được cung cấp, phiên/chat và ngày; không tự đoán phiên bản model. B không tự sửa source, test chính thức, đặc tả, ADR, PLAN hoặc báo cáo của A để làm bản bàn giao đạt. B có thể kiểm tra thêm bằng test thăm dò ở bản sao tạm, lưu cách tái hiện và đề nghị A tích hợp regression test.

A phase ghi evidence của mình sau report B; một A được chỉ định làm điều phối là người duy nhất ghi PLAN trung tâm. A không sửa kết luận của B hoặc tự thay vai để duyệt phần mình vừa làm. Nếu B trực tiếp triển khai một bản sửa thì bản sửa đó phải có người/model khác review; không tự coi nó đã được review độc lập.

## 3. Đơn vị giao việc

- Mặc định một task cho mỗi lượt bàn giao. Có thể gom 2–3 task gắn chặt với nhau thành một nhóm nhỏ, ghi danh sách/lý do/dependency và AC từng task trong PLAN trước khi triển khai, áp dụng cho từng chat phase; không gom cả phase làm một lượt code.
- Dependency ngoài nhóm phải hoàn thành và có review còn hiệu lực. Dependency nội bộ nhóm có thể được triển khai nối tiếp để tạo một đầu ra kiểm tra được; các task trong nhóm vẫn chưa hoàn thành cho tới khi cả nhóm đủ nghiệm thu.
- A tự tiếp tục trong phạm vi triển khai đã giao. Tới điểm cần B, ghi Chờ review và bàn giao, không hỏi lại người dùng để xác nhận từng bước thông thường hoặc tự vượt qua cổng review.
- A có thể làm task độc lập khác trong phạm vi được giao nếu không làm thay đổi bản B đang kiểm tra. Trong cùng một checkout, giữ nguyên toàn bộ đầu vào của bản bàn giao trong suốt lượt B; chọn làm nối tiếp khi không tách được phần công việc.
- Những phần chưa thực hiện hoặc task tương lai không được đánh giá là đạt chỉ vì có stub, thiết kế hay tên test.

## 4. Xác định đúng bản được bàn giao

A cung cấp một **candidate ID** cho mỗi lượt, ví dụ `P01-T04-r01`, và bản gốc dùng so sánh. ID này phải trỏ tới nội dung xác định, không chỉ là nhãn tự đặt.

| Thành phần | Bằng chứng bắt buộc |
|---|---|
| Baseline | Commit nền nếu có; liệt kê thay đổi đã tồn tại trước task để phân biệt phần A sửa |
| Candidate | Commit cụ thể nếu đã có; nếu workspace chưa commit, ghi danh sách file/trạng thái và SHA-256 của đầu vào cần review, kèm diff và nội dung file mới |
| Phạm vi thay đổi | Source, test, manifest, lockfile, build/CI config và consumer liên quan; gồm file untracked thuộc task |
| Căn cứ | Phiên bản/hash phần đặc tả, ADR và task áp dụng; ghi nguyên phạm vi/AC đã nhận từ PLAN |
| Môi trường | Toolchain, target, OS/terminal/tool version và cấu hình/fixture không chứa bí mật |
| Kết quả | Lệnh, số ca liên quan, kết quả, log và candidate mà chúng thực sự kiểm tra |

Không tự commit, force-add tài liệu nội bộ hoặc đưa dữ liệu người dùng vào snapshot chỉ để có bản review. Một commit ID không đại diện cho thay đổi chưa commit, file mới chưa được theo dõi hoặc tài liệu bị ignore. Manifest phải bao phủ cả các đầu vào đó khi chúng ảnh hưởng task. Nếu không có baseline đủ để tạo diff, ghi rõ giới hạn và bàn giao toàn bộ đầu ra trong phạm vi; B không được giả định đã xem đủ thay đổi.

B đối chiếu candidate lúc bắt đầu và trước khi kết luận. Source/test/dependency/config hoặc yêu cầu liên quan thay đổi thì tạo lượt mới; kết luận cũ không tự áp dụng. Báo cáo, log sinh khi test và cập nhật trạng thái thuần túy không thuộc nội dung sản phẩm được đóng băng. Thay đổi phạm vi/AC trong PLAN vẫn làm thay đổi căn cứ review.

Khi dùng workspace khác, chuyển đúng bản code và bộ tài liệu áp dụng cho candidate. Tài liệu trong `docs/` đã được người dùng cho phép đưa lên GitHub có thể được duyệt qua PR. Chia sẻ riêng `PLAN.md`, báo cáo và bằng chứng nội bộ theo quyền người dùng đã cấp. B thiếu đầu vào bắt buộc phải ghi Chưa đủ bằng chứng.

## 5. Trình tự A → B → sửa → kiểm tra lại

1. **A nhận việc:** đọc AGENTS, PLAN, task/nhóm và dependency; giữ căn cứ là đặc tả/thiết kế, ghi baseline và phạm vi.
2. **A triển khai:** hoàn thành hành vi, test tương ứng và tự review. Ghi cả ca lỗi/biên, lỗi đã biết và kiểm tra chưa chạy. Task tài liệu chỉ chạy phép kiểm tài liệu/thử nghiệm được task yêu cầu.
3. **A bàn giao:** tạo báo cáo theo [mẫu bàn giao](templates/TASK_HANDOFF.md), xác định candidate và chuyển Chờ review. Không tự đánh dấu Hoàn thành.
4. **B đọc yêu cầu trước:** tự xác định AC, đường lỗi, ranh giới và side effect cần kiểm tra. Sau đó xem diff, file mới, consumer liên quan, code test và báo cáo A; báo cáo A là bằng chứng cần đối chiếu.
5. **B kiểm chứng:** tự chạy test liên quan tới phần thay đổi, ca lỗi quan trọng và các test bị sửa; thử thêm trường hợp thiếu khi cần. Xác minh test thực sự được phát hiện/chạy và assertion kiểm hành vi mong đợi. Không chỉ lặp lại kết luận của A hoặc đọc số test xanh.
6. **B kết luận:** viết [báo cáo review](templates/TASK_REVIEW.md), gắn từng AC với bằng chứng và phát hiện với vị trí/cách tái hiện. Chọn Đạt, Cần sửa hoặc Chưa đủ bằng chứng theo mục 6.
7. **A xử lý:** sửa phát hiện có căn cứ, thêm regression test phù hợp, hoặc trả lời bằng đặc tả/test khi không đồng ý. Bản bàn giao lượt mới ghi từng finding ID đã xử lý, diff từ lượt trước và candidate mới.
8. **B kiểm tra lại:** kiểm bản sửa, regression và phần bị ảnh hưởng; đóng/tái mở từng finding có lý do. Giữ báo cáo lượt trước. Review đạt trên bản cũ không thay kết luận cho bản mới.
9. **A cập nhật:** khi review Đạt và đủ bằng chứng bắt buộc, cập nhật mục 16 của phase và gửi điều phối cập nhật PLAN, rồi nhận task phụ thuộc tiếp theo trong phạm vi được giao.

Review không yêu cầu chạy lại toàn bộ test không liên quan ở mọi lượt. B ghi rõ test tự chạy, bằng chứng CI/thủ công đã đối chiếu, phạm vi chưa kiểm và lý do. Thiếu môi trường cho kiểm tra bắt buộc không được thay bằng suy đoán; giữ chưa xác minh. Test tự động vẫn offline/deterministic, không gọi provider trả phí.

## 6. Trạng thái và điều kiện chuyển bước

```text
Chưa bắt đầu → Đang làm → Chờ review → Hoàn thành
                             ↓
                          Cần sửa → Đang làm → Chờ review (lượt mới)

Thiếu kiểm tra bắt buộc → Chờ kiểm chứng
Thiếu dependency/thông tin/môi trường cần thiết → Bị chặn
```

| Kết luận B | Khi dùng | Hành động của A |
|---|---|---|
| Đạt | Đủ AC và kiểm tra bắt buộc; candidate rõ và còn đúng; không còn lỗi chặn | Được đánh dấu Hoàn thành trong đúng phạm vi B đã review |
| Cần sửa | Có lỗi đã được xác lập làm sai hành vi/AC hoặc vi phạm yêu cầu bắt buộc | Ghi Cần sửa, xử lý findings rồi gửi lượt mới |
| Chưa đủ bằng chứng | Chưa chạy/đối chiếu được kiểm tra bắt buộc, thiếu đầu vào, candidate thay đổi hoặc chưa rõ yêu cầu | Ghi Chờ kiểm chứng hoặc Bị chặn kèm điều kiện còn thiếu; chưa đóng task |

Nếu vừa có lỗi đã xác lập vừa thiếu bằng chứng, kết luận Cần sửa và liệt kê đủ phần chưa xác minh; không coi việc sửa lỗi đó là đã bù bằng chứng. Nghi ngờ chưa chứng minh được phải được gắn rõ là câu hỏi/điểm cần kiểm, không dựng thành lỗi chắc chắn. Khi mọi yêu cầu đã đạt, B có thể ghi không tìm thấy lỗi chặn; không cần bịa lỗi để chứng tỏ có review.

Mỗi task chỉ hoàn thành khi có đủ AC/test, review B Đạt còn hiệu lực và tài liệu đã đồng bộ. Review độc lập không thay kiểm tra terminal/reader thật hoặc người rà nghĩa Nhật–Việt khi đặc tả yêu cầu. Không có B thì vẫn Chờ review; A có thể tiếp tục phần độc lập được giao nhưng không tự cấp kết luận B.

## 7. Phát hiện, mức ảnh hưởng và bất đồng

Mỗi phát hiện dùng ID ổn định như `P01-T04-F01`: mức ảnh hưởng, yêu cầu/AC bị vi phạm, file:dòng hoặc đoạn tài liệu, điều kiện kích hoạt, mong đợi/thực tế, bằng chứng/cách tái hiện và hướng kiểm chứng bản sửa. Test không chạy được phải ghi rõ; phân tích tĩnh có thể là bằng chứng nếu đường lỗi được chứng minh cụ thể.

| Mức | Cách dùng |
|---|---|
| P0 | Nguy cơ nghiêm trọng đã có căn cứ như mất dữ liệu hoặc lộ bí mật; chặn nghiệm thu |
| P1 | Luồng chính sai, treo, vi phạm quyền chạy/tiền/hợp đồng quan trọng; chặn nghiệm thu |
| P2 | Lỗi có phạm vi hẹp nhưng vẫn sai AC/hành vi bắt buộc; phải sửa trước nghiệm thu phạm vi đó |
| P3 / Gợi ý | Cải tiến tùy chọn về cách tổ chức/trình bày; ghi việc sau nếu hữu ích, không tự chặn task |

Không hạ một vi phạm AC bắt buộc thành gợi ý để cho qua. Yêu cầu mở rộng ngoài phạm vi và lỗi baseline được ghi riêng, nêu rõ có ảnh hưởng việc nghiệm thu hiện tại hay không. Không yêu cầu refactor theo sở thích khi chưa chỉ ra vi phạm, rủi ro hoặc lợi ích cần thiết cho task.

A trả lời từng finding: đã sửa, đề nghị bác bỏ kèm bằng chứng, hoặc cần làm rõ. B kiểm tra và ghi trạng thái đóng/mở; A không tự xóa finding. Nếu còn bất đồng vì đặc tả mơ hồ, đưa đúng câu hỏi sản phẩm cho người dùng và tiếp tục phần độc lập. Không sửa/nới test, đổi AC hoặc thay đặc tả chỉ để đạt review.

## 8. Review cuối phase và cổng M0

Task review kiểm phần thay đổi; phase review kiểm kết quả kết hợp trên bản cuối. Dùng [mẫu review phase](templates/PHASE_REVIEW.md), đối chiếu mục 12–16 của phase:

- Truy vết tất cả AC/task tới bằng chứng và review còn hiệu lực; kiểm tác động của các thay đổi sau lần review từng task.
- Kiểm luồng tích hợp, demo, recovery, migration, bảo mật và tương thích được phase yêu cầu.
- Xác minh bộ lệnh Rust bắt buộc cùng các kiểm tra riêng đạt trên candidate cuối khi có thay đổi Rust; ghi target/toolchain thực tế. Task tài liệu không dùng code cũ làm bằng chứng đạt.
- Đối chiếu kiểm tra thủ công, nền tảng và artifact đúng phạm vi; không dùng headless/fake thay kết quả thật bắt buộc.
- Chỉ ghi phase/cổng đạt khi không còn lỗi chặn hoặc thiếu bằng chứng bắt buộc; ghi đầu vào bàn giao cho phase tiếp theo.

Riêng Phase 00: **M0-01** review T01–T05 và phần bàn giao T10 cho nền tảng; **M0-02** review T06–T08 và phần T10 cho storage. Kết luận M0-01 không đóng toàn T10 hoặc toàn Phase 00. Toàn M0 còn cần lịch T09 và các điều kiện mục 15; mỗi phạm vi review ghi rõ để không biến ADR tương lai thành điều kiện của M0-01.

## 9. Hồ sơ và cách ghi phiên bản

Tất cả file dưới đây là tài liệu nội bộ trong `docs/roadmap/evidence/phase-XX/`, chỉ tạo khi thực hiện:

| File | Người ghi | Nội dung / mẫu |
|---|---|---|
| `PXX-TYY.handoff-r01.md` | A | [Bàn giao task](templates/TASK_HANDOFF.md): căn cứ, baseline/candidate, AC, test, câu trả lời findings |
| `PXX-TYY.review-r01.md` | B | [Review task](templates/TASK_REVIEW.md): kiểm chứng độc lập, findings, kết luận |
| `M1.handoff-r01.md` và `M1.review-r01.md` | A / B, file riêng | Bàn giao candidate của cổng và [review phase](templates/PHASE_REVIEW.md) |

Lượt sửa tăng `r02`, `r03`…; giữ nguyên báo cáo đã bàn giao, không ghi đè để xóa lịch sử. Finding ID giữ ổn định giữa các lượt. Nhóm task phải liệt kê đủ task/AC trong báo cáo; tên file có thể dùng dải như `P01-T07-T08`. Bàn giao sớm dùng phạm vi rõ như `P00-T10-M0-01` và review cổng `M0-01`.

PLAN giữ các dòng chat/task đang chạy, ownership, inputs và checkpoint; chỉ điều phối cập nhật từ reports A/B. Ngay lúc bàn giao, A ghi đường dẫn báo cáo B dự kiến theo lượt và đánh dấu chưa có kết quả; chưa tạo link tới file không tồn tại. B tạo báo cáo tại đường dẫn đó, trả đường dẫn trong kết quả bàn giao để A đọc mà không cần B sửa PLAN. Mục 16 của phase lưu sổ task/gate và link bằng chứng. A cập nhật evidence phase và gửi điều phối cập nhật PLAN sau bàn giao/kết luận; B ghi báo cáo riêng để tránh cùng sửa trạng thái. Nếu dữ liệu lệch, đối chiếu candidate và bằng chứng thay vì chọn trạng thái tiến xa hơn.

## 10. Mẫu câu giao việc

### Giao model A task hiện tại

```text
Bạn là model A triển khai. Đọc AGENTS.md, PLAN.md và
docs/roadmap/REVIEW_WORKFLOW.md.
Thực hiện task/nhóm hiện hành theo đặc tả, dependency và phạm vi được giao.
Viết/chạy test phù hợp, tự review, tạo báo cáo bàn giao có candidate xác định.
Cập nhật Chờ review; không tự đánh dấu Hoàn thành hoặc đóng vai model B.
```

### Giao model B kiểm tra

```text
Bạn là model B review độc lập. Đọc AGENTS.md, PLAN.md và
docs/roadmap/REVIEW_WORKFLOW.md; đọc yêu cầu/AC của task trước khi
kiểm code và báo cáo bàn giao của A.
Xác nhận candidate; tự kiểm code, test, ca lỗi và chạy kiểm chứng liên quan.
Không sửa source, test chính thức, đặc tả hoặc trạng thái PLAN.
Viết báo cáo review riêng: findings có căn cứ, kiểm tra đã/chưa chạy,
và kết luận Đạt / Cần sửa / Chưa đủ bằng chứng cho đúng candidate.
```

### Giao A sửa sau review

```text
Bạn là model A. Đọc review mới nhất cho task hiện hành trong PLAN.
Xử lý từng finding, bổ sung regression test phù hợp và trả lời có bằng chứng.
Tạo candidate và báo cáo bàn giao lượt mới, giữ lịch sử và finding ID.
Cập nhật Chờ review để B kiểm tra lại; không tự đóng findings hoặc task.
```

### Giao B nghiệm thu phase/cổng

```text
Bạn là model B. Review cổng hiện hành trong PLAN theo mẫu
docs/roadmap/templates/PHASE_REVIEW.md.
Đối chiếu candidate cuối, toàn bộ AC, review task, kiểm thử tích hợp,
demo/thủ công và điều kiện đóng phase. Ghi bằng chứng còn thiếu rõ ràng.
Viết báo cáo review cổng; chỉ kết luận Đạt cho phạm vi thực sự đủ bằng chứng.
```

## 11. Review checkpoint và công bố sớm

Mỗi checkpoint 1–3 task có review tích hợp đúng candidate trước commit/push; dùng PHASE_REVIEW và ghi rõ CP/phạm vi. Có thể gộp report task đơn với CP nếu kiểm đủ cả hai phạm vi. CP Đạt thì A commit/push ngay theo [PARALLEL_EXECUTION §6](PARALLEL_EXECUTION.md#git-checkpoint), cập nhật draft PR, không đợi hết phase. Nếu thay baseline, contract, schema hoặc resolve conflict, B kiểm lại phần ảnh hưởng. Consumer ngoài phase chỉ nhận checkpoint đã công bố. Cổng cuối M2–M9 vẫn cần milestone trước, toàn bộ task/AC/manual và review bản ghép cuối.


## 12. Merge và bảo vệ phần đã nghiệm thu

Áp dụng [MERGE_WORKFLOW](MERGE_WORKFLOW.md) cho ủy quyền 14/11/2026: điều phối merge tuần tự khi đủ cổng, B xác nhận bản ghép đúng candidate, base/head/tree; review lại phần ảnh hưởng khi inputs đổi. Phần đã nghiệm thu và merge được khóa, kể cả khi hậu kiểm lỗi; conflict thuần test có ngoại lệ hẹp theo MERGE_WORKFLOW, còn sửa core do conflict hoặc sửa phần bảo vệ ngoài ngoại lệ cần Master phê duyệt cụ thể trước sửa. CP code chỉ công bố cho consumer sau merge và hậu kiểm; CP docs-only nhận manifest/review theo dependency. Không thay AC, status hoặc candidate đang review.


## 13. Scratch và evidence sau checkpoint

A và B phân định owner, mục đích, thời hạn giữ scratch trước review; giữ candidate ổn định, dùng manifest chọn lọc và build dir tái sử dụng theo [dọn checkpoint](MERGE_WORKFLOW.md#checkpoint-cleanup). B review chính sách/kế hoạch trước push; A dọn sau xác nhận push/PR hoặc sau B Đạt và bàn giao docs-only, không tạo vòng lặp review. Chỉ xử lý scratch B sau khi review kết thúc theo phân công owner; giữ report và bằng chứng bắt buộc.
