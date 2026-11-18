# PHASE 06 — WORKFLOW DỊCH–REVIEW–RESEARCH–REPAIR ĐẦY ĐỦ

> Lịch task song song và checkpoint commit/push tại [mục 10](#p06-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Draft 0.5 — 09/11/2026 · Chưa triển khai.** Lập từ đặc tả và thiết kế theo yêu cầu của người dùng; code hiện tại không là chuẩn thiết kế, đầu vào tái sử dụng hoặc bằng chứng hoàn thành. W7-10/11/12 và Research/cache retention còn mở; điều kiện và mục 6 nay phản ánh đúng trạng thái proposal, với lịch owner/hạn tại [ROADMAP §4.1](../ROADMAP.md#decision-schedule). Lịch thực thi v2 ngày 12/11/2026; thay lịch v0.1 chưa triển khai.
>
> Cấu trúc theo [Phase 01](PHASE-01.md). Task, đường dẫn module và tên test là kế hoạch ban đầu; rà lại trước khi giao việc khi contract của phase trước thay đổi. Không tự thay yêu cầu đã chốt để khớp implementation.
>
> **Cập nhật 07/11/2026 (P00-T01-r02):** P06-T02 cite Q13; đối chiếu hash Skill/mapping/generation ngay trước lượt chưa gửi; P06-TC02 phân biệt payload chưa gửi (nền mới), watcher chậm/trễ và lượt đã gửi (giữ nền cũ). Review P00-T01-r01 — finding F04.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P06 / M6 |
| Tên | Workflow Dịch–Review–Research–Repair đầy đủ |
| Trạng thái | Chưa bắt đầu; mọi AC và kiểm thử bên dưới chưa có kết quả thực thi |
| Đầu vào chính | M5 executor/Coordinator/budget/recovery; M4 Review; M3 rules/dependency |
| Gói giao diện | UI-08, UI-09, UI-10 |
| Phân rã | 5 gói việc, 16 task |
| Nhánh khi triển khai | rewrite/rust; giữ Go theo R-REPO |
| Người thực hiện / rà soát | Model A triển khai / model B khác review độc lập; chưa phân công |

Xem [roadmap chung](../ROADMAP.md) và [phạm vi tại Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-06). Tài liệu và bằng chứng nội bộ ở trong `docs/`, ngoài Git.

## 2. Mục tiêu và demo cuối phase

Hoàn thiện tác vụ dịch phần đã chọn hoặc tự động đến hết EPUB, đi qua mọi điểm chờ và sửa mà giữ đúng thứ tự, căn cứ và quyền người dùng. Research cung cấp bằng chứng; repair thay đúng phạm vi; kết luận chương chỉ hình thành khi đủ coverage và căn cứ còn hiệu lực.

1. Chọn all-text hoặc story, chọn chương hoặc to-end độc lập, bắt đầu tác vụ một lần.
2. Fixture đi qua local Review lỗi → sửa cục bộ → Review lại → kiểm tra chương → chương tiếp.
3. Gặp tên/ruby mới, chọn đúng nhóm thuật ngữ và xác nhận; task chỉ tiếp tục khi đủ điều kiện/quyền chạy.
4. Gặp nghi vấn cần Research, đọc nguồn có bằng chứng và loại dữ kiện tương lai/chưa rõ mốc.
5. Tạo năm vòng sửa thất bại; bàn giao người dùng; sửa tay hoặc giữ ngoại lệ đúng revision.
6. Restart trong chờ duyệt/retry/repair và tiếp tục đúng số vòng, tiền và phạm vi đã giao.

## 3. Phạm vi

Task scope và automatic-to-end; context preflight; Review evidence đầy đủ; repair M1/M2/M3/M4; counters/exception; Research internal/web/cache/spoiler; propagation; UI-08/09/10; CLI nghiệp vụ cần cho các luồng này.

Ảnh/Preview/EPUB packaging ở Phase 07. Hoàn thiện các luồng điều hướng/CLI trên toàn ứng dụng ở Phase 08. Đánh giá model thật với bộ chuẩn đầy đủ ở Phase 09; CI vẫn dùng fixture và oracle được rà soát.

## 4. Điều kiện bắt đầu

P06-T01 nhận executor/recovery P05-T12, chapter gate P04-T09 và approval P03-T10 đã Đạt. Repair và Research mở theo dependency từng task. P06-T08/T12 bàn giao exceptions, scope, coverage và resume để P07-T01 kiểm readiness trên dữ liệu thật. P06-T16/M6 vẫn cần M5 và mọi task P06.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-TRANS | [Giai đoạn Dịch](../../spec/TRANSLATION_STAGE_SPEC.md) |
| S-REVIEW | [Giai đoạn Review](../../spec/REVIEW_STAGE_SPEC.md) |
| S-RUBRIC | [Rubric S0–S3 và M1–M4](../../spec/REVIEW_RUBRIC_SPEC.md) |
| S-RESEARCH | [Research và bằng chứng](../../spec/RESEARCH_STAGE_SPEC.md) |
| S-WORKFLOW | [Điều phối Dịch–Review](../../spec/WORKFLOW_ORCHESTRATION_SPEC.md) |
| S-MODEL | [Model, retry và chi phí](../../spec/MODEL_STRATEGY_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-SKILL | [Skills và duyệt thuật ngữ](../../spec/SKILLS_GLOSSARY_SPEC.md) |
| S-ADDRESS | [Xưng hô và ngữ cảnh](../../spec/ADDRESSING_CONTINUITY_SPEC.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| D-WORKFLOW | [Bước 7 — State machine và recovery](../../desgin/WORKFLOW_STATE_MACHINE_STEP7.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Ký hiệu Qxx trong tài liệu là quyết định xx của S-Q. Hành vi đã chốt ưu tiên hơn phương án kỹ thuật còn đề xuất. Thư viện, tên bảng/trait và SQL nháp phải được rà theo consumer thực tế; không tự áp dụng DDL thiết kế vào workspace.

- Content mode và selected/to-end là hai lựa chọn; task giữ effective scope/config/budget qua chương và restart.
- Review ngay sau lượt; chapter gate trước chương tiếp; không tự thêm tập/chương ngoài phạm vi.
- S0 là gợi ý; S1–S3 cần bằng chứng; severity không tự quyết định M. Core chọn phạm vi nhỏ nhất chứng minh được.
- M4 cần lý do chọn lọc không đủ và lượt kiểm chứng nguồn bổ sung có quyền thu hẹp/bác.
- Một failed repair round chỉ tính khi bản sửa hợp lệ được Review lại và vẫn xác nhận lỗi của đợt; network/structure/wait tách riêng.
- Hết năm failed rounds thì chờ người dùng; manual edits/restart/đổi scope không xóa lịch sử vòng đã dùng.
- Ngoại lệ gắn người duyệt/lý do/issue/scope/revision/căn cứ; đổi nền phải xét lại; không bỏ điều kiện kỹ thuật/coverage/thuật ngữ.
- Research tra nội bộ trước, web search và page reader riêng; Brave Search API là adapter đầu tiên theo đặc tả.
- Giới hạn thử nghiệm Research mỗi câu hỏi 3 queries, 5 pages, 15 phút; model deadline lấy phần nhỏ hơn còn lại; phí tool riêng cùng task.
- Dữ kiện có source/hash/time/entity/story time/earliest reader position; unknown spoiler boundary không vào context hiện tại.
- Batch approval có thể cho task đang chờ đúng mục tiếp tục khi đủ điều kiện; task user-paused vẫn cần resume.

## 6. ADR và điểm còn mở

| Vấn đề | Cần trước | Nội dung |
|---|---|---|
| W7-10/11/12, ADR-007/008 | P06-T02/T04/T12 theo consumer | Còn mở: revalidation, repair plan/round, event schema và recovery policy; lịch owner/hạn tại [ROADMAP §4.1](../ROADMAP.md#decision-schedule). Không đánh dấu đã chốt chỉ từ state machine. |
| Research provider/read transport | T09 | Brave contract, nguồn ưu tiên/chặn, timeout/size/redirect policy và error mapping |
| DB-11, evidence/cache | T10/T11 | Retention, source hashes, cache keys, revalidation và quyền lưu search response |
| Eligibility/wake-up | T12/T15 | Chờ quyết định khác chủ động paused; recheck permission/budget/dependencies |
| UI task scope/CLI inputs | T13 | Tách content mode, selected/to-end và effective configuration |

Trạng thái ban đầu của các quyết định trên là cần đối chiếu/ghi nhận, không phải đã được thông qua. Dùng ADR đã có nếu còn phù hợp; chỉ mở rộng phần mới. Ghi lựa chọn, đánh đổi, task phụ thuộc và phép kiểm chứng. Chỉ phần thật sự thay đổi yêu cầu sản phẩm mới cần quyết định sản phẩm bổ sung.

## 7. Thiết kế triển khai

Application workflow điều phối Translation, Review, Research và RepairPlan; tất cả request model đi qua executor P05. `nt-research` chỉ hiện thực search/read/cache IO theo port, không tự approve rules hay gọi Translation. Domain sở hữu severity/scope/round/exception invariants; storage lưu evidence/plan/checkpoint và migration; TUI gọi cùng commands.

Luồng chính: resolve effective scope → prepare context → Translate → validate/commit → Review source/compare → nếu lỗi thì xác minh/correct cause/repair → Review lại → chapter gate → next assigned chapter. Research/approval/budget/pause là điểm chờ được lưu, không nhánh bỏ qua Review.

Contract có consumer thực tế mới được công khai. Domain/Application giữ quy tắc và quyền công bố; adapter thực hiện IO qua port; TUI/CLI gọi cùng use case. Không nối hai adapter qua implementation nội bộ để né ownership hoặc tạo dependency vòng.

### 7.1. Đường đi và dấu vết của một tác vụ

1. Lưu phạm vi hiệu lực theo tập/chương và chế độ nội dung. Dùng lại lựa chọn người dùng, giới hạn và bộ đếm khi mở lại task.
2. Trước từng lượt chưa gửi, tra đúng bản nguồn, căn cứ, ranh giới người đọc và bản Skill đã đồng bộ. Ghim các phiên bản thực dùng vào lần gửi. Nếu căn cứ mới đến khi đang chờ, chạy lại chuẩn bị và kiểm ngân sách/quyền trước khi gửi.
3. So kết quả Review với đúng tập revision. Lưu kế hoạch sửa cùng vùng ảnh hưởng đã xác minh. Ghi bản sửa và trạng thái chờ Review sao cho mở lại có thể Review đúng revision đã commit.
4. Giữ quyết định người dùng, chờ nguồn và chủ động pause thành các nguyên nhân riêng. Chỉ xét chạy tiếp khi điều kiện của đúng nguyên nhân đã được giải quyết. Qua chapter gate mới chọn phần kế tiếp trong task plan.

Khi bàn giao sang M7, đưa một fixture có cả chương đạt, chương chờ duyệt và chương có ngoại lệ còn hiệu lực. Ghi source/translation/Review IDs, coverage, căn cứ và trạng thái của từng phần để bên xuất có thể kiểm readiness mà không gọi model.

## 8. Dữ liệu, lỗi và phục hồi

| Lỗi/điểm ngắt | Hành vi |
|---|---|
| Mapping/rule/context là nguyên nhân sai | Sửa căn cứ trước; invalidation đúng chuỗi rồi mới repair target |
| Bản sửa commit, chưa Review | Review revision đó, không tạo thêm vòng vô lý |
| Năm vòng thất bại | Giữ counter/evidence, chuyển chờ người dùng; không tạo task mới để né giới hạn |
| Research giới hạn/lỗi/không đọc được trang | Trả bằng chứng hiện có và điều còn thiếu; không bịa quote/URL |
| Source web đổi/mất | Giữ đoạn đã lưu với thời điểm, đánh dấu cần kiểm chứng lại |
| Approval xong nhưng task user-paused | Không tự chạy; resume vẫn đi qua queue và preflight |
| Exception target/căn cứ đổi | Không áp exemption cũ cho dữ liệu mới |

Partial response/uncertain usage/retry/handoff dùng nguyên giao thức P05. Recovery của workflow không được reset repair rounds, tool budgets hoặc quyền đã bị thu hồi.

Mỗi lỗi cần giữ định danh phạm vi và nguyên nhân đã redaction, cùng bước tiếp theo. Không dùng thông báo thành công thay cho trạng thái đang chờ. Mọi thay đổi schema đã áp dụng phải có migration/version và kiểm thử tương thích; dữ liệu, lịch sử và sửa tay được bảo toàn.

## 9. Phần TUI/CLI

UI-08: form chọn tập/content mode và selected/to-end, hiện model/budget có hiệu lực; start có chủ đích, không hỏi lại mỗi chương đủ điều kiện. UI-09: Review lại exact revision, evidence Research có nguồn/mốc/còn thiếu; Editor return giữ thẻ lỗi. UI-10 nối chọn file/thư mục Skill, bật/tắt thư viện và preview Skill riêng theo [Phase 03 §7.4](PHASE-03.md#p03-skill-format). Cùng màn xử lý chọn nhiều candidates, lựa chọn/scope từng mục, nút xác nhận đúng số đã chọn, conflict và chờ đồng bộ.

Nối `translate`, `review`, `issues`, `terms list/confirm`, `pause/resume/status` theo S-TUI. Điểm chờ non-interactive trả checkpoint/task/next_action. Không thêm shortcut bỏ duyệt hoặc tự tăng giới hạn.

Trên cây nội dung, nối hành động **Nhờ Research đề xuất** sau import cho trường hợp phân loại chưa rõ (Q02). Đây là yêu cầu có chủ đích, chịu ngân sách; người dùng chốt loại trang. Quyết định gắn EPUB hash, mapping version và vị trí, được đối chiếu lại khi nguồn/mapping đổi.

Thông báo sản phẩm dùng ngôn ngữ người dùng hiểu được; mã task/ADR/crate chỉ nằm trong tài liệu kỹ thuật. Điều hướng, cuộn, resize và mở trợ giúp không tự gửi model hoặc thay quyết định nghiệp vụ.

<a id="p06-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P06 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P06-T01 nhận executor/recovery P05-T12, chapter gate P04-T09 và approval P03-T10 đã Đạt. Repair và Research mở theo dependency từng task. P06-T08/T12 bàn giao exceptions, scope, coverage và resume để P07-T01 kiểm readiness trên dữ liệu thật. P06-T16/M6 vẫn cần M5 và mọi task P06. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p06-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p06-cp01"></a>P06-CP01 — Scope, context và evidence | [P06-T01](#p06-t01), [P06-T02](#p06-t02), [P06-T03](#p06-t03) | [P03-CP05](PHASE-03.md#p03-cp05), [P04-CP04](PHASE-04.md#p04-cp04), [P05-CP06](PHASE-05.md#p05-cp06) | P06-T01: V-01 trong phạm vi AC/TC của task; P06-T02: V-01 trong phạm vi AC/TC của task; P06-T03: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp02"></a>P06-CP02 — Repair cục bộ và theo cảnh | [P06-T04](#p06-t04), [P06-T05](#p06-t05) | [P06-CP01](#p06-cp01) | P06-T04: V-03 trong phạm vi AC/TC của task; P06-T05: V-03 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp03"></a>P06-CP03 — Dịch lại chương, số vòng và ngoại lệ | [P06-T06](#p06-t06), [P06-T07](#p06-t07), [P06-T08](#p06-t08) | [P06-CP01](#p06-cp01), [P06-CP02](#p06-cp02) | P06-T06: V-03 trong phạm vi AC/TC của task; P06-T07: V-03 trong phạm vi AC/TC của task; P06-T08: V-03 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp04"></a>P06-CP04 — Research, cache và nguồn chứng cứ | [P06-T09](#p06-t09), [P06-T10](#p06-t10), [P06-T11](#p06-t11) | [P06-CP01](#p06-cp01) | P06-T09: V-04 trong phạm vi AC/TC của task; P06-T10: V-04 trong phạm vi AC/TC của task; P06-T11: V-04 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp05"></a>P06-CP05 — Propagation và resume | [P06-T12](#p06-t12) | [P03-CP05](PHASE-03.md#p03-cp05), [P06-CP01](#p06-cp01), [P06-CP02](#p06-cp02), [P06-CP03](#p06-cp03), [P06-CP04](#p06-cp04) | P06-T12: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp06"></a>P06-CP06 — Giao diện Dịch và Review | [P06-T13](#p06-t13), [P06-T14](#p06-t14) | [P02-CP06](PHASE-02.md#p02-cp06), [P04-CP06](PHASE-04.md#p04-cp06), [P06-CP01](#p06-cp01), [P06-CP04](#p06-cp04), [P06-CP05](#p06-cp05) | P06-T13: V-06 trong phạm vi AC/TC của task; P06-T14: V-06 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp07"></a>P06-CP07 — Giao diện Skills và approval | [P06-T15](#p06-t15) | [P03-CP03](PHASE-03.md#p03-cp03), [P03-CP05](PHASE-03.md#p03-cp05), [P06-CP05](#p06-cp05) | P06-T15: V-06 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |
| <a id="p06-cp08"></a>P06-CP08 — Nghiệm thu M6 | [P06-T16](#p06-t16) | M5 Đạt, [P06-CP01](#p06-cp01), [P06-CP02](#p06-cp02), [P06-CP03](#p06-cp03), [P06-CP04](#p06-cp04), [P06-CP05](#p06-cp05), [P06-CP06](#p06-cp06), [P06-CP07](#p06-cp07) | P06-T16: V-07, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M6; V-00 cho code, B review đúng candidate. | feature/workflow; A chat P06 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Các đường dẫn dưới đây là vị trí mục tiêu từ thiết kế. Không sửa phần ngoài phạm vi chỉ để tiện triển khai. Tất cả task khởi tạo ở trạng thái **Chưa bắt đầu**; lệnh V-* nằm ở mục 13.

<a id="p06-t01"></a>

### P06-T01 — Phạm vi tác vụ và tự động đến hết

- **Mục tiêu / đầu ra:** Task plan lưu scope/config hiệu lực, thứ tự và phần còn thiếu.
- **Căn cứ / phụ thuộc:** Q01/24/29, S-WORKFLOW; M5. Dependency thực thi: [P05-T12](PHASE-05.md#p05-t12), [P04-T09](PHASE-04.md#p04-t09), [P03-T10](PHASE-03.md#p03-t10).
- **Phạm vi:** Application task planning/workflow; storage task config.
- **Việc làm:** Tách content mode khỏi selected/to-end; gồm phụ trợ; giữ phần đã hợp lệ; tự chuyển chương đã được giao khi đủ gate.
- **Ca lỗi / biên:** All-text bỏ phụ trang; to-end tự sang tập khác; đổi default reset budget; chọn lại chương đã hoàn tất.
- **Nghiệm thu:** `P06-AC01` — Không mở rộng phạm vi hoặc reset cấu hình/counter; hoàn tất theo đúng mọi phần bắt buộc.
- **Kiểm chứng:** `P06-TC01`, V-01; dữ liệu kiểm tra: Hai tập, nhiều chương/phụ trang, một phần đã dịch.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Phân giải lựa chọn nội dung và kiểu chạy thành danh sách phần bắt buộc theo thứ tự spine trong tập được giao.
2. Phân biệt phần đã hợp lệ với phần cần dịch hoặc Review lại trên nền đang dùng. Lưu lựa chọn, danh sách và cấu hình hiệu lực cùng task ID.
3. Đọc lại checkpoint khi resume và chỉ chuyển phần sau khi chapter gate ghi nhận coverage cần thiết.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC01.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T01 |
|---|---|
| Đầu vào bắt buộc | [P05-T12](PHASE-05.md#p05-t12), [P04-T09](PHASE-04.md#p04-t09), [P03-T10](PHASE-03.md#p03-t10). Nhận executor/Coordinator/recovery P05-T12 và coverage/rules P03-T10/P04-T09 đã duyệt. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Task plan lưu content mode riêng với selected/to-end, effective config/budget và thứ tự phần còn thiếu. Owner đầu ra là A-P06-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T01 giữ Application task planning/workflow; storage task config. Phần chung do M5 giữ executor/checkpoint, T01 giữ task scope. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC01 / P06-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Expected scope/order/remaining content đúng; không có request ngoài tập được giao. |
| Điểm tích hợp | [P06-CP01](#p06-cp01) — Scope, context và evidence; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t02"></a>

### P06-T02 — Context preflight đầy đủ trước mỗi lượt

- **Mục tiêu / đầu ra:** Request chỉ dùng context/rules đã đồng bộ đúng thời điểm.
- **Căn cứ / phụ thuộc:** Q09/12/13/16, S-TRANS; T01 và resolver P03. Dependency thực thi: [P06-T01](#p06-t01).
- **Phạm vi:** Application translation preparation/executor preflight.
- **Việc làm:** Kiểm Skill hashes, mapping/generation, unresolved conflicts; lọc reader boundary; pin supplied dependencies; đối chiếu hash Skill/mapping/generation ngay trước lượt chưa gửi.
- **Ca lỗi / biên:** File ngoài đổi, alias/rule mới, unknown listener, candidate chưa xác minh; Skill sửa giữa hai lượt (payload chưa gửi dùng nền mới hợp lệ; lượt đã gửi giữ nền cũ; watcher chậm vẫn an toàn nhờ hash check).
- **Nghiệm thu:** `P06-AC02` — Phần chưa gửi dùng nền mới hợp lệ; request đã gửi giữ nền thực dùng, không tráo history.
- **Kiểm chứng:** `P06-TC02`, V-01; dữ liệu kiểm tra: Skill edit giữa hai lượt (payload chưa gửi dùng nền mới; watcher chậm/trễ; lượt đã gửi giữ nền cũ) và flashback/alias ambiguity.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Nạp resolver M3 theo mục tiêu, ranh giới người đọc và mapping đã duyệt. Ghi ID/version của đúng phần Skill và facts đưa vào payload.
2. Ngay trước khi gửi, đối chiếu hash và generation kể cả khi chưa nhận watcher event. Làm mới phần chưa gửi, hoặc trả lý do chờ nếu căn cứ chưa đủ.
3. Lưu căn cứ thực dùng theo attempt. Khi kết quả về muộn, revalidate trước khi công bố revision theo contract W7/P05.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC02.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T02 |
|---|---|
| Đầu vào bắt buộc | [P06-T01](#p06-t01). Nhận task scope, resolver M3 và current Skill/mapping generation. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Payload chưa gửi đối chiếu hash tại preflight, attempt đã gửi giữ căn cứ thực dùng và publication revalidates. Owner đầu ra là A-P06-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T02 giữ Application translation preparation/executor preflight. Phần chung do P03-T07 giữ resolver, T01 giữ scope, T02 giữ context preflight. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC02 / P06-TC02, gồm các ca chi tiết §13.3. V-01. Đối chiếu Payload/dependency IDs đúng theo từng lượt; nền của lượt đã gửi không bị tráo; blocking reason theo căn cứ thiếu. |
| Điểm tích hợp | [P06-CP01](#p06-cp01) — Scope, context và evidence; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t03"></a>

### P06-T03 — Review evidence và ý nghĩa có kiểm chứng

- **Mục tiêu / đầu ra:** Issue/conclusion đủ source/target/provenance và xử lý bất đồng.
- **Căn cứ / phụ thuộc:** Q21–22/24, S-REVIEW/S-RUBRIC; P04 Review, T02. Dependency thực thi: [P06-T02](#p06-t02), [P04-T07](PHASE-04.md#p04-t07), [P04-T09](PHASE-04.md#p04-t09).
- **Phạm vi:** Application Review verification/evidence acceptance.
- **Việc làm:** Kiểm source findings, bidirectional coverage; đọc rộng khi mâu thuẫn; chỉ verified facts mới đủ điều kiện thành context. Nếu nền nguồn/độ phủ sai, mở lại toàn bộ phạm vi Review bị ảnh hưởng.
- **Ca lỗi / biên:** Quote tồn tại nhưng suy luận sai; dịch khác cách vẫn đúng; nguồn đọc chưa đủ; Review tự bịa sự kiện.
- **Nghiệm thu:** `P06-AC03` — Bác nhận định sai hoặc giữ cần xác minh; không coi quote match là chứng minh ngữ nghĩa.
- **Kiểm chứng:** `P06-TC03`, V-02; dữ liệu kiểm tra: Oracle phủ định/chủ thể ẩn/false quote/acceptable paraphrase.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Nối source-only và Compare với cùng source map. Đối chiếu evidence span và hai chiều phủ nguồn–đích trên tập revision mục tiêu.
2. Tách nhận định đã xác minh, nhận định bị bác và nhận định cần thêm căn cứ. Mở rộng vùng đọc khi lỗi phụ thuộc chủ thể hoặc câu lân cận.
3. Khi nguồn hay coverage đổi, mở lại Review ở phạm vi bị ảnh hưởng. Chỉ cấp facts đã xác minh cho resolver.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC03.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T03 |
|---|---|
| Đầu vào bắt buộc | [P06-T02](#p06-t02), [P04-T07](PHASE-04.md#p04-t07), [P04-T09](PHASE-04.md#p04-t09). Nhận source/Compare findings M4 và source-context basis hiện hành. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Review evidence phân verified/rejected/needs-check, quote và bidirectional coverage cùng exact revision. Owner đầu ra là A-P06-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T03 giữ Application Review verification/evidence acceptance. Phần chung do P04-T06/T07 giữ findings, T03 giữ evidence meaning contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC03 / P06-TC03, gồm các ca chi tiết §13.3. V-02. Đối chiếu So expected evidence status và false positives; ca chưa có oracle không được tính đạt nghĩa. |
| Điểm tích hợp | [P06-CP01](#p06-cp01) — Scope, context và evidence; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t04"></a>

### P06-T04 — Repair M1 và M2

- **Mục tiêu / đầu ra:** Bản sửa đúng các vị trí/đoạn đã chứng minh.
- **Căn cứ / phụ thuộc:** Q23, S-RUBRIC; T03. Dependency thực thi: [P06-T03](#p06-t03).
- **Phạm vi:** Domain repair scope; application selective repair.
- **Việc làm:** M1 precise patches kể cả nhiều vị trí; M2 paragraph; validate bounds/markers; commit revision và Review lại.
- **Ca lỗi / biên:** Patch chạm vùng ngoài plan; base stale; sửa lỗi cũ nhưng thêm lỗi mới.
- **Nghiệm thu:** `P06-AC04` — Phần ngoài scope còn hợp lệ được giữ; sửa luôn tạo revision và không tự đánh dấu đạt.
- **Kiểm chứng:** `P06-TC04`, V-03; dữ liệu kiểm tra: Một lỗi cục bộ nghiêm trọng và nhiều exact spans.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Tạo plan M1 cho tập span chính xác hoặc M2 cho block đã xác định. Ghim base revision và phạm vi trước khi chuẩn bị request.
2. Kiểm overlap, giới hạn vùng thay, token bảo toàn và revision hiện hành trước khi commit.
3. Ghi revision mới và việc phải Review lại. Đối chiếu diff phần ngoài vùng sửa và giữ lịch sử gốc.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC04.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T04 |
|---|---|
| Đầu vào bắt buộc | [P06-T03](#p06-t03). Nhận verified issue spans, target base và source markers. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | M1/M2 plan ghim vùng được sửa, commit revision mới rồi Review lại, lỗi bounds giữ bản trước. Owner đầu ra là A-P06-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T04 giữ Domain repair scope; application selective repair. Phần chung do T03 giữ issue evidence, T04 giữ W7-10 repair plan. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC04 / P06-TC04, gồm các ca chi tiết §13.3. V-03. Đối chiếu Diff/coverage chỉ vùng cho phép; phần mới có Review target đúng. |
| Điểm tích hợp | [P06-CP02](#p06-cp02) — Repair cục bộ và theo cảnh; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t05"></a>

### P06-T05 — Repair M3 và phụ thuộc xuyên cảnh

- **Mục tiêu / đầu ra:** Plan cảnh/nhóm đoạn có lý do và danh sách phạm vi.
- **Căn cứ / phụ thuộc:** Q09/23, S-RUBRIC; T04. Dependency thực thi: [P06-T04](#p06-t04).
- **Phạm vi:** Application repair planning/dependency expansion.
- **Việc làm:** Sửa nguyên nhân mapping/rule/context trước; mở rộng theo edges/occurrences; giữ vùng độc lập.
- **Ca lỗi / biên:** Severity cao làm mở cả chương; dependency cycle; nguồn căn cứ sai chưa bị vô hiệu.
- **Nghiệm thu:** `P06-AC05` — M3 chỉ gồm vùng có quan hệ chứng minh được; context sai không tiếp tục cấp cho lượt sau.
- **Kiểm chứng:** `P06-TC05`, V-03; dữ liệu kiểm tra: Sai tiền đề ở đầu cảnh ảnh hưởng hai đoạn sau và một đoạn độc lập.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Xác định mapping, rule hoặc ngữ cảnh gốc gây sai. Đi qua quy trình duyệt và công bố của M3 nếu căn cứ cần thay đổi.
2. Tính tập ảnh hưởng từ edge và occurrence có thể truy vết. Ghi lý do từng cảnh hoặc đoạn phải sửa trước khi mở rộng plan.
3. Cấp context mới sau khi đồng bộ. Kiểm diff để giữ đoạn độc lập và sửa tay ngoài tập ảnh hưởng.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC05.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T05 |
|---|---|
| Đầu vào bắt buộc | [P06-T04](#p06-t04). Nhận repair plan và dependency/occurrence M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | M3 sửa căn cứ trước, mở rộng affected set có edges và giữ vùng độc lập/sửa tay. Owner đầu ra là A-P06-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T05 giữ Application repair planning/dependency expansion. Phần chung do T04 giữ plan, P03-T04 giữ graph, T05 giữ scope expansion. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC05 / P06-TC05, gồm các ca chi tiết §13.3. V-03. Đối chiếu Affected set/diff đúng oracle, history và sửa tay ngoài phạm vi được giữ. |
| Điểm tích hợp | [P06-CP02](#p06-cp02) — Repair cục bộ và theo cảnh; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t06"></a>

### P06-T06 — M4 với kiểm chứng nguồn bổ sung

- **Mục tiêu / đầu ra:** Quyết định dịch lại chương có chứng cứ và quyền thu hẹp/bác.
- **Căn cứ / phụ thuộc:** Q23, S-RUBRIC; T03/T05. Dependency thực thi: [P06-T03](#p06-t03), [P06-T05](#p06-t05).
- **Phạm vi:** Application repair escalation/additional verification.
- **Việc làm:** Đòi lý do selective repair không đủ; request nguồn bổ sung; ghi original/proposed/confirmed scope.
- **Ca lỗi / biên:** Chỉ có nhiều S1/S3 mà thiếu lan truyền; verifier bất đồng; coverage verifier thiếu.
- **Nghiệm thu:** `P06-AC06` — Không M4 vì số lỗi hoặc severity đơn lẻ; chỉ công bố plan M4 khi điều kiện đạt.
- **Kiểm chứng:** `P06-TC06`, V-03; dữ liệu kiểm tra: M4 hợp lệ, M4 bị thu hẹp và đề xuất bị bác.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lưu đề xuất M4 kèm dấu hiệu lỗi toàn chương và lý do vùng nhỏ hơn chưa đủ. Gọi lượt kiểm chứng nguồn riêng qua executor.
2. Dựa trên evidence bổ sung, ghi quyết định giữ M4, thu hẹp hoặc bác cùng vùng và căn cứ tương ứng.
3. Nếu thiếu nguồn, ngân sách hoặc lượt kiểm chứng hợp lệ, đưa việc về trạng thái chờ và giữ đề xuất để truy vết.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC06.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T06 |
|---|---|
| Đầu vào bắt buộc | [P06-T03](#p06-t03), [P06-T05](#p06-t05). Nhận evidence và lý do selective repair chưa đủ. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | M4 cần request kiểm chứng nguồn bổ sung, lưu original/proposed/confirmed scope và quyền thu hẹp/bác. Owner đầu ra là A-P06-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T06 giữ Application repair escalation/additional verification. Phần chung do T03 giữ evidence, T05 giữ affected scope, T06 giữ M4 verification. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC06 / P06-TC06, gồm các ca chi tiết §13.3. V-03. Đối chiếu Request/decision trace chứng minh bổ sung; phạm vi sau quyết định đúng. |
| Điểm tích hợp | [P06-CP03](#p06-cp03) — Dịch lại chương, số vòng và ngoại lệ; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t07"></a>

### P06-T07 — Repair rounds và giới hạn năm vòng

- **Mục tiêu / đầu ra:** Counter theo repair plan/issue group/base/new revision.
- **Căn cứ / phụ thuộc:** Q25, S-MODEL; T04–T06. Dependency thực thi: [P06-T04](#p06-t04), [P06-T05](#p06-t05), [P06-T06](#p06-t06).
- **Phạm vi:** Domain repair rounds; application outcome recording; storage history.
- **Việc làm:** Chỉ tăng failed round sau valid repair + Review vẫn xác nhận lỗi; giữ qua mở rộng/manual/restart.
- **Ca lỗi / biên:** Network retry/JSON lỗi tăng nhầm; nhiều requests một round; reset khi chuyển chương/model.
- **Nghiệm thu:** `P06-AC07` — Round thất bại thứ năm đưa về người dùng; các counter network/structure riêng, không reset lịch sử.
- **Kiểm chứng:** `P06-TC07`, V-03; dữ liệu kiểm tra: 2 transient, 2 structural, 5 failed repair rounds và manual revision.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Gắn counter với đợt issue và cập nhật theo kết quả Review của bản sửa hợp lệ. Phân biệt attempt mạng, lỗi cấu trúc và vòng sửa.
2. Bảo toàn counter khi mở rộng M, resume hoặc đổi bản sửa tay trong cùng đợt issue. Chống áp lại cùng kết quả Review khi replay.
3. Đưa đợt issue về chờ người dùng ngay khi vòng thất bại thứ năm được ghi. Giữ evidence và lựa chọn tiếp theo.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC07.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T07 |
|---|---|
| Đầu vào bắt buộc | [P06-T04](#p06-t04), [P06-T05](#p06-t05), [P06-T06](#p06-t06). Nhận repair plan/revision và kết quả Review của từng đợt issue. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Counter tăng khi repair hợp lệ được Review xác nhận vẫn lỗi, replay/mở rộng/restart giữ đợt và dừng vòng thứ năm. Owner đầu ra là A-P06-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T07 giữ Domain repair rounds; application outcome recording; storage history. Phần chung do T04–T06 giữ plans, T07 giữ round identity/counter. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC07 / P06-TC07, gồm các ca chi tiết §13.3. V-03. Đối chiếu Đếm đúng từng loại; không có round tự động thứ sáu. |
| Điểm tích hợp | [P06-CP03](#p06-cp03) — Dịch lại chương, số vòng và ngoại lệ; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t08"></a>

### P06-T08 — Ngoại lệ người dùng gắn revision và căn cứ

- **Mục tiêu / đầu ra:** Quyết định giữ lỗi có provenance và validity.
- **Căn cứ / phụ thuộc:** Q25, S-RUBRIC; T03/T07. Dependency thực thi: [P06-T03](#p06-t03), [P06-T07](#p06-t07).
- **Phạm vi:** Application exception use case; storage exception records.
- **Việc làm:** Ghi reviewer/issue/severity/scope/revision/basis và người duyệt/lý do; revalidate khi nền đổi.
- **Ca lỗi / biên:** Ngoại lệ revision cũ; reason trống; thiếu coverage hoặc rule approval.
- **Nghiệm thu:** `P06-AC08` — Hiển thị có ngoại lệ; chỉ miễn đúng vấn đề, không vượt technical/coverage/approval gates.
- **Kiểm chứng:** `P06-TC08`, V-03; dữ liệu kiểm tra: Exception hợp lệ rồi target/rule đổi; technical invalid draft.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Hiển thị đúng issue, severity, target revision và căn cứ khi người dùng chọn giữ ngoại lệ. Lưu danh tính/quyết định và lý do theo contract.
2. Đối chiếu lại scope, revision và dependency versions khi tính chapter gate hoặc readiness.
3. Giữ nhãn ngoại lệ còn hiệu lực hoặc cần xét lại trong lịch sử. Chạy đầy đủ các gate kỹ thuật, coverage và phê duyệt khác.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC08.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T08 |
|---|---|
| Đầu vào bắt buộc | [P06-T03](#p06-t03), [P06-T07](#p06-t07). Nhận verified issue, severity và failed-round state. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Ngoại lệ lưu người/lý do/scope/revision/basis, đổi căn cứ phải xét lại và giữ các gate kỹ thuật khác. Owner đầu ra là A-P06-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T08 giữ Application exception use case; storage exception records. Phần chung do T03 giữ evidence, T07 giữ episode, T08 giữ exception validity. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC08 / P06-TC08, gồm các ca chi tiết §13.3. V-03. Đối chiếu Gate khác vẫn chặn; ngoại lệ cũ không áp vào revision mới. |
| Điểm tích hợp | [P06-CP03](#p06-cp03) — Dịch lại chương, số vòng và ngoại lệ; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P07-T01](PHASE-07.md#p07-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t09"></a>

### P06-T09 — Research nội bộ, Brave search và page reader

- **Mục tiêu / đầu ra:** Công cụ tìm URL/đọc trang tách vai trò, có nguồn.
- **Căn cứ / phụ thuộc:** Q02/26, S-RESEARCH/S-SEC; M5 và T03. Dependency thực thi: [P06-T03](#p06-t03).
- **Phạm vi:** nt-research adapters, application research use case.
- **Việc làm:** Tìm nguyên tác/glossary/verified results trước; dùng Brave được cấu hình khi cần; đọc đoạn liên quan rồi mới cite. Nhận câu hỏi phân loại từ hành động người dùng sau import; kết quả vẫn là đề xuất.
- **Ca lỗi / biên:** Search snippet không có page; URL không đọc được; nguồn chặn; trang chứa chỉ dẫn giả hoặc đích truy cập không được phép.
- **Nghiệm thu:** `P06-AC09` — Không cite trang chưa đọc, không làm theo chỉ dẫn trong nội dung web; tool fees/requests qua policy chung.
- **Kiểm chứng:** `P06-TC09`, V-04; dữ liệu kiểm tra: Search/page transcripts, redirects/errors và untrusted page.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Đưa cùng câu hỏi qua tra nguyên tác, glossary và findings đã xác minh. Chỉ mở web route khi được phép và thiếu căn cứ.
2. Chốt contract Brave/page reader cùng policy nguồn, redirect, size và timeout ở T09. Search result cung cấp nơi đọc, bằng chứng cần phần trang đã đọc.
3. Gắn tool attempts, lỗi và chi phí với task qua chính sách P05. Giữ yêu cầu phân loại từ người dùng thành đề xuất để họ chốt.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC09.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T09 |
|---|---|
| Đầu vào bắt buộc | [P06-T03](#p06-t03). Nhận evidence schema và executor/budget M5. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Tra nguồn nội bộ trước, search chỉ dẫn tới page read, quote dựa trang đã đọc và tool attempts gắn task. Owner đầu ra là A-P06-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T09 giữ nt-research adapters, application research use case. Phần chung do T03 giữ evidence, M5 giữ send/money, T09 giữ Research ports. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC09 / P06-TC09, gồm các ca chi tiết §13.3. V-04. Đối chiếu Call sequence nội bộ→search→read đúng; không dùng key thật hoặc mạng ngoài. |
| Điểm tích hợp | [P06-CP04](#p06-cp04) — Research, cache và nguồn chứng cứ; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t10"></a>

### P06-T10 — Research limits, cache và thời hạn

- **Mục tiêu / đầu ra:** Budget/tool calls/cache có provenance và hiệu lực.
- **Căn cứ / phụ thuộc:** Q26, S-MODEL; T09. Dependency thực thi: [P06-T09](#p06-t09).
- **Phạm vi:** Application research limits/cache policy; storage research cache.
- **Việc làm:** 3 queries/5 pages/15min defaults; deadline min; phí tool riêng; cache key scope/entity/reader/source config/basis; TTL web 7 ngày.
- **Ca lỗi / biên:** Hết deadline giữa call; query/page cap; nguồn đổi trước TTL; thiếu quyền lưu raw response.
- **Nghiệm thu:** `P06-AC10` — Dừng đúng giới hạn, trả bằng chứng còn thiếu; không reset task/model cap hoặc dùng cache sai mốc.
- **Kiểm chứng:** `P06-TC10`, V-04; dữ liệu kiểm tra: Fake clock boundary TTL/deadline, changed basis và tool fees.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Tạo giới hạn hiệu lực cho mỗi câu hỏi. Dùng clock xác định deadline còn lại trước search/read/model và ghi từng khoản phí theo loại.
2. Tra cache theo câu hỏi, thực thể, mốc truyện, scope nguồn, cấu hình nguồn và phiên bản căn cứ. Hết hạn web hoặc có mâu thuẫn thì kiểm lại.
3. Khi chạm giới hạn hoặc resume, giữ số lượt đã dùng và trả evidence còn dùng được cùng phần thiếu.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC10.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T10 |
|---|---|
| Đầu vào bắt buộc | [P06-T09](#p06-t09). Nhận question/task identity, search/read attempts và nguồn price. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Research áp query/page/time caps, cache theo scope/entity/mốc/basis, giữ phí và counters khi resume. Owner đầu ra là A-P06-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T10 giữ Application research limits/cache policy; storage research cache. Phần chung do T09 giữ IO, P05-T04/T05 giữ ledger, T10 giữ cache/limits DB-11. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC10 / P06-TC10, gồm các ca chi tiết §13.3. V-04. Đối chiếu No extra call khi hết cap/time; cache reuse/revalidate đúng điều kiện. |
| Điểm tích hợp | [P06-CP04](#p06-cp04) — Research, cache và nguồn chứng cứ; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t11"></a>

### P06-T11 — Evidence Research và giới hạn spoiler

- **Mục tiêu / đầu ra:** Fact-level evidence đủ mốc/phạm vi/verification.
- **Căn cứ / phụ thuộc:** Q27, S-RESEARCH/S-ADDRESS; T09–T10. Dependency thực thi: [P06-T09](#p06-t09), [P06-T10](#p06-t10).
- **Phạm vi:** Domain evidence/context event; application admissibility.
- **Việc làm:** Lưu quote/hash/time/entity/story time/earliest reader position; lọc từng fact trong cùng trang; terms mới về candidates.
- **Ca lỗi / biên:** Trang có cả hiện tại/tương lai; không biết mốc; nguồn mất hoặc trái nguyên tác.
- **Nghiệm thu:** `P06-AC11` — Unknown/future facts giữ trong hồ sơ nhưng không vào context cảnh hiện tại; conflict cần xác minh.
- **Kiểm chứng:** `P06-TC11`, V-04; dữ liệu kiểm tra: Một trang chứa ba loại fact và cache xuyên tập.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lưu từng fact kèm đoạn căn cứ, định danh nguồn, hash, thời điểm và thực thể. Xác định story time và mốc sớm nhất người đọc biết nếu có căn cứ.
2. Lọc từng fact ở lúc dựng context. Fact mâu thuẫn, chưa rõ mốc hoặc tương lai ở lại hồ sơ để xác minh.
3. Gửi thuật ngữ mới vào candidate pipeline M3. Căn cứ trang mất hoặc đổi vẫn giữ provenance cũ và có trạng thái cần kiểm lại.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC11.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T11 |
|---|---|
| Đầu vào bắt buộc | [P06-T09](#p06-t09), [P06-T10](#p06-t10). Nhận page quote/hash/time và cache scope/versions. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Fact có source và mốc biết sớm nhất, future/unknown giữ ngoài context hiện tại, term mới vào candidate M3. Owner đầu ra là A-P06-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T11 giữ Domain evidence/context event; application admissibility. Phần chung do T03 giữ evidence status, T10 giữ retention, T11 giữ fact/spoiler model. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC11 / P06-TC11, gồm các ca chi tiết §13.3. V-04. Đối chiếu Context chỉ chứa facts đủ điều kiện, source history không bị xóa. |
| Điểm tích hợp | [P06-CP04](#p06-cp04) — Research, cache và nguồn chứng cứ; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t12"></a>

### P06-T12 — Propagation, chờ và resume toàn workflow

- **Mục tiêu / đầu ra:** Workflow mở đúng vùng bị ảnh hưởng và giữ stop intent.
- **Căn cứ / phụ thuộc:** Q09/24/28–29, S-WORKFLOW; T01–T11. Dependency thực thi: [P06-T01](#p06-t01), [P06-T02](#p06-t02), [P06-T03](#p06-t03), [P06-T04](#p06-t04), [P06-T05](#p06-t05), [P06-T06](#p06-t06), [P06-T07](#p06-t07), [P06-T08](#p06-t08), [P06-T09](#p06-t09), [P06-T10](#p06-t10), [P06-T11](#p06-t11), [P03-T10](PHASE-03.md#p03-t10).
- **Phạm vi:** Application workflow/dependency/wake-up; checkpoint schema.
- **Việc làm:** Nối rule/evidence changes, approval publication, repair và chapter gate; recheck budget/permit trước wake.
- **Ca lỗi / biên:** Approval tự resume user-paused; chapter kết luận trên revision cũ; Research lỗi làm mất nháp.
- **Nghiệm thu:** `P06-AC12` — Chỉ task chờ đúng quyết định và còn được phép mới tiếp tục; mọi kết luận dùng exact current basis.
- **Kiểm chứng:** `P06-TC12`, V-05; dữ liệu kiểm tra: Approval wait vs user-paused, rejected event và queued resume.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Nối event publication với đúng task đang chờ và tập dependency ảnh hưởng. Chốt replay/dedup và transaction boundaries theo W7.
2. Sau publication, đọc lại preflight, budget và quyền chạy của P05. Lưu lý do tiếp tục chờ, xếp hàng hoặc đủ điều kiện.
3. Khôi phục sau ngắt tại approval, repair commit và chapter gate. Dùng lại checkpoint và counters cùng identity của task.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC12.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T12 |
|---|---|
| Đầu vào bắt buộc | [P06-T01](#p06-t01), [P06-T02](#p06-t02), [P06-T03](#p06-t03), [P06-T04](#p06-t04), [P06-T05](#p06-t05), [P06-T06](#p06-t06), [P06-T07](#p06-t07), [P06-T08](#p06-t08), [P06-T09](#p06-t09), [P06-T10](#p06-t10), [P06-T11](#p06-t11), [P03-T10](PHASE-03.md#p03-t10). Nhận scope/evidence/repair/Research và approval publication M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Wake/replay chỉ task chờ đúng quyết định, kiểm lại budget/permit/deps, task user-paused giữ pause. Owner đầu ra là A-P06-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T12 giữ Application workflow/dependency/wake-up; checkpoint schema. Phần chung do T01–T11 giữ outputs, P05-T11/T12 giữ permission/recovery, T12 giữ propagation. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC12 / P06-TC12, gồm các ca chi tiết §13.3. V-05. Đối chiếu Request trace/counters/generation đúng trước và sau restart. |
| Điểm tích hợp | [P06-CP05](#p06-cp05) — Propagation và resume; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P07-T01](PHASE-07.md#p07-t01), [P07-T04](PHASE-07.md#p07-t04), [P08-T02](PHASE-08.md#p08-t02). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t13"></a>

### P06-T13 — UI-08 và bắt đầu Dịch theo phạm vi

- **Mục tiêu / đầu ra:** Form scope/config rõ ràng và start đúng task.
- **Căn cứ / phụ thuộc:** S-TUI/S-WORKFLOW; T01/T12. Dependency thực thi: [P06-T01](#p06-t01), [P06-T12](#p06-t12), [P02-T12](PHASE-02.md#p02-t12).
- **Phạm vi:** nt-tui volumes/start; app translate command.
- **Việc làm:** Tách all-text/story khỏi selected/to-end; preview effective model/budget; giữ selection khi lỗi; dedup start. Nối Nhờ Research đề xuất ở cây nội dung, giữ quyền chốt phân loại cho người dùng.
- **Ca lỗi / biên:** Không chọn chương, scope chưa phân loại, start lặp, đổi default khi task đang chạy.
- **Nghiệm thu:** `P06-AC13` — Task lưu đúng cả hai lựa chọn; không hỏi lại ở từng chương đã được giao; không tự tăng hạn mức.
- **Kiểm chứng:** `P06-TC13`, V-06; dữ liệu kiểm tra: Hai mode nội dung × hai mode chạy, CLI flags conflicts.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dựng lựa chọn riêng cho chế độ nội dung và kiểu chạy. Đọc effective scope/model/budget từ use case và hiển thị trước xác nhận.
2. Gửi đúng command và identity của lần bắt đầu qua core. Giữ lựa chọn và vị trí khi báo lỗi hoặc chờ quyền.
3. Đưa Nhờ Research đề xuất vào thao tác rõ ràng trên nội dung chưa phân loại. Hiển thị kết quả để người dùng quyết định trên đúng phần nguồn.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC13.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T13 |
|---|---|
| Đầu vào bắt buộc | [P06-T01](#p06-t01), [P06-T12](#p06-t12), [P02-T12](PHASE-02.md#p02-t12). Nhận task scope/effective config và start/wake command. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | UI chọn hai loại phạm vi, khử lặp start và Nhờ Research chỉ từ intent rõ của người dùng. Owner đầu ra là A-P06-T13. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T13 giữ nt-tui volumes/start; app translate command. Phần chung do T01/T12 giữ commands, T13 giữ UI-08, người tích hợp giữ router. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC13 / P06-TC13, gồm các ca chi tiết §13.3. V-06. Đối chiếu UI/CLI sinh cùng command/config; start một lần không tạo hai tasks. |
| Điểm tích hợp | [P06-CP06](#p06-cp06) — Giao diện Dịch và Review; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t14"></a>

### P06-T14 — UI-09 Review lại và Research evidence

- **Mục tiêu / đầu ra:** Hành động và bằng chứng mở đúng revision/source.
- **Căn cứ / phụ thuộc:** S-TUI, Q22/27; T03/T11/T12 và UI-05. Dependency thực thi: [P06-T03](#p06-t03), [P06-T11](#p06-t11), [P06-T12](#p06-t12), [P04-T12](PHASE-04.md#p04-t12).
- **Phạm vi:** nt-tui reviewer actions/evidence subviews; app review/issues.
- **Việc làm:** Review lại target đã chọn qua core; hiện nguồn/mốc/còn thiếu; giữ Editor return context.
- **Ca lỗi / biên:** Target đổi khi bấm, stale issue, quote dài, bằng chứng mâu thuẫn.
- **Nghiệm thu:** `P06-AC14` — Không Review revision khác trong im lặng; nguồn/cảnh cần xác minh được nhìn thấy.
- **Kiểm chứng:** `P06-TC14`, V-06; dữ liệu kiểm tra: Review cũ/current mới, Research unknown spoiler và long evidence.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chọn issue hoặc revision hiện hành rồi xác nhận mục tiêu Review lại. Đọc kết quả mới qua read model dùng chung.
2. Hiển thị nguồn, mốc đọc, điều chưa xác minh và trạng thái cũ/mới. Giữ thứ tự, bộ lọc và vị trí khi xem evidence dài.
3. Khi sang Editor, truyền exact target và lưu route quay lại. Nếu target đổi, báo xung đột và cho người dùng chọn lại.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC14.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T14 |
|---|---|
| Đầu vào bắt buộc | [P06-T03](#p06-t03), [P06-T11](#p06-t11), [P06-T12](#p06-t12), [P04-T12](PHASE-04.md#p04-t12). Nhận issue/revision, Research facts và workflow status. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Review lại exact target, evidence giữ nguồn/mốc/trạng thái thiếu, route Editor giữ return context. Owner đầu ra là A-P06-T14. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T14 giữ nt-tui reviewer actions/evidence subviews; app review/issues. Phần chung do T03/T11/T12 giữ read models, T14 giữ UI-09, người tích hợp giữ Editor routing. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC14 / P06-TC14, gồm các ca chi tiết §13.3. V-06. Đối chiếu Command target và route state đúng, không biến view thành AI request. |
| Điểm tích hợp | [P06-CP06](#p06-cp06) — Giao diện Dịch và Review; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t15"></a>

### P06-T15 — UI-10 duyệt Skills và ngữ cảnh

- **Mục tiêu / đầu ra.** Người dùng chọn và xem trước Skill, rồi duyệt đúng nhóm/phương án/phạm vi.
- **Căn cứ / phụ thuộc.** Q11–15/35, S-SKILL, [P03-T05](PHASE-03.md#p03-t05), P03 publication và T12.
- **Phạm vi.** nt-tui Skills và ngữ cảnh. Dùng use case nạp/preview/confirm của M3.
- **Việc làm.** Chọn file/thư mục thư viện chung, xem loại phần và bật/tắt theo scope. Chọn Skill riêng của bộ, xem ba nhóm và xác nhận phần rõ với hash hiện hành. Trong màn duyệt, Space chọn và nút Chốt N xác nhận đúng nhóm. Mở chi tiết giữ trạng thái, conflict giữ lựa chọn và chờ đồng bộ được hiển thị rõ.
- **Ca lỗi / biên.** File đổi sau preview, nhầm scope, mục mới xuất hiện, item version đổi, 1/10 conflict, sync lỗi và paused task.
- **Nghiệm thu.** `P06-AC15`. Nạp đúng lựa chọn và bảo toàn file gốc. Chỉ đúng nhóm/version đã chọn được xác nhận, không ghi một phần hoặc tự resume tác vụ người dùng đã pause.
- **Kiểm chứng.** `P06-TC15`, V-06 và P06-UT06. Fixture có thư viện chung, Skill riêng thiếu trường, preview stale, batch 10→9 và publication chậm.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Nối chọn thư viện Markdown chung và Skill riêng vào hai đường nạp M3. Hiển thị scope, phần dùng được, thiếu căn cứ và mục trùng/mâu thuẫn theo preview.
2. Chốt preview bằng hash và chỉ các item/version đã chọn. Giữ selections và hiển thị conflict nếu cả batch chưa được ghi.
3. Phân biệt đã ghi quyết định và đã đồng bộ active set. Giữ file gốc, vị trí danh sách và trạng thái task đang chờ hoặc pause.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC15.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T15 |
|---|---|
| Đầu vào bắt buộc | [P06-T12](#p06-t12), [P03-T05](PHASE-03.md#p03-t05), [P03-T09](PHASE-03.md#p03-t09), [P03-T10](PHASE-03.md#p03-t10). Nhận P03-T05 preview/import, P03-T09/T10 batch/publication và wake state. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | UI-10 xác nhận đúng hash hoặc ID/version đã chọn, giữ file gốc, hiển thị pending sync và task pause. Owner đầu ra là A-P06-T15. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T15 giữ nt-tui Skills và ngữ cảnh. Dùng use case nạp/preview/confirm của M3. Phần chung do M3 giữ import/approval, T12 giữ wake, T15 giữ Skill selection UI. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P05-T13](PHASE-05.md#p05-t13), [P05-T14](PHASE-05.md#p05-t14), [P05-T15](PHASE-05.md#p05-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC15 / P06-TC15, gồm các ca chi tiết §13.3. V-06. Đối chiếu Nạp/xác nhận đúng scope và hash, file gốc còn nguyên. UI khớp intent/active set. Phím nhập không kích hoạt phía sau. |
| Điểm tích hợp | [P06-CP07](#p06-cp07) — Giao diện Skills và approval; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T04](PHASE-08.md#p08-t04), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p06-t16"></a>

### P06-T16 — Nghiệm thu workflow đầy đủ

- **Mục tiêu / đầu ra:** M6 có corpus test và failure/restart evidence.
- **Căn cứ / phụ thuộc:** Q20–29/36; T01–T15. Dependency thực thi: [P06-T01](#p06-t01), [P06-T02](#p06-t02), [P06-T03](#p06-t03), [P06-T04](#p06-t04), [P06-T05](#p06-t05), [P06-T06](#p06-t06), [P06-T07](#p06-t07), [P06-T08](#p06-t08), [P06-T09](#p06-t09), [P06-T10](#p06-t10), [P06-T11](#p06-t11), [P06-T12](#p06-t12), [P06-T13](#p06-t13), [P06-T14](#p06-t14), [P06-T15](#p06-t15), M5 Đạt.
- **Phạm vi:** App end-to-end/migration tests, oracle/evidence.
- **Việc làm:** Chạy selected/to-end, terms/ruby waits, M1–M4, five-round stop, Research/spoiler, exception và resume.
- **Ca lỗi / biên:** Đi tắt chapter gate; fake success che thiếu coverage; oracle chưa duyệt; counters lost.
- **Nghiệm thu:** `P06-AC16` — Mọi nhánh scope/wait/repair giữ contract; ca nghĩa có oracle và ca chưa xác nhận được báo riêng.
- **Kiểm chứng:** `P06-TC16`, V-07, V-00; dữ liệu kiểm tra: Chương ngắn tự soạn, adversarial responses và expected outcomes.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chuẩn bị fixture nhỏ có source map, reference spans, trạng thái và request trace mong đợi. Đặt điểm ngắt để chạy lại cùng kịch bản.
2. Chạy workflow từ entry point CLI/TUI qua fake model và fake Research. Đối chiếu revision, evidence, chi phí, thứ tự queue và chapter gate.
3. Tách ca tự động, ca oracle nghĩa và ca thao tác bàn phím. Đóng gói đầu ra đã rà cho M7 và báo phần còn chặn trước khi đề nghị B xét M6.

Ca thử chi tiết tại [mục 13.3](#p06-cases) theo mã P06-TC16.

**Sáu trường giao việc**

| Trường | Nội dung cho P06-T16 |
|---|---|
| Đầu vào bắt buộc | [P06-T01](#p06-t01), [P06-T02](#p06-t02), [P06-T03](#p06-t03), [P06-T04](#p06-t04), [P06-T05](#p06-t05), [P06-T06](#p06-t06), [P06-T07](#p06-t07), [P06-T08](#p06-t08), [P06-T09](#p06-t09), [P06-T10](#p06-t10), [P06-T11](#p06-t11), [P06-T12](#p06-t12), [P06-T13](#p06-t13), [P06-T14](#p06-t14), [P06-T15](#p06-t15), M5 Đạt. Nhận candidate workflow đủ các nhánh với oracle và fixtures có request trace. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Hồ sơ M6 so revision/evidence/fees/queue/coverage từ CLI/TUI qua restart, bàn giao readiness inputs. Owner đầu ra là A-P06-T16. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P06-T16 giữ App end-to-end/migration tests, oracle/evidence. Phần chung do owners T01–T15 nhận sửa theo boundary, A-T16 tích hợp. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P07-T01](PHASE-07.md#p07-t01), [P07-T02](PHASE-07.md#p07-t02), [P07-T03](PHASE-07.md#p07-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P06-AC16 / P06-TC16, gồm các ca chi tiết §13.3. V-07, V-00. Đối chiếu So DB/history/requests/UI/CLI với oracle; không gọi dịch vụ thật. |
| Điểm tích hợp | [P06-CP08](#p06-cp08) — Nghiệm thu M6; A chat P06 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P06-AC01 | Không mở rộng phạm vi hoặc reset cấu hình/counter; hoàn tất theo đúng mọi phần bắt buộc. | P06-T01 | P06-TC01; V-01 |
| P06-AC02 | Phần chưa gửi dùng nền mới hợp lệ; request đã gửi giữ nền thực dùng, không tráo history. | P06-T02 | P06-TC02; V-01 |
| P06-AC03 | Bác nhận định sai hoặc giữ cần xác minh; không coi quote match là chứng minh ngữ nghĩa. | P06-T03 | P06-TC03; V-02 |
| P06-AC04 | Phần ngoài scope còn hợp lệ được giữ; sửa luôn tạo revision và không tự đánh dấu đạt. | P06-T04 | P06-TC04; V-03 |
| P06-AC05 | M3 chỉ gồm vùng có quan hệ chứng minh được; context sai không tiếp tục cấp cho lượt sau. | P06-T05 | P06-TC05; V-03 |
| P06-AC06 | Không M4 vì số lỗi hoặc severity đơn lẻ; chỉ công bố plan M4 khi điều kiện đạt. | P06-T06 | P06-TC06; V-03 |
| P06-AC07 | Round thất bại thứ năm đưa về người dùng; các counter network/structure riêng, không reset lịch sử. | P06-T07 | P06-TC07; V-03 |
| P06-AC08 | Hiển thị có ngoại lệ; chỉ miễn đúng vấn đề, không vượt technical/coverage/approval gates. | P06-T08 | P06-TC08; V-03 |
| P06-AC09 | Không cite trang chưa đọc, không làm theo chỉ dẫn trong nội dung web; tool fees/requests qua policy chung. | P06-T09 | P06-TC09; V-04 |
| P06-AC10 | Dừng đúng giới hạn, trả bằng chứng còn thiếu; không reset task/model cap hoặc dùng cache sai mốc. | P06-T10 | P06-TC10; V-04 |
| P06-AC11 | Unknown/future facts giữ trong hồ sơ nhưng không vào context cảnh hiện tại; conflict cần xác minh. | P06-T11 | P06-TC11; V-04 |
| P06-AC12 | Chỉ task chờ đúng quyết định và còn được phép mới tiếp tục; mọi kết luận dùng exact current basis. | P06-T12 | P06-TC12; V-05 |
| P06-AC13 | Task lưu đúng cả hai lựa chọn; không hỏi lại ở từng chương đã được giao; không tự tăng hạn mức. | P06-T13 | P06-TC13; V-06 |
| P06-AC14 | Không Review revision khác trong im lặng; nguồn/cảnh cần xác minh được nhìn thấy. | P06-T14 | P06-TC14; V-06 |
| P06-AC15 | Nạp đúng file/scope và giữ file gốc. Xác nhận đúng preview hoặc nhóm/version đã chọn. Không ghi một phần nhóm hoặc tự resume tác vụ đã pause. | P06-T15 | P06-TC15 · V-06 |
| P06-AC16 | Mọi nhánh scope/wait/repair giữ contract; ca nghĩa có oracle và ca chưa xác nhận được báo riêng. | P06-T16 | P06-TC16; V-07, V-00 |

Các AC không thay thế nhau. Build thành công, fixture trả đúng kết quả định sẵn hoặc một màn hiển thị đẹp không chứng minh toàn bộ hành vi nghiệp vụ.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P06-TC01 / P06-AC01 | Hai tập, nhiều chương/phụ trang, một phần đã dịch. | Expected scope/order/remaining content đúng; không có request ngoài tập được giao. | V-01 |
| P06-TC02 / P06-AC02 | Skill edit giữa hai lượt (payload chưa gửi dùng nền mới; watcher chậm/trễ; lượt đã gửi giữ nền cũ) và flashback/alias ambiguity. | Payload/dependency IDs đúng theo từng lượt; nền của lượt đã gửi không bị tráo; blocking reason theo căn cứ thiếu. | V-01 |
| P06-TC03 / P06-AC03 | Oracle phủ định/chủ thể ẩn/false quote/acceptable paraphrase. | So expected evidence status và false positives; ca chưa có oracle không được tính đạt nghĩa. | V-02 |
| P06-TC04 / P06-AC04 | Một lỗi cục bộ nghiêm trọng và nhiều exact spans. | Diff/coverage chỉ vùng cho phép; phần mới có Review target đúng. | V-03 |
| P06-TC05 / P06-AC05 | Sai tiền đề ở đầu cảnh ảnh hưởng hai đoạn sau và một đoạn độc lập. | Affected set/diff đúng oracle, history và sửa tay ngoài phạm vi được giữ. | V-03 |
| P06-TC06 / P06-AC06 | M4 hợp lệ, M4 bị thu hẹp và đề xuất bị bác. | Request/decision trace chứng minh bổ sung; phạm vi sau quyết định đúng. | V-03 |
| P06-TC07 / P06-AC07 | 2 transient, 2 structural, 5 failed repair rounds và manual revision. | Đếm đúng từng loại; không có round tự động thứ sáu. | V-03 |
| P06-TC08 / P06-AC08 | Exception hợp lệ rồi target/rule đổi; technical invalid draft. | Gate khác vẫn chặn; ngoại lệ cũ không áp vào revision mới. | V-03 |
| P06-TC09 / P06-AC09 | Search/page transcripts, redirects/errors và untrusted page. | Call sequence nội bộ→search→read đúng; không dùng key thật hoặc mạng ngoài. | V-04 |
| P06-TC10 / P06-AC10 | Fake clock boundary TTL/deadline, changed basis và tool fees. | No extra call khi hết cap/time; cache reuse/revalidate đúng điều kiện. | V-04 |
| P06-TC11 / P06-AC11 | Một trang chứa ba loại fact và cache xuyên tập. | Context chỉ chứa facts đủ điều kiện, source history không bị xóa. | V-04 |
| P06-TC12 / P06-AC12 | Approval wait vs user-paused, rejected event và queued resume. | Request trace/counters/generation đúng trước và sau restart. | V-05 |
| P06-TC13 / P06-AC13 | Hai mode nội dung × hai mode chạy, CLI flags conflicts. | UI/CLI sinh cùng command/config; start một lần không tạo hai tasks. | V-06 |
| P06-TC14 / P06-AC14 | Review cũ/current mới, Research unknown spoiler và long evidence. | Command target và route state đúng, không biến view thành AI request. | V-06 |
| P06-TC15 / P06-AC15 | File chung/riêng, preview stale, batch 10→9, mục mới và publication chậm. | Nạp/xác nhận đúng scope và hash, file gốc còn nguyên. UI khớp intent/active set. Phím nhập không kích hoạt phía sau. | V-06 |
| P06-TC16 / P06-AC16 | Chương ngắn tự soạn, adversarial responses và expected outcomes. | So DB/history/requests/UI/CLI với oracle; không gọi dịch vụ thật. | V-07, V-00 |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo trong task tương ứng; nếu đổi phải cập nhật kế hoạch. Lệnh exit 0 với 0 ca liên quan không được ghi là đạt. 

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | cargo test -p nt-application --test task_scope_context --locked |
| V-02 | cargo test -p nt-application --test review_evidence --locked |
| V-03 | cargo test -p nt-application --test repair_policy --locked |
| V-04 | cargo test -p nt-research --test research_contract --locked<br>cargo test -p nt-application --test research_policy --locked |
| V-05 | cargo test -p nt-application --test workflow_resume --locked |
| V-06 | cargo test -p nt-tui --test workflow_actions --locked<br>cargo test -p novel-translate --test workflow_cli --locked |
| V-07 | cargo test -p novel-translate --test phase06_workflow --locked; demo UI-08/09/10 bằng bàn phím |

**V-00 — Bộ kiểm tra Rust bắt buộc trước PR Rust và trước đóng phase có thay đổi Rust:**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

Test tự động dùng fixture tự soạn, adapter/clock/ID kiểm soát được và thư mục tạm. Chuẩn bị dependency/công cụ trước bước test; không gọi provider trả phí, không dùng key hoặc EPUB cá nhân. `--offline` chỉ hạn chế network của Cargo; cô lập transport của ứng dụng trong harness để không mở dịch vụ thật. Kiểm tra UI trên terminal thật và kiểm tra liên tiến trình được ghi riêng với unit test.

### 13.2. Cách dùng bằng chứng

Test phải kiểm tra kết quả quan sát được ở boundary, cả ca thành công và lỗi làm sai dữ liệu/quyền. Không viết test chỉ nhắc lại implementation. Đối với fault injection, ghi chính xác điểm ngắt, trạng thái trước/sau reopen và side effect đã/không xảy ra. Lỗi CI/môi trường và lỗi sản phẩm được phân biệt; chưa chạy thì ghi chưa xác minh.

<a id="p06-cases"></a>

### 13.3. Ca thử chi tiết theo task

Các mã hậu tố a, b, c, d phân rã ca chính trong bảng trên và dùng cùng nhóm V tương ứng. Dùng clock và adapter giả để điều khiển thời điểm chờ, gửi, phản hồi và crash. Ghi expected source span, issue, scope hoặc tập fact của từng fixture trước khi chạy. Ca kết luận nghĩa cần đối chiếu với đáp án người rà Nhật–Việt đã xác nhận.

| Mã ca | Fixture và thao tác | Kết quả cần đối chiếu |
|---|---|---|
| P06-TC01a | Chọn một chương giữa tập ở chế độ story. | Task plan giữ chương được chọn và phần thuộc phạm vi theo Q01. |
| P06-TC01b | Chạy to-end/all-text qua phụ trang và chương đã đạt. | Phụ trang bắt buộc có mặt, phần hợp lệ được giữ và request dừng ở cuối tập. |
| P06-TC01c | Đổi model mặc định rồi restart giữa hai chương. | Cùng task ID, phạm vi, model hiệu lực và số tiền đã dùng. |
| P06-TC02a | Đổi Skill khi lượt đang xếp hàng và giữ watcher im. | Hash check phát hiện thay đổi trước khi adapter nhận request. |
| P06-TC02b | Đổi mapping khi request đã gửi. | Attempt giữ version cũ. Việc công bố phải qua kiểm căn cứ hiện hành. |
| P06-TC02c | Cảnh flashback có alias chưa phân giải và fact tương lai. | Payload thiếu fact vượt mốc. Trường hợp chưa đủ căn cứ có lý do và bước tiếp theo. |
| P06-TC02d | Nguồn có một block vượt cửa sổ và thiếu điểm cắt an toàn. | Giữ chặn theo Q05 với Chương, Block ID và hướng replan. |
| P06-TC03a | Quote tồn tại nhưng nguồn phủ định điều finding khẳng định. | Oracle bác finding sai hoặc yêu cầu xác minh thêm. |
| P06-TC03b | Chủ thể ẩn chỉ được xác định từ đoạn trước. | Ghi span bổ sung và kết luận đúng chủ thể theo oracle. |
| P06-TC03c | Compare paraphrase hợp lệ và một câu nguồn bị bỏ. | Chỉ câu bị bỏ tạo lỗi coverage cần xử lý. |
| P06-TC03d | Remap block đã có kết luận đạt. | Kết luận dựa trên span cũ không còn được dùng ở chapter gate. |
| P06-TC04a | Hai span M1 trong cùng block, thứ tự phản hồi đảo. | Chỉ các span được duyệt đổi và patch vẫn khớp base. |
| P06-TC04b | Patch lấn qua marker hoặc span khác. | Từ chối patch và giữ revision trước sửa. |
| P06-TC04c | M2 thay một block rồi ngắt ngay sau commit. | Mở lại Review đúng revision M2 một lần, các block khác còn nguyên. |
| P06-TC05a | Một tiền đề sai ảnh hưởng hai cảnh và bỏ qua một cảnh độc lập. | Tập repair khớp edges đã biết và giữ nguyên cảnh độc lập. |
| P06-TC05b | Mapping mới đã duyệt nhưng publication còn chờ. | Lượt repair liên quan vẫn chờ, payload chưa dùng tập nửa cũ nửa mới. |
| P06-TC05c | Một đoạn ngoài tập ảnh hưởng được sửa tay. | Bản sửa tay và lịch sử giữ nguyên qua repair M3. |
| P06-TC06a | Một lỗi S3 chỉ nằm trong một câu. | Plan chọn vùng cục bộ có căn cứ dù severity cao. |
| P06-TC06b | Oracle xác nhận hiểu sai tiền đề xuyên chương. | Có attempt kiểm chứng, evidence và quyết định M4 trước khi gửi repair. |
| P06-TC06c | Kiểm chứng giới hạn lỗi trong một cảnh. | Chỉ phạm vi được thu hẹp đi vào repair. |
| P06-TC06d | Lượt kiểm chứng bị chặn ngân sách hoặc bác đề xuất. | Chưa gửi yêu cầu dịch lại toàn chương. |
| P06-TC07a | Timeout đã xác định, phản hồi sai cấu trúc và repair hợp lệ đang chờ Review. | Ba loại lượt có trace riêng, failed-round chưa tăng. |
| P06-TC07b | Round 4 đổi M2 sang M3 rồi restart. | Counter giữ 4 và chỉ tăng khi Review xác nhận đợt lỗi còn. |
| P06-TC07c | Replay kết quả Review thứ năm rồi resume. | Counter vẫn 5, trạng thái chờ người dùng và không gửi vòng thứ sáu. |
| P06-TC08a | Giữ đúng một issue đã xác minh. | Chương hiển thị ngoại lệ và issue còn truy được trong lịch sử. |
| P06-TC08b | Sửa revision hoặc đổi căn cứ của cùng issue. | Ngoại lệ cũ cần xét lại, gate dùng dữ liệu mới. |
| P06-TC08c | Có ngoại lệ hợp lệ nhưng thiếu coverage hoặc draft lỗi cấu trúc. | Gate vẫn trả chặn tương ứng. |
| P06-TC09a | Nguồn nội bộ đủ trả lời. | Trace có lookup nội bộ và zero search/read calls. |
| P06-TC09b | Kết quả search có snippet nhưng page đọc lỗi. | Ghi thiếu căn cứ, snippet không thành quote xác nhận. |
| P06-TC09c | Redirect sang địa chỉ ngoài policy hoặc trang chứa chỉ dẫn gọi tool. | Policy chặn route không được phép và nội dung chỉ được đọc như dữ liệu. |
| P06-TC09d | Người dùng mở cây nhập rồi chủ động nhờ Research phân loại. | Chỉ thao tác xác nhận tạo intent, class nguồn chưa đổi khi mới có đề xuất. |
| P06-TC10a | Đã dùng 3 search hoặc 5 page reads theo cấu hình mặc định. | Lượt kế tiếp cùng loại bị chặn và ledger giữ phí đã có. |
| P06-TC10b | Còn 20 giây Research, model timeout lớn hơn. | Deadline cấp cho model tối đa 20 giây. Hết thời hạn thì không gửi thêm. |
| P06-TC10c | Cùng câu hỏi ở mốc khác hoặc sau 7 ngày. | Cache tách theo mốc. Nguồn web hết hạn được kiểm lại trước reuse. |
| P06-TC10d | Restart sau một lượt có phí. | Số lượt và chi phí vẫn thuộc task/câu hỏi cũ. |
| P06-TC11a | Một trang có fact hiện tại, tương lai và chưa rõ mốc. | Chỉ fact hiện tại vào payload của cảnh đang xử lý. |
| P06-TC11b | Tên từ web trùng alias đã duyệt nhưng khác thực thể. | Tạo mục cần xác minh, active mapping giữ nguyên. |
| P06-TC11c | Trang biến mất sau khi đã trích một đoạn. | Giữ quote/hash/thời điểm đã lưu và gắn trạng thái cần kiểm lại. |
| P06-TC12a | Batch duyệt liên quan hai task, một task đang user-paused. | Task đủ điều kiện đi vào queue, task paused giữ trạng thái. |
| P06-TC12b | Publication xong nhưng task B đang giữ permit. | Task A chờ đúng hàng và không gửi request vượt permit. |
| P06-TC12c | Ngắt sau commit publication trước khi nhận wake event. | Replay nối đúng task một lần, giữ generation và số lượt. |
| P06-TC12d | Căn cứ bị thu hồi trước khi task tới đầu hàng. | Preflight chặn lượt chưa gửi và chỉ ra mục cần đồng bộ. |
| P06-TC13a | Thử cả bốn tổ hợp content mode và kiểu chạy. | UI và CLI tạo cùng phạm vi/config theo từng tổ hợp. |
| P06-TC13b | Enter lặp trong lúc start đang xử lý. | Một intent tạo một task, màn hình chỉ ra trạng thái hiện có. |
| P06-TC13c | Đổi revision nguồn khi đề xuất phân loại còn mở. | Confirm yêu cầu đối chiếu lại nguồn, lựa chọn đang xem vẫn truy được. |
| P06-TC14a | Mở finding của revision cũ, chọn Review lại khi current đã đổi. | Hiện hai revision và cần người dùng chọn đúng target. |
| P06-TC14b | Cuộn evidence dài có fact chưa rõ mốc. | Nhãn chưa xác minh và nguồn còn thấy, mở màn hình không sinh model call. |
| P06-TC14c | Qua Editor rồi quay lại khi có kết quả mới. | Giữ vị trí theo issue còn tồn tại hoặc báo issue đã thay đổi. |
| P06-TC15a | Nạp thư viện chung có bảng và Skill riêng thiếu trường. | Markdown chung được đăng ký đúng scope, preview riêng nêu mục thiếu và file gốc giữ nguyên. |
| P06-TC15b | Sửa file giữa preview và confirm. | Hash không khớp, yêu cầu preview lại, chưa thêm mục nào. |
| P06-TC15c | Chọn 9/10, có item mới và một version conflict. | Tập chốt giữ đúng lựa chọn. Cả batch được từ chối nếu version đã đổi. |
| P06-TC15d | Publication còn chờ khi một task đã user-paused. | UI hiện đã ghi/chờ đồng bộ, task paused giữ nguyên. |
| P06-TC16a | Selected chạy qua M1 và duyệt ruby rồi qua chapter gate. | Request/DB/evidence có đúng target và không mở rộng phạm vi. |
| P06-TC16b | To-end qua Research, M3/M4 và restart. | Counter, provenance và thứ tự phần tiếp theo khớp oracle. |
| P06-TC16c | Đợt lỗi hết 5 vòng, sau đó ghi ngoại lệ đúng revision. | Tới chờ người dùng rồi qua gate tương ứng khi còn đủ điều kiện khác. |
| P06-TC16d | Demo UI-08/09/10 bằng bàn phím trên terminal thật. | Quyết định, scope, trạng thái chờ và resume quan sát được theo mục 16.4. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md): A thực hiện task, kiểm thử và tự review rồi bàn giao **Chờ review**; B đối chiếu yêu cầu, kiểm bản bàn giao và tự chạy kiểm chứng phù hợp trong phiên riêng. B ghi báo cáo độc lập, không sửa source/test chính thức hoặc trạng thái PLAN. A xử lý findings; B kiểm tra lại trên candidate mới. Quy tắc này áp dụng cả task tài liệu/ADR/thử nghiệm của phase.

- Severity và repair scope có bị đồng nhất? M4 có đủ verification bổ sung?
- Bộ đếm có reset/lẫn nhau khi retry, scope change hoặc manual edit?
- Evidence có quote tồn tại nhưng kết luận không có căn cứ; unknown có lọt context?
- Research có đường tự gửi model/chi phí ngoài executor?
- Approval wake-up có tôn trọng paused/queue và exact decision versions?

Rà thêm diff ngoài phạm vi, API công khai không có consumer, dependency cấm, error bị nuốt và test không được phát hiện/chạy. Không xóa hoặc nới test chỉ để qua kiểm tra. Sửa lỗi ở owner của quy tắc thay vì thêm ngoại lệ trong UI.

## 15. Điều kiện đóng phase

- [ ] Mọi task/nhóm trong phạm vi cổng có review B **Đạt** còn hiệu lực; đủ AC, kiểm tra bắt buộc và không còn finding chặn. A không tự duyệt phần mình triển khai.
- [ ] B đã review bản tích hợp cuối theo [mẫu review phase](../templates/PHASE_REVIEW.md); candidate và căn cứ khớp, bằng chứng các task còn áp dụng hoặc đã được kiểm lại theo ảnh hưởng.

- [ ] Selected/to-end và các điểm chờ qua cùng workflow/Coordinator.
- [ ] M1–M4, five-round limit, exception validity, Research budgets/spoiler và dependency propagation có bằng chứng.
- [ ] UI-08/09/10 cùng CLI đạt; migration P05→P06 giữ checkpoint/counters/history.
- [ ] Tất cả task và AC của phase có bằng chứng, điểm chặn đã được giải quyết; ca chưa kiểm tra được ghi đúng trạng thái.
- [ ] V-00 áp dụng cho thay đổi Rust đã đạt; checks riêng trong mục 13 và các kiểm tra Go bị ảnh hưởng đã có kết quả.
- [ ] README được cập nhật nếu command/layout/nền tảng thay đổi; tài liệu nội bộ, key, EPUB, workspace và đầu ra tiếp tục ngoài Git.
- [ ] Bàn giao mục 16 đủ cho phase sau; không tự xóa đường Go hoặc công bố feature parity.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md); B dùng [mẫu review task](../templates/TASK_REVIEW.md), và [mẫu review phase](../templates/PHASE_REVIEW.md) khi nghiệm thu cổng. Báo cáo nội bộ đặt theo lượt trong `docs/roadmap/evidence/phase-06/`: `P06-TYY.handoff-r01.md` và `P06-TYY.review-r01.md`; giữ lượt trước khi sửa. Hồ sơ cổng dùng ID M0-01/M0-02/M0 hoặc M1…M9 theo phạm vi. Các mẫu chưa phải bằng chứng thực thi.

Sổ task/cổng khi thực hiện ghi trạng thái, candidate, link báo cáo A, review B, finding còn mở và bước tiếp theo. A phase cập nhật sổ này và gửi điều phối cập nhật PLAN trung tâm sau bàn giao/kết luận; B chỉ ghi báo cáo review. Thiếu review B hoặc kiểm tra bắt buộc thì chưa đánh dấu hoàn thành.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 8 checkpoint của lịch v2: [P06-CP01](#p06-cp01), [P06-CP02](#p06-cp02), [P06-CP03](#p06-cp03), [P06-CP04](#p06-cp04), [P06-CP05](#p06-cp05), [P06-CP06](#p06-cp06), [P06-CP07](#p06-cp07), [P06-CP08](#p06-cp08); xem [§10.2](#p06-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task | Trạng thái | Bằng chứng |
|---|---|---|---|
| P06-A | T01–T03 | Chưa bắt đầu | Chưa có |
| P06-B | T04–T08 | Chưa bắt đầu | Chưa có |
| P06-C | T09–T11 | Chưa bắt đầu | Chưa có |
| P06-D | T12–T15 | Chưa bắt đầu | Chưa có |
| P06-E | T16 | Chưa bắt đầu | Chưa có |

### 16.2. Báo cáo mỗi task

Ghi task/AC, thay đổi hành vi, file/module, quyết định và tác động migration/compatibility; lệnh đã chạy, toolchain/OS, tên/số ca liên quan, kết quả; test thủ công và ca chưa chạy; rủi ro còn lại và task có thể nhận tiếp. Nếu lưu log/báo cáo, dùng `docs/roadmap/evidence/phase-06/`, đã redaction, không chứa dữ liệu cá nhân hoặc nội dung sách thật.

### 16.3. Đầu ra cho bước tiếp theo

[Phase 07](PHASE-07.md) nhận readiness read model theo content mode, exact revision/Review sets, exceptions/rules còn hiệu lực và coverage phụ trợ. [Phase 08](PHASE-08.md) hoàn thiện interactions xuyên màn. [Phase 09](PHASE-09.md) mở rộng corpus Q36 và đo chất lượng model thật thủ công; fake workflow pass không thay kết quả đánh giá đó.

Trạng thái kế hoạch không phải trạng thái implementation. Chỉ cập nhật hoàn thành khi có bằng chứng của hành vi thực tế.

<a id="p06-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử candidate **M6** và bộ kịch bản offline về phạm vi dịch, Review lại, duyệt thuật ngữ, Research và tiếp tục sau khi dừng. Khi bàn giao, A ghi candidate, lệnh mở app, cách chọn kịch bản và checkpoint mong đợi. B lặp các ca trên dữ liệu mẫu mới.

#### 16.4.2. Chuẩn bị

Dành khoảng 20 phút. Chuẩn bị tập mẫu có hai chương, một trang chưa rõ loại, một lượt cần sửa, ba ứng viên thuật ngữ, hai file Skill tự soạn và nguồn Research giả lập. Gói M6 chỉ rõ chương, trang, hai mục cần chọn và kịch bản chạm giới hạn sửa. Dùng workspace và hồ sơ thử được tạo riêng.

#### 16.4.3. Mở đúng bản

Nạp bộ mẫu và mở bản Rust M6 theo lệnh bàn giao. Vào **Tập và chương**, chọn tập mẫu và mở form bắt đầu Dịch. Đối chiếu phạm vi, model giả lập và ngân sách thử đang hiển thị trước khi bắt đầu.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P06-UT01 — Chương đã chọn và tự động đến hết | Chạy mẫu chỉ với chương được chỉ định. Sau đó lấy bản sao mới và thử chế độ tự động đến hết với kịch bản đi được qua cả hai chương. | Lượt đầu giữ nguyên chương ngoài phạm vi. Lượt sau đi theo đúng thứ tự đến chương cuối, qua các bước Review cần thiết. |
| P06-UT02 — Sửa và Review lại | Chạy mẫu có một lượt cần sửa. Mở thẻ Review của lượt đó và xem lại sau khi bộ thử chạy sửa. | Review lại gắn với revision sau khi sửa. Lượt/chương kế tiếp chỉ tiếp tục khi phần bắt buộc đã đạt. |
| P06-UT03 — Duyệt đúng nhóm | Trong **Skills và ngữ cảnh**, chọn hai mục đã chỉ định. Xem số mục trên nút xác nhận rồi xác nhận. | Chỉ hai mục được chọn đổi trạng thái. Mục thứ ba còn chờ. Tác vụ thể hiện đúng lý do nếu vẫn còn điều kiện chờ. |
| P06-UT04 — Research có căn cứ | Mở trang mơ hồ trong cây, chọn **Nhờ Research đề xuất** theo kịch bản và mở bằng chứng. | Có nguồn, mốc và điều còn thiếu để đối chiếu. Loại trang chỉ đổi khi cậu xác nhận lựa chọn. |

#### 16.4.5. Thử ca lỗi và mở lại

**P06-UT05 — Chạm giới hạn sửa và mở lại.** Chạy kịch bản không sửa được sau năm vòng. Mở chi tiết tác vụ để kiểm trạng thái bàn giao người dùng và đoạn cần xử lý. Đóng rồi mở lại, đối chiếu ID, revision và số vòng đã dùng. Ca đạt khi giữ đúng điểm chờ và lượt sửa chỉ tiếp tục qua thao tác phù hợp.

**P06-UT06 — Nạp Skill và xem lại khi file đổi.** Trong **Skills và ngữ cảnh**, chọn thư viện chung mẫu rồi bật cho scope chỉ định. Chọn file Skill riêng mẫu và đối chiếu ba nhóm. Thay một dòng trong bản sao đầu vào sau preview rồi thử xác nhận. App yêu cầu xem lại và chưa áp phần cũ. Xem preview mới, xác nhận phần rõ và mở lại. Bản riêng mới đúng mẫu sáu mục. File gốc vẫn khớp byte ngay trước lần xác nhận.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Kèm candidate M6, tên kịch bản, ID tác vụ, chapter/revision đang xem và vị trí khác kết quả mẫu nếu có.

#### 16.4.7. Thử lại từ đầu

Tạo lại workspace và hồ sơ thử bằng lệnh đã bàn giao. Nạp đúng kịch bản trước mỗi lần thử lại để các trạng thái duyệt và số vòng về ban đầu.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-06. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P06 rà **Dịch/Review, Research, repair, approval và resume** tại các mốc **P06-CP03/CP04/CP05/CP06/CP07; kiểm toàn workflow tại M6**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Đối chiếu luồng người dùng, chờ duyệt, lỗi và mở lại trên Rust; rà caller liên quan đến export và UI trước khi gỡ từng phần Go. Bằng chứng chất lượng model thật bắt buộc vẫn theo hồ sơ nghiệm thu, không tự coi offline workflow là đủ.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
