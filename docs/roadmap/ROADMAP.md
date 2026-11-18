# ROADMAP — KẾ HOẠCH TRIỂN KHAI RUST

**Phiên bản:** 2.0 · **Ngày:** 12/11/2026  
**Trạng thái:** M0-r03 và M1-r01 đã B Đạt trên Windows; PR Phase 01 đã merge vào rewrite/rust. Phase 02–09 chưa triển khai; [kế hoạch song song v2 và dọn Go đã B r04 Đạt](evidence/planning/P02-P09-parallel-v2.review-r04.md). Bước nhận việc theo PLAN.
**Căn cứ:** [36 quyết định](../spec/DESIGN_DECISIONS_36.md), [thiết kế Bước 9](../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) và [quy tắc repo](../../AGENTS.md).

Theo yêu cầu của người dùng, kế hoạch được lập từ đặc tả và thiết kế; code hiện tại không được dùng làm chuẩn, đầu vào tái sử dụng hoặc căn cứ đánh dấu hoàn thành. Các tài liệu này nằm trong `docs/`, là tài liệu nội bộ được loại khỏi Git.

## 1. Cách dùng bộ kế hoạch

- Bắt đầu tại [PLAN.md](../../PLAN.md) để nhận task hiện hành; [AGENTS.md](../../AGENTS.md) quy định cách thực thi, kiểm chứng và cập nhật tiến độ.
- Mỗi phase có 16 mục: thông tin; mục tiêu/demo; phạm vi; điều kiện bắt đầu; nguồn/bất biến; ADR; thiết kế; dữ liệu/lỗi/recovery; TUI/CLI; gói việc; task; AC; ma trận test; review code; exit gate; bàn giao.
- Có **10 phase, 136 task** trong bản kế hoạch ban đầu. Đây là phân rã để giao việc và rà soát; chi tiết task tương lai có thể được điều chỉnh theo bằng chứng của phase trước.
- Trước mỗi task, đọc đầu ra dependency đã công bố theo [lịch song song](PARALLEL_EXECUTION.md) và [graph task](PARALLEL_TASKS.json), giải quyết quyết định thực sự chặn task và điều chỉnh chi tiết triển khai nếu cần. Yêu cầu đã chốt không đổi ngầm.
- Mỗi chat nhận một phase; từng lượt một task hoặc nhóm 2–3 task gắn chặt được điều phối ghi trước trong PLAN. Mỗi checkpoint có 1–3 task; commit/push ngay khi đủ checks và B review bản ghép Đạt. Theo [quy trình hai model](REVIEW_WORKFLOW.md), A triển khai/test/tự review; B review độc lập; A sửa và B kiểm tra lại. Chưa có B Đạt và đủ bằng chứng thì chưa đóng task/nhóm hoặc chuyển phần phụ thuộc ngoài nhóm.
- Tên module/package/test/command kiểm chứng là mục tiêu cần tạo khi triển khai. Lệnh exit 0 với 0 ca liên quan không được ghi đạt.
- Tiến độ cập nhật bằng bằng chứng. Kế hoạch viết xong không đồng nghĩa ADR đã chốt hoặc sản phẩm đã có chức năng.

## 2. Danh mục phase và milestone

| Phase | Mục tiêu | Cổng nghiệm thu | Đầu vào | UI | Task |
|---|---|---|---|---|---:|
| [00](phases/PHASE-00.md) | Quyết định kiến trúc | M0-01 cho nền tảng; M0-02 cho storage/import | Không có | Contract UI-01 | 10 |
| [01](phases/PHASE-01.md) | Nền tảng Rust/CLI/TUI | Build/test/lint, boundaries và shell dùng được | M0-01 | UI-01 | 14 |
| [02](phases/PHASE-02.md) | Workspace và EPUB | Import → commit → reopen đúng source/mapping | M1 + M0-02 | UI-02/03 | 13 |
| [03](phases/PHASE-03.md) | Revision và context | Revision/Skill/dependency, sửa tay, approval và reuse có recovery | M2 | UI-04; terms CLI | 15 |
| [04](phases/PHASE-04.md) | Dịch–Review giả lập | Nhiều lượt, source-only, chapter gate, checkpoint tối thiểu | M3 | UI-05 | 13 |
| [05](phases/PHASE-05.md) | Model và Coordinator | Attempt/tiền/quyền chạy/recovery an toàn | M4 + ADR money/IPC | UI-06/07 | 16 |
| [06](phases/PHASE-06.md) | Workflow đầy đủ | Selected/to-end, Research/Repair/approval/exception | M5 | UI-08/09/10 | 16 |
| [07](phases/PHASE-07.md) | Ảnh/Preview/EPUB | Bundle và EPUB 3.3, publish không ghi đè | M6 | UI-11/12/13 | 15 |
| [08](phases/PHASE-08.md) | TUI/CLI hoàn chỉnh | Tám màn và commands cùng core, keyboard/resize/non-interactive | M2–M7 | UI-14/15; tích hợp UI trước | 12 |
| [09](phases/PHASE-09.md) | Nghiệm thu/release | Recovery/compatibility/security/quality và artifact có bằng chứng | M1–M8 | Nghiệm thu toàn bộ | 12 |

**Phase 00/01 đã Đạt trong phạm vi Windows; Phase 02–09 chưa bắt đầu.** Bằng chứng chi tiết ở mục 16 của từng file.

## 3. Thứ tự nghiệm thu và các điểm bàn giao

```text
Phase 00: M0-01 → Phase 01
          M0-02 + M1 → Phase 02
Nghiệm thu đầy đủ: M2 → M3 → M4 → M5 → M6 → M7 → M8 → M9
Code song song: mở từng task qua checkpoint trong PARALLEL_TASKS.json
```

Đầu vào cột milestone ở §2 là nền hoàn chỉnh để nghiệm thu; điều kiện bắt đầu task sớm được ghi tại §4/§11 từng phase và graph v2. Có 64 checkpoint, mỗi checkpoint 1–3 task; không chờ hết phase mới push. Các chat dùng worktree riêng, một điều phối PLAN/shared files và B gpt-6.1-sol/max riêng theo [quy trình](PARALLEL_EXECUTION.md).

M0-01 chốt đủ nền tảng, CLI/terminal/API và chiến lược test. M0-02 chốt storage, identity/offset và generation/migration. Phase 01 có thể bắt đầu khi M0-01 đủ; không cần đợi quyết định IPC/export của các phase sau.

| Bên giao → bên nhận | Contract/đầu ra cần nhận | Điều không được suy diễn |
|---|---|---|
| P00 → P01 | [ADR-001](../decisions/ADR-001.md) và phần nền tảng [ADR-008](../decisions/ADR-008.md), kèm bằng chứng M0-01 | Có bản ghi hướng lựa chọn nghĩa là phiên bản/API và thử nghiệm đã hoàn tất |
| P00/P01 → P02 | Workspace boundaries, error/UI harness, ADR storage/source/commit | DDL nháp là migration production |
| P02 → P03 | Snapshot/Block/Range/mapping, immutable store, writer/CAS | Đổi mapping cho phép sửa identity nguồn hoặc mất history |
| P03 → P04 | Revision/approval/context/dependency, file sync và reuse có provenance | Candidate hoặc ví dụ là official rule |
| P04 → P05 | Gateway/Review/workflow và checkpoint tối thiểu | Fake restart đã chứng minh send gap/uncertain usage/IPC |
| P05 → P06 | Executor/attempt/tiền/permit/recovery | Research/repair được mở transport hoặc tự cấp quyền riêng |
| P06 → P07 | Exact revision/Review sets, exceptions, coverage và content mode | Thân truyện đạt là đủ xuất toàn bộ sách |
| P07 → P08 | Bundle/Preview/export history và các use case UI | UI có thể tự bỏ readiness/safety gate |
| P02–P08 → P09 | Regression suites, fixtures, protocol versions, known limitations | CI fake xanh chứng minh chất lượng model thật hoặc feature parity |

Phase 08 tích hợp UI đã làm theo tính năng. Kiểm thử hệ thống ở Phase 09 không thay kiểm thử trong từng task/phase.

## 4. Thời điểm giải quyết quyết định kỹ thuật

Các bản ghi ADR và prototype chỉ chốt phần được ghi rõ trong phạm vi của chúng. “Đã chọn” dưới đây không thay kiểm tra consumer; “mở” và “hoãn có điều kiện” vẫn là trạng thái chưa quyết định, không phải quyết định đã thông qua. Mọi hạn đều là **trước task/consumer đầu tiên**; các mục tương lai không chặn M0-01/M0-02 khi chưa được dùng.

| Hồ sơ | Nội dung | Chậm nhất trước |
|---|---|---|
| [ADR-001](../decisions/ADR-001.md) | Toolchain/MSRV/framework/dependency/features/platform; review Đạt trong phạm vi đã ghi | P01 scaffold và consumer thư viện tương ứng |
| [ADR-002](../decisions/ADR-002.md) | SQLite driver/worker/pragmas; B r04 Đạt trên spike Windows, chưa phải storage adapter production | P02-T01/T03 |
| [ADR-003](../decisions/ADR-003.md) | BlockID/hash/source-range baseline; B r04 Đạt, P02 parser/mapping integration còn phải kiểm | P02-T01/T06; mở rộng reuse tại P03-T14 |
| [ADR-004](../decisions/ADR-004.md) | Bootstrap/generation/CAS/file-reference/schema migration baseline; B r04 Đạt, durability/updater còn mở | P02-T01/T03; mở rộng revision tại P03-T01/T02; updater tại P09-T05 |
| Các ID TD/DB/W7 | Trạng thái từng ID, owner nhận và hạn cụ thể | Bảng truy vết đầy đủ ngay dưới đây |

<a id="decision-schedule"></a>

### 4.1. Lịch đầy đủ TD/DB/W7

Cột owner/hạn giữ mốc consumer được giao từ M0 để truy vết. Các consumer P01 đã được kiểm trong [M1-r01 trên Windows](evidence/phase-01/M1.review-r01.md); không mở lại việc P01 đã Đạt chỉ vì bảng còn ghi hạn trước P01. Consumer mới ở P02–P09 vẫn phải kiểm riêng, và những quyết định ghi mở/hoãn vẫn chưa được chốt.

| ID | Trạng thái hiện tại, trong phạm vi nào | Owner nhận / hạn chốt | Tác động tới cổng sớm |
|---|---|---|---|
| TD-01 | Đã chọn Rust Stable 1.99.1, pin, MSRV 1.88, Edition 2024 cho Windows; Linux native hoãn theo người dùng | P01-T02; P01-T13 trước khi nhận lại hỗ trợ Linux | Không chặn M0-01 Windows; Linux chưa được tuyên bố hỗ trợ |
| TD-02 | Mở: Tokio chưa được chọn vì Phase 01 chưa có async consumer | P05-T02 trước khi chọn async provider transport/runtime | Không chặn P01; provider gateway chưa bắt đầu |
| TD-03 | Đã chọn Ratatui/Crossterm theo version/features ADR-001 | P01-T06 trước khi nối event/terminal adapter thật | M1 đã Đạt trên Windows: implementation và terminal thật đã được B kiểm; xem [M1 §3.2](evidence/phase-01/M1.review-r01.md). UI nghiệp vụ ở phase sau kiểm riêng |
| TD-04 | Đã chọn Clap cho parser tại app boundary | P01-T04 trước parser sản phẩm | Không chặn M0-01 ngoài contract đã review |
| TD-05 | Spike chọn Rusqlite + worker được B Đạt trên Windows; production adapter/chính sách concurrency chưa được nghiệm thu | P02-T01/T03 trước schema/write adapter; prototype worker/recovery tại đây | Chỉ là baseline cho M0-02; P02 chưa bắt đầu |
| TD-06 | Đã chọn Serde/serde_json ở DTO boundary, không serialize Domain mặc định | P01-T04 trước CLI JSON DTO | Không chặn M0-01 ngoài contract đã review |
| TD-07 | Đã chọn thiserror cho lỗi có kiểu và anyhow tại binary boundary | P01-T05 trước error/diagnostic consumer | Không chặn M0-01 ngoài contract đã review |
| TD-08 | Mở: reqwest chưa được chọn cho HTTP/provider | P05-T02 trước provider transport; ghi version/features và test adapter | Không chặn P01/P02 |
| TD-09 | Đã chọn tracing/subscriber căn bản; sink, filter và redaction phải được kiểm tại consumer | P01-T05/T06 trước diagnostic sink TUI/CLI | M1 đã kiểm diagnostic/redaction/sentinel trên Windows; xem [review M1](evidence/phase-01/M1.review-r01.md). Consumer mới phải kiểm lại phần ảnh hưởng |
| TD-10 | Mở: Windows Named Pipes chỉ là đề xuất, chưa có transport được duyệt | P05-T09 trước IPC contract/prototype; P05-T16 kiểm đa tiến trình/recovery trước đóng M5 | Không chặn P01–P04 |
| TD-11 | Mở: Coordinator process riêng/on-demand chỉ là đề xuất | P05-T09 trước khi chọn process lifecycle/lock/queue; P05-T16 kiểm recovery | Không chặn P01–P04 |
| TD-12 | Đã chốt nguyên tắc Application ports, adapter implementations theo ADR-008 | P01-T03 kiểm graph/API package thực | M1 đã kiểm architecture_boundaries 10/10 trên graph sản phẩm; xem [M1 §3.1](evidence/phase-01/M1.review-r01.md). Kiểm lại khi thêm consumer |
| TD-13 | Đã chốt Domain không nhận async/framework/IO theo ADR-001/008 | P01-T03 kiểm dependency graph; kiểm lại khi thêm consumer P05 | Không chặn M0-01 |
| TD-14 | Đã chốt typed contracts/errors và ranh giới P01; version compatibility cụ thể vẫn thuộc TD-15 | P01-T03/T04 trước public API/CLI contract | Không chặn M0-01 ngoài contract đã review |
| TD-15 | Mở theo từng boundary: không có một version scheme chung đã được chốt | P01-T04 trước JSON schema evolution; P02-T01/T03 trước persisted schema; P05-T09 trước IPC | Không chặn phase trước consumer; mỗi boundary phải có version check/migration phù hợp |
| TD-16 | Chưa có nhu cầu plugin runtime; nguyên tắc không tạo framework sớm chưa phải cam kết tính năng v1 | P03-T05 chỉ khi scope Skill cần extension runtime; quyết định trước thiết kế/triển khai extension; nếu không phát sinh thì hoãn | Không chặn P03 Skill cơ bản hoặc M0 |
| DB-01 | ADR-003 chốt BlockID; format/canonical ID cho entity DB khác còn mở | P02-T01 trước migration đầu | Chặn schema migration P02 cho tới khi được chốt; không chặn M0 |
| DB-02 | Mở: precision, UTC storage và timezone conversion | P02-T01 trước migration timestamp đầu | Chặn schema migration P02 cho tới khi được chốt |
| DB-03 | ADR-003 chốt SHA-256 encoding cho BlockID vector; storage hash format/API tổng quát còn mở | P02-T01/T02 trước schema/object-store adapter | Chặn schema/object consumer P02 cho tới khi được chốt |
| DB-04 | Mở: nội dung revision lưu file, SQLite hay hybrid | P03-T01 trước revision persistence; P03-T02 thực hiện outbox/projection theo lựa chọn | Không chặn P02/M0-02 |
| DB-05 | Contract canonical path/half-open source range đã B Đạt ở ADR-003; parser integration chưa làm | P02-T06 trước source mapping implementation/test | Không chặn M0-02; P02 mapping chưa bắt đầu |
| DB-06 | Mở: slot boundary và exact coverage validation | P04-T02 trước lập lượt Translation/Review | Không chặn P01–P03 |
| DB-07 | Mở: composite FK/triggers và cùng snapshot/volume integrity trong schema production | P02-T01/T03 trước migration/write path | Chặn schema/write path P02 cho tới khi được chốt |
| DB-08 | Bootstrap/generation/CAS/replay baseline đã B Đạt ở ADR-004; production schema/write implementation còn thiếu | P02-T01/T03 trước write path | Không chặn M0-02; P02 phải hiện thực và kiểm lại |
| DB-09 | Mở: Decimal precision, currencies và FX snapshot | P05-T04/T05 trước cost ledger, budget hoặc reservation | Không chặn P01–P04 |
| DB-10 | Mở: Coordinator handoff/reconciliation/fencing protocol | P05-T09/T11 trước multiprocess dispatch/handoff; P05-T16 fault campaign | Không chặn P01–P04 |
| DB-11 | Mở: retention/redaction cho raw source, Research và model material | P06-T10 trước Research cache; P09-T06 trước release/security sign-off | Không chặn P01–P05 |
| DB-12 | ADR-004 chốt backup/rollback cơ bản cho schema staging; đổi đồng bộ DB+object và updater Windows còn mở | P02-T03 trước migration write; P09-T05 trước release updater | Không chặn P01; không tuyên bố updater đã an toàn |
| DB-13 | SQLite pragma/worker baseline được B Đạt trên spike Windows; production settings còn phải kiểm | P02-T01/T03 trước storage adapter | Không chặn M0-02; P02 cần kiểm trên DB adapter |
| DB-14 | Mở: index tuning theo workload đo được | P09-T07 trước performance acceptance | Không chặn phase trước khi có workload/benchmark |
| DB-15 | Mở: export version reservation/publish protocol | P07-T08/T09 trước reserve/publish; P09-T04 kiểm crash recovery | Không chặn P01–P06 |
| DB-16 | Mở: Skill approval batch đồng bộ file Markdown bên ngoài atomically | P03-T09/T10 trước approval/publish | Không chặn P01/P02 |
| DB-17 | Mở: integrity policy cho generic entity-kind/id relationships | P03-T04 trước dependency engine/schema | Không chặn P01/P02 |
| DB-18 | ADR-004 ghi nguyên tắc vertical slice/minimal baseline; số nhóm migration/schema production còn mở | P02-T01/T03 trước scaffold/schema implementation | Chặn schema scaffold P02 cho tới khi slice đầu được xác định |
| W7-01 | Mở: cách biểu diễn workflow step | P04-T08 trước full workflow state implementation | Không chặn P01–P03 |
| W7-02 | Mở: checkpoint version và xử lý checkpoint cũ | P04-T10 trước persisted checkpoint schema/reopen | Không chặn P01–P03 |
| W7-03 | Mở: checkpoint granularity/frequency | P04-T10 trước checkpoint persistence | Không chặn P01–P03 |
| W7-04 | Mở: intent-to-send attempt dispatch protocol | P05-T06 trước gửi request provider thật | Không chặn fake-provider P04 |
| W7-05 | Mở: late response fencing và điều kiện publish | P05-T08 trước publish attempt result | Không chặn P01–P04 |
| W7-06 | Mở: Coordinator handoff intent/ack/release | P05-T11 trước multiprocess handoff | Không chặn P01–P04 |
| W7-07 | Mở: quyền tiếp quản/reconciliation sau Coordinator crash | P05-T11/T12 trước recovery resume; fault test tại P05-T16 | Không chặn P01–P04 |
| W7-08 | Mở: retry classification cho transient/uncertain/fatal | P05-T07 trước retry provider | Không chặn P01–P04 |
| W7-09 | Mở: cancellation ở tool/transport/provider boundary | P05-T07/T08 trước timeout/cancel transport | Không chặn P01–P04 |
| W7-10 | Mở: repair plan/round/attempt state; không được coi là đã chốt từ state diagram | P06-T04 trước M1/M2 repair implementation | Không chặn P01–P05 |
| W7-11 | Mở: dependency revalidation time/scope | P06-T02 trước context preflight; P06-T12 trước propagation/resume | Không chặn P01–P05 |
| W7-12 | Mở: event schema version/payload/retention cho propagation | P06-T12 trước durable event propagation; đối chiếu DB-11 retention | Không chặn P01–P05 |
| W7-13 | Mở: policy khi workflow recovery thất bại | P06-T12 trước recovery/resume policy | Không chặn P01–P05 |
| W7-14 | Mở: fault/crash/deterministic-time testing strategy | P04-T01 trước workflow test harness; P05-T16/P09-T04 mở rộng trước M5/release | Không chặn P01–P03 |

<a id="adr-followups"></a>

### 4.2. Mục mở trong ADR chưa có ID

| Mục mở / trạng thái | Owner nhận / hạn | Tác động |
|---|---|---|
| ADR-001: Linux native build/link/test được người dùng hoãn | P01-T13 trước khi nhận lại Linux support | Không chặn phạm vi Windows hiện tại; không tuyên bố Linux đã hỗ trợ |
| ADR-002: production storage adapter, cross-process workspace lease, queue back-pressure và benchmark tải thật chưa có evidence | P02-T01/T03 trước storage adapter/write path; P05-T09 trước multi-process lease; P05-T10 trước queue dispatch; P09-T07 trước performance acceptance | Các nguyên tắc worker của spike không tự nghiệm thu consumer production |
| ADR-003: parser version, node path ổn định, ruby/marker mapping và EPUB parser integration còn cần nghiệm thu | P02-T06 trước source mapping; P03-T14 trước rebase/reuse | Không tái dùng spike vector như parser acceptance |
| ADR-004: garbage collection/orphan inventory | P02-T02 trước khi thêm GC | Không chặn file store ban đầu; không xóa object trong publish path |
| ADR-004: generation graph/operation ledger đầy đủ | P02-T03 trước production write path; P05-T11 trước cross-process reconciliation | Baseline r04 chỉ chốt slice bootstrap/CAS/replay đã mô tả |
| ADR-004: Markdown outbox/projection | P03-T02 trước khi ghi projection ra file ngoài | Không chặn schema/import Phase 02 |
| ADR-004: power-loss/directory-entry durability và recovery | P09-T04 trước release trên filesystem/OS được hỗ trợ | Process-crash Windows spike không thay power-loss check |
| ADR-004: thay/restore đồng bộ DB và object, cleanup backup và updater | P09-T05 trước release updater | Không tuyên bố hai hệ thống lưu trữ atomic hoặc updater an toàn |
| ADR-005/006: IPC, Coordinator, provider attempt/retry/fencing và recovery chưa có quyết định đã chốt | P05-T06–T12 trước provider dispatch/attempt recovery; P05-T09/T11 trước IPC, Coordinator và handoff; P05-T16 trước đóng M5 | Thuộc các consumer Phase 05; prototype/recovery evidence phải có trước khi khóa quyết định tương ứng |
| ADR-007: workflow step và checkpoint tối thiểu còn mở | P04-T08 trước full workflow state; P04-T10 trước checkpoint schema/persist/reopen | Chặn consumer P04 tương ứng cho tới khi chốt; không chặn P01–P03 |
| ADR-007: phần mở rộng attempt/retry/fencing/recovery | P05-T06–T12 trước provider attempt, dispatch, retry, fencing hoặc recovery consumer; P05-T16 trước đóng M5 | Phần này không dời quyết định step/checkpoint nền tảng khỏi hạn P04 ở hàng trên |
| ADR-008: import diagnostics, partial/unsupported status, volume registration và ý nghĩa `import --volume` còn mở | P02-T05 trước inventory/unsupported diagnostics; P02-T09 trước partial publication và import diagnostics/idempotency; P02-T12 trước RegisterVolume và CLI/TUI `import --volume` | Mỗi contract phải được chốt và kiểm trước consumer tương ứng; không coi API import đã có trong ADR-008 Phase 01 |
| ADR-008: gateway DTO, request/response schema, Translation/Review evidence và error mapping còn mở | P04-T01 trước ModelGateway port/fake; P04-T02 trước planning request contract; P04-T03 trước response validation; P04-T04 trước candidate/Review eligibility và evidence | Mỗi contract phải được chốt và kiểm trước consumer tương ứng; DB-06 không thay contract gateway P04-T01 |

Đối chiếu ADR-008 §8 theo cùng quy tắc owner/hạn: persistence/source được phân tại ADR-002/003/004 và TD-05, DB-01–08/12/13/18; revision/Skill tại DB-04/16/17 và TD-16; workflow/checkpoint tại ADR-007 cùng W7-01–03; money/provider/IPC tại TD-02/08/10/11, DB-09/10, W7-04–09 và ADR-005/006; Research/repair tại DB-11, W7-10–13; preview/export tại DB-15. Các ID này vẫn mở theo đúng consumer ghi trong §4.1; không hàm ý ADR-008 đã chọn trước API/adapter tương lai. Không có framework/provider/IPC/export nào được coi là chọn sẵn từ danh mục, và các nhóm không bị gộp thành một protocol chung.

ADR-001/008 và các phần kỹ thuật P00-T03–T05 được B review Đạt cho phạm vi P01; ADR-002/003/004 chứa baseline spike T06–T08 được B review Đạt trên r04 trong phạm vi Windows. Các owner/hạn mới ở §4.1–4.2 là công việc T09, không mở lại hành vi kỹ thuật đã review. M0-01-r01 và M0-02-r01 đã được review tích hợp Đạt riêng; dossier T10 của mỗi cổng giữ phạm vi tương ứng.

Đã có [ADR-001](../decisions/ADR-001.md), [ADR-008](../decisions/ADR-008.md) và ADR-002–004. ADR-001/008 và T03–T05 được B review Đạt trong phạm vi báo cáo. Review kỹ thuật r03 ghi F04/F05 Cần sửa; A xử lý ở r04. B độc lập r04 xác nhận candidate **Đạt** cho AC06–08, 32/32 ca mỗi toolchain và manifest 120/120; F01–F05 đóng. M0-01-r01 và M0-02-r01 đều đã B review tích hợp Đạt; M1 vẫn là dependency trước Phase 02. Các dòng được ghi “mở/hoãn” phía trên không được xem là đã chốt chỉ vì đã có owner.

## 5. Truy vết 36 quyết định

Bảng xác định task chủ đạo; bất biến xuyên phase vẫn áp dụng ở mọi consumer. AC và fixture cụ thể nằm trong file phase. Cột "Nghiệm thu / kiểm tra" ghi cặp AC · TC tại phase chủ đạo đã được rà khớp tồn tại (khớp tiêu đề mục 12–13 của từng phase); PHASE-01 dùng `AC-xx · TC-xx`, các phase khác dùng `Pxx-ACxx · Pxx-TCxx`. Ký hiệu (a)–(h) xem "Ghi chú rà truy vết" cuối mục. Cập nhật lượt r02–r03 (07/11/2026): các hàng Q02, Q05, Q08, Q13, Q31, Q32, Q35 mở rộng theo finding F01–F07 của review B (r01: Cần sửa); r03 đóng thêm ranh giới người dùng của F01; xem [báo cáo r02](evidence/phase-00/P00-T01.handoff-r02.md) và [báo cáo r03](evidence/phase-00/P00-T01.handoff-r03.md).

| Quyết định | Chủ đề | Task chủ đạo | Nghiệm thu / kiểm tra |
|---|---|---|---|
| [Q01](../spec/DESIGN_DECISIONS_36.md#decision-01) | Nội dung dịch mặc định | [P02-T08](phases/PHASE-02.md#p02-t08), [P02-T12](phases/PHASE-02.md#p02-t12) (a), [P06-T01](phases/PHASE-06.md#p06-t01), [P06-T13](phases/PHASE-06.md#p06-t13) | P02-AC08 · TC08, P02-AC12 · TC12, P06-AC01 · TC01, P06-AC13 · TC13; chữ trong bitmap qua luồng ảnh [P07-T02](phases/PHASE-07.md#p07-t02)/[P07-T03](phases/PHASE-07.md#p07-t03) (P07-AC02/03 · TC02/03) (a) |
| [Q02](../spec/DESIGN_DECISIONS_36.md#decision-02) | Phân loại EPUB | [P02-T08](phases/PHASE-02.md#p02-t08), [P06-T09](phases/PHASE-06.md#p06-t09), [P06-T13](phases/PHASE-06.md#p06-t13) | P02-AC08 · TC08 (gồm tái dùng/đối chiếu lại theo hash EPUB + version mapping), P06-AC09 · TC09, P06-AC13 · TC13 |
| [Q03](../spec/DESIGN_DECISIONS_36.md#decision-03) | Chương và chỉnh mapping | [P02-T07](phases/PHASE-02.md#p02-t07), [P02-T10](phases/PHASE-02.md#p02-t10), [P03-T14](phases/PHASE-03.md#p03-t14) | P02-AC07 · TC07, P02-AC10 · TC10, P03-AC14 · TC14; điểm bàn giao P02→P03→P04/P06 xem (f) |
| [Q04](../spec/DESIGN_DECISIONS_36.md#decision-04) | BlockID/SourceRange và reuse | [P02-T06](phases/PHASE-02.md#p02-t06), [P03-T14](phases/PHASE-03.md#p03-t14), [P04-T02](phases/PHASE-04.md#p04-t02); nền [ADR-003](../decisions/ADR-003.md): [P00-T07](phases/PHASE-00.md#p00-t07) (g) | P02-AC06 · TC06, P03-AC14 · TC14, P04-AC02 · TC02; P00-AC07 · TC07 (nền) |
| [Q05](../spec/DESIGN_DECISIONS_36.md#decision-05) | Chia lượt theo token | [P04-T02](phases/PHASE-04.md#p04-t02), [P05-T03](phases/PHASE-05.md#p05-t03), [P05-T13](phases/PHASE-05.md#p05-t13) | P04-AC02 · TC02 (lõi: block vượt trần, chặn gửi, vị trí nguồn, re-plan), P05-AC03 · TC03, P05-AC13 · TC13 (UI-06: vị trí/nguyên nhân + chỉnh điểm chia/giới hạn → core); CLI: [P05-T15](phases/PHASE-05.md#p05-t15) (P05-AC15 · TC15: issue vị trí + next_action, exit 7); (b) |
| [Q06](../spec/DESIGN_DECISIONS_36.md#decision-06) | Workspace/schema/migration | [P02-T01](phases/PHASE-02.md#p02-t01), [P09-T05](phases/PHASE-09.md#p09-t05); nền storage [ADR-002](../decisions/ADR-002.md)/[ADR-004](../decisions/ADR-004.md) qua P00-T06/08 | P02-AC01 · TC01, P09-AC05 · TC05; P00-AC06/08 nền; ghi chú workspace Go cũ — (c) |
| [Q07](../spec/DESIGN_DECISIONS_36.md#decision-07) | Generation và atomic commit | [P02-T03](phases/PHASE-02.md#p02-t03), [P02-T09](phases/PHASE-02.md#p02-t09), [P03-T02](phases/PHASE-03.md#p03-t02); nền [ADR-004](../decisions/ADR-004.md): [P00-T08](phases/PHASE-00.md#p00-t08) | P02-AC03 · TC03, P02-AC09 · TC09, P03-AC02 · TC02; P00-AC08 (nền); điểm bàn giao P02→P03 — (d), (f) |
| [Q08](../spec/DESIGN_DECISIONS_36.md#decision-08) | Markdown và revision sửa tay | [P03-T03](phases/PHASE-03.md#p03-t03), [P03-T12](phases/PHASE-03.md#p03-t12) | P03-AC03 · TC03, P03-AC12 · TC12 (gồm gắn lại đơn vị nguồn khi marker thiếu/trùng) |
| [Q09](../spec/DESIGN_DECISIONS_36.md#decision-09) | Dependency/invalidation | [P03-T04](phases/PHASE-03.md#p03-t04), [P06-T12](phases/PHASE-06.md#p06-t12) | P03-AC04 · TC04, P06-AC12 · TC12 |
| [Q10](../spec/DESIGN_DECISIONS_36.md#decision-10) | Ba trục trạng thái | [P03-T01](phases/PHASE-03.md#p03-t01), [P08-T02](phases/PHASE-08.md#p08-t02) | P03-AC01 · TC01, P08-AC02 · TC02 |
| [Q11](../spec/DESIGN_DECISIONS_36.md#decision-11) | Mẫu Skill riêng | [P03-T05](phases/PHASE-03.md#p03-t05) | P03-AC05 · TC05 |
| [Q12](../spec/DESIGN_DECISIONS_36.md#decision-12) | Mandatory/example/reference | [P03-T05](phases/PHASE-03.md#p03-t05), [P03-T07](phases/PHASE-03.md#p03-t07) | P03-AC05 · TC05, P03-AC07 · TC07 |
| [Q13](../spec/DESIGN_DECISIONS_36.md#decision-13) | Skill external sync | [P03-T10](phases/PHASE-03.md#p03-t10), [P06-T02](phases/PHASE-06.md#p06-t02) | P03-AC10 · TC10, P06-AC02 · TC02 (P03 → P06; hash check trước lượt chưa gửi, lượt đã gửi giữ nền cũ) |
| [Q14](../spec/DESIGN_DECISIONS_36.md#decision-14) | Candidates và occurrences | [P03-T08](phases/PHASE-03.md#p03-t08), [P04-T04](phases/PHASE-04.md#p04-t04) | P03-AC08 · TC08, P04-AC04 · TC04 |
| [Q15](../spec/DESIGN_DECISIONS_36.md#decision-15) | Approval batch | [P03-T09](phases/PHASE-03.md#p03-t09), [P03-T10](phases/PHASE-03.md#p03-t10), [P06-T15](phases/PHASE-06.md#p06-t15) | P03-AC09 · TC09, P03-AC10 · TC10, P06-AC15 · TC15; wake-up sau publish đi qua [P06-T12](phases/PHASE-06.md#p06-t12) (P06-AC12 · TC12) — (f) |
| [Q16](../spec/DESIGN_DECISIONS_36.md#decision-16) | Xưng hô và hai mốc thời gian | [P03-T06](phases/PHASE-03.md#p03-t06), [P03-T07](phases/PHASE-03.md#p03-t07), [P06-T02](phases/PHASE-06.md#p06-t02) | P03-AC06 · TC06, P03-AC07 · TC07, P06-AC02 · TC02 |
| [Q17](../spec/DESIGN_DECISIONS_36.md#decision-17) | Model/provider/credentials | [P05-T01](phases/PHASE-05.md#p05-t01), [P05-T02](phases/PHASE-05.md#p05-t02) | P05-AC01 · TC01, P05-AC02 · TC02 |
| [Q18](../spec/DESIGN_DECISIONS_36.md#decision-18) | Tiền/usage/pricing/FX | [P05-T04](phases/PHASE-05.md#p05-t04), [P05-T05](phases/PHASE-05.md#p05-t05) | P05-AC04 · TC04, P05-AC05 · TC05 |
| [Q19](../spec/DESIGN_DECISIONS_36.md#decision-19) | Timeout/retry/late response | [P05-T06](phases/PHASE-05.md#p05-t06), [P05-T07](phases/PHASE-05.md#p05-t07), [P05-T08](phases/PHASE-05.md#p05-t08) | P05-AC06 · TC06, P05-AC07 · TC07, P05-AC08 · TC08 |
| [Q20](../spec/DESIGN_DECISIONS_36.md#decision-20) | JSON Dịch và validation | [P04-T03](phases/PHASE-04.md#p04-t03), [P04-T04](phases/PHASE-04.md#p04-t04), [P04-T05](phases/PHASE-04.md#p04-t05) | P04-AC03 · TC03, P04-AC04 · TC04, P04-AC05 · TC05 |
| [Q21](../spec/DESIGN_DECISIONS_36.md#decision-21) | Source-only Review | [P04-T06](phases/PHASE-04.md#p04-t06), [P06-T03](phases/PHASE-06.md#p06-t03) | P04-AC06 · TC06, P06-AC03 · TC03 |
| [Q22](../spec/DESIGN_DECISIONS_36.md#decision-22) | Evidence/coverage và kết luận | [P04-T07](phases/PHASE-04.md#p04-t07), [P06-T03](phases/PHASE-06.md#p06-t03) | P04-AC07 · TC07, P06-AC03 · TC03 |
| [Q23](../spec/DESIGN_DECISIONS_36.md#decision-23) | M1–M4 | [P06-T04](phases/PHASE-06.md#p06-t04), [P06-T05](phases/PHASE-06.md#p06-t05), [P06-T06](phases/PHASE-06.md#p06-t06) | P06-AC04 · TC04, P06-AC05 · TC05, P06-AC06 · TC06 |
| [Q24](../spec/DESIGN_DECISIONS_36.md#decision-24) | Review toàn chương | [P04-T09](phases/PHASE-04.md#p04-t09), [P06-T12](phases/PHASE-06.md#p06-t12) | P04-AC09 · TC09, P06-AC12 · TC12 |
| [Q25](../spec/DESIGN_DECISIONS_36.md#decision-25) | Repair rounds và ngoại lệ | [P06-T07](phases/PHASE-06.md#p06-t07), [P06-T08](phases/PHASE-06.md#p06-t08) | P06-AC07 · TC07, P06-AC08 · TC08 |
| [Q26](../spec/DESIGN_DECISIONS_36.md#decision-26) | Research search/read/limits/cache | [P06-T09](phases/PHASE-06.md#p06-t09), [P06-T10](phases/PHASE-06.md#p06-t10) | P06-AC09 · TC09, P06-AC10 · TC10 |
| [Q27](../spec/DESIGN_DECISIONS_36.md#decision-27) | Evidence và spoiler boundary | [P06-T11](phases/PHASE-06.md#p06-t11) | P06-AC11 · TC11 |
| [Q28](../spec/DESIGN_DECISIONS_36.md#decision-28) | Queue/permit/handoff | [P05-T09](phases/PHASE-05.md#p05-t09), [P05-T10](phases/PHASE-05.md#p05-t10), [P05-T11](phases/PHASE-05.md#p05-t11) | P05-AC09 · TC09, P05-AC10 · TC10, P05-AC11 · TC11 |
| [Q29](../spec/DESIGN_DECISIONS_36.md#decision-29) | Checkpoint/pause/resume/recovery | [P04-T10](phases/PHASE-04.md#p04-t10), [P05-T12](phases/PHASE-05.md#p05-t12), [P06-T12](phases/PHASE-06.md#p06-t12) | P04-AC10 · TC10, P05-AC12 · TC12, P06-AC12 · TC12; chuỗi minimal→đầy đủ xem (f) |
| [Q30](../spec/DESIGN_DECISIONS_36.md#decision-30) | Ảnh thay và stable capture | [P07-T02](phases/PHASE-07.md#p07-t02), [P07-T03](phases/PHASE-07.md#p07-t03), [P07-T12](phases/PHASE-07.md#p07-t12) | P07-AC02 · TC02, P07-AC03 · TC03, P07-AC12 · TC12 |
| [Q31](../spec/DESIGN_DECISIONS_36.md#decision-31) | Preview/session/sanitization | [P07-T05](phases/PHASE-07.md#p07-t05), [P07-T10](phases/PHASE-07.md#p07-t10), [P07-T11](phases/PHASE-07.md#p07-t11), [P07-T13](phases/PHASE-07.md#p07-t13) | P07-AC05 · TC05, P07-AC10 · TC10, P07-AC11 · TC11, P07-AC13 · TC13 (Stale → reload tạo bundle hợp lệ mới, giữ bundle cũ, không export từ view) |
| [Q32](../spec/DESIGN_DECISIONS_36.md#decision-32) | EPUB 3.3 và validator/readers | [P07-T06](phases/PHASE-07.md#p07-t06), [P07-T07](phases/PHASE-07.md#p07-t07), [P07-T15](phases/PHASE-07.md#p07-t15), [P09-T09](phases/PHASE-09.md#p09-t09) | P07-AC06 · TC06, P07-AC07 · TC07 (validation trước publish), P07-AC15 · TC15 + V-10 (reader Calibre + trình đọc mục tiêu tại M7), P09-AC09 · TC09 (nghiệm thu release) |
| [Q33](../spec/DESIGN_DECISIONS_36.md#decision-33) | Bundle và export versions | [P07-T04](phases/PHASE-07.md#p07-t04), [P07-T08](phases/PHASE-07.md#p07-t08), [P07-T09](phases/PHASE-07.md#p07-t09), [P07-T14](phases/PHASE-07.md#p07-t14) | P07-AC04 · TC04, P07-AC08 · TC08, P07-AC09 · TC09, P07-AC14 · TC14 |
| [Q34](../spec/DESIGN_DECISIONS_36.md#decision-34) | CLI contract | [P01-T04](phases/PHASE-01.md#p01-t04), [P08-T07](phases/PHASE-08.md#p08-t07), [P08-T08](phases/PHASE-08.md#p08-t08), [P08-T09](phases/PHASE-08.md#p08-t09) | AC-03/04 · TC-03/04 (PHASE-01), P08-AC07 · TC07, P08-AC08 · TC08, P08-AC09 · TC09 |
| [Q35](../spec/DESIGN_DECISIONS_36.md#decision-35) | Tám màn, keyboard và resize | [P01-T07](phases/PHASE-01.md#p01-t07), [P01-T08](phases/PHASE-01.md#p01-t08), [P01-T10](phases/PHASE-01.md#p01-t10), [P08-T04](phases/PHASE-08.md#p08-t04), [P08-T05](phases/PHASE-08.md#p08-t05), [P08-T06](phases/PHASE-08.md#p08-t06), [P04-T12](phases/PHASE-04.md#p04-t12), [P08-T10](phases/PHASE-08.md#p08-t10) | AC-07/08/10/11 · TC-07/08/10/11 (PHASE-01), P08-AC04 · TC04, P08-AC05 · TC05, P08-AC06 · TC06; P04-AC12 · TC12 và P08-AC10 · TC10 (Reviewer → Editor → quay lại đúng thẻ) |
| [Q36](../spec/DESIGN_DECISIONS_36.md#decision-36) | Fixture, oracle và acceptance | [P00-T05](phases/PHASE-00.md#p00-t05), [P02-T04](phases/PHASE-02.md#p02-t04) (e), [P09-T01](phases/PHASE-09.md#p09-t01), [P09-T02](phases/PHASE-09.md#p09-t02), [P09-T03](phases/PHASE-09.md#p09-t03), [P09-T04](phases/PHASE-09.md#p09-t04), [P09-T06](phases/PHASE-09.md#p09-t06), [P09-T09](phases/PHASE-09.md#p09-t09) | P00-AC05 · TC05, P02-AC04 · TC04, P09-AC01 · TC01, P09-AC02 · TC02, P09-AC03 · TC03, P09-AC04 · TC04, P09-AC06 · TC06, P09-AC09 · TC09 |

### 5.1. Ghi chú rà truy vết — 07/11/2026 (lượt P00-T01-r01, r02 và r03, model A)

**Lượt r01** — chuỗi Q → task → AC/TC đã được kiểm chứng tự động ngày 07/11/2026: mọi link task trong bảng đều trỏ đúng phase và tồn tại; mọi AC/TC tham chiếu trong task card tồn tại trong mục 12 (AC) / mục 13 (test matrix) của phase tương ứng; mọi anchor `#decision-01`…`#decision-36` tồn tại trong [S-Q](../spec/DESIGN_DECISIONS_36.md); mọi link tương đối trong ROADMAP.md và PHASE-00.md phân giải được tới file/anchor thật. Không phát hiện yêu cầu sản phẩm bị đổi ngầm so với S-Q; các sai lệch đều được ghi chú (a)–(g) thay vì chỉnh âm thầm. Số liệu đếm: **103 lượt link trong 36 hàng; 122 trong toàn §5; 124 trong toàn ROADMAP** (regex kiểm mã r01 không nhận dạng dạng `AC-xx`/`TC-xx` của PHASE-01 — hạn chế đã biết; review B tự xác minh lại bằng phép kiểm riêng, xem báo cáo B r01 §2).

**Lượt r02 (sau review B **Cần sửa**, 7 finding F01–F07, 07/11/2026)** — đã xử lý từng finding; chi tiết và câu trả lời từng ID xem [báo cáo r02](evidence/phase-00/P00-T01.handoff-r02.md). Thay đổi truy vết: Q02 → TC08 của [P02-T08](phases/PHASE-02.md#p02-t08) nay gồm tái dùng/đối chiếu lại theo hash EPUB + version mapping (F02); Q05 → owner [P04-T02](phases/PHASE-04.md#p04-t02) (F01, xem (b)); Q08 → AC12/TC12 của [P03-T12](phases/PHASE-03.md#p03-t12) gồm gắn lại đơn vị nguồn khi marker thiếu/trùng (F03); Q13 → thêm [P06-T02](phases/PHASE-06.md#p06-t02) hash check Skill trước lượt chưa gửi, lượt đã gửi giữ nền cũ (F04); Q31 → thêm [P07-T13](phases/PHASE-07.md#p07-t13) Stale → reload (F05); Q32 → thêm [P07-T15](phases/PHASE-07.md#p07-t15) reader tại M7, phân biệt validation trước publish (P07-T07) và nghiệm thu release (P09-T09) (F06); Q35 → thêm [P04-T12](phases/PHASE-04.md#p04-t12) và [P08-T10](phases/PHASE-08.md#p08-t10) cho Reviewer → Editor → quay lại đúng thẻ (F07). Các finding F02–F07 chỉ bổ sung AC/TC/hàng truy vết cho hành vi đã chốt trong S-Q; không đổi yêu cầu, không đổi AC nào theo hướng nới lỏng.

**Lượt r03 (sau review B r02 **Cần sửa**, còn duy nhất F01, 07/11/2026; chi tiết xem [báo cáo r03](evidence/phase-00/P00-T01.handoff-r03.md))** — F01 được đóng thêm ranh giới người dùng: [P05-T13](phases/PHASE-05.md#p05-t13) (UI-06 — màn hình Tasks) hiển thị vị trí Chương · Block ID · nguyên nhân từ core, chỉnh điểm chia/giới hạn tại màn hình và lập lại lượt qua core (P05-AC13 · TC13); [P05-T15](phases/PHASE-05.md#p05-t15) (CLI) trả issue có vị trí + next_action, exit 7 (P05-AC15 · TC15). Màn hình/control/trình bày chi tiết là lựa chọn triển khai; F02–F07 giữ nguyên Đóng. Không đổi yêu cầu/AC theo hướng nới lỏng.

- (a) **Q01 — yêu cầu phụ trợ “chữ trong bitmap đi qua luồng ảnh” và nhận/lưu phạm vi.** Phạm vi lưu và hiệu lực trước lượt dịch đầu được nhận bởi [P02-T12](phases/PHASE-02.md#p02-t12) (UI-03: lưu scope đúng đối tượng, P02-AC12 · TC12 — đã bổ sung vào hàng Q01) và [P06-T13](phases/PHASE-06.md#p06-t13) (hiển thị cấu hình hiệu lực, P06-AC13 · TC13). Nhánh “chữ trong bitmap” thuộc pipeline ảnh Phase 07 ([P07-T02](phases/PHASE-07.md#p07-t02)/[P07-T03](phases/PHASE-07.md#p07-t03), Q30); ghi chú để không bị coi là thiếu — không tạo task mới.
- (b) **Q05 — phụ trợ “hiển thị vị trí để chỉnh điểm chia hoặc giới hạn” cho block không có chỗ ngắt an toàn — ĐÃ ĐÓNG CHUỖI (r02 lõi, r03 ranh giới người dùng; finding F01).** [P04-T02](phases/PHASE-04.md#p04-t02) (P04-AC02 · TC02) sở hữu lõi: chặn gửi, diagnostic kèm vị trí nguồn (XHTML path/block ID/range), re-plan chỉ qua điểm chia/giới hạn người dùng cấp, protected inline không bị cắt; fixture “một block vượt trần, mọi điểm cắt đều trong vùng protected/inline”. **Ranh giới người dùng (r03):** [P05-T13](phases/PHASE-05.md#p05-t13) (UI-06, P05-AC13 · TC13) — màn hình Tasks hiển thị Chương · Block ID · nguyên nhân, chỉnh điểm chia/giới hạn tại màn hình, lập lại lượt qua core; [P05-T15](phases/PHASE-05.md#p05-t15) (CLI, P05-AC15 · TC15) — issue vị trí + next_action, exit 7. Cách trình bày chi tiết là lựa chọn triển khai.
- (c) **Q06 — workspace test Go cũ.** S-Q chốt “xử lý bằng luồng tạo workspace mới và import EPUB theo quyết định đã chốt”, không migrate tại chỗ; các phase nhất quán ([PHASE-02 §5](phases/PHASE-02.md): “không sửa dữ liệu Go cũ”; PHASE-09: “Không yêu cầu migrate workspace test Go cũ”). Nơi thực thi là luồng create/import ([P02-T01](phases/PHASE-02.md#p02-t01), [P02-T09](phases/PHASE-02.md#p02-t09)); không cần task riêng.
- (d) **Q07 — phụ trợ “File chưa được tham chiếu có thể thu gom sau”.** S-Q cho phép thu gom sau, không bắt buộc trước M0/M2. Review B ghi nhận **không chặn P00-AC01** (review-r01 §4). P02-T02 nhận chính sách/orphan inventory trước khi thêm garbage collection; việc chưa có GC không chặn file store đầu tiên hoặc M0-01/M0-02. Xem [lịch DB/ADR](#decision-schedule) và [P02-T02](phases/PHASE-02.md#p02-t02).
- (e) **Q36 — giới hạn corpus đầu vào (10.000 mục ZIP, 128 MiB/mục, 1 GiB sau giải nén, kiểm byte thực tế và collision).** Nơi thực thi/kiểm chứng là [P02-T04](phases/PHASE-02.md#p02-t04) (P02-AC04 · TC04: fixture dưới/đúng/trên giới hạn) và invariant ở PHASE-02 mục 5; đã bổ sung vào hàng Q36.
- (f) **Bàn giao giữa các phase nhận cùng trách nhiệm — không phát hiện thiếu điểm bàn giao:** Q03 (P02 mapping version → [P03-T14](phases/PHASE-03.md#p03-t14) rebase/reuse → [P04-T09](phases/PHASE-04.md#p04-t09)/[P06-T12](phases/PHASE-06.md#p06-t12) rà lại chương); Q07 (P02 generation/CAS → [P03-T02](phases/PHASE-03.md#p03-t02) outbox/projection); Q15 (approval publication → wake đúng điều kiện qua [P06-T12](phases/PHASE-06.md#p06-t12) dù card không cite Q15 — ghi nhận, không sửa); Q29 ([P04-T10](phases/PHASE-04.md#p04-t10) checkpoint tối thiểu → [P05-T12](phases/PHASE-05.md#p05-t12) recovery đầy đủ → [P06-T12](phases/PHASE-06.md#p06-t12) wake-up); Q36 (P00-T05 chiến lược → P02-T04 giới hạn container → P09-T02 corpus nghiệm thu); Q32 (P07-T07 validation trước publish → P07-T15 reader tại M7 → P09-T09 nghiệm thu release, mỗi bước một vai trò, không trùng trách nhiệm) (r02, finding F06). Q33 (P07-T07 gate xuất) → P09-T09 nghiệm thu đọc thực là hai phép kiểm khác vai trò (build-time vs release), không trùng trách nhiệm.
- (g) **Q04 — P00-T07** (P00-AC07 · TC07) là bước chuẩn bị (ADR-003: ID/hash/offset/source mapping), không thay nơi triển khai/kiểm chứng [P02-T06](phases/PHASE-02.md#p02-t06)/[P03-T14](phases/PHASE-03.md#p03-t14).
- (h) **Nguồn bằng chứng bàn giao (r02–r03).** Manifest r01 chỉ có hash spec/ADR/template/phase chính; r02 bổ sung hash của D-ROADMAP và cả 10 PHASE-*.md cùng diff r01 hash-verified (`P00-T01.diff-r01.patch`, tái tạo bản trước r01 bằng revert các edit và khớp hash baseline A đã ghi). r03 giữ patch của r02 ở định dạng byte Git nguyên vẹn theo góp ý B r02 §4. Ngoài ra A ghi lại giới hạn regex r01 (không nhận dạng `AC-xx/TC-xx` của PHASE-01) và số đếm 103/122/124 thay vì chỉ “124/124”.

Ngoài các ghi chú trên, mọi quyết định Q01–Q36 đều có ít nhất một nơi triển khai và một nơi kiểm tra được rà khớp trong bảng; mục nào còn mở ((d) không thuộc M0) được nêu rõ thay vì đánh dấu đạt.

## 6. Phân bố giao diện

| Gói | Phase | Kết quả |
|---|---|---|
| UI-01 | 01 | Shell, focus, help, resize và terminal lifecycle |
| UI-02/03 | 02 | Tạo/mở workspace, nhập EPUB, cây nguồn, phân loại, mapping |
| UI-04 | 03 | Editor nguồn/nháp/lịch sử, Ctrl+S và conflict |
| UI-05 | 04 | Reviewer/evidence, mở Editor và trở lại đúng thẻ |
| UI-06/07 | 05 | Tasks/queue/model/cost và Settings/connections |
| UI-08/09/10 | 06 | Start scope/to-end, Review/Research, Skills approval |
| UI-11/12/13 | 07 | Ảnh, readiness/Preview, export/version history |
| UI-14/15 | 08 | Tổng quan/báo cáo/help và keyboard/responsive xuyên tám màn |
| Acceptance UI | 09 | Kiểm trên artifact/nền tảng dự kiến phát hành |

Nhờ Research đề xuất phân loại được nối ở P06 sau import; import P02 vẫn cục bộ. Core approval có ở P03 với CLI tối thiểu, giao diện duyệt nhóm đầy đủ ở P06. Shell tám route không có nghĩa tám màn nghiệp vụ đã hoàn thiện.

## 7. Kiểm thử và cách báo tiến độ

- Mỗi task có phạm vi, dependency, output, lỗi/biên, AC, fixture và cách kiểm chứng. Phase có thêm luồng tích hợp/recovery/demo phù hợp.
- CI dùng dữ liệu tự soạn và model/công cụ giả lập; EPUB được tạo tạm. Không gọi provider trả phí.
- Parser/DB/filesystem/IPC/terminal/EPUBCheck cần kiểm adapter thật tại boundary phù hợp; không thay mọi phép kiểm bằng mock.
- Báo riêng automated, terminal/reader manual và live model evaluation. Cờ offline của Cargo không tự chặn network do test mở; harness/transport phải được cô lập.
- Bộ Rust bắt buộc theo AGENTS.md chạy trước PR Rust; schema/layout/command/platform thay đổi có migration hoặc README tương ứng.
- Bằng chứng ở `docs/roadmap/evidence/phase-XX/`, đã redaction. [Bàn giao A](templates/TASK_HANDOFF.md) và [review B](templates/TASK_REVIEW.md) là hai báo cáo riêng theo lượt, gắn baseline/candidate, phiên bản yêu cầu, lệnh/số ca/kết quả, phần chưa chạy và findings. Giữ báo cáo cũ khi có bản sửa.
- B tự đối chiếu AC, code và test, chạy kiểm chứng liên quan, ghi Đạt / Cần sửa / Chưa đủ bằng chứng. A phase cập nhật evidence riêng, điều phối cập nhật PLAN trung tâm; không tự thay kết luận B. Trạng thái Chờ review không đồng nghĩa hoàn thành.
- Cuối phase hoặc cổng M0 có [review tích hợp riêng](templates/PHASE_REVIEW.md) của B trên candidate cuối. Kiểm tra bắt buộc chưa đủ giữ chưa xác minh; kết luận trên bản cũ không tự áp dụng cho bản đã thay đổi.

## 8. Phạm vi hiện đã thực hiện

Phase 00 M0-r03 và Phase 01 M1-r01 đã B Đạt trên Windows; [review M1](evidence/phase-01/M1.review-r01.md). PR #1 đã merge; commit/baseline và giới hạn đối chiếu ghi trong [PLAN](../../PLAN.md). Lịch song song v2 cho Phase 02–09 đã được soạn, chưa thực hiện code; trạng thái review kế hoạch ở PLAN. Giữ task IDs/AC/TC và bài tự test, chia lại 64 checkpoint 1–3 task và mở dependency từng phần. Bản v0.1 chưa triển khai được lưu lịch sử.

Bước tiếp là giao chat P02 và P09 chuẩn bị theo kế hoạch đã B Đạt, rồi mở P03/P07 theo P02-CP05. Dọn Go dần theo [PLAN §8](../../PLAN.md#go-retirement-plan), trên phạm vi được điều phối cấp và có review bản sau dọn; dữ liệu người dùng giữ nguyên. GO-FINAL là việc bắt buộc để đạt mục tiêu chỉ Rust. Lượt lập kế hoạch chưa xóa code; merge checkpoint vào rewrite/rust theo ủy quyền ngày 14/11/2026 và [quy trình merge](MERGE_WORKFLOW.md); publish release vẫn do người dùng quyết định.


Quy trình hiện hành: [MERGE_WORKFLOW](MERGE_WORKFLOW.md). CP code chỉ mở dependency ngoài phase sau merge và hậu kiểm; docs-only dùng manifest/review. Phần đã nghiệm thu và merge được bảo vệ; áp dụng ngoại lệ conflict thuần test và yêu cầu phê duyệt core theo quy trình trên. AC, task và graph không đổi.
