# PHASE 04 — LUỒNG DỊCH–REVIEW ĐẦU TIÊN BẰNG MODEL GIẢ LẬP

> Lịch thực thi v2 · 12/11/2026.

> Lịch task song song và checkpoint commit/push tại [mục 10](#p04-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Kế hoạch triển khai v0.3 · 09/11/2026 · Chưa bắt đầu.** M4 nhận revision, context và quyết định đã duyệt từ M3. Hành vi theo đặc tả và ADR. Fixture giả lập chứng minh thứ tự, cách lưu và kiểm bằng chứng của ứng dụng.
>
> Giữ yêu cầu Q05 đã bổ sung ở P00-T01-r02 sau finding F01. Khi một block không có chỗ ngắt an toàn, T02/AC02/TC02 phải **chặn gửi**, chỉ vị trí nguồn, giữ protected inline và chỉ lập lại lượt sau khi nhận điểm chia hoặc giới hạn rõ ràng.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P04 / M4 |
| Tên | Luồng Dịch–Review đầu tiên bằng model giả lập |
| Trạng thái | Kế hoạch đã chi tiết. Chờ M3, chưa triển khai hoặc chạy test M4. |
| Đầu vào chính | M3 revision/context/approval, M2 source mapping và M1 shell |
| Giao diện | UI-05 Reviewer, nối Editor UI-04 |
| Phân rã | 5 gói, 13 task |
| Nhánh | Nhánh theo phần việc `feature/...`, PR Rust vào `rewrite/rust`. |
| Vai trò | A triển khai, kiểm và bàn giao. B khác review độc lập. |

Xem [roadmap chung](../ROADMAP.md), [Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-04) và [quy tắc repo](../../../AGENTS.md). Kế hoạch, fixture riêng và bằng chứng lưu ngoài Git.

## 2. Mục tiêu và demo cuối phase

Chạy từ nguyên tác của một chương qua lập lượt, Dịch, kiểm cấu trúc, lưu nháp, Review hai bước và kết luận chương. Dùng model giả lập trả kịch bản cố định. Kiểm rằng ứng dụng gửi đúng dữ liệu, gắn bằng chứng đúng revision, giữ vùng đang chờ và tiếp tục từ dữ liệu đã lưu.

1. Nạp fixture từ M3 có một chương ít nhất hai lượt và chương kế để kiểm thứ tự. Có ruby, ảnh/link, điểm nối phủ định hoặc đổi người nói, quy tắc riêng đã duyệt và đoạn thư viện chung phù hợp.
2. Chạy bộ thử offline. So request với bảng kỳ vọng về slot, ngân sách và phần Skill được chọn. Kiểm trace `Dịch 1 → Đọc nguồn 1 → Đối chiếu 1 → Dịch 2` trên ca đạt.
3. Chạy riêng các ca response thiếu ID, marker sai, quote giả, quy ước mới chưa duyệt và block không thể chia an toàn. Đối chiếu trạng thái giữ ứng viên, cần sửa/cần xác minh hoặc chờ người dùng.
4. Khi local Review đạt, kiểm điểm nối và độ phủ hai chiều trước kết luận chương. Cố ý đổi revision ở một lượt hoặc cho lỗi nối để kiểm chương sau chưa bắt đầu.
5. Mở Reviewer → Editor đúng revision/range → quay lại đúng thẻ. Ngắt sau response đã lưu và sau nháp đã commit, rồi mở lại để kiểm bước kế tiếp và số request.

Phần cậu thao tác nằm ở [mục 16.4](#p04-user-test). Độ đúng nghĩa trong demo dựa trên oracle tự soạn. Chất lượng model thật được đo riêng ở Phase 09.

## 3. Phạm vi

| M4 thực hiện | Phase tiếp nhận |
|---|---|
| ModelGateway, profile và bộ đếm fixture, kịch bản giả lập | Phase 05 nối provider/tokenizer, cấu hình model, tiền và attempt. |
| Lập lượt, validator Dịch, gating quyết định và lưu revision qua M3 | Phase 05 thêm guard gửi thật. Phase 06 nối repair, Research và tiếp tục đúng điều kiện. |
| Review đọc nguồn, đối chiếu, evidence, phủ hai chiều, kết luận chương | Phase 06 mở rộng Review lại, xác minh và sửa M1–M4 theo rubric. |
| State machine trong app, checkpoint tối thiểu và restart | Phase 05 thêm intent/permit/retry/uncertain và Coordinator. |
| UI-05 đọc danh sách lỗi/thẻ và mở Editor đúng chỗ | Phase 06 nối thao tác Review lại. Phase 08 hoàn thiện luồng qua nhiều màn. |

M4 dùng dữ liệu M2/M3 qua use case và adapter Rust. Model giả lập chỉ nằm trong test hoặc bản chạy demo phát triển được chọn rõ. Bản chạy sản phẩm quảng bá phần đã nối. Lệnh chưa khả dụng trả lỗi theo ADR-008. [Lộ trình thay Go](PHASE-03.md#p03-go-retirement) dùng bằng chứng thay thế ở đúng phạm vi. Luồng giả lập M4 chưa thay đường Dịch–Review thật của Go.

## 4. Điều kiện bắt đầu

P04-T01 nhận resolver P03-T07 qua P03-CP04. P04-T04 chờ riêng approval/publication P03-T08/T09/T10; P04-T12 chờ Editor P03-T12. P04-T01 và T02 sau review/công bố có thể mở adapter/planner P05 trước khi M4 xong. P04-T13/M4 vẫn cần M3.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-TRANS | [Giai đoạn Dịch](../../spec/TRANSLATION_STAGE_SPEC.md) |
| S-REVIEW | [Giai đoạn Review](../../spec/REVIEW_STAGE_SPEC.md) |
| S-RUBRIC | [Rubric S0–S3 và M1–M4](../../spec/REVIEW_RUBRIC_SPEC.md) |
| S-SKILL | [Skills và duyệt thuật ngữ](../../spec/SKILLS_GLOSSARY_SPEC.md) |
| S-MODEL | [Model, retry và chi phí](../../spec/MODEL_STRATEGY_SPEC.md) |
| S-WORKFLOW | [Điều phối Dịch–Review](../../spec/WORKFLOW_ORCHESTRATION_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-WORKFLOW | [Bước 7 — State machine và recovery](../../desgin/WORKFLOW_STATE_MACHINE_STEP7.md) |
| D-TECH | [Bước 5 — Nền tảng và contract](../../desgin/TECHNICAL_FOUNDATION_STEP5.md) |
| D-UI | [Bước 8 — UI/UX](../../desgin/UI_UX_DESIGN_STEP8.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REVIEW | [Quy trình hai model](../REVIEW_WORKFLOW.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Qxx là quyết định xx trong S-Q. Chốt tên API, schema và thư viện ở task có consumer. Giữ các bất biến khi chọn kỹ thuật.

- **Q05.** Đếm toàn request đã dựng. Kiểm input, output và tổng context riêng. Cắt ngữ cảnh tùy chọn trước, giữ quy ước bắt buộc và protected units. Lượt không thể chia an toàn giữ chờ với vị trí nguồn.
- **Q13/17.** Kiểm hash/version Skill trước request mới. Chọn theo scope và giữ phiên bản đã gửi. ModelGateway có role, request ID và kết quả/lỗi có kiểu. Model thực tế và usage chưa biết giữ trạng thái chưa xác định.
- **Q20.** Request/response có `schema_version`, slots theo thứ tự, target và marker đối chiếu được. Lỗi cấu trúc tối đa hai yêu cầu sửa. Phản hồi bị cắt đi qua lập lại kế hoạch. Quy ước chưa duyệt chỉ giữ vùng ứng viên.
- **Q21–22.** Review đọc nguồn độc lập rồi đối chiếu bản Việt. Kiểm exact snapshot/range/revision/quote và phủ hai chiều. Quote tồn tại là điều kiện kiểm được bằng code. Kết luận ý nghĩa cần ngữ cảnh và evidence đủ. Mâu thuẫn giữ các kết quả để xác minh.
- **Q23–24.** Mức S và phạm vi sửa M tách riêng. Phase 04 ghi nhận vấn đề và phạm vi sửa đề xuất có căn cứ. Luồng tự sửa thuộc Phase 06. Kết luận chương cần local Review còn hiệu lực, các điểm nối và toàn bộ phạm vi được giao.
- **Q29.** Reopen đọc step/checkpoint và đối chiếu phiên bản. Có response/revision/local result hợp lệ thì tiếp tục từ đó. Phần attempt, tiền và quyền chạy thật giữ consumer Phase 05.

## 6. ADR và điểm còn mở

Dùng [ADR-003](../../decisions/ADR-003.md) cho SourceRange, [ADR-004](../../decisions/ADR-004.md) cho generation/operation và [ADR-008](../../decisions/ADR-008.md) cho ranh giới/API/lỗi. M3 bàn giao context, approval và source reuse còn hiệu lực.

| ID | Quyết định cần chốt và thử | Owner / hạn |
|---|---|---|
| P04-D01 | Gateway DTO, role, request/response ID, kết quả hoàn tất/dở/lỗi và ghi request test. Chốt harness có clock/barrier và điểm ngắt, W7-14. | T01 trước gateway/fake. |
| P04-D02 | DB-06. Slot ID/range, thứ tự và exact coverage, phần chồng chỉ đọc. Chốt đường đếm payload đầy đủ, phần bao và trần profile fixture. | T02 trước planner. |
| P04-D03 | Q20. Schema JSON Dịch, encoding marker và validate ID/target. Chốt hai lần sửa cấu trúc và đường phản hồi bị cắt. | T03 trước validate response. |
| P04-D04 | Candidate evidence/eligibility, phạm vi nào chờ và dữ liệu ghim cho commit. Review DTO/error mapping nối ADR-008. | T04 trước gating và T05 trước publish. |
| P04-D05 | Source-only input/schema, key cache theo căn cứ, hai chiều coverage, issue evidence/quote và join contexts. | T06/T07 trước Review, T09 trước chapter conclusion. |
| P04-D06 | ADR-007/W7-01. Step/event, điều kiện hợp lệ để chạy, chờ, pause và chuyển local/chapter. | T08 trước state machine. |
| P04-D07 | ADR-007/W7-02/03. Checkpoint version, mốc lưu, artifact references, điều kiện replay và xử lý version cũ/lỗi. | T10 trước schema/reopen. |

A ghi phương án, tiêu chí và giới hạn vào ADR của consumer. [Lịch DB/W7](../ROADMAP.md#decision-schedule) · [Mục mở ADR](../ROADMAP.md#adr-followups). Tokenizer thật, HTTP, attempt reconciliation và durable quyền chạy thuộc owner Phase 05.

## 7. Thiết kế triển khai

### 7.1. Trách nhiệm và chiều gọi

| Thành phần | Sở hữu |
|---|---|
| `nt-domain` | Kiểu slot/range, quy tắc coverage, trạng thái và căn cứ hiệu lực thuần. |
| `nt-application` | ModelGateway port, prepare/validate/publish, Review và chọn bước kế. Dùng context/approval của M3. |
| `nt-storage` | Lưu response/Review/coverage/checkpoint qua port. Mở rộng migration M3 và current/history theo generation. |
| Fake/harness | Kịch bản xác định, capture payload, failpoints và bộ đếm. Gắn qua port trong test/demo. |
| App và `nt-tui` | Composition, đọc kết quả Reviewer và mở Editor. Không chứa quy tắc lập lượt hoặc xác nhận Review. |

Tách request **Đọc nguồn** thành DTO chỉ có source và context nguồn đã kiểm chứng. Request **Đối chiếu** mới có target exact revision cùng source findings. Các kiểu ứng dụng không phụ thuộc adapter.

### 7.2. Lập lượt, chọn Skill và công bố nháp

1. Đọc source snapshot, mapping, scope tập và revision nền. Resolve Skill theo M3, ghim ID/version và chỉ lấy đoạn cần dùng. Quy tắc chưa công bố hoặc xung đột chặn đúng vùng phụ thuộc.
2. Tạo danh sách slot giao việc từ SourceRange. Overlap là context chỉ đọc, không được trả lại như slot Dịch. Đếm token trên payload đã dựng bằng bộ đếm fixture, gồm phần hướng dẫn, khung JSON và phần bao. Tính dự trù output/reasoning theo profile.
3. Kiểm trần input/output/tổng context. Chỉ gửi khi hợp lệ và còn căn cứ. Cắt tùy chọn rồi giảm phạm vi qua ranh giới an toàn. Với block không chia được, giữ vị trí và yêu cầu thay đổi có chủ đích.
4. Lưu response giả lập có ID/finish reason trước validate. Kiểm JSON, slot và marker, evidence đề xuất. Chỉ phần đủ điều kiện mới có thể công bố revision.
5. Trước commit kiểm source/mapping/context/base. Ghi revision chờ Review, supplied dependencies, current/history và Markdown outbox qua M3. Response stale ở lịch sử để đối chiếu.

### 7.3. Review, kết luận chương và tiếp tục

1. Dùng source/context nguồn đã xác minh để tạo lần đọc độc lập. Có thể tách nhiều lần để phủ đủ phạm vi. Chỉ dùng lại cache khi phạm vi, nguồn và phiên bản căn cứ đúng.
2. Đối chiếu target/revision đã ghim với findings. Kiểm quote/range/ID, bằng chứng, phạm vi và phủ nguồn→dịch lẫn dịch→nguồn. Giữ **Cần xác minh** khi chưa đủ căn cứ hoặc kết quả mâu thuẫn.
3. Lưu local result trước khi quyết định bước kế. Lượt kế chỉ bắt đầu khi phần đang xét đủ điều kiện và dữ liệu còn hiệu lực. Khi xong local, lập hồ sơ chương với tập revision, joins, issues và coverage.
4. Lưu step/checkpoint theo D06/D07 và liên kết artifact immutable. Khi reopen, xác minh trước khi chạy tiếp. Chỉ chuyển chương sau khi hồ sơ chương đủ điều kiện. Giữ intent tạm dừng.

## 8. Dữ liệu, lỗi và phục hồi

| Tình huống | Trạng thái hoặc hành động cần giữ |
|---|---|
| Thiếu giới hạn, vượt trần hoặc block không chia an toàn | Không gọi fake. Giữ source location, lý do và đầu vào cần thay đổi trước re-plan. |
| Response dở/bị cắt, JSON hoặc marker sai | Giữ artifact/chẩn đoán. Bị cắt thì lập lại phạm vi/dự trù. Lỗi cấu trúc sửa có bộ đếm tối đa hai. |
| Quy ước chưa duyệt hoặc Skill còn chờ sync | Giữ ứng viên, decision IDs và phần phụ thuộc. Chỉ publish sau đủ căn cứ. |
| Source, Skill hoặc base đổi trước commit | Revalidate. Lượt chưa gửi dựng lại nếu cần. Giữ response cũ và sửa tay, không công bố stale. |
| Quote/range sai hoặc Review mâu thuẫn | Trả vấn đề về kiểm chứng hoặc Review lại vùng ảnh hưởng. Bảo toàn bằng chứng hai phía. |
| Ngắt sau lưu response/commit/local result | Dùng lại artifact hợp lệ và bước kế. Đọc đúng revision và kiểm lại context/version. |
| Checkpoint không hỗ trợ hoặc ref hỏng | Báo lỗi/version và phạm vi. Giữ history, không tự chạy bước mới. |
| Pause hoặc chờ người dùng | Reopen giữ intent và đúng lý do. Không cấp bước gửi mới chỉ từ thao tác mở. |

Lỗi có mã, phạm vi, lý do và bước tiếp theo. Log test đã lọc chỉ chứa dữ liệu tổng hợp. M4 kiểm process restart của luồng giả lập, không dùng kết quả đó để suy ra an toàn resend, usage hoặc chi phí provider.

## 9. Phần TUI/CLI

| Phần | Hành vi cần bàn giao |
|---|---|
| Danh sách Reviewer | Đúng workspace/tập/chương/revision, ba trục và bước hiện tại. Hiển thị pending, verified, rejected hoặc cần xác minh theo kết quả đã lưu. Danh sách rỗng có trạng thái riêng khi chưa đủ coverage. |
| Thẻ evidence | Hiện nguyên tác, bản Việt, phạm vi, quote, giải thích, tình trạng bằng chứng và S/M nếu có. Với lỗi thêm, thấy đoạn Việt và vùng nguồn đã so. Với bỏ sót, thấy nguyên tác và vị trí thiếu. |
| Mở Editor | Dùng exact revision/range của thẻ. Nếu là bản lịch sử, hiển thị rõ. Nếu ref không còn, báo lý do. Quay lại đúng thẻ, bộ lọc, focus và cuộn. |
| Luồng demo | Harness chạy model giả lập và tạo workspace mẫu để TUI đọc. Ghi lệnh chạy thực ở mục 16.4 khi bàn giao. |

Giữ điều hướng bàn phím và kích thước của M1/M3. Chỉ read model hợp lệ được hiển thị là kết quả Review. UI không chạy IO nặng trong render. Core T02 trả vị trí/giới hạn cho block quá dài và nhận override đã kiểm. Giao diện sửa điểm chia và CLI kết quả đầy đủ nhận ở Phase 05. [P05-T13](PHASE-05.md#p05-t13) · [P05-T15](PHASE-05.md#p05-t15)

<a id="p04-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P04 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P04-T01 nhận resolver P03-T07 qua P03-CP04. P04-T04 chờ riêng approval/publication P03-T08/T09/T10; P04-T12 chờ Editor P03-T12. P04-T01 và T02 sau review/công bố có thể mở adapter/planner P05 trước khi M4 xong. P04-T13/M4 vẫn cần M3. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p04-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p04-cp01"></a>P04-CP01 — Gateway và planner giả lập | [P04-T01](#p04-t01), [P04-T02](#p04-t02) | [P03-CP04](PHASE-03.md#p03-cp04) | P04-T01: V-01 trong phạm vi AC/TC của task; P04-T02: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |
| <a id="p04-cp02"></a>P04-CP02 — Dịch, candidate và công bố nháp | [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P04-T05](#p04-t05) | [P03-CP01](PHASE-03.md#p03-cp01), [P03-CP03](PHASE-03.md#p03-cp03), [P03-CP05](PHASE-03.md#p03-cp05), [P04-CP01](#p04-cp01) | P04-T03: V-02 trong phạm vi AC/TC của task; P04-T04: V-02 trong phạm vi AC/TC của task; P04-T05: V-03 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |
| <a id="p04-cp03"></a>P04-CP03 — Review đọc nguồn | [P04-T06](#p04-t06) | [P04-CP01](#p04-cp01) | P04-T06: V-04 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |
| <a id="p04-cp04"></a>P04-CP04 — Đối chiếu và workflow cấp chương | [P04-T07](#p04-t07), [P04-T08](#p04-t08), [P04-T09](#p04-t09) | [P04-CP01](#p04-cp01), [P04-CP02](#p04-cp02), [P04-CP03](#p04-cp03) | P04-T07: V-04 trong phạm vi AC/TC của task; P04-T08: V-05 trong phạm vi AC/TC của task; P04-T09: V-04, V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |
| <a id="p04-cp05"></a>P04-CP05 — Lưu bước và restart | [P04-T10](#p04-t10) | [P04-CP04](#p04-cp04) | P04-T10: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |
| <a id="p04-cp06"></a>P04-CP06 — Reviewer và mở Editor | [P04-T11](#p04-t11), [P04-T12](#p04-t12) | [P03-CP02](PHASE-03.md#p03-cp02), [P04-CP04](#p04-cp04) | P04-T11: V-06, V-08 trong phạm vi AC/TC của task; P04-T12: V-06, V-08 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-08 phần UI-05/Reviewer–Editor T11/T12 trên core đã công bố; toàn M4 ở CP07. V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |
| <a id="p04-cp07"></a>P04-CP07 — Nghiệm thu M4 | [P04-T13](#p04-t13) | M3 Đạt, [P04-CP01](#p04-cp01), [P04-CP02](#p04-cp02), [P04-CP03](#p04-cp03), [P04-CP04](#p04-cp04), [P04-CP05](#p04-cp05), [P04-CP06](#p04-cp06) | P04-T13: V-07, V-08, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M4; V-00 cho code, B review đúng candidate. | feature/translation-review; A chat P04 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Nhận thẻ cùng mục 5–8, AC và ca kiểm tương ứng. Tên module là dự kiến theo trách nhiệm. Tạo đúng phần có caller. Sau khi chạy kiểm, bàn giao bằng fixture, trace và trạng thái đọc lại trên cùng candidate.

<a id="p04-t01"></a>

### P04-T01 — ModelGateway contract và fake xác định

- **Đầu ra.** Port ModelGateway tối thiểu và bộ thử giả lập dùng cùng contract với adapter thật sau này.
- **Đầu vào.** Dependency thực thi: [P03-T07](PHASE-03.md#p03-t07). Căn cứ kỹ thuật: Q17/20–21, D-TECH, ADR-008, D01/W7-14.
- **Vùng sửa.** Application ports/DTO, fake trong test/demo và app composition của bộ thử.

1. Chốt role, logical request ID, payload/profile version, finish reason, response artifact và lỗi có kiểu. Cùng kết quả hoàn tất cho response một lần hoặc gom từng phần. Usage/model thực tế chưa biết giữ giá trị chưa xác định.
2. Tách Translation, SourceRead và Compare theo kiểu dữ liệu được phép. Port dùng cùng shape kết quả, scope và correlation ID. SourceRead và Compare là hai thao tác của role Review. DTO Research thuộc consumer sau.
3. Fake nhận kịch bản theo ID, role và payload kỳ vọng. Capture dữ liệu gửi thực tế, trả success/error/cut theo script. Gặp request ngoài kịch bản thì báo sai test. Fake không ghi DB hoặc tự chuyển bước.
4. Tạo seam clock/barrier/failpoint để kiểm thứ tự/reopen. Giữ raw fixture ở thư mục tạm và log đã lọc. Chạy khi mạng bị cô lập và không có key.
5. Kiểm test/demo mới gắn fake. Binary sản phẩm giữ lỗi cấu hình phù hợp khi chưa có provider. Test boundary dùng actual DTO và serialization.

**Kiểm chứng.** P04-AC01, P04-TC01, V-01. B kiểm fake không tự đổi state và payload capture đủ phát hiện việc lộ target vào lần đọc nguồn.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T01 |
|---|---|
| Đầu vào bắt buộc | [P03-T07](PHASE-03.md#p03-t07). Nhận source/context/revision đã kiểm ở P03-CP01/CP03/CP04 và contract CLI/DTO theo ADR-008. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Gateway nhận role, request identity và payload, trả complete/cut/error. Fake capture nguyên payload và không tự ghi state. Owner đầu ra là A-P04-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T01 giữ Application ports/DTO, fake trong test/demo và app composition của bộ thử. Phần chung do T01 giữ gateway DTO/harness W7-14, người tích hợp giữ app composition thử. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T08](PHASE-03.md#p03-t08), [P03-T09](PHASE-03.md#p03-t09). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC01 / P04-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Payload sai làm test fail. DB không đổi khi chỉ gọi fake. |
| Điểm tích hợp | [P04-CP01](#p04-cp01) — Gateway và planner giả lập; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P05-T01](PHASE-05.md#p05-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t02"></a>

### P04-T02 — Lập lượt từ toàn bộ request

- **Đầu ra.** Kế hoạch slot theo SourceRange, request đủ trường và chẩn đoán an toàn khi vượt trần.
- **Đầu vào.** Dependency thực thi: [P04-T01](#p04-t01). Căn cứ kỹ thuật: T01, Q05/20, S-INPUT, DB-06/D02, M2 source units và M3 context.
- **Vùng sửa.** Domain range/coverage và application planner/budget profile fixture.

1. Chốt slot ID và vùng được giao theo thứ tự nguồn. Giữ source snapshot/mapping/parser version và protected units. Overlap chỉ đọc có nhãn riêng.
2. Dựng payload cuối từ source, hướng dẫn, quy tắc/Skill đã chọn, context, khung JSON và phần bao adapter. Đếm token của toàn bộ payload bằng bộ đếm fixture xác định. Dự trù output/reasoning một lần theo profile.
3. Kiểm trần input, output và tổng context riêng. Giữ mặc định thử nghiệm Q05 là 3.000 token nguồn, dự trù output Dịch khoảng 2 lần nguồn cộng phần bao JSON và biên 5% với bộ đếm phù hợp hoặc 10% khi ước lượng. Fixture có thể dùng trần nhỏ để thử đúng ngưỡng. Cắt ngữ cảnh tùy chọn trước, giữ quy tắc bắt buộc. Khi thiếu giới hạn đáng tin thì báo chờ.
4. Gom block liên tiếp. Ưu tiên điểm part/cảnh ở 20% cuối phần vừa chứa, rồi cuối đoạn/cuối câu hợp lệ. Kiểm đúng grapheme và biên ruby/ảnh/marker theo ADR-003.
5. Block không có chỗ ngắt an toàn thì **chặn gửi**, giữ diagnostic có chương, canonical XHTML path, BlockID/range, trần và lý do. Đổi điểm chia hoặc giới hạn qua input rõ của người dùng, validate lại rồi lập lại phần chưa gửi. Bảo toàn phần hoàn thành còn hiệu lực.
6. So tập được giao với oracle độc lập. Mỗi vùng nguồn trong phạm vi xuất hiện đúng một lần. Hai đoạn trùng chữ và context overlap không làm trùng coverage.

**Kiểm chứng.** P04-AC02, P04-TC02, V-02. B phải thử ba trần riêng, không cắt protected inline, không gọi fake ở block không chia được và re-plan hợp lệ.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T02 |
|---|---|
| Đầu vào bắt buộc | [P04-T01](#p04-t01). Nhận Gateway DTO, SourceRange M2 và context provenance M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Planner trả slot/ranges và full-request budget, overlap chỉ đọc, diagnostic Q05 chỉ đúng block. Owner đầu ra là A-P04-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T02 giữ Domain range/coverage và application planner/budget profile fixture. Phần chung do T01 giữ gateway, T02 giữ DB-06/slot và planner. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T08](PHASE-03.md#p03-t08), [P03-T09](PHASE-03.md#p03-t09). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC02 / P04-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu So budget/coverage với oracle. Không gửi khi bị chặn. Một re-plan cho đầu vào hợp lệ đã xác nhận. |
| Điểm tích hợp | [P04-CP01](#p04-cp01) — Gateway và planner giả lập; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P05-T03](PHASE-05.md#p05-t03). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t03"></a>

### P04-T03 — JSON Dịch và sửa cấu trúc có giới hạn

- **Đầu ra.** Response đi qua validator thành ứng viên hợp lệ hoặc chẩn đoán đúng loại.
- **Đầu vào.** Dependency thực thi: [P04-T01](#p04-t01), [P04-T02](#p04-t02). Căn cứ kỹ thuật: T01–T02, Q20, S-TRANS và D03.
- **Vùng sửa.** Application schema/validator và đường structural repair.

1. Chốt schema_version và fields theo Q20. Request đã ghim là căn cứ cho thứ tự/ID. Kiểm đủ ID, đúng thứ tự, duy nhất, không ID ngoài scope và target có chữ.
2. Kiểm marker, ruby, ảnh, link và chú thích theo định danh đã bảo vệ. Trả lỗi có ID, range và expected vs actual. Lưu raw artifact tách khỏi current.
3. Với response đầy đủ nhưng lỗi cấu trúc, dựng yêu cầu sửa cụ thể cùng model/profile đã ghim. Giới hạn **hai yêu cầu sửa** ngoài lần Dịch đầu. Lưu counter riêng với số vòng sửa chất lượng.
4. Với finish reason bị cắt, lưu phần dở để đối chiếu và chuyển lập lại ngân sách/phạm vi. Giữ tiến độ còn hợp lệ. Kết quả dở không thành draft.
5. Khi hết lượt sửa hoặc schema không xử lý được, đưa về chờ với nguyên nhân. T10 lưu counter qua restart.

**Kiểm chứng.** P04-AC03, P04-TC03, V-02. B thử chuỗi ba phản hồi sai, phản hồi bị cắt và ca marker có đúng số nhưng sai ID.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T03 |
|---|---|
| Đầu vào bắt buộc | [P04-T01](#p04-t01), [P04-T02](#p04-t02). Nhận slot IDs, output schema theo Q20 và request role. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Validate JSON/markers/IDs thành candidate hoặc structural error có giới hạn hai lần sửa theo D03. Owner đầu ra là A-P04-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T03 giữ Application schema/validator và đường structural repair. Phần chung do T02 giữ slot, T03 giữ schema/validator Dịch. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T08](PHASE-03.md#p03-t08), [P03-T09](PHASE-03.md#p03-t09). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC03 / P04-TC03, gồm các ca chi tiết §13.3. V-02. Đối chiếu Không tạo current sai hoặc tự lặp sau giới hạn. |
| Điểm tích hợp | [P04-CP02](#p04-cp02) — Dịch, candidate và công bố nháp; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t04"></a>

### P04-T04 — Kiểm candidates và chặn quy ước chưa duyệt

- **Đầu ra.** Candidate/uncertainty có căn cứ và vùng chờ rõ. Phần đủ điều kiện chuyển cho T05.
- **Đầu vào.** Dependency thực thi: [P04-T03](#p04-t03), [P03-T08](PHASE-03.md#p03-t08), [P03-T09](PHASE-03.md#p03-t09), [P03-T10](PHASE-03.md#p03-t10). Căn cứ kỹ thuật: T03, Q14–15/20, S-SKILL, D04 và publication M3.
- **Vùng sửa.** Application candidate validation và eligibility, gọi contract M3.

1. Đọc `uncertainties`, `continuity_candidates` và `rule_candidates` theo response schema. Kiểm source snapshot/quote/range và đề xuất liên quan tới slot nào.
2. Nối với kho đề xuất của M3. Giữ cách diễn đạt thử, dedup theo nguyên ngữ/nghĩa/entity/phạm vi và lưu provenance. Bằng chứng sai giữ chẩn đoán hoặc cần xác minh theo contract.
3. Xác định vùng thật sự phụ thuộc quy ước chưa duyệt, kể cả ruby có ý nghĩa riêng. Giữ vùng đó dưới dạng ứng viên và decision IDs. Nếu không khoanh được vùng an toàn, giữ cả lượt để đối chiếu.
4. Đọc version/scope đã duyệt và tình trạng Skill sync. Sau quyết định hợp lệ, chuẩn bị lại phần phụ thuộc rồi validate, ghi nháp và Review theo luồng. P06 nối thao tác tiếp tục có quyền.
5. Dữ kiện nguồn và quy ước bản Việt có trạng thái riêng. Chấp nhận draft hoặc fact không tự duyệt cách gọi, thuật ngữ hoặc quan hệ.

**Kiểm chứng.** P04-AC04, P04-TC04, V-02. B kiểm quote giả, version còn chờ sync, ruby chưa chốt và phạm vi bị chặn.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T04 |
|---|---|
| Đầu vào bắt buộc | [P04-T03](#p04-t03), [P03-T08](PHASE-03.md#p03-t08), [P03-T09](PHASE-03.md#p03-t09), [P03-T10](PHASE-03.md#p03-t10). Nhận candidate hợp cấu trúc và approval/publication M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Kiểm term/ruby uncertainty theo evidence, trả vùng đủ điều kiện hoặc chờ phê duyệt. Owner đầu ra là A-P04-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T04 giữ Application candidate validation và eligibility, gọi contract M3. Phần chung do T03 giữ schema, P03-T09/T10 giữ approval, T04 giữ eligibility Dịch. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC04 / P04-TC04, gồm các ca chi tiết §13.3. V-02. Đối chiếu Không áp quy tắc hoặc publish vùng phụ thuộc trước đủ điều kiện. |
| Điểm tích hợp | [P04-CP02](#p04-cp02) — Dịch, candidate và công bố nháp; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t05"></a>

### P04-T05 — Công bố nháp và supplied dependencies

- **Đầu ra.** Revision chờ Review gắn đúng request, context, phiên bản và dữ liệu hiện hành.
- **Đầu vào.** Dependency thực thi: [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P03-T02](PHASE-03.md#p03-t02), [P03-T04](PHASE-03.md#p03-t04). Căn cứ kỹ thuật: T03–T04, Q07–10/20, S-DATA, D04 và M3 commit/CAS.
- **Vùng sửa.** Application publish và adapter persist qua đường revision M3.

1. Giữ snapshot, mapping/ranges, exact base revision, model profile và toàn bộ context đã gửi. Lưu cả các mục model không báo đã dùng để invalidation theo M3.
2. Trước commit revalidate scope, source/version, base, Skill sync và điều kiện đã duyệt. Dùng operation ID và CAS của M2/M3.
3. Công bố revision chờ Review cùng current/history, supplied dependencies và outbox. Verify object trước khi tham chiếu nếu M3 dùng file. ACK phản ánh revision và tình trạng dựng Markdown.
4. Nếu source/rule/base đổi, giữ response/candidate để đối chiếu, không ghi đè bản người dùng. Nếu projection lỗi sau commit, tiếp tục từ revision đã lưu.
5. So DB/history, phần context được ghim và readback khi reopen. Phản hồi chưa kiểm hoặc ứng viên còn chờ không có current pointer.

**Kiểm chứng.** P04-AC05, P04-TC05, V-03. B kiểm cạnh tranh r1 → sửa tay r2, replay cùng operation và lỗi dựng Markdown.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T05 |
|---|---|
| Đầu vào bắt buộc | [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P03-T02](PHASE-03.md#p03-t02), [P03-T04](PHASE-03.md#p03-t04). Nhận draft hợp lệ, supplied dependency versions và writer/CAS M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Publish nhận base revision/context, ghi một revision chờ Review hoặc trả stale/conflict. Owner đầu ra là A-P04-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T05 giữ Application publish và adapter persist qua đường revision M3. Phần chung do P03-T01/T02 giữ write protocol, T04 giữ eligibility, T05 giữ publication. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC05 / P04-TC05, gồm các ca chi tiết §13.3. V-03. Đối chiếu Một generation đúng với revision/current/deps và trạng thái file. |
| Điểm tích hợp | [P04-CP02](#p04-cp02) — Dịch, candidate và công bố nháp; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P05-T08](PHASE-05.md#p05-t08). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t06"></a>

### P04-T06 — Review đọc nguồn độc lập

- **Đầu ra.** Source findings có phạm vi, quote và trạng thái bằng chứng, dùng lại được khi căn cứ còn khớp.
- **Đầu vào.** Dependency thực thi: [P04-T01](#p04-t01), [P04-T02](#p04-t02). Căn cứ kỹ thuật: T01/T02, Q21, S-REVIEW, D05, source M2 và context nguồn M3.
- **Vùng sửa.** Application source read, validator, cache/persistence phù hợp.

1. Dựng DTO chỉ có source và context nguồn đã kiểm chứng, đúng phạm vi và mốc độc giả. Dùng allowlist cho trường dữ liệu. Capture payload hoàn chỉnh để test target sentinel không xuất hiện, kể cả lồng trong context.
2. Nếu quá dung lượng, dùng planner để chia request nguồn liên tiếp. Đánh dấu overlap chỉ đọc, so union phạm vi với vùng được giao.
3. Kiểm snapshot/range/quote, người nói/người nghe, thời điểm, phủ định/giả định và điểm chưa rõ. Gom theo vị trí nguồn. Quote có thật mà nghĩa chưa chắc vẫn cần xác minh.
4. Lưu findings cùng version và scope. Chốt cache key gồm nguồn/ranges, context nguồn và profile/schema/prompt cần thiết. Đổi căn cứ làm cache cũ không đủ điều kiện.
5. Chỉ chuyển findings hợp lệ sang đối chiếu. Kết quả chưa xác minh giữ lý do, không tự thành fact chính thức.

**Kiểm chứng.** P04-AC06, P04-TC06, V-04. B kiểm serialized payload, source split/coverage và ca cache miss khi thay riêng một căn cứ.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T06 |
|---|---|
| Đầu vào bắt buộc | [P04-T01](#p04-t01), [P04-T02](#p04-t02). Nhận source ranges/context nguồn và planner đã kiểm. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Source-only DTO loại target, findings kèm quote/scope/cache basis và độ phủ đọc nguồn. Owner đầu ra là A-P04-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T06 giữ Application source read, validator, cache/persistence phù hợp. Phần chung do T01/T02 giữ gateway/planner, T06 giữ source findings D05. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T08](PHASE-03.md#p03-t08), [P03-T09](PHASE-03.md#p03-t09). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC06 / P04-TC06, gồm các ca chi tiết §13.3. V-04. Đối chiếu Capture không lộ target, coverage đủ và cache đổi theo input. |
| Điểm tích hợp | [P04-CP03](#p04-cp03) — Review đọc nguồn; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t07"></a>

### P04-T07 — Comparison, issue và coverage

- **Đầu ra.** Kết quả local Review gắn exact revision, bằng chứng, độ phủ và trạng thái rõ.
- **Đầu vào.** Dependency thực thi: [P04-T05](#p04-t05), [P04-T06](#p04-t06). Căn cứ kỹ thuật: T05–T06, Q22, S-REVIEW/S-RUBRIC và D05.
- **Vùng sửa.** Application comparison/evidence, domain coverage và storage/read model.

1. Dựng request đối chiếu từ source findings hợp lệ và đúng target revision/range đã ghim. Giữ nguồn, revision, rule/context versions và request ID. Không tự lấy current khác.
2. Kiểm issue ID, loại, phạm vi, quote và evidence state. Bỏ sót cần đoạn nguồn và vị trí thiếu trong bản Việt. Thêm ý cần đoạn Việt và vùng nguồn đã so để chỉ căn cứ.
3. Đối chiếu phủ nguồn→dịch và dịch→nguồn với tập source/target kỳ vọng. Overlap không cộng coverage lần hai. Bằng chứng thiếu hoặc phạm vi nền sai giữ chưa xác minh.
4. Phân biệt quote khớp với kết luận có đủ căn cứ. Dùng oracle kiểm phủ định, thêm/bỏ ý, cách diễn đạt khác vẫn đúng và ruby. Giữ cả hai kết quả mâu thuẫn. Gợi ý S/M gắn evidence, việc tự sửa thuộc Phase 06.
5. Lưu kết quả và trạng thái local đúng revision. Khi revision/source/context đổi, dùng dependency để đánh dấu cần kiểm. Issue list trống không tự thành Đạt.

**Kiểm chứng.** P04-AC07, P04-TC07, V-04. B kiểm quote thật nhưng kết luận sai, phạm vi thiếu hai chiều và kết quả gắn nhầm revision.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T07 |
|---|---|
| Đầu vào bắt buộc | [P04-T05](#p04-t05), [P04-T06](#p04-t06). Nhận revision Dịch cùng source findings và source map còn hiệu lực. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Compare trả issue/evidence/status theo exact revision, kiểm cả nguồn→đích và đích→nguồn. Owner đầu ra là A-P04-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T07 giữ Application comparison/evidence, domain coverage và storage/read model. Phần chung do T05 giữ target revision, T06 giữ findings, T07 giữ issue/coverage contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC07 / P04-TC07, gồm các ca chi tiết §13.3. V-04. Đối chiếu Độ phủ hai chiều và issue states đúng oracle. |
| Điểm tích hợp | [P04-CP04](#p04-cp04) — Đối chiếu và workflow cấp chương; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P06-T03](PHASE-06.md#p06-t03). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t08"></a>

### P04-T08 — Workflow nhiều lượt có Review ngay

- **Đầu ra.** State machine xác định thứ tự Dịch → Đọc nguồn → Đối chiếu → local result cho từng lượt.
- **Đầu vào.** Dependency thực thi: [P04-T02](#p04-t02), [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P04-T05](#p04-t05), [P04-T06](#p04-t06), [P04-T07](#p04-t07). Căn cứ kỹ thuật: T02–T07, Q24/29, S-WORKFLOW, D06/W7-01.
- **Vùng sửa.** Application workflow state, event và contract step của ADR-007.

1. Ghi bước, event, scope và điều kiện chuyển cho prepared, validate, publish, Review, waiting, pause và chapter pending. Giữ 3 trục theo M3.
2. Mở lượt trong phạm vi đã giao. Kiểm user intent, source/context/version và điều kiện chờ trước bước gửi. Chỉ dùng fake trong harness.
3. Sau nháp hợp lệ, Review đúng revision. Lưu local result và bước kế trước khi tiếp tục. Với source read cache còn hiệu lực, ghi rõ sử dụng lại findings trong trace.
4. Nếu cần duyệt, thiếu evidence, lỗi hoặc invalidation, giữ lý do và checkpoint. Lượt tiếp theo chưa được gửi. S0 và các ngoại lệ phải theo điều kiện hiệu lực của S-RUBRIC.
5. Khi xong tất cả local Review đủ điều kiện, chuyển chờ kết luận chương. Trạng thái hoàn tất không được set chỉ vì danh sách issue rỗng.

**Kiểm chứng.** P04-AC08, P04-TC08, V-05. B so request trace ca cold hai lượt và số request ở ca wait/fail/pause.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T08 |
|---|---|
| Đầu vào bắt buộc | [P04-T02](#p04-t02), [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P04-T05](#p04-t05), [P04-T06](#p04-t06), [P04-T07](#p04-t07). Nhận planner, publication và local Review cùng event identities. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Step/event điều phối dịch → Review → lượt tiếp, chờ/pause giữ intent theo W7-01. Owner đầu ra là A-P04-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T08 giữ Application workflow state, event và contract step của ADR-007. Phần chung do T02/T05/T07 giữ outputs, T08 giữ workflow event/state. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC08 / P04-TC08, gồm các ca chi tiết §13.3. V-05. Đối chiếu So trace và số request bị cấm. |
| Điểm tích hợp | [P04-CP04](#p04-cp04) — Đối chiếu và workflow cấp chương; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t09"></a>

### P04-T09 — Review tổng thể chương tối thiểu

- **Đầu ra.** Hồ sơ chương đúng tập revision, local results, độ phủ, điểm nối và vấn đề còn mở.
- **Đầu vào.** Dependency thực thi: [P04-T07](#p04-t07), [P04-T08](#p04-t08). Căn cứ kỹ thuật: T07–T08, Q24, S-REVIEW và D05.
- **Vùng sửa.** Application chapter check, read model và chapter gate.

1. Chụp tập revision hiện hành và các local results. Kiểm mọi result còn hiệu lực, đúng phạm vi và phủ hai chiều. Với local thiếu, đưa vùng thiếu về Review.
2. Lập request kiểm từng điểm nối/quan hệ xuyên cảnh bằng source và bản Việt thực. Lấy ít nhất một đoạn ở mỗi bên nếu có. Overlap là context, không tăng coverage.
3. Kiểm phủ định, người nói, thời gian, quan hệ, thuật ngữ và thêm/bớt ý bằng fixture có kỳ vọng riêng. Khi cần mở rộng, trả vùng và lý do thay vì kết luận sớm.
4. Lưu kết luận cùng tập revision, joins và blockers. Nếu lượt đầu sai hoặc đổi revision, tra dependency để mở lại vùng bị ảnh hưởng. Giữ local results độc lập còn hợp lệ.
5. Chỉ bước sang chương kế trong phạm vi đã giao khi kết luận chương còn hiệu lực và mọi điểm chặn đã xử lý. Dùng chương kế làm sentinel trong harness.

**Kiểm chứng.** P04-AC09, P04-TC09, V-04/V-05. B ưu tiên ca từng lượt đều đạt nhưng nối sai nghĩa và ca stale revision sau local Review.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T09 |
|---|---|
| Đầu vào bắt buộc | [P04-T07](#p04-t07), [P04-T08](#p04-t08). Nhận local issues/coverage, workflow và revision set chương. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Chapter conclusion gắn đúng tập revision/joins, stale basis mở lại gate và giữ chương sau chờ. Owner đầu ra là A-P04-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T09 giữ Application chapter check, read model và chapter gate. Phần chung do T07 giữ coverage, T08 giữ transitions, T09 giữ chapter read model. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC09 / P04-TC09, gồm các ca chi tiết §13.3. V-04, V-05. Đối chiếu Kết luận cũ hạ hiệu lực và chương sau giữ chờ. |
| Điểm tích hợp | [P04-CP04](#p04-cp04) — Đối chiếu và workflow cấp chương; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P06-T01](PHASE-06.md#p06-t01), [P06-T03](PHASE-06.md#p06-t03), [P07-T01](PHASE-07.md#p07-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t10"></a>

### P04-T10 — Checkpoint tối thiểu và restart

- **Đầu ra.** Reopen đúng step và dùng lại artifact hợp lệ của luồng giả lập.
- **Đầu vào.** Dependency thực thi: [P04-T08](#p04-t08), [P04-T09](#p04-t09). Căn cứ kỹ thuật: T08–T09, Q29, D-WORKFLOW, D07/W7-02/03, ADR-004.
- **Vùng sửa.** App workflow resume, storage checkpoint và migration M3→M4 khi có schema mới.

1. Chốt format/version checkpoint và mốc ghi. Lưu task/step/slot, request ID, source/mapping/context/base, refs tới response/revision/results và trạng thái waiting/paused. Dùng refs tới bản bất biến.
2. Ghi khi chuẩn bị, trước bước gửi giả lập, sau nhận đủ response, sau validate/commit và khi đổi sang Review, wait hoặc complete. Giữ counter sửa cấu trúc cùng công việc.
3. Reopen kiểm schema/version, artifact hash và căn cứ hiện hành. Có response hợp lệ thì validate. Có nháp commit thì Review bản đó. Có local result hợp lệ thì tiếp tục đúng lượt hoặc kiểm chương.
4. Với source/Skill/base đổi, dùng dependency để xác định phần cần re-plan/Review lại. Giữ phần cũ trong history. Checkpoint lỗi hoặc ref thiếu giữ chờ với chẩn đoán.
5. Giữ pause và ý định người dùng. Ca giả lập đã ghi mốc gửi nhưng chưa có response theo D07. Phase 05 nối intent, đối chiếu attempt và điều kiện gửi thật.
6. Tiêm ngắt từng mốc và kiểm số fake calls cùng current/history. Marker xác nhận child tới đúng chỗ khi dùng process crash.

**Kiểm chứng.** P04-AC10, P04-TC10, V-05. B thử response còn hợp lệ, revision đã commit, local result đã lưu, version lỗi và pause.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T10 |
|---|---|
| Đầu vào bắt buộc | [P04-T08](#p04-t08), [P04-T09](#p04-t09). Nhận workflow step, chapter gate và generation/version M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Checkpoint lưu artifact refs trước bước tiếp, resume đúng trạng thái và không lặp phần đã commit. Owner đầu ra là A-P04-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T10 giữ App workflow resume, storage checkpoint và migration M3→M4 khi có schema mới. Phần chung do T08 giữ event schema, T10 giữ ADR-007/W7-02/03 và migration. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC10 / P04-TC10, gồm các ca chi tiết §13.3. V-05. Đối chiếu Không gọi lại phần đã lưu hoặc tự biến pause thành quyền chạy. |
| Điểm tích hợp | [P04-CP05](#p04-cp05) — Lưu bước và restart; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P05-T06](PHASE-05.md#p05-t06), [P05-T12](PHASE-05.md#p05-t12). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t11"></a>

### P04-T11 — UI-05 danh sách lỗi

- **Đầu ra.** Reviewer list đọc đúng dữ liệu và ba trục trạng thái của target.
- **Đầu vào.** Dependency thực thi: [P04-T07](#p04-t07), [P04-T09](#p04-t09). Căn cứ kỹ thuật: T07/T09, S-TUI/D-UI và shell M1.
- **Vùng sửa.** TUI Reviewer list/state và application read model.

1. Nạp kết quả đã lưu theo workspace/tập/chương/revision và step. Phân biệt chưa có Review, thiếu coverage, cần xác minh và kết quả đã đạt.
2. Hiện evidence state, loại issue, S/M khi có căn cứ, nguồn và exact revision. Model/bước hiển thị từ request thực của demo, thiếu ID thực thì giữ chưa xác định.
3. Giữ mục chọn, filter, cuộn và focus khi refresh, đổi scope và resize. Khi target cũ, hiển thị rõ thay vì nhập chung vào bản hiện hành.
4. Kiểm trên hai revision có issue khác nhau và ca rỗng thiếu coverage. Render và thao tác đọc giữ nguyên generation.

**Kiểm chứng.** P04-AC11, P04-TC11, V-06 và phần danh sách của V-08. B đối chiếu IDs/state thực với fixture.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T11 |
|---|---|
| Đầu vào bắt buộc | [P04-T07](#p04-t07), [P04-T09](#p04-t09). Nhận issue list, revision set và trạng thái chapter gate. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Reviewer list lọc theo scope/version, refresh giữ selected issue, view chỉ đọc. Owner đầu ra là A-P04-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T11 giữ TUI Reviewer list/state và application read model. Phần chung do T07/T09 giữ read model, người tích hợp M4 giữ router. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T03](PHASE-03.md#p03-t03), [P03-T11](PHASE-03.md#p03-t11), [P03-T12](PHASE-03.md#p03-t12). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC11 / P04-TC11, gồm các ca chi tiết §13.3. V-06, V-08. Đối chiếu IDs/status/focus đúng, xem không ghi. |
| Điểm tích hợp | [P04-CP06](#p04-cp06) — Reviewer và mở Editor; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t12"></a>

### P04-T12 — UI-05 thẻ bằng chứng và mở Editor

- **Đầu ra.** Từ thẻ lỗi mở đúng đoạn nguồn/bản Việt của revision đã Review, rồi quay về đúng chỗ.
- **Đầu vào.** Dependency thực thi: [P04-T11](#p04-t11), [P03-T12](PHASE-03.md#p03-t12). Căn cứ kỹ thuật: T11, Q22/35, S-TUI/D-UI và UI-04 của M3.
- **Vùng sửa.** TUI evidence card, điều hướng Reviewer→Editor và read contract.

1. Hiện nguyên tác, bản Việt, quote/ranges, trạng thái bằng chứng, giải thích và S/M nếu có. Hiện lý do khi còn cần xác minh.
2. Tạo hành động mở Editor với workspace/tập/source/target/range/revision và return state. Revalidate ref lúc mở. Bản lịch sử có nhãn rõ, ref mất báo chẩn đoán.
3. Dùng cùng Editor M3 để xem hoặc sửa theo quyền của revision. Giữ bản đang soạn. Thao tác mở và quay lại chưa tự tạo revision hoặc request Review mới.
4. Quay lại giữ thẻ, filter, focus và vị trí cuộn. Kiểm card Nhật–Việt dài, resize và issue ở r1 khi current đã là r2.

**Kiểm chứng.** P04-AC12, P04-TC12, V-06/V-08. B đối chiếu route args với revision hiển thị thực và thao tác terminal.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T12 |
|---|---|
| Đầu vào bắt buộc | [P04-T11](#p04-t11), [P03-T12](PHASE-03.md#p03-t12). Nhận issue selection và Editor M3 với exact source/target. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Card evidence và route Editor mang issue/revision/source span, back giữ thẻ/bộ lọc/scroll. Owner đầu ra là A-P04-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T12 giữ TUI evidence card, điều hướng Reviewer→Editor và read contract. Phần chung do T11 giữ Reviewer route, P03-T11/T12 giữ Editor, T12 giữ liên kết hai màn. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T13](PHASE-03.md#p03-t13), [P03-T14](PHASE-03.md#p03-t14), [P05-T01](PHASE-05.md#p05-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC12 / P04-TC12, gồm các ca chi tiết §13.3. V-06, V-08. Đối chiếu Đúng route args, text và UI state thực. |
| Điểm tích hợp | [P04-CP06](#p04-cp06) — Reviewer và mở Editor; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P06-T14](PHASE-06.md#p06-t14), [P08-T01](PHASE-08.md#p08-t01), [P08-T04](PHASE-08.md#p08-t04). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p04-t13"></a>

### P04-T13 — Nghiệm thu luồng offline và M4

- **Đầu ra.** Evidence tích hợp từ source đến Reviewer/Editor, gồm các ca bị chặn và restart.
- **Đầu vào.** Dependency thực thi: [P04-T01](#p04-t01), [P04-T02](#p04-t02), [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P04-T05](#p04-t05), [P04-T06](#p04-t06), [P04-T07](#p04-t07), [P04-T08](#p04-t08), [P04-T09](#p04-t09), [P04-T10](#p04-t10), [P04-T11](#p04-t11), [P04-T12](#p04-t12), M3 Đạt. Căn cứ kỹ thuật: T01–T12, Q20–24/29/36, M3, R-REPO.
- **Vùng sửa.** App test/demo offline, fixture/README khi cần và hồ sơ M4.

1. Dựng workspace mẫu từ M3. Có hai lượt ở chương đầu và chương kế, inline bảo vệ, context Skill đã chọn và bảng kỳ vọng về range/quote/ý nghĩa.
2. Chạy toàn luồng trên app nối adapter thật và fake. So request order, IDs, markers, revision/coverage, chapter conclusion và M3 write/history. Chạy ca cấu trúc sai, quy ước chờ, quote giả và join sai.
3. Ngắt ở các mốc response, commit và local Review, mở lại và so data versions, trạng thái cùng số fake calls. Nếu migration M4 đổi schema, kiểm bản M3 sau nâng và khi lỗi.
4. Chạy terminal thật và bài tự test mục 16.4. Ghi lệnh dựng mẫu, cách mở đúng build, expected UI và hạn chế. B tự đi qua hướng dẫn trên cùng candidate.
5. Chạy V-00 và V-09 nếu đổi dependency. A tạo handoff M4, B review bản tích hợp theo mục 15. Ghi phần bàn giao provider/repair và phạm vi chưa nghiệm thu.

**Kiểm chứng.** P04-AC13, P04-TC13, V-07/V-08/V-00. B so oracle độc lập với trace/DB/UI. Phạm vi M4 là workflow giả lập.

**Sáu trường giao việc**

| Trường | Nội dung cho P04-T13 |
|---|---|
| Đầu vào bắt buộc | [P04-T01](#p04-t01), [P04-T02](#p04-t02), [P04-T03](#p04-t03), [P04-T04](#p04-t04), [P04-T05](#p04-t05), [P04-T06](#p04-t06), [P04-T07](#p04-t07), [P04-T08](#p04-t08), [P04-T09](#p04-t09), [P04-T10](#p04-t10), [P04-T11](#p04-t11), [P04-T12](#p04-t12), M3 Đạt. Nhận candidate Dịch–Review, checkpoint, Reviewer/Editor và fixture oracle. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Báo cáo M4 so trace/DB/file/UI trên cùng candidate, ca nghĩa theo oracle đã xác nhận. Owner đầu ra là A-P04-T13. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P04-T13 giữ App test/demo offline, fixture/README khi cần và hồ sơ M4. Phần chung do owners T01–T12 sửa tại boundary, A-T13 tích hợp. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T01](PHASE-05.md#p05-t01), [P05-T02](PHASE-05.md#p05-t02), [P05-T03](PHASE-05.md#p05-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P04-AC13 / P04-TC13, gồm các ca chi tiết §13.3. V-07, V-08, V-00. Đối chiếu So trace/DB/file/UI, ghi đúng giới hạn giả lập. |
| Điểm tích hợp | [P04-CP07](#p04-cp07) — Nghiệm thu M4; A chat P04 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P04-AC01 | Fake dùng cùng contract, capture payload thực và không tự ghi state. | P04-T01 | P04-TC01 · V-01 |
| P04-AC02 | Phạm vi nguồn được giao đúng một lần. Kiểm đủ input/output/context và giữ phần bắt buộc. Block không có chỗ ngắt an toàn bị chặn kèm vị trí. Re-plan bằng điểm chia/giới hạn tường minh, protected inline nguyên vẹn. | P04-T02 | P04-TC02 · V-02 |
| P04-AC03 | Response sai giữ ở vùng ứng viên/chẩn đoán. Tối đa hai lần sửa cấu trúc, tách khỏi sửa chất lượng. Phản hồi cắt đi qua lập lại phạm vi. | P04-T03 | P04-TC03 · V-02 |
| P04-AC04 | Candidate có provenance và chỉ phần đủ điều kiện được lưu nháp. Quy ước chưa duyệt chặn đúng phạm vi và chưa active. | P04-T04 | P04-TC04 · V-02 |
| P04-AC05 | Commit đúng căn cứ thành nháp chờ Review. Giữ supplied dependencies, sửa tay và response stale trong lịch sử. | P04-T05 | P04-TC05 · V-03 |
| P04-AC06 | Lần đọc nguồn độc lập giữ target ngoài payload. Findings/cache chỉ dùng khi đúng nguồn, context và phạm vi. | P04-T06 | P04-TC06 · V-04 |
| P04-AC07 | Issue/coverage gắn exact revision và bằng chứng đúng. Thiếu phủ hoặc mâu thuẫn giữ cần xác minh. | P04-T07 | P04-TC07 · V-04 |
| P04-AC08 | Review hợp lệ của lượt trước đi trước lượt tiếp. Chờ/paused/invalidated giữ đúng bước và không thành complete. | P04-T08 | P04-TC08 · V-05 |
| P04-AC09 | Kết luận chương còn hiệu lực ghim đúng tập revision, joins và coverage. Chương kế chỉ bắt đầu sau đủ điều kiện. | P04-T09 | P04-TC09 · V-04, V-05 |
| P04-AC10 | Reopen dùng lại artifact hợp lệ và đúng step. Counter và pause giữ nguyên. Checkpoint lạ có chẩn đoán. | P04-T10 | P04-TC10 · V-05 |
| P04-AC11 | Danh sách Reviewer đọc đúng target, evidence và ba trục. Rỗng khi chưa có kết quả hoặc thiếu phủ không hiển thị Đạt. | P04-T11 | P04-TC11 · V-06, V-08 |
| P04-AC12 | Thẻ mở đúng source/target/revision. Ref cũ hoặc thiếu được báo rõ. Quay lại đúng thẻ và vị trí. | P04-T12 | P04-TC12 · V-06, V-08 |
| P04-AC13 | Luồng offline tích hợp có bằng chứng thứ tự, coverage, lỗi, restart, migration và UI đúng scope. | P04-T13 | P04-TC13 · V-07, V-08, V-00 |

A nối mỗi AC với test hoặc ca thủ công đã chạy. B đối chiếu phạm vi, bằng chứng và test có thể thất bại khi hành vi sai.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P04-TC01 / P04-AC01 | Success/cut/error và request ngoài script | Payload sai làm test fail. DB không đổi khi chỉ gọi fake. | V-01 |
| P04-TC02 / P04-AC02 | Profile fixture, overlap, ruby, ngưỡng và block không chia được | So budget/coverage với oracle. Không gửi khi bị chặn. Một re-plan cho đầu vào hợp lệ đã xác nhận. | V-02 |
| P04-TC03 / P04-AC03 | JSON/ID/marker sai, ba response lỗi và finish reason cắt | Không tạo current sai hoặc tự lặp sau giới hạn. | V-02 |
| P04-TC04 / P04-AC04 | Thuật ngữ/ruby mới, quote giả và Skill pending | Không áp quy tắc hoặc publish vùng phụ thuộc trước đủ điều kiện. | V-02 |
| P04-TC05 / P04-AC05 | Cạnh tranh r1/r2, context đổi, outbox lỗi | Một generation đúng với revision/current/deps và trạng thái file. | V-03 |
| P04-TC06 / P04-AC06 | Target sentinel, source split, quote sai và cache keys | Capture không lộ target, coverage đủ và cache đổi theo input. | V-04 |
| P04-TC07 / P04-AC07 | Phủ định, thêm/bỏ, diễn đạt đúng, ruby và quote giả | Độ phủ hai chiều và issue states đúng oracle. | V-04 |
| P04-TC08 / P04-AC08 | Hai lượt, local pass/fail, wait và pause | So trace và số request bị cấm. | V-05 |
| P04-TC09 / P04-AC09 | Local pass nhưng nối sai, revision đổi, chương sentinel | Kết luận cũ hạ hiệu lực và chương sau giữ chờ. | V-04, V-05 |
| P04-TC10 / P04-AC10 | Failpoints, versions và fake call counts | Không gọi lại phần đã lưu hoặc tự biến pause thành quyền chạy. | V-05 |
| P04-TC11 / P04-AC11 | Nhiều revision, issue list rỗng và refresh | IDs/status/focus đúng, xem không ghi. | V-06, V-08 |
| P04-TC12 / P04-AC12 | Issue r1 khi current r2, card Nhật–Việt dài | Đúng route args, text và UI state thực. | V-06, V-08 |
| P04-TC13 / P04-AC13 | Hai lượt/hai chương, dữ liệu tự soạn, terminal thật | So trace/DB/file/UI, ghi đúng giới hạn giả lập. | V-07, V-08, V-00 |

### 13.1. Lệnh và môi trường

Tên package/test là mục tiêu của task. Ghi tên ca được phát hiện và đã chạy. Exit 0 với 0 ca liên quan chưa chứng minh AC.

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | `cargo test -p nt-application --test model_gateway_contract --locked` |
| V-02 | `cargo test -p nt-application --test translation_contract --locked` |
| V-03 | `cargo test -p nt-storage --test translation_commit --locked` |
| V-04 | `cargo test -p nt-application --test review_contract --locked` |
| V-05 | `cargo test -p nt-application --test phase04_workflow --locked` |
| V-06 | `cargo test -p nt-tui --test reviewer_evidence --locked` |
| V-07 | `cargo test -p novel-translate --test phase04_slice --locked` |
| V-08 | Demo Reviewer→Editor trên terminal Windows thật. M4 chạy trọn [bài tự test](#p04-user-test) với lệnh dựng bộ mẫu và bản chạy được bàn giao. |
| V-09 | Kiểm MSRV khi thêm/đổi dependency theo ADR-001. Với mốc 1.88.0 dùng `cargo +1.88.0 test --workspace --all-features --locked`. |

**V-00 — Bộ kiểm tra Rust trước PR và trước đóng M4.**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

Chuẩn bị toolchain/dependency trước rồi chạy offline theo test strategy. Fixture tự soạn, dùng thư mục tạm, clock/ID/barrier kiểm soát được, cô lập mạng ứng dụng. Không dùng provider, key hoặc sách riêng. V-07 chạy adapter lưu thật với fake. V-08 tách khỏi test buffer.

### 13.2. Cách dùng bằng chứng

- **Oracle.** Viết trước bảng slot/ranges, token kỳ vọng, request order, exact revisions, quotes và nhãn ý nghĩa cho ví dụ tự soạn. Dùng cặp đúng/sai rõ về phủ định, người nói, thêm/bỏ ý và đoạn nối. Fake trả theo kịch bản, test so output ứng dụng với oracle.
- **Đo payload thật.** Assert dữ liệu đã serialize, cả request bị chặn và request không được gửi. Scope giao và overlap tách riêng. B kiểm mutation đổi một điều kiện trọng yếu có thể làm test fail.
- **Restart.** Ghi artifact/state trước ngắt và khi reopen. Với process crash xác nhận marker đúng điểm. So số fake calls, DB/history và hash. Giữ lỗi môi trường riêng với ca chưa đạt.
- **Giới hạn.** JSON/quote/coverage test chứng minh phép kiểm cấu trúc và routing. Độ đúng nghĩa trên corpus và model thật thuộc Phase 09.

### 13.3. Chi tiết ca thử theo task

Hậu tố phân rã từng nhóm TC. Nối từng ca với tên test hoặc bước thủ công lúc handoff.

| Ca | Thao tác trên fixture | Kết quả phải đối chiếu |
|---|---|---|
| P04-TC01a | Gửi request đúng ID/role/schema, thử trả một lần và gom nhiều phần. | Kết quả hoàn tất có cùng contract và scope. Capture đúng payload. |
| P04-TC01b | Đổi một trường hoặc gửi request không có trong script. | Fake báo lỗi test. Không tự chọn phản hồi thành công. |
| P04-TC01c | Giả lập response cut, error và thiếu model/usage thực tế. | Giữ finish reason/lỗi có kiểu, giá trị chưa biết và artifact dở. |
| P04-TC01d | Chạy với mạng cô lập, không key. Kiểm app release chưa cấu hình. | Demo dùng fake, app release không âm thầm chọn fake. Fake không ghi DB. |
| P04-TC02a | Đếm payload có nguồn, Skill, context, khung JSON và phần bao. Thử thiếu giới hạn hoặc profile ước lượng. | Kết quả đúng oracle từng thành phần. Có trạng thái ước lượng, thiếu giới hạn thì chặn. |
| P04-TC02b | Lần lượt chạm/vượt input, output và tổng context. Thử reasoning đã nằm trong output. | Chỉ trần bị vượt gây chặn đúng lý do. Reasoning không đếm hai lần. |
| P04-TC02c | Bố trí part/cảnh ở 20% cuối, cuối đoạn/câu và context tùy chọn. | Chọn đúng điểm ưu tiên sau khi cắt tùy chọn. Phần bắt buộc và inline giữ nguyên. |
| P04-TC02d | Block vượt trần, mọi điểm cắt tiềm năng nằm trong protected units. | Không gọi fake. Diagnostic đúng chương/XHTML path/BlockID/range, trần và lý do. |
| P04-TC02e | Cấp điểm chia hoặc giới hạn tường minh. Thử đầu vào sai trước và đúng sau. | Sai giữ blocked. Hợp lệ lập lại đúng một lần cho vùng chưa gửi, giữ phần đạt còn hiệu lực. |
| P04-TC02f | Hai đoạn giống chữ, nhiều slot và overlap chỉ đọc. | Tập slot khớp oracle, đủ thứ tự và mỗi vùng giao đúng một lần. |
| P04-TC03a | Gửi JSON hỏng, thiếu/trùng/sai thứ tự ID, ID ngoài scope hoặc target rỗng. | Trả lỗi có vị trí. Không có current mới. |
| P04-TC03b | Marker cùng số nhưng sai ID, ruby/link/ảnh/note mất hoặc lệch. | Lỗi đúng marker/slot. Không tự điền target. |
| P04-TC03c | Lần Dịch đầu và hai phản hồi sửa cấu trúc đều sai. | Có tối đa hai yêu cầu sửa. Giữ bộ đếm chất lượng riêng và bước chờ. |
| P04-TC03d | Response bị cắt, so với ca sửa cấu trúc thành công. | Ca cắt giữ artifact và lập lại phạm vi/dự trù. Ca sửa hợp lệ vẫn qua toàn validator. |
| P04-TC04a | Response dùng quy ước mới hoặc ruby chưa chốt, có scope rõ. | Phần phụ thuộc giữ candidate và decision IDs. Không đưa vào official context. |
| P04-TC04b | Candidate có quote/range sai hoặc nói dữ kiện chưa chứng minh. | Có chẩn đoán hoặc cần xác minh. Không tạo rule/fact active. |
| P04-TC04c | Giữ đề xuất hợp lệ nhưng phần khác không phụ thuộc. | Dedup có provenance. Không áp quy tắc mới cho vùng độc lập. |
| P04-TC04d | Xác nhận đúng version, thử còn pending sync rồi bản đã active. | Pending vẫn chờ. Đủ điều kiện thì dựng lại đúng phần và validate. |
| P04-TC05a | Commit draft đủ điều kiện với context đã chọn từ M3. | Nháp chờ Review và supplied dependencies gồm toàn bộ đã gửi, kể cả item model không nhắc. |
| P04-TC05b | Hai bản dùng cùng base, người dùng lưu r2 trước publish candidate. | Stale giữ history, không ghi đè r2. Ref/current giữ nguyên. |
| P04-TC05c | Đổi source/mapping/Skill hoặc policy trước commit. | Chặn công bố trên căn cứ cũ và chỉ đúng vùng cần kiểm lại. |
| P04-TC05d | Tiêm lỗi dựng Markdown sau commit rồi reopen/replay. | Revision đã lưu đúng một lần. Outbox pending hoặc hoàn tất đúng job. |
| P04-TC06a | Gắn target sentinel vào bản Việt, current draft và context đầu vào. Capture request nguồn thật. | Payload đã serialize chỉ chứa source và context nguồn được phép, không có target sentinel. |
| P04-TC06b | Source quá ngân sách, tách nhiều request và có overlap. | Union đúng vùng được giao. Overlap chỉ đọc, không đếm lại. |
| P04-TC06c | Response có quote giả, range sai hoặc thiếu source coverage. | Không chấp nhận làm findings đủ điều kiện. Lỗi chỉ đúng vùng. |
| P04-TC06d | Cache hit cùng căn cứ. Đổi riêng source, phạm vi hoặc context/profile/schema liên quan. | Chỉ hit khi key và trạng thái hợp lệ. Phần đổi cần đọc lại. |
| P04-TC07a | Đối chiếu exact target với local findings hợp lệ, có S0 tùy chọn. | Phủ cả nguồn→dịch và dịch→nguồn. Kết quả theo đúng điều kiện S-RUBRIC. |
| P04-TC07b | So ca mất phủ định, bỏ câu và thêm câu. | Bỏ sót có quote nguồn + vị trí thiếu. Thêm ý có quote Việt + vùng nguồn đã đối chiếu. |
| P04-TC07c | Quote không tồn tại hoặc đúng chữ nhưng ở sai revision/range. | Từ chối evidence sai. Kết quả không apply lên revision khác. |
| P04-TC07d | Fixture có bản diễn đạt khác vẫn đúng và ruby chỉ phát âm. | Oracle không xem đây là thiếu dịch. Ruby có nghĩa riêng vẫn cần căn cứ theo lựa chọn. |
| P04-TC07e | Có quote thật nhưng kết luận bị ngữ cảnh hoặc lượt kiểm chứng khác bác. | Giữ cả hai kết quả và đưa về Cần xác minh. Không nâng thành lỗi chắc chắn chỉ vì quote khớp. |
| P04-TC07f | Source phủ đủ nhưng target có phần chưa đối chiếu, overlap bị tính lặp. | Giữ chưa đủ coverage, chỉ đúng phần thiếu. Không đạt vì issue list rỗng. |
| P04-TC08a | Chạy ca cold hai lượt đều đủ điều kiện. | Trace Dịch 1 → Đọc nguồn 1 → Đối chiếu 1 → Dịch 2. Local result đã lưu trước bước kế. |
| P04-TC08b | Local result lỗi/cần xác minh hoặc ứng viên chờ quyết định. | Giữ bước và lý do. Lượt kế không có request. |
| P04-TC08c | Pause hoặc căn cứ đổi trước bước gửi kế. | Kiểm lại scope/version và intent. Không gửi trên bản cũ hoặc tự resume. |
| P04-TC08d | Hết lượt local nhưng chưa có conclusion chương. | Giữ trạng thái chờ kiểm chương, chưa đánh dấu complete. |
| P04-TC09a | Hai local pass và điểm nối có đoạn hai phía. | Kết luận ghim đúng revisions, joins và phủ hai chiều. |
| P04-TC09b | Mỗi lượt đúng riêng nhưng nối sai người nói, phủ định hoặc thời gian. | Có evidence nối và phạm vi cần kiểm. Chương sentinel chưa chạy. |
| P04-TC09c | Đổi một revision hoặc phát hiện local coverage thiếu. | Hạ hiệu lực conclusion cũ và mở đúng vùng ảnh hưởng. Phần độc lập còn hiệu lực. |
| P04-TC09d | Kết luận chương đạt rồi chuyển chương đã giao. | Lưu kết luận trước request kế. Không sang chương ngoài scope. |
| P04-TC10a | Ngắt sau lưu response và trước validate. | Reopen validate cùng artifact đúng hash. Không gọi Dịch lần nữa. |
| P04-TC10b | Ngắt sau commit draft rồi sau local result lưu. | Review exact revision hoặc tiếp tục bước kế. Không lặp Dịch/Review đã đủ căn cứ. |
| P04-TC10c | Checkpoint version lạ, ref thiếu hoặc source/context đổi. | Chẩn đoán hoặc re-plan/Review lại đúng scope. Giữ lịch sử, không tự mở bước sai. |
| P04-TC10d | Reopen lúc paused/chờ và sau một lần sửa cấu trúc. | Giữ intent, lý do và counter. Không có request mới ngoài điều kiện. |
| P04-TC11a | Mở Reviewer ở hai tập và hai revision cùng chương. | Issue IDs và ba trục đúng target. Bản cũ có nhãn rõ. |
| P04-TC11b | Issue list rỗng nhưng chưa Review hoặc thiếu coverage. | Hiện trạng thái tương ứng, không hiện Đạt. |
| P04-TC11c | Refresh, đổi filter, resize và quay lại mục đang xem. | Giữ lựa chọn/focus theo đúng scope. Không đổi generation. |
| P04-TC11d | Có requested model nhưng chưa có actual model, kết quả pending/verified/rejected. | Bước và model/issue states đúng với dữ liệu đã lưu. |
| P04-TC12a | Mở thẻ có quote/ranges, lỗi thêm và bỏ sót trên fixture. | Nguyên tác, bản Việt, vùng so và evidence state đúng. |
| P04-TC12b | Mở Editor từ issue r1 khi current là r2. | Mở đúng r1 có nhãn lịch sử. Ref thiếu báo lý do, không nhảy nhầm. |
| P04-TC12c | Quay lại sau đổi kích thước và xem card Nhật–Việt dài. | Còn thẻ/filter/focus/cuộn đúng. Draft đang soạn được giữ. |
| P04-TC12d | Chỉ mở/đóng thẻ, xem nguồn và quay lại. | Không tạo revision, Review hoặc request mới. |
| P04-TC13a | Chạy app offline với M3, hai lượt/hai chương, Skill và inline fixture. | So trace, source/target, refs, dependencies, coverage và conclusion với oracle. |
| P04-TC13b | Chạy cấu trúc sai, rule wait, quote giả và điểm nối sai. | Dừng đúng bước và số request. Readback giữ dữ liệu còn hợp lệ. |
| P04-TC13c | Ngắt ở mốc đã lưu rồi reopen. Nếu có migration, chạy thành công và tiêm lỗi trên bản M3. | Đúng artifact/step/counter. Source, revision và mốc trước được bảo toàn. |
| P04-TC13d | Chạy V-00 và V-08 trên candidate cuối, đối chiếu tài liệu mẫu. | Đủ bằng chứng app/terminal và giới hạn nền tảng. Bộ thử không gọi provider. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md). A làm trong phạm vi task/nhóm, tự kiểm rồi tạo handoff có baseline, candidate, căn cứ và kết quả thật. B đọc AC, kiểm độc lập cùng đầu vào và ghi report. Nếu code, spec hoặc fixture ảnh hưởng hành vi đổi, tạo lượt mới để B kiểm phần ảnh hưởng.

| Phần | B cần đối chiếu |
|---|---|
| T01–T02 | Fake ở đúng composition. So serialized payload, token oracle và exact slots. Chặn gửi khi thiếu giới hạn hoặc block không có điểm ngắt an toàn. |
| T03–T05 | Schema, ID và marker. Hai lần sửa cấu trúc. Candidate chưa duyệt không publish. Revalidate versions và giữ đủ supplied dependencies. |
| T06–T07 | Source-only không lộ target. Quote/scope/coverage đúng exact revision. Mâu thuẫn và thiếu căn cứ hiện đúng. |
| T08–T10 | Thứ tự thực của request, điều kiện local/chapter, checkpoint và reopen. Kiểm số lần gọi khi có sẵn artifact và khi paused. |
| T11–T13 | UI đọc đúng thẻ/revision, không tự tạo request. Demo thật trên đúng build, fixture và hệ điều hành. |

Rà chiều phụ thuộc, API có consumer, IO ngoài event loop, log đã lọc và ca test có thể phát hiện lỗi. B ghi phép kiểm tự chạy, bằng chứng đã đối chiếu và điều kiện chưa xác minh. Terminal thật thuộc V-08.

A commit phần đã được B duyệt theo [Git workflow](../../../AGENTS.md#git-workflow). Khi cả phạm vi PR đã qua kiểm tra và review tích hợp, A push nhánh theo phần việc và mở/cập nhật Draft PR vào `rewrite/rust`. Commit và PR mô tả thay đổi thực bằng tiếng Anh đơn giản. Hồ sơ fixture, request/response riêng và bằng chứng ở máy người dùng.

## 15. Điều kiện đóng phase

- [ ] T01–T13 đủ AC và ca kiểm liên quan. D01–D07 có quyết định tại consumer. Review B Đạt còn hiệu lực cho các phần đã dùng.
- [ ] B review Đạt bản tích hợp theo [mẫu review phase](../templates/PHASE_REVIEW.md). Có định danh code, test, config, dependency và căn cứ. Phần thay đổi sau lượt trước đã được kiểm lại.
- [ ] Bộ thử offline chứng minh exact slots, đủ ba trần, case block không chia được, chặn gửi và re-plan có chủ đích. Fixture có protected inline và Skill có provenance.
- [ ] Có trace Dịch → Đọc nguồn → Đối chiếu cho từng lượt, phủ hai chiều, kết luận chương đúng tập revision và điểm nối. Không gửi lượt/chương sau khi còn chặn.
- [ ] Evidence thiếu/giả/trái nhau và candidate chờ được xử lý đúng. Response stale không ghi đè nháp tay. Reopen giữ artifact, step và pause/counter.
- [ ] Reviewer→Editor chạy trên terminal thật. Lệnh dựng mẫu và hướng dẫn tự test khớp candidate. Ca UI buffer và thủ công có bằng chứng riêng.
- [ ] V-00 và các phép kiểm mục 13 đạt. V-09 chạy nếu có thay đổi dependency. Nếu chạm phần chung Go, có phép kiểm đường Go còn dùng. Giữ tài liệu, key, sách và output riêng.
- [ ] Hồ sơ M4 ghi giới hạn giả lập và đủ đầu ra để Phase 05/06 nhận. Kết quả chất lượng model thật và quyết định thay Go giữ phạm vi đã ghi ở các phase sau.

## 16. Bàn giao và bằng chứng

A dùng [mẫu handoff task](../templates/TASK_HANDOFF.md). B dùng [mẫu review task](../templates/TASK_REVIEW.md) và [mẫu review phase](../templates/PHASE_REVIEW.md) cho M4. Báo cáo trong `docs/roadmap/evidence/phase-04/` theo lượt, ví dụ `P04-T01.handoff-r01.md`, `P04-T01.candidate-r01.sha256` và `P04-T01.review-r01.md`. Giữ finding IDs và report cũ khi sửa.

A phase cập nhật sổ sau handoff/verdict; điều phối cập nhật PLAN trung tâm từ báo cáo đó. B chỉ ghi report riêng. Thiếu bằng chứng bắt buộc giữ đúng trạng thái.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 7 checkpoint của lịch v2: [P04-CP01](#p04-cp01), [P04-CP02](#p04-cp02), [P04-CP03](#p04-cp03), [P04-CP04](#p04-cp04), [P04-CP05](#p04-cp05), [P04-CP06](#p04-cp06), [P04-CP07](#p04-cp07); xem [§10.2](#p04-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task / cổng | Trạng thái | Candidate / báo cáo A–B |
|---|---|---|---|
| P04-A | [T01](#p04-t01) | Chưa bắt đầu | — |
| P04-A | [T02](#p04-t02) | Chưa bắt đầu | — |
| P04-A | [T03](#p04-t03) | Chưa bắt đầu | — |
| P04-A | [T04](#p04-t04) | Chưa bắt đầu | — |
| P04-A | [T05](#p04-t05) | Chưa bắt đầu | — |
| P04-B | [T06](#p04-t06) | Chưa bắt đầu | — |
| P04-B | [T07](#p04-t07) | Chưa bắt đầu | — |
| P04-C | [T08](#p04-t08) | Chưa bắt đầu | — |
| P04-C | [T09](#p04-t09) | Chưa bắt đầu | — |
| P04-C | [T10](#p04-t10) | Chưa bắt đầu | — |
| P04-D | [T11](#p04-t11) | Chưa bắt đầu | — |
| P04-D | [T12](#p04-t12) | Chưa bắt đầu | — |
| P04-E | [T13](#p04-t13) | Chưa bắt đầu | — |
| P04-E | M4 | Chưa bắt đầu | — |

### 16.2. Báo cáo mỗi task

1. Ghi task/AC, phạm vi và hành vi đã đổi, ADR và baseline.
2. Định danh candidate bằng commit nếu có, diff và hashes của file liên quan, kể cả untracked, config/dependency, fixture và tài liệu.
3. Nối test/ca thủ công với AC. Ghi command, môi trường, số ca, kết quả và phần chưa chạy. Giữ trace payload, request order, IDs, checkpoint và readback đã lọc.
4. Ghi findings, phần cần sửa/chưa xác minh và việc tiếp theo. Khi tới mốc Git, ghi commit/PR và checks đã chạy trên đúng bản.

### 16.3. Đầu ra cho bước tiếp theo

| Nơi nhận | Nội dung bàn giao |
|---|---|
| [Phase 05](PHASE-05.md) | Gateway DTO và adapter conformance fixtures. Profile/budget/diagnostic, checkpoint version và mốc lưu, request IDs. Nối token/provider/attempt/permit thật, báo lỗi block không chia được qua UI-06/CLI. |
| [Phase 06](PHASE-06.md) | Review target/evidence, local/chapter result, phạm vi cần xác minh, waiting và dependency của từng lượt. Nối repair, Research, approval wake-up và tiếp tục có quyền. |
| [Phase 07](PHASE-07.md) | Kết luận chương, exact revision set, coverage và blockers dùng khi kiểm sẵn sàng xuất. |
| [Phase 08](PHASE-08.md) | Read model Reviewer, route đến Editor và hành vi trạng thái/resize dùng trong kiểm CLI/TUI. |
| [Phase 09](PHASE-09.md) | Fixture nguồn–bản Việt với oracle và trace regressions. Dùng làm căn cứ cho corpus thực, thử chất lượng model và xem phạm vi thay Go. |

Khi P03-CP04 đã Đạt/công bố và PLAN giao Phase 04, bắt đầu ở [P04-T01](#p04-t01). Đọc bàn giao context đã nhận, chốt D01 cho DTO/fake và W7-14, chạy V-01 rồi bàn giao B trước planner.

<a id="p04-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử candidate **M4** cùng bộ chạy thử dùng phản hồi model giả lập. Bài thử tập trung vào thứ tự Dịch–Review, thẻ lỗi, đường mở Editor và lưu điểm tiếp tục. A ghi đúng lệnh chạy bộ thử, lệnh mở executable và vị trí các dữ liệu mẫu. B dùng lại các lệnh trên bản sao sạch.

#### 16.4.2. Chuẩn bị

Dành khoảng 15 phút. Dùng chương mẫu có ít nhất hai lượt, một bộ phản hồi đủ điều kiện và một bộ phản hồi có lỗi đã mô tả. Chuẩn bị bảng thứ tự sự kiện và dòng Nhật–Việt cần đối chiếu ở chỗ nối lượt. Bài thử chạy cục bộ với workspace mẫu.

#### 16.4.3. Mở đúng bản

Chạy bộ thử offline ghi trong bàn giao M4 để tạo hoặc mở workspace mẫu. Mở **Reviewer** bằng bản Rust M4 với đúng workspace vừa tạo. Bản bàn giao chỉ rõ thao tác nào chạy trong bộ thử và thao tác nào thực hiện trên TUI.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P04-UT01 — Hai lượt liên tiếp | Chạy kịch bản hợp lệ và đối chiếu thứ tự ở báo cáo của bộ thử. | Dịch lượt 1 rồi Review lượt 1 trước khi bắt đầu Dịch lượt 2. Kết quả chương chỉ hoàn tất sau kiểm tra tổng thể. |
| P04-UT02 — Mở đúng chỗ từ thẻ lỗi | Nạp bộ mẫu có issue, chọn thẻ đã chỉ định trong **Reviewer** và mở Editor từ thẻ. Sau đó quay lại. | Thẻ nêu nguyên tác, bản Việt và revision có vấn đề. Editor mở đúng đoạn/revision và khi quay lại vẫn chọn đúng thẻ. |
| P04-UT03 — Thiếu bằng chứng | Mở mẫu có câu trích không tồn tại hoặc thiếu một đoạn đã chỉ rõ. Đối chiếu vị trí lỗi với bảng mẫu. | Kết quả cho biết chỗ không đủ bằng chứng. Chương còn ở trạng thái chưa hoàn tất. |

#### 16.4.5. Thử ca lỗi và mở lại

**P04-UT04 — Dừng rồi tiếp tục.** Dùng kịch bản dừng ở mốc đã lưu ghi trong bàn giao. Mở lại workspace và tiếp tục qua bộ thử. Ca đạt khi đúng lượt/revision được lấy lại và báo cáo cho thấy phần đã lưu không được chạy lại.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Kèm candidate M4, tên kịch bản, thứ tự bước thực tế hoặc thẻ lỗi và revision đang mở nếu kết quả khác bảng mẫu.

#### 16.4.7. Thử lại từ đầu

Tạo workspace mẫu mới bằng lệnh của bộ thử. Chọn lại đúng bộ phản hồi trước khi chạy lại ca.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-04. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P04 rà **Phần chuẩn bị request, lưu kết quả và Review** tại các mốc **P04-CP02/CP04/CP05; đánh giá lại khi P05/P06 nhận consumer thật**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Chỉ dọn helper Go không còn caller nếu có bằng chứng riêng. Luồng model giả lập Phase 04 không đủ điều kiện xóa đường dịch/model/Review thật của Go; ghi phần này chờ P05/P06 và phép kiểm sản phẩm tương ứng.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
