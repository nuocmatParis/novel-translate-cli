# PHASE 08 — HOÀN THIỆN TRẢI NGHIỆM TUI VÀ CLI

> Lịch task song song và checkpoint commit/push tại [mục 10](#p08-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Draft 0.4 — 09/11/2026 · Chưa triển khai.** Lập từ đặc tả và thiết kế theo yêu cầu của người dùng; code hiện tại không là chuẩn thiết kế, đầu vào tái sử dụng hoặc bằng chứng hoàn thành. Lịch thực thi v2 ngày 12/11/2026; thay lịch v0.1 chưa triển khai.
>
> Cấu trúc theo [Phase 01](PHASE-01.md). Task, đường dẫn module và tên test là kế hoạch ban đầu; rà lại trước khi giao việc khi contract của phase trước thay đổi. Không tự thay yêu cầu đã chốt để khớp implementation.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P08 / M8 |
| Tên | Hoàn thiện trải nghiệm TUI và CLI |
| Trạng thái | Chưa bắt đầu; mọi AC và kiểm thử bên dưới chưa có kết quả thực thi |
| Đầu vào chính | M2–M7 cung cấp use cases và UI-02–13; M1 shell/UI-01 |
| Gói giao diện | UI-14, UI-15; nghiệm thu tích hợp UI-01–13 |
| Phân rã | 5 gói việc, 12 task |
| Nhánh khi triển khai | rewrite/rust; giữ Go theo R-REPO |
| Người thực hiện / rà soát | Model A triển khai / model B khác review độc lập; chưa phân công |

Xem [roadmap chung](../ROADMAP.md) và [phạm vi tại Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-08). Tài liệu và bằng chứng nội bộ ở trong `docs/`, ngoài Git.

## 2. Mục tiêu và demo cuối phase

Hoàn thiện tám màn và CLI trên các use case đã có, đặc biệt các luồng qua nhiều màn, state khi đổi workspace/resize, hành vi script và sự tương đương giữa CLI/TUI. Phase này nối và sửa phần còn thiếu ở Presentation; lỗi nghiệp vụ được sửa tại owner trong core.

1. Từ Tổng quan mở đúng tác vụ/điểm chờ; điều hướng đủ tám màn bằng bàn phím.
2. Reviewer → Editor sửa nháp → resize → quay lại đúng thẻ; thử xung đột và không mất bản đang soạn.
3. Xem tác vụ B khi A đang chạy, pause/resume qua TUI và CLI cùng dữ liệu.
4. Chọn 2 trong nhiều thuật ngữ, xác nhận đúng 2; giữ trạng thái nếu publication chưa xong.
5. Dùng CLI non-interactive đi qua chờ duyệt/budget/queue; đọc JSON và status rồi resume đúng ID.
6. Check/Preview/Export từ cả TUI/CLI cho cùng kết quả và giữ version history.

## 3. Phạm vi

UI-14 Dashboard/reports/help, UI-15 keyboard/responsive/state; rà toàn bộ tám màn đã nối; đầy đủ CLI grammar/flags/JSON/exit codes; integration/UI tests; hướng dẫn user-facing và sửa lỗi usability có tác động đến contract.

Không xây GUI/web UI quản lý, không thêm màn chính thứ chín, không viết lại nghiệp vụ trong UI. Packaging, live model quality eval và quyết định phát hành thuộc Phase 09.

## 4. Điều kiện bắt đầu

P08-T01 nhận các checkpoint giao diện P02–P07 ghi trong thẻ; ma trận được đối chiếu trên hành vi thật cùng các gap còn thiếu. Có thể đọc spec/lập nháp trước, nhưng chưa đóng T01 bằng bảng dự kiến. Code T02–T11 mở theo đầu ra UI/core cụ thể đã duyệt, không phải đợi toàn bộ M7. P08-T12/M8 vẫn cần M7 và nghiệm thu tám màn/toàn CLI.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| S-WORKFLOW | [Điều phối Dịch–Review](../../spec/WORKFLOW_ORCHESTRATION_SPEC.md) |
| S-MODEL | [Model, retry và chi phí](../../spec/MODEL_STRATEGY_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| S-SKILL | [Skills và duyệt thuật ngữ](../../spec/SKILLS_GLOSSARY_SPEC.md) |
| S-REVIEW | [Giai đoạn Review](../../spec/REVIEW_STAGE_SPEC.md) |
| S-IMAGE | [Ảnh nguồn và ảnh thay](../../spec/IMAGE_EDIT_SPEC.md) |
| S-OUTPUT | [EPUB đầu ra](../../spec/EPUB_OUTPUT_SPEC.md) |
| D-UI | [Bước 8 — UI/UX](../../desgin/UI_UX_DESIGN_STEP8.md) |
| D-MODULE | [Bước 4 — Module và ownership](../../desgin/MODULE_DESIGN_STEP4.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Ký hiệu Qxx trong tài liệu là quyết định xx của S-Q. Hành vi đã chốt ưu tiên hơn phương án kỹ thuật còn đề xuất. Thư viện, tên bảng/trait và SQL nháp phải được rà theo consumer thực tế; không tự áp dụng DDL thiết kế vào workspace.

- Tám màn đúng S-TUI, subview/dialog không tăng main route.
- Arrow/Tab/Enter/Space/Esc/F1 theo focus; Enter trong editor xuống dòng, Ctrl+S lưu; Esc không tự mất nháp/thoát app.
- Phím / f e p r chỉ khi khả dụng ngoài ô nhập; input/modal không kích hoạt màn dưới.
- Từ 80×24 dùng được; từ 120 cột dùng hai vùng ở màn phù hợp; dưới ngưỡng giữ state và chỉ hiện control an toàn.
- Navigation state theo workspace/volume/document: selection/filter/scroll/draft/base revision và return route.
- Global status nhận diện task/role/model thực đang chạy dù xem task khác; quality/validity/execution tách.
- CLI/TUI cùng quyền duyệt/budget/Coordinator/core; không có shortcut vượt điều kiện.
- JSON có schema_version, command, state, task_id, scope, result, issues, next_action; stdout máy đọc, stderr đã redaction.
- Exit 0 complete, 2 args/config, 3 operational, 4 approval/conflict, 5 money/call cap, 6 queued/duplicate active, 7 paused/incomplete; status 0 khi đọc thành công.
- Non-interactive không chờ prompt, lưu checkpoint và trả next_action; không auto-approve hoặc tự tăng limits.

## 6. ADR và điểm còn mở

| Vấn đề | Cần trước | Nội dung |
|---|---|---|
| Navigation/read model contract | T01/T04 | State key, stale events, return route, UI errors và draft ownership |
| CLI output compatibility | T07/T08 | Schema/nullability/syntax/precedence, exit mapping; giữ help/version P01 |
| UX chi tiết | T02–T06 | Bố cục, label, focus order, validation inline, help theo context |
| Terminal test matrix | T05/T06/T12 | OS/terminal, Unicode/input behavior, PTY/manual coverage |
| Scope gaps | T01 | Use case thiếu chuyển đúng owner/task, không triển khai giả ở Presentation |

Trạng thái ban đầu của các quyết định trên là cần đối chiếu/ghi nhận, không phải đã được thông qua. Dùng ADR đã có nếu còn phù hợp; chỉ mở rộng phần mới. Ghi lựa chọn, đánh đổi, task phụ thuộc và phép kiểm chứng. Chỉ phần thật sự thay đổi yêu cầu sản phẩm mới cần quyết định sản phẩm bổ sung.

## 7. Thiết kế triển khai

Presentation nhận read model/events có version và gửi command/query vào Application. Router/focus giữ UI state; event updates chỉ làm refresh phần liên quan, không tự thay task intent. Binary composition root/CLI dùng cùng contracts; output mapping là nơi duy nhất đổi core result sang JSON/exit.

| Nhóm | Điểm hoàn thiện |
|---|---|
| Tổng quan | Tiến độ theo content, blockers và deep links |
| Tập/chương, Tác vụ | Scope/start, queue/active status, pause/resume |
| Editor, Reviewer | Exact revision, evidence, draft/conflict và return route |
| Skills/ngữ cảnh | Selection/version/scope và atomic approval state |
| Ảnh/xuất | Readiness/stale/validation/version history |
| Cấu hình/báo cáo/trợ giúp | Effective config, chi phí có nguồn, contextual help |

Contract có consumer thực tế mới được công khai. Domain/Application giữ quy tắc và quyền công bố; adapter thực hiện IO qua port; TUI/CLI gọi cùng use case. Không nối hai adapter qua implementation nội bộ để né ownership hoặc tạo dependency vòng.

### 7.1. Ranh giới thao tác và trạng thái hiển thị

- Ghim ngữ cảnh thao tác theo workspace, tập, tài liệu và revision. Trạng thái màn đang xem giữ selection, filter, scroll và nháp. Trạng thái task đang thực thi đến từ Coordinator và vẫn hiển thị khi chuyển màn.
- Mỗi thao tác ghi dữ liệu đi từ control hoặc CLI parser tới cùng use case với target/version đã chọn. Sau khi nhận kết quả, refresh read model tương ứng. Nếu mục tiêu đổi, cho người dùng thấy xung đột cùng dữ liệu cần chọn lại.
- Sự kiện bàn phím vào modal, ô nhập hoặc vùng focus có quyền nhận trước. Route quyết định shortcut khả dụng. Resize chỉ tính lại bố cục và đưa vị trí hiển thị về mục neo.
- Map lỗi core ở CLI thành code/envelope và ở TUI thành vị trí, lý do, việc tiếp theo. Với Q05 giữ Chương, Block ID và hướng lập lại lượt ở cả hai đường.

### 7.2. Bảng đối chiếu phải bàn giao

T01 lập từng hàng theo màn hoặc lệnh, thao tác, target, use case, AC/TC, trạng thái chờ/lỗi và owner. T12 ghi fixture, kích thước/terminal, command hoặc phím đã dùng và kết quả thực tế. M9 nhận bảng này để chạy lại trên executable đóng gói. Ghi riêng các thao tác TUI có trong đặc tả và các thao tác chung cần test parity.

## 8. Dữ liệu, lỗi và phục hồi

| Tình huống | Hành vi UI/CLI |
|---|---|
| Core trả conflict | Giữ nháp/selections và chỉ rõ base/current; không refresh mất input |
| Event cũ/duplicate | Không hạ state mới; re-query nếu cần theo contract |
| Đổi workspace/volume | State đúng khóa; không áp draft/filter của document khác |
| Resize dưới ngưỡng | Giữ draft/task, không gửi lại request; expand khôi phục vị trí |
| CLI chờ duyệt/queue/budget | Checkpoint + task ID + next_action/exit; tiến trình kết thúc hữu hạn |
| Application lỗi/restart | Dữ liệu bền vững từ core; UI state trong phiên giữ theo spec; chưa tự hứa giữ draft chưa lưu qua process crash |

Không biến điều hướng hoặc refresh thành retry/resume. Secret vẫn bị che trong reports/error details/copyable output. Technical errors giữ next_action đủ để người dùng xử lý mà không phơi raw payload.

Mỗi lỗi cần giữ định danh phạm vi và nguyên nhân đã redaction, cùng bước tiếp theo. Không dùng thông báo thành công thay cho trạng thái đang chờ. Mọi thay đổi schema đã áp dụng phải có migration/version và kiểm thử tương thích; dữ liệu, lịch sử và sửa tay được bảo toàn.

## 9. Phần TUI/CLI

### Hợp đồng CLI phải phủ đủ

| Nhóm | Cú pháp theo đặc tả |
|---|---|
| Workspace | `workspace create <path>`, `workspace open <path>` |
| Import/scope | `import <epub> --volume <id>`, `scope set --volume <id> --content all-text\|story` |
| Dịch | `translate --volume <id>` với `--chapter <id>` lặp được hoặc `--to-end`; budget flags theo S-TUI |
| Review/issue | `review --volume <id> --chapter <id>`, `issues --task <id>` |
| Thuật ngữ | `terms list`, `terms confirm --file <json>` |
| Tác vụ | `status [--task <id>]`, `pause <task-id>`, `resume <task-id>` |
| Đầu ra | `check --volume <id>`, `preview --volume <id>`, `export --volume <id>` |
| Kết nối | `connection set <name> --key-env <env-name>`, `connection list`, `connection remove <name>` |

Kiểm tra `--workspace`, `--json`, `--non-interactive` ở nơi thích hợp; `--cost-limit`/`--currency` và `--unlimited-cost` có validation xung đột. Giữ tên executable và help/version của Phase 01. Không thêm command chỉ để né thiếu use case.

Thông báo sản phẩm dùng ngôn ngữ người dùng hiểu được; mã task/ADR/crate chỉ nằm trong tài liệu kỹ thuật. Điều hướng, cuộn, resize và mở trợ giúp không tự gửi model hoặc thay quyết định nghiệp vụ.

<a id="p08-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P08 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P08-T01 nhận các checkpoint giao diện P02–P07 ghi trong thẻ; ma trận được đối chiếu trên hành vi thật cùng các gap còn thiếu. Có thể đọc spec/lập nháp trước, nhưng chưa đóng T01 bằng bảng dự kiến. Code T02–T11 mở theo đầu ra UI/core cụ thể đã duyệt, không phải đợi toàn bộ M7. P08-T12/M8 vẫn cần M7 và nghiệm thu tám màn/toàn CLI. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p08-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p08-cp01"></a>P08-CP01 — Ma trận thiếu/đủ UI và CLI | [P08-T01](#p08-t01) | [P02-CP06](PHASE-02.md#p02-cp06), [P03-CP06](PHASE-03.md#p03-cp06), [P04-CP06](PHASE-04.md#p04-cp06), [P05-CP08](PHASE-05.md#p05-cp08), [P06-CP06](PHASE-06.md#p06-cp06), [P06-CP07](PHASE-06.md#p06-cp07), [P07-CP06](PHASE-07.md#p07-cp06), [P07-CP07](PHASE-07.md#p07-cp07) | P08-T01: V-01 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp02"></a>P08-CP02 — Tổng quan, cấu hình và báo cáo | [P08-T02](#p08-t02), [P08-T03](#p08-t03) | [P05-CP07](PHASE-05.md#p05-cp07), [P05-CP08](PHASE-05.md#p05-cp08), [P06-CP05](PHASE-06.md#p06-cp05), [P07-CP07](PHASE-07.md#p07-cp07), [P08-CP01](#p08-cp01) | P08-T02: V-02 trong phạm vi AC/TC của task; P08-T03: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp03"></a>P08-CP03 — Navigation theo dữ liệu | [P08-T04](#p08-t04) | [P03-CP02](PHASE-03.md#p03-cp02), [P04-CP06](PHASE-04.md#p04-cp06), [P05-CP08](PHASE-05.md#p05-cp08), [P06-CP07](PHASE-06.md#p06-cp07), [P07-CP07](PHASE-07.md#p07-cp07), [P08-CP01](#p08-cp01) | P08-T04: V-03 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp04"></a>P08-CP04 — Bàn phím, bố cục và Unicode | [P08-T05](#p08-t05), [P08-T06](#p08-t06) | [P08-CP02](#p08-cp02), [P08-CP03](#p08-cp03) | P08-T05: V-03 trong phạm vi AC/TC của task; P08-T06: V-04 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp05"></a>P08-CP05 — CLI và non-interactive | [P08-T07](#p08-t07), [P08-T08](#p08-t08) | [P02-CP06](PHASE-02.md#p02-cp06), [P03-CP06](PHASE-03.md#p03-cp06), [P05-CP08](PHASE-05.md#p05-cp08), [P06-CP06](PHASE-06.md#p06-cp06), [P06-CP07](PHASE-06.md#p06-cp07), [P07-CP07](PHASE-07.md#p07-cp07), [P08-CP01](#p08-cp01) | P08-T07: V-05 trong phạm vi AC/TC của task; P08-T08: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp06"></a>P08-CP06 — Parity hai giao diện | [P08-T09](#p08-t09) | [P08-CP02](#p08-cp02), [P08-CP04](#p08-cp04), [P08-CP05](#p08-cp05) | P08-T09: V-06 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp07"></a>P08-CP07 — Luồng xuyên màn | [P08-T10](#p08-t10), [P08-T11](#p08-t11) | [P08-CP03](#p08-cp03), [P08-CP04](#p08-cp04), [P08-CP05](#p08-cp05), [P08-CP06](#p08-cp06) | P08-T10: V-07 trong phạm vi AC/TC của task; P08-T11: V-07 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |
| <a id="p08-cp08"></a>P08-CP08 — Nghiệm thu M8 | [P08-T12](#p08-t12) | M7 Đạt, [P08-CP01](#p08-cp01), [P08-CP02](#p08-cp02), [P08-CP03](#p08-cp03), [P08-CP04](#p08-cp04), [P08-CP05](#p08-cp05), [P08-CP06](#p08-cp06), [P08-CP07](#p08-cp07) | P08-T12: V-08, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M8; V-00 cho code, B review đúng candidate. | feature/interface; A chat P08 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Các đường dẫn dưới đây là vị trí mục tiêu từ thiết kế. Không sửa phần ngoài phạm vi chỉ để tiện triển khai. Tất cả task khởi tạo ở trạng thái **Chưa bắt đầu**; lệnh V-* nằm ở mục 13.

<a id="p08-t01"></a>

### P08-T01 — Lập coverage matrix màn, hành động và commands

- **Mục tiêu / đầu ra:** Danh mục thao tác đủ tám màn và toàn CLI, gap có owner.
- **Căn cứ / phụ thuộc:** Q34–35, D-UI; M2–M7. Dependency thực thi: [P02-T12](PHASE-02.md#p02-t12), [P03-T13](PHASE-03.md#p03-t13), [P04-T12](PHASE-04.md#p04-t12), [P05-T13](PHASE-05.md#p05-t13), [P05-T15](PHASE-05.md#p05-t15), [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15), [P07-T12](PHASE-07.md#p07-t12), [P07-T14](PHASE-07.md#p07-t14).
- **Phạm vi:** UI/CLI acceptance matrix và task scope.
- **Việc làm:** Map control→use case→AC→test; phân chưa nối UI với core thiếu; ghi trạng thái empty/loading/error/wait/conflict.
- **Ca lỗi / biên:** Mock success thay core; action có chuột nhưng không có bàn phím; bỏ state lỗi.
- **Nghiệm thu:** `P08-AC01` — Mỗi hành động S-TUI có nơi thực thi/test; gap được giao đúng owner, không tự đánh dấu đã đủ.
- **Kiểm chứng:** `P08-TC01`, V-01; dữ liệu kiểm tra: S-TUI command/screen table và use case catalog.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lập hàng cho tám màn của S-TUI §8.2 và từng lệnh của §8.1. Gắn action, use case, AC, test và trạng thái empty/loading/error/wait/conflict.
2. Ghi thao tác chung và thao tác riêng của từng giao diện. Kiểm mỗi thao tác ghi có đúng target và quyền sử dụng.
3. Phân gap theo thiếu nối giao diện, thiếu read model hay thiếu hành vi core. Giao đúng owner và giữ dependency còn chặn trong PLAN khi triển khai.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC01.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T01 |
|---|---|
| Đầu vào bắt buộc | [P02-T12](PHASE-02.md#p02-t12), [P03-T13](PHASE-03.md#p03-t13), [P04-T12](PHASE-04.md#p04-t12), [P05-T13](PHASE-05.md#p05-t13), [P05-T15](PHASE-05.md#p05-t15), [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15), [P07-T12](PHASE-07.md#p07-t12), [P07-T14](PHASE-07.md#p07-t14). Nhận shell M1 và các checkpoint giao diện đã duyệt ở P02–P07 theo danh sách dependency; đối chiếu từng control/command thật, ghi gap có owner. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Ma trận action→target→use case→AC/TC nêu rõ gap và owner cho success/error/wait/conflict. Owner đầu ra là A-P08-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T01 giữ UI/CLI acceptance matrix và task scope. Phần chung do T01 giữ matrix, owner core từ phase tương ứng giữ nghiệp vụ. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC01 / P08-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Walkthrough từng control/command với dữ liệu thành công và ít nhất một state không thành công. |
| Điểm tích hợp | [P08-CP01](#p08-cp01) — Ma trận thiếu/đủ UI và CLI; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t02"></a>

### P08-T02 — UI-14 Tổng quan và deep links

- **Mục tiêu / đầu ra:** Dashboard đưa tới đúng nội dung/tác vụ đang cần xử lý.
- **Căn cứ / phụ thuộc:** S-TUI, S-DATA; T01. Dependency thực thi: [P08-T01](#p08-t01), [P06-T12](PHASE-06.md#p06-t12), [P07-T14](PHASE-07.md#p07-t14), [P05-T13](PHASE-05.md#p05-t13).
- **Phạm vi:** nt-tui dashboard/overview read model wiring.
- **Việc làm:** Hiện bộ/tập/content progress/blockers; links tới task/issue/term; phân dịch hoàn tất và export thành công.
- **Ca lỗi / biên:** Task khác active, zero data, stale blocker, retained source tính như translated.
- **Nghiệm thu:** `P08-AC02` — Click/keyboard mở đúng identity; số liệu không trộn quality/execution hoặc gọi AI.
- **Kiểm chứng:** `P08-TC02`, V-02; dữ liệu kiểm tra: Nhiều volumes/tasks, partial coverage và export states.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dựng Tổng quan từ read model theo bộ/tập, phạm vi nội dung và các trục tiến độ. Tách dịch/Review, tác vụ chạy và file xuất.
2. Tạo lối tắt theo target ID cho task, issue và mục chờ duyệt. Kiểm target và workspace khi mở route.
3. Giữ ngữ cảnh Tổng quan khi quay về. Với dữ liệu đã đổi, refresh danh sách và hiển thị lý do mục cũ chưa mở được.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC02.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T02 |
|---|---|
| Đầu vào bắt buộc | [P08-T01](#p08-t01), [P06-T12](PHASE-06.md#p06-t12), [P07-T14](PHASE-07.md#p07-t14), [P05-T13](PHASE-05.md#p05-t13). Nhận ma trận và progress/blockers read model M3–M7. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Dashboard tách tiến độ nội dung/tác vụ/xuất, deep link bằng target ID và refresh khi target đổi. Owner đầu ra là A-P08-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T02 giữ nt-tui dashboard/overview read model wiring. Phần chung do T01 giữ coverage, owners core giữ read model, người tích hợp giữ router. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC02 / P08-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu Displayed counts/links đúng core snapshot; navigation có zero model dispatch. |
| Điểm tích hợp | [P08-CP02](#p08-cp02) — Tổng quan, cấu hình và báo cáo; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t03"></a>

### P08-T03 — UI-14 Cấu hình, báo cáo và help

- **Mục tiêu / đầu ra:** Reports/help phản ánh hành vi và cấu hình thực.
- **Căn cứ / phụ thuộc:** S-TUI/S-MODEL/S-SEC; T01. Dependency thực thi: [P08-T01](#p08-t01), [P05-T14](PHASE-05.md#p05-t14), [P07-T14](PHASE-07.md#p07-t14).
- **Phạm vi:** nt-tui settings/reports/help.
- **Việc làm:** Effective configs, pricing provenance/unknown, operation reports; F1 theo màn/focus; shortcuts đúng capability.
- **Ca lỗi / biên:** Report lộ key, data stale, pending save, help quảng bá chức năng chưa nối.
- **Nghiệm thu:** `P08-AC03` — Thông tin đủ phân estimated/provider/unknown, lỗi có bước xử lý; help khớp phím thực.
- **Kiểm chứng:** `P08-TC03`, V-02; dữ liệu kiểm tra: Fake credentials, missing pricing, reports lỗi và empty.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Hiển thị model và cấu hình hiệu lực, nguồn giá, phần chi phí đã biết/chưa biết và lịch sử operation theo task.
2. Dùng trạng thái capability và vùng focus để tạo F1/help. Gắn liên kết tới thao tác, báo cáo hoặc phần cấu hình được hỗ trợ.
3. Áp cùng redaction khi trình bày connection, provider error và báo cáo. Giữ focus khi đóng trợ giúp.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC03.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T03 |
|---|---|
| Đầu vào bắt buộc | [P08-T01](#p08-t01), [P05-T14](PHASE-05.md#p05-t14), [P07-T14](PHASE-07.md#p07-t14). Nhận capability/help, effective config và ledger/report M5–M7. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Settings/report/F1 phản ánh model/giá/unknown thực và redaction, đóng help về đúng focus. Owner đầu ra là A-P08-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T03 giữ nt-tui settings/reports/help. Phần chung do P05-T01/T04 giữ config/money, T03 giữ help/report UI. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC03 / P08-TC03, gồm các ca chi tiết §13.3. V-02. Đối chiếu Redaction trên render/details; F1 không đổi task hoặc config. |
| Điểm tích hợp | [P08-CP02](#p08-cp02) — Tổng quan, cấu hình và báo cáo; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t04"></a>

### P08-T04 — State navigation theo dữ liệu đang xem

- **Mục tiêu / đầu ra:** Router giữ selection/filter/scroll/draft/return context.
- **Căn cứ / phụ thuộc:** Q35, S-TUI; T01 và routes P01. Dependency thực thi: [P08-T01](#p08-t01), [P03-T12](PHASE-03.md#p03-t12), [P04-T12](PHASE-04.md#p04-t12), [P05-T13](PHASE-05.md#p05-t13), [P06-T15](PHASE-06.md#p06-t15), [P07-T14](PHASE-07.md#p07-t14).
- **Phạm vi:** nt-tui navigation/state/read-model events.
- **Việc làm:** Key state theo workspace/volume/document; xử lý stale events; back/refresh không reset nháp.
- **Ca lỗi / biên:** ID trùng giữa workspace, chuyển volume khi đang soạn, duplicate update, removed item.
- **Nghiệm thu:** `P08-AC04` — Không rò draft/selection sang document khác; quay lại khôi phục đúng trạng thái còn hợp lệ.
- **Kiểm chứng:** `P08-TC04`, V-03; dữ liệu kiểm tra: Hai workspace với IDs dễ nhầm và event out-of-order.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dùng identity workspace/tập/tài liệu để lưu route, filter, scroll, selected ID và nháp cùng base revision.
2. Refresh theo event version và query scope. Xử lý target bị xóa hoặc chuyển revision bằng trạng thái cần chọn lại.
3. Khôi phục return route khi back. Khi chuyển dữ liệu có nháp, đi qua quyết định lưu/giữ/hủy của Editor.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC04.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T04 |
|---|---|
| Đầu vào bắt buộc | [P08-T01](#p08-t01), [P03-T12](PHASE-03.md#p03-t12), [P04-T12](PHASE-04.md#p04-t12), [P05-T13](PHASE-05.md#p05-t13), [P06-T15](PHASE-06.md#p06-t15), [P07-T14](PHASE-07.md#p07-t14). Nhận route M1 và identity/version của read models đã giao. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Navigation state theo workspace/tập/tài liệu, event stale không tráo target, back/refresh bảo vệ draft. Owner đầu ra là A-P08-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T04 giữ nt-tui navigation/state/read-model events. Phần chung do T04 giữ router/state key chung, Editor M3 giữ draft/save. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC04 / P08-TC04, gồm các ca chi tiết §13.3. V-03. Đối chiếu State identities và draft bytes đúng qua chuỗi navigation. |
| Điểm tích hợp | [P08-CP03](#p08-cp03) — Navigation theo dữ liệu; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t05"></a>

### P08-T05 — UI-15 keyboard và input routing

- **Mục tiêu / đầu ra:** Toàn bộ control thao tác được không cần chuột.
- **Căn cứ / phụ thuộc:** Q35, S-TUI; T02–T04. Dependency thực thi: [P08-T02](#p08-t02), [P08-T03](#p08-t03), [P08-T04](#p08-t04).
- **Phạm vi:** nt-tui focus/action/inputs/modals.
- **Việc làm:** Arrow/Tab/Enter/Space/Esc/F1; Ctrl+S; scope /f/e/p/r; rõ focus/disabled state; không event xuyên modal.
- **Ca lỗi / biên:** Space/Enter trong editor, Esc ở root/dirty draft, nested modal, key repeats.
- **Nghiệm thu:** `P08-AC05` — Phím đúng vùng; không mất draft hoặc tạo command phía sau; không focus trap.
- **Kiểm chứng:** `P08-TC05`, V-03; dữ liệu kiểm tra: Event sequences cho tám màn và inputs nhiều dòng.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Rà bảng phím P01 đã duyệt và Q35. Nối phím theo focus và capability của từng màn, có dấu hiệu rõ khi action bị vô hiệu.
2. Xử lý modal/ô nhập trước shortcut màn. Enter trong Editor xuống dòng, Ctrl+S đi qua use case lưu với base revision.
3. Giữ quy tắc press/repeat/release của M1 cho thao tác một lần. Kiểm đóng trợ giúp, modal và thoát ứng dụng trên terminal thật.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC05.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T05 |
|---|---|
| Đầu vào bắt buộc | [P08-T02](#p08-t02), [P08-T03](#p08-t03), [P08-T04](#p08-t04). Nhận các control/focus/route và input protocol M1. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Input/modal nhận trước shortcut, action theo capability, Ctrl+S/Enter/repeat giữ contract đã duyệt. Owner đầu ra là A-P08-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T05 giữ nt-tui focus/action/inputs/modals. Phần chung do T04 giữ navigation, T05 giữ keyboard routing. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC05 / P08-TC05, gồm các ca chi tiết §13.3. V-03. Đối chiếu Action trace đúng expected; thử bằng bàn phím trên terminal thật. |
| Điểm tích hợp | [P08-CP04](#p08-cp04) — Bàn phím, bố cục và Unicode; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t06"></a>

### P08-T06 — UI-15 layout và Unicode trên tám màn

- **Mục tiêu / đầu ra:** Hẹp/rộng/quá nhỏ giữ đầy đủ dữ liệu thao tác.
- **Căn cứ / phụ thuộc:** Q35, D-UI; T04–T05. Dependency thực thi: [P08-T04](#p08-t04), [P08-T05](#p08-t05).
- **Phạm vi:** nt-tui layout/wrap/scroll components/screens.
- **Việc làm:** 80×24,119/120 breakpoint, two-pane phù hợp, details dài, global status nhìn thấy; dưới ngưỡng safe controls.
- **Ca lỗi / biên:** Resize khi đang gõ/chờ retry, wide/combining characters, quote dài, modal vượt khung.
- **Nghiệm thu:** `P08-AC06` — Không mất draft/selection; mở được đầy đủ evidence; không restart/cancel task do resize.
- **Kiểm chứng:** `P08-TC06`, V-04; dữ liệu kiểm tra: Tám màn × sizes với Nhật/Việt tự soạn.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Rà tất cả tám màn ở 80×24, rộng dưới/đúng 120 và dưới ngưỡng. Màn phù hợp chuyển hai vùng theo S-TUI, giữ global status.
2. Dùng phép đo độ rộng terminal và cụm ký tự cho text Nhật, Việt, emoji và trường dài. Kiểm wrap/truncate không làm mất vị trí đang thao tác.
3. Khi resize, tính lại viewport theo mục neo, focus, scroll và nháp. Dưới ngưỡng hiện hành động an toàn và khôi phục được.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC06.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T06 |
|---|---|
| Đầu vào bắt buộc | [P08-T04](#p08-t04), [P08-T05](#p08-t05). Nhận route/focus/viewport state và keyboard actions. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | 80×24, breakpoint120 và dưới ngưỡng giữ anchor/draft, đo Unicode theo cụm ký tự. Owner đầu ra là A-P08-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T06 giữ nt-tui layout/wrap/scroll components/screens. Phần chung do T04/T05 giữ input/state, T06 giữ layout/width. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC06 / P08-TC06, gồm các ca chi tiết §13.3. V-04. Đối chiếu State/buffer và terminal thực đều đúng; không chỉ snapshot màu. |
| Điểm tích hợp | [P08-CP04](#p08-cp04) — Bàn phím, bố cục và Unicode; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t07"></a>

### P08-T07 — CLI grammar, flags và JSON schema

- **Mục tiêu / đầu ra:** Command parser/output đầy đủ theo S-TUI.
- **Căn cứ / phụ thuộc:** Q34, S-TUI; T01, ADR-008. Dependency thực thi: [P08-T01](#p08-t01), [P02-T12](PHASE-02.md#p02-t12), [P03-T13](PHASE-03.md#p03-t13), [P05-T15](PHASE-05.md#p05-t15), [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15), [P07-T14](PHASE-07.md#p07-t14).
- **Phạm vi:** App cli args/commands/output; contract tests.
- **Việc làm:** Phủ bảng lệnh S-TUI §8.1, workspace/path quoting, conflicting flags, schema/fields/types, stdout/stderr và các mã 0, 2–7.
- **Ca lỗi / biên:** Missing args, chapter/to-end cùng lúc, cost-limit/unlimited conflict, wrong currency, JSON lẫn ANSI/log.
- **Nghiệm thu:** `P08-AC07` — Parser reject đúng; machine output parse toàn bộ; status read success trả 0, state công việc ở JSON.
- **Kiểm chứng:** `P08-TC07`, V-05; dữ liệu kiểm tra: Command table valid/invalid, Unicode paths và mọi core result class.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Tạo parser và help theo S-TUI §8.1. Kiểm cờ tương thích, tham số bắt buộc và đường dẫn trước khi mở workspace hoặc terminal.
2. Map core result thành envelope schema v1 gồm tám trường theo ADR-008 và exit 0, 2–7 theo đặc tả. Giữ help/version và status read theo hợp đồng.
3. Chạy process harness để tách stdout JSON và stderr đã che. Kiểm Unicode và đường dẫn có khoảng trắng theo quy tắc từng shell.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC07.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T07 |
|---|---|
| Đầu vào bắt buộc | [P08-T01](#p08-t01), [P02-T12](PHASE-02.md#p02-t12), [P03-T13](PHASE-03.md#p03-t13), [P05-T15](PHASE-05.md#p05-t15), [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15), [P07-T14](PHASE-07.md#p07-t14). Nhận S-TUI §8.1, ADR-008 và use cases đủ từ M7. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Parser kiểm cờ trước IO, envelope tám fields và exit 0/2–7, status read/help/version giữ contract. Owner đầu ra là A-P08-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T07 giữ App cli args/commands/output; contract tests. Phần chung do ADR-008 owner giữ envelope, T07 giữ CLI args/output, người tích hợp cấp registry. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC07 / P08-TC07, gồm các ca chi tiết §13.3. V-05. Đối chiếu Subprocess exit/output đúng từng trường hợp; help/version giữ contract P01. |
| Điểm tích hợp | [P08-CP05](#p08-cp05) — CLI và non-interactive; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t08"></a>

### P08-T08 — CLI non-interactive và tiếp tục tác vụ

- **Mục tiêu / đầu ra:** Script không bị prompt và có checkpoint/next action.
- **Căn cứ / phụ thuộc:** Q29/34, S-TUI/S-WORKFLOW; T07 và recovery P05/P06. Dependency thực thi: [P08-T07](#p08-t07).
- **Phạm vi:** App command execution/wait/output mapping.
- **Việc làm:** Lưu trước trả waiting/queued/budget/conflict; task IDs rõ; resume cùng core và counters.
- **Ca lỗi / biên:** Pipe stdin, need approval, no credential, paused, duplicate active request, pending Skill publication.
- **Nghiệm thu:** `P08-AC08` — Kết thúc hữu hạn đúng code, không auto-approve/fallback; resume giữ tiến độ/chi phí.
- **Kiểm chứng:** `P08-TC08`, V-05; dữ liệu kiểm tra: Process harness có time bound, fake core/real temporary storage.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Nối non-interactive vào cùng entry point nghiệp vụ và trả khi đạt trạng thái cần quyết định, xếp hàng, hết giới hạn, pause hoặc lỗi.
2. Lưu checkpoint trước trả task ID và next_action. Phân biệt lỗi trước khi tạo task với task đã được lưu.
3. Dùng resume đã có để đối chiếu căn cứ, counter, budget và permit. Không tự chọn thay người dùng khi thiếu dữ liệu.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC08.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T08 |
|---|---|
| Đầu vào bắt buộc | [P08-T07](#p08-t07). Nhận parser/result mapping cùng recovery/queue M5/M6. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Non-interactive không prompt, lưu checkpoint trước return, task ID và next_action khớp state/counters. Owner đầu ra là A-P08-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T08 giữ App command execution/wait/output mapping. Phần chung do T07 giữ parser, P05-T12/P06-T12 giữ resume, T08 giữ CLI lifecycle. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC08 / P08-TC08, gồm các ca chi tiết §13.3. V-05. Đối chiếu Không đọc input ngoài ý muốn; checkpoint readback đúng next_action. |
| Điểm tích hợp | [P08-CP05](#p08-cp05) — CLI và non-interactive; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t09"></a>

### P08-T09 — Parity CLI và TUI tại boundary

- **Mục tiêu / đầu ra:** Cùng command/input dẫn tới cùng state/quyền.
- **Căn cứ / phụ thuộc:** S-TUI, R-REPO; T07–T08, use cases P02–P07. Dependency thực thi: [P08-T07](#p08-t07), [P08-T08](#p08-t08), [P08-T02](#p08-t02), [P08-T03](#p08-t03), [P08-T06](#p08-t06).
- **Phạm vi:** App/TUI integration adapters và differential fixtures.
- **Việc làm:** Chạy hai cách trên bản sao fixture độc lập; so core outcome, history, money/permit/approval và export.
- **Ca lỗi / biên:** CLI bỏ validation, UI auto-resume, write path riêng, error mapping khác nghĩa.
- **Nghiệm thu:** `P08-AC09` — Khác cách trình bày nhưng cùng điều kiện và tác động nghiệp vụ; không có đường tắt.
- **Kiểm chứng:** `P08-TC09`, V-06; dữ liệu kiểm tra: Import, save/approval, resume/budget và export blocked/success.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chạy các thao tác chung từ CLI/TUI trên bản sao độc lập của cùng fixture. Chuẩn hóa ID/ngày tạo trong phép so theo contract.
2. So outcome ở boundary qua revision, history, decision, permit, budget và export. Đối chiếu cả lỗi và trạng thái chờ.
3. Tách khác biệt do trình bày với sai lệch nghiệp vụ. Giao sai lệch core cho owner rồi chạy lại cả hai đường.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC09.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T09 |
|---|---|
| Đầu vào bắt buộc | [P08-T07](#p08-t07), [P08-T08](#p08-t08), [P08-T02](#p08-t02), [P08-T03](#p08-t03), [P08-T06](#p08-t06). Nhận hai entry points cùng use cases/snapshot từ M7. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Parity so normalized core outcomes/history/fees/permit/approval/export trên fixture độc lập. Owner đầu ra là A-P08-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T09 giữ App/TUI integration adapters và differential fixtures. Phần chung do owners core giữ commands, T09 giữ differential test harness. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC09 / P08-TC09, gồm các ca chi tiết §13.3. V-06. Đối chiếu Normalize IDs/time rồi so state/effects theo oracle; không chỉ so chuỗi thông báo. |
| Điểm tích hợp | [P08-CP06](#p08-cp06) — Parity hai giao diện; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t10"></a>

### P08-T10 — Luồng Reviewer–Editor và bảo vệ nháp

- **Mục tiêu / đầu ra:** Vòng qua nhiều màn giữ exact target và bản đang soạn.
- **Căn cứ / phụ thuộc:** Q08/22/35; T04–T06 và UI-04/05/09. Dependency thực thi: [P08-T04](#p08-t04), [P08-T05](#p08-t05), [P08-T06](#p08-t06).
- **Phạm vi:** nt-tui reviewer/editor navigation/conflict integration.
- **Việc làm:** Issue→source/target→edit/save/history→back; re-query khi revision đổi; conflict resolution có chủ đích.
- **Ca lỗi / biên:** Background Review/Skill change, resize, user Esc, stale issue target.
- **Nghiệm thu:** `P08-AC10` — Không đánh tráo revision; draft không mất; quay lại đúng card/filter/scroll và validity mới.
- **Kiểm chứng:** `P08-TC10`, V-07; dữ liệu kiểm tra: r1 issue/current r2, dirty draft và background update.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Mở Editor từ issue với source span và revision nền cụ thể. Lưu route, issue ID, filter, selection và scroll để quay lại.
2. Giữ nháp theo tài liệu. Khi lưu, dùng use case tạo revision sửa tay và giữ bản nền; conflict yêu cầu lựa chọn rõ ràng.
3. Quay lại Reviewer theo ngữ cảnh đã lưu, refresh issue và cho Review lại revision mới qua core.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC10.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T10 |
|---|---|
| Đầu vào bắt buộc | [P08-T04](#p08-t04), [P08-T05](#p08-t05), [P08-T06](#p08-t06). Nhận Reviewer/Editor UI-04/05/09 và navigation đã duyệt. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Issue→Editor→save/back giữ source/target/base và draft, revision đổi phải xử lý conflict rõ. Owner đầu ra là A-P08-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T10 giữ nt-tui reviewer/editor navigation/conflict integration. Phần chung do T04 giữ return route, P03-T12 giữ save, T10 giữ cross-screen flow. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC10 / P08-TC10, gồm các ca chi tiết §13.3. V-07. Đối chiếu Source/target/draft IDs/text giữ đúng qua toàn chuỗi. |
| Điểm tích hợp | [P08-CP07](#p08-cp07) — Luồng xuyên màn; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t11"></a>

### P08-T11 — Luồng Tasks–Skills–Export và quyền chạy

- **Mục tiêu / đầu ra:** UI phản ánh đúng queue/approval/readiness khi cập nhật liên tục.
- **Căn cứ / phụ thuộc:** Q15/28/33/35; T04–T09 và UI-06/10/12/13. Dependency thực thi: [P08-T04](#p08-t04), [P08-T05](#p08-t05), [P08-T06](#p08-t06), [P08-T07](#p08-t07), [P08-T08](#p08-t08), [P08-T09](#p08-t09).
- **Phạm vi:** nt-tui integrated workflows/read model refresh.
- **Việc làm:** Xem B khi A active; duyệt nhóm đúng selection; pending sync; paused resume qua queue; stale export readiness.
- **Ca lỗi / biên:** Candidate mới giữa chọn/confirm, A late response, B chưa độc lập, bundle stale khi bấm export.
- **Nghiệm thu:** `P08-AC11` — Global status đúng A; approval đúng nhóm; UI không cấp quyền/chốt readiness riêng.
- **Kiểm chứng:** `P08-TC11`, V-07; dữ liệu kiểm tra: Hai tasks/workspaces, selected terms và changing bundle.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Hiện active permit và task đang xem riêng. Nối pause/resume qua Coordinator, đọc trạng thái sau mỗi event.
2. Giữ batch selection theo item/version. Hiện ghi quyết định và chờ đồng bộ rõ ràng khi approval publication chưa xong.
3. Tính lại readiness khi inputs đổi và nối người dùng tới blocker/Preview stale. Giữ task user-paused qua approval.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC11.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T11 |
|---|---|
| Đầu vào bắt buộc | [P08-T04](#p08-t04), [P08-T05](#p08-t05), [P08-T06](#p08-t06), [P08-T07](#p08-t07), [P08-T08](#p08-t08), [P08-T09](#p08-t09). Nhận queue/approval/Preview read models và parity nền. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Tasks→Skills→Export giữ active permit, exact batch và stale readiness qua refresh, paused vẫn cần resume. Owner đầu ra là A-P08-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T11 giữ nt-tui integrated workflows/read model refresh. Phần chung do P05/P06/P07 giữ core, T11 giữ flow xuyên màn. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02), [P03-T14](PHASE-03.md#p03-t14). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC11 / P08-TC11, gồm các ca chi tiết §13.3. V-07. Đối chiếu Action trace và core state đúng; navigation không tăng send count. |
| Điểm tích hợp | [P08-CP07](#p08-cp07) — Luồng xuyên màn; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p08-t12"></a>

### P08-T12 — Nghiệm thu tám màn và toàn CLI

- **Mục tiêu / đầu ra:** M8 với keyboard/terminal/process evidence.
- **Căn cứ / phụ thuộc:** Q34–36, S-TUI; T01–T11. Dependency thực thi: [P08-T01](#p08-t01), [P08-T02](#p08-t02), [P08-T03](#p08-t03), [P08-T04](#p08-t04), [P08-T05](#p08-t05), [P08-T06](#p08-t06), [P08-T07](#p08-t07), [P08-T08](#p08-t08), [P08-T09](#p08-t09), [P08-T10](#p08-t10), [P08-T11](#p08-t11), M7 Đạt.
- **Phạm vi:** App acceptance tests, manual scripts, README/help.
- **Việc làm:** Chạy luồng đầu-cuối, ma trận trạng thái/kích thước/OS; rà commands/help; cập nhật hướng dẫn khả năng thật.
- **Ca lỗi / biên:** Headless pass nhưng terminal lỗi; output pipe bị treo; control không truy cập bàn phím; help lệch code.
- **Nghiệm thu:** `P08-AC12` — Tám màn/CLI đạt bảng coverage; ca chưa chạy trên OS/terminal nào được nêu, không tuyên bố hỗ trợ chưa kiểm.
- **Kiểm chứng:** `P08-TC12`, V-08, V-00; dữ liệu kiểm tra: Workspace tự soạn từ P02–P07 và test profiles.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chạy ma trận T01 trên bản tích hợp. Ghi command/help, fixture, kết quả process và thao tác bàn phím theo kích thước terminal.
2. Lặp luồng import → dịch → Review/sửa → duyệt → xuất với provider giả. Thu bằng chứng xử lý chờ, conflict, resume và readback.
3. Kiểm đóng TUI, lỗi khởi động và terminal cleanup. Bàn giao danh sách nền tảng đã kiểm và phần hoãn cùng báo cáo M8.

Ca thử chi tiết tại [mục 13.3](#p08-cases) theo mã P08-TC12.

**Sáu trường giao việc**

| Trường | Nội dung cho P08-T12 |
|---|---|
| Đầu vào bắt buộc | [P08-T01](#p08-t01), [P08-T02](#p08-t02), [P08-T03](#p08-t03), [P08-T04](#p08-t04), [P08-T05](#p08-t05), [P08-T06](#p08-t06), [P08-T07](#p08-t07), [P08-T08](#p08-t08), [P08-T09](#p08-t09), [P08-T10](#p08-t10), [P08-T11](#p08-t11), M7 Đạt. Nhận candidate tám màn/CLI, fixture M2–M7 và platform matrix. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | M8 chứng minh action/help/JSON/terminal thật và cleanup theo target, ca Linux native ghi hoãn. Owner đầu ra là A-P08-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P08-T12 giữ App acceptance tests, manual scripts, README/help. Phần chung do owners T01–T11 giữ phần việc, A-T12 tích hợp. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P08-AC12 / P08-TC12, gồm các ca chi tiết §13.3. V-08, V-00. Đối chiếu Lưu evidence theo screen/command/size/OS; không chạy provider thật để thử giao diện. |
| Điểm tích hợp | [P08-CP08](#p08-cp08) — Nghiệm thu M8; A chat P08 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P08-AC01 | Mỗi hành động S-TUI có nơi thực thi/test; gap được giao đúng owner, không tự đánh dấu đã đủ. | P08-T01 | P08-TC01; V-01 |
| P08-AC02 | Click/keyboard mở đúng identity; số liệu không trộn quality/execution hoặc gọi AI. | P08-T02 | P08-TC02; V-02 |
| P08-AC03 | Thông tin đủ phân estimated/provider/unknown, lỗi có bước xử lý; help khớp phím thực. | P08-T03 | P08-TC03; V-02 |
| P08-AC04 | Không rò draft/selection sang document khác; quay lại khôi phục đúng trạng thái còn hợp lệ. | P08-T04 | P08-TC04; V-03 |
| P08-AC05 | Phím đúng vùng; không mất draft hoặc tạo command phía sau; không focus trap. | P08-T05 | P08-TC05; V-03 |
| P08-AC06 | Không mất draft/selection; mở được đầy đủ evidence; không restart/cancel task do resize. | P08-T06 | P08-TC06; V-04 |
| P08-AC07 | Parser reject đúng; machine output parse toàn bộ; status read success trả 0, state công việc ở JSON. | P08-T07 | P08-TC07; V-05 |
| P08-AC08 | Kết thúc hữu hạn đúng code, không auto-approve/fallback; resume giữ tiến độ/chi phí. | P08-T08 | P08-TC08; V-05 |
| P08-AC09 | Khác cách trình bày nhưng cùng điều kiện và tác động nghiệp vụ; không có đường tắt. | P08-T09 | P08-TC09; V-06 |
| P08-AC10 | Không đánh tráo revision; draft không mất; quay lại đúng card/filter/scroll và validity mới. | P08-T10 | P08-TC10; V-07 |
| P08-AC11 | Global status đúng A; approval đúng nhóm; UI không cấp quyền/chốt readiness riêng. | P08-T11 | P08-TC11; V-07 |
| P08-AC12 | Tám màn/CLI đạt bảng coverage; ca chưa chạy trên OS/terminal nào được nêu, không tuyên bố hỗ trợ chưa kiểm. | P08-T12 | P08-TC12; V-08, V-00 |

Các AC không thay thế nhau. Build thành công, fixture trả đúng kết quả định sẵn hoặc một màn hiển thị đẹp không chứng minh toàn bộ hành vi nghiệp vụ.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P08-TC01 / P08-AC01 | S-TUI command/screen table và use case catalog. | Walkthrough từng control/command với dữ liệu thành công và ít nhất một state không thành công. | V-01 |
| P08-TC02 / P08-AC02 | Nhiều volumes/tasks, partial coverage và export states. | Displayed counts/links đúng core snapshot; navigation có zero model dispatch. | V-02 |
| P08-TC03 / P08-AC03 | Fake credentials, missing pricing, reports lỗi và empty. | Redaction trên render/details; F1 không đổi task hoặc config. | V-02 |
| P08-TC04 / P08-AC04 | Hai workspace với IDs dễ nhầm và event out-of-order. | State identities và draft bytes đúng qua chuỗi navigation. | V-03 |
| P08-TC05 / P08-AC05 | Event sequences cho tám màn và inputs nhiều dòng. | Action trace đúng expected; thử bằng bàn phím trên terminal thật. | V-03 |
| P08-TC06 / P08-AC06 | Tám màn × sizes với Nhật/Việt tự soạn. | State/buffer và terminal thực đều đúng; không chỉ snapshot màu. | V-04 |
| P08-TC07 / P08-AC07 | Command table valid/invalid, Unicode paths và mọi core result class. | Subprocess exit/output đúng từng trường hợp; help/version giữ contract P01. | V-05 |
| P08-TC08 / P08-AC08 | Process harness có time bound, fake core/real temporary storage. | Không đọc input ngoài ý muốn; checkpoint readback đúng next_action. | V-05 |
| P08-TC09 / P08-AC09 | Import, save/approval, resume/budget và export blocked/success. | Normalize IDs/time rồi so state/effects theo oracle; không chỉ so chuỗi thông báo. | V-06 |
| P08-TC10 / P08-AC10 | r1 issue/current r2, dirty draft và background update. | Source/target/draft IDs/text giữ đúng qua toàn chuỗi. | V-07 |
| P08-TC11 / P08-AC11 | Hai tasks/workspaces, selected terms và changing bundle. | Action trace và core state đúng; navigation không tăng send count. | V-07 |
| P08-TC12 / P08-AC12 | Workspace tự soạn từ P02–P07 và test profiles. | Lưu evidence theo screen/command/size/OS; không chạy provider thật để thử giao diện. | V-08, V-00 |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo trong task tương ứng; nếu đổi phải cập nhật kế hoạch. Lệnh exit 0 với 0 ca liên quan không được ghi là đạt. 

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | Rà coverage matrix screen/control/use case/command/AC; mọi gap có task và owner. |
| V-02 | cargo test -p nt-tui --test dashboard_settings_help --locked |
| V-03 | cargo test -p nt-tui --test navigation_keyboard --locked |
| V-04 | cargo test -p nt-tui --test responsive_unicode --locked; kiểm terminal thật 80×24/120×24 và dưới ngưỡng |
| V-05 | cargo test -p novel-translate --test cli_contract --locked |
| V-06 | cargo test -p novel-translate --test cli_tui_parity --locked |
| V-07 | cargo test -p nt-tui --test cross_screen_flows --locked |
| V-08 | cargo test -p novel-translate --test phase08_acceptance --locked; thực hiện checklist tám màn bằng bàn phím trên matrix đã chọn |

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

<a id="p08-cases"></a>

### 13.3. Ca thử chi tiết theo task

Dùng cùng mã V của TC gốc. Snapshot hiển thị kiểm phần chữ/trạng thái, event test kiểm intent và subprocess kiểm exit/stdout/stderr. Ghi baseline state và side effects trước/sau với mỗi ca reject hoặc conflict. Kiểm Unicode, resize, bàn phím và terminal cleanup trên terminal Windows thật, ngoài test tự động.

| Mã ca | Fixture và thao tác | Kết quả cần đối chiếu |
|---|---|---|
| P08-TC01a | Đối chiếu tám màn và toàn bộ CLI help với bảng S-TUI. | Mỗi hành vi có một hàng truy vết và subview không tăng số màn chính. |
| P08-TC01b | Một control đã có core và một control chưa có use case. | Ma trận phân được hai loại gap và nêu task xử lý. |
| P08-TC01c | Một lệnh hoặc màn chỉ có ca thành công. | Hàng nêu rõ các trạng thái biên và check còn thiếu. |
| P08-TC02a | Tập dịch xong có một blocker xuất. | Dashboard tách tiến độ dịch và readiness, mở đúng việc cần xử lý. |
| P08-TC02b | Mục chờ duyệt chuyển scope sau khi Dashboard đã render. | Deep link báo mục đổi và không mở một mục trùng tên của tập khác. |
| P08-TC02c | Workspace trống rồi thêm một tập mới. | Empty state có hướng đi, refresh hiển thị đúng tập và không mất focus. |
| P08-TC03a | Task dùng model khác default và usage chưa xác định. | Báo cáo ghi đúng model hiệu lực và phần chi phí chưa biết. |
| P08-TC03b | F1 trong Reviewer, Editor và modal nhập. | Help khớp ngữ cảnh, thoát về đúng focus. |
| P08-TC03c | Provider error chứa sentinel giống key. | Màn cấu hình và báo cáo chỉ hiện bản đã che. |
| P08-TC04a | Hai workspace có cùng ID cục bộ và tên tập. | Chuyển qua lại khôi phục đúng mục và nháp của từng workspace. |
| P08-TC04b | Event cũ đến muộn sau khi chuyển tập. | Event không thay selection hoặc dữ liệu của route hiện tại. |
| P08-TC04c | Mục đang chọn bị xóa khi refresh. | UI báo mục đổi và không gửi action cho mục kế bên. |
| P08-TC05a | Nhập / f e p r, Enter và paste trong editor hoặc ô tìm. | Chữ vào đúng ô, task nền không pause/resume và route giữ nguyên. |
| P08-TC05b | Giữ Enter hoặc Space trong hộp xác nhận. | Một lần kích hoạt theo contract input đã duyệt, batch/task không tạo lặp. |
| P08-TC05c | Chỉ dùng Tab, mũi tên, F1, Esc và Enter ở các control. | Đi được đủ vùng và quay lại đúng focus. |
| P08-TC05d | Ctrl+S với revision nền đã đổi. | Đi qua đường báo xung đột, nháp vẫn còn. |
| P08-TC06a | Resize 80×24 → 119×24 → 120×24 → 80×24 khi đang xem evidence. | Bố cục đúng, cùng issue/focus và mốc đọc. |
| P08-TC06b | Giảm dưới ngưỡng rồi tăng lại khi có nháp. | Nháp, base revision và mục chọn giữ nguyên, control không đủ chỗ bị khóa an toàn. |
| P08-TC06c | Text Nhật, tiếng Việt có dấu tổ hợp và emoji dài. | Không cắt hỏng cụm ký tự hoặc đè vào label/control. |
| P08-TC06d | Xem task B trong khi A chạy, cửa sổ hẹp. | Global status vẫn chỉ đúng task/role/model thực đang chạy. |
| P08-TC07a | Ma trận lệnh hợp lệ gồm tạo/mở, nhập/scope, dịch/Review/terms, task, xuất và kết nối. | Help và parser công bố đúng lệnh/cờ đã hỗ trợ. |
| P08-TC07b | Truyền chapter cùng to-end, hai kiểu hạn mức, hoặc workspace rỗng. | Exit 2 trước terminal/workspace IO, không tạo task. |
| P08-TC07c | Giả lập mọi lớp kết quả 0, 2–7 và status đọc task paused. | Mã thoát đúng bảng, status đọc thành công exit 0 với state paused. |
| P08-TC07d | Gửi --json khi lỗi với đường dẫn Unicode và lỗi provider có sentinel. | Một envelope đầy đủ, stdout không ANSI/log, stderr đã che. |
| P08-TC08a | Pipe stdin bỏ trống khi đang chờ duyệt thuật ngữ. | Kết thúc hữu hạn code 4, có task ID và bước cần làm. |
| P08-TC08b | Khởi động lại rồi resume task đang xếp hàng hoặc hết ngân sách. | Giữ checkpoint/counter, trả state và mã đúng, không tăng hạn mức. |
| P08-TC08c | Thiếu credential hoặc lượt còn chờ Skill publication. | Không chờ nhập hay tự kích hoạt connection, trả nguyên nhân theo core. |
| P08-TC08d | Lượt chặn Q05 vì block không cắt an toàn. | Exit 7 cùng Chương, Block ID và hành động lập lại lượt. |
| P08-TC09a | Cùng scope/translate trên fixture tương đương. | Task plan, constraints, checkpoint và tác động ledger tương đương. |
| P08-TC09b | Cùng tập ID/version bị conflict khi xác nhận nhóm. | Cả hai từ chối nguyên batch, dữ liệu active giữ nguyên. |
| P08-TC09c | Export sẵn sàng và export bị chặn vì Review cũ. | Cùng gate/bundle/proof, lỗi có cùng target và việc tiếp theo. |
| P08-TC10a | Mở một issue giữa danh sách đã lọc, sửa rồi quay lại. | Trở về đúng thẻ/focus nếu còn, có nhãn revision mới cần Review. |
| P08-TC10b | Revision nền đổi từ tiến trình khác trong lúc viết nháp. | Lưu bị chặn do conflict, nháp và source/base vẫn truy được. |
| P08-TC10c | Issue được thay khi Editor đang mở. | Back báo mục đã đổi, không tự chọn và sửa issue khác. |
| P08-TC10d | Resize trong Editor và thử Esc khi nháp chưa lưu. | Giữ bản đang soạn và cho người dùng chọn hành động. |
| P08-TC11a | Xem task B trong khi A giữ permit, chọn resume B. | UI hiển thị A active và B queued, không có lượt vượt permit. |
| P08-TC11b | Chọn 9/10 mục, có conflict khi danh sách refresh. | Selection được bảo toàn, không xác nhận ngoài batch đã chọn. |
| P08-TC11c | Duyệt xong khi task paused và Preview cũ còn mở. | Task vẫn paused, readiness/Preview phản ánh căn cứ đã đổi. |
| P08-TC12a | Dùng bàn phím đi qua đủ tám màn ở 80×24 và bố cục rộng. | Đủ thao tác chính, quay lại đúng ngữ cảnh và thấy global status. |
| P08-TC12b | Chạy toàn bảng lệnh và trạng thái chờ qua subprocess. | Help khớp capability, exit/envelope/checkpoint đúng kết quả. |
| P08-TC12c | Thoát thường và đi qua lỗi có kiểm soát khi TUI đang mở. | Terminal trả lại trạng thái và diagnostic hiển thị đúng. |
| P08-TC12d | Thực hiện checklist mục 16.4 trên terminal Windows đã ghi phiên bản. | Bằng chứng thủ công gắn exact candidate, Linux native được ghi hoãn. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md): A thực hiện task, kiểm thử và tự review rồi bàn giao **Chờ review**; B đối chiếu yêu cầu, kiểm bản bàn giao và tự chạy kiểm chứng phù hợp trong phiên riêng. B ghi báo cáo độc lập, không sửa source/test chính thức hoặc trạng thái PLAN. A xử lý findings; B kiểm tra lại trên candidate mới. Quy tắc này áp dụng cả task tài liệu/ADR/thử nghiệm của phase.

- Screen có chứa logic approve/budget/permit thay vì gọi core?
- State key/return route có gắn đúng workspace/volume/document/revision?
- Một event có bị xử lý ở modal và màn sau cùng lúc?
- Exit code/JSON có đúng trạng thái thực tế đã lưu, đặc biệt status và pending operations?
- Footer/help/disabled controls có phản ánh hành động thật và thiếu dữ liệu?

Rà thêm diff ngoài phạm vi, API công khai không có consumer, dependency cấm, error bị nuốt và test không được phát hiện/chạy. Không xóa hoặc nới test chỉ để qua kiểm tra. Sửa lỗi ở owner của quy tắc thay vì thêm ngoại lệ trong UI.

## 15. Điều kiện đóng phase

- [ ] Mọi task/nhóm trong phạm vi cổng có review B **Đạt** còn hiệu lực; đủ AC, kiểm tra bắt buộc và không còn finding chặn. A không tự duyệt phần mình triển khai.
- [ ] B đã review bản tích hợp cuối theo [mẫu review phase](../templates/PHASE_REVIEW.md); candidate và căn cứ khớp, bằng chứng các task còn áp dụng hoặc đã được kiểm lại theo ảnh hưởng.

- [ ] Coverage matrix đủ tám màn, các commands/flags và states bắt buộc.
- [ ] Keyboard, 80/120/dưới ngưỡng, Unicode, dirty draft và global active model được kiểm chứng.
- [ ] CLI/TUI parity, non-interactive và migration/read-model compatibility đạt; README/help cập nhật.
- [ ] Tất cả task và AC của phase có bằng chứng, điểm chặn đã được giải quyết; ca chưa kiểm tra được ghi đúng trạng thái.
- [ ] V-00 áp dụng cho thay đổi Rust đã đạt; checks riêng trong mục 13 và các kiểm tra Go bị ảnh hưởng đã có kết quả.
- [ ] README được cập nhật nếu command/layout/nền tảng thay đổi; tài liệu nội bộ, key, EPUB, workspace và đầu ra tiếp tục ngoài Git.
- [ ] Bàn giao mục 16 đủ cho phase sau; không tự xóa đường Go hoặc công bố feature parity.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md); B dùng [mẫu review task](../templates/TASK_REVIEW.md), và [mẫu review phase](../templates/PHASE_REVIEW.md) khi nghiệm thu cổng. Báo cáo nội bộ đặt theo lượt trong `docs/roadmap/evidence/phase-08/`: `P08-TYY.handoff-r01.md` và `P08-TYY.review-r01.md`; giữ lượt trước khi sửa. Hồ sơ cổng dùng ID M0-01/M0-02/M0 hoặc M1…M9 theo phạm vi. Các mẫu chưa phải bằng chứng thực thi.

Sổ task/cổng khi thực hiện ghi trạng thái, candidate, link báo cáo A, review B, finding còn mở và bước tiếp theo. A phase cập nhật sổ này và gửi điều phối cập nhật PLAN trung tâm sau bàn giao/kết luận; B chỉ ghi báo cáo review. Thiếu review B hoặc kiểm tra bắt buộc thì chưa đánh dấu hoàn thành.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 8 checkpoint của lịch v2: [P08-CP01](#p08-cp01), [P08-CP02](#p08-cp02), [P08-CP03](#p08-cp03), [P08-CP04](#p08-cp04), [P08-CP05](#p08-cp05), [P08-CP06](#p08-cp06), [P08-CP07](#p08-cp07), [P08-CP08](#p08-cp08); xem [§10.2](#p08-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task | Trạng thái | Bằng chứng |
|---|---|---|---|
| P08-A | T01–T03 | Chưa bắt đầu | Chưa có |
| P08-B | T04–T06 | Chưa bắt đầu | Chưa có |
| P08-C | T07–T09 | Chưa bắt đầu | Chưa có |
| P08-D | T10–T11 | Chưa bắt đầu | Chưa có |
| P08-E | T12 | Chưa bắt đầu | Chưa có |

### 16.2. Báo cáo mỗi task

Ghi task/AC, thay đổi hành vi, file/module, quyết định và tác động migration/compatibility; lệnh đã chạy, toolchain/OS, tên/số ca liên quan, kết quả; test thủ công và ca chưa chạy; rủi ro còn lại và task có thể nhận tiếp. Nếu lưu log/báo cáo, dùng `docs/roadmap/evidence/phase-08/`, đã redaction, không chứa dữ liệu cá nhân hoặc nội dung sách thật.

### 16.3. Đầu ra cho bước tiếp theo

[Phase 09](PHASE-09.md) nhận ma trận screen/command/platform, subprocess/UI acceptance suites, known limitations và kịch bản demo thực tế. Phase 08 không thay release criteria hoặc kết quả đánh giá chất lượng model bằng việc UI tests xanh.

Trạng thái kế hoạch không phải trạng thái implementation. Chỉ cập nhật hoàn thành khi có bằng chứng của hành vi thực tế.

<a id="p08-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử candidate **M8**, tập trung vào thao tác qua nhiều màn, giữ trạng thái giao diện và kết quả nhất quán giữa TUI và CLI. A ghi candidate, terminal, bản chạy và cách dựng mẫu. B làm thử cùng phiên bản với cả TUI lẫn CLI.

#### 16.4.2. Chuẩn bị

Dành khoảng 15–20 phút. Dùng hai workspace mẫu khác tên, một thẻ Review có thể mở Editor và một tác vụ ở điểm chờ đã biết. Dùng hồ sơ thử có model giả lập. Chuẩn bị lệnh có tham số xung đột do gói bàn giao cung cấp và bộ dữ liệu đủ để chạy kiểm tra xuất.

#### 16.4.3. Mở đúng bản

Mở bản Rust M8 theo lệnh bàn giao. Dùng TUI với workspace mẫu A và giữ một PowerShell khác trỏ đến cùng executable cho các bước CLI. Đối chiếu tên bộ trên **Tổng quan** trước khi thao tác.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P08-UT01 — Từ Tổng quan tới đúng chỗ chờ | Chọn lối mở tác vụ đang chờ trong **Tổng quan**, sau đó mở help tại màn đích. | Đúng tác vụ và điểm chờ được chọn. Help mô tả các phím đang dùng được tại đó. |
| P08-UT02 — Giữ nháp khi chuyển màn | Từ thẻ **Reviewer** mở **Editor**, sửa thử một dòng chưa lưu, đổi kích thước 80×24 sang 120×24 rồi quay lại thẻ. Mở lại Editor. | Quay về đúng thẻ. Mở lại Editor thấy nháp đúng đoạn, giữ focus hoặc mục chọn theo quy tắc của màn. |
| P08-UT03 — Tách trạng thái hai workspace | Chọn một mục và vị trí cuộn trong A. Chuyển sang B rồi trở lại A. | B hiện dữ liệu của B. Khi về A, vị trí đang xem của A được khôi phục. |
| P08-UT04 — TUI và CLI đọc cùng kết quả | Ghi ID và trạng thái tác vụ từ TUI. Chạy `status --task <id> --json --non-interactive` với workspace tương ứng theo lệnh bàn giao. Đối chiếu thêm kết quả `check --volume <id>` với màn xuất. | Lệnh đọc trạng thái trả mã 0 và một JSON có đúng ID/trạng thái. Kết quả kiểm tra sẵn sàng xuất nêu cùng điều kiện với TUI. |

#### 16.4.5. Thử ca lỗi và mở lại

**P08-UT05 — Cờ CLI xung đột và đọc lại trạng thái.** Chạy lệnh mẫu có đồng thời `--cost-limit` và `--unlimited-cost` trong chế độ không tương tác. App báo lỗi tham số, trả mã 2 và kết thúc ngay. Mở lại cùng workspace và đối chiếu danh sách tác vụ trước/sau. Danh sách và trạng thái đã lưu giữ nguyên.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Khi sai, ghi candidate M8, tên terminal, kích thước, workspace, màn/phím hoặc lệnh, mã thoát và kết quả mong đợi/thực tế.

#### 16.4.7. Thử lại từ đầu

Tạo bản sao mới của hai workspace, nạp điểm chờ mẫu theo lệnh bàn giao và đưa terminal về kích thước ban đầu.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-08. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P08 rà **CLI/TUI Go và lệnh/script gọi giao diện cũ** tại các mốc **P08-CP05/CP06/CP07; kiểm đủ tám màn/lệnh tại M8**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Kiểm route, lệnh/option, exit/output, terminal và hướng dẫn chuyển sang Rust. Parity giữa CLI và TUI Rust không tự chứng minh parity Go–Rust; phải đối chiếu từng khả năng được cam kết thay. Giữ entrypoint/CI Go nếu còn khả năng chưa chuyển.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
