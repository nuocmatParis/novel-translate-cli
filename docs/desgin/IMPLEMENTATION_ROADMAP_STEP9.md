# BƯỚC 9 — IMPLEMENTATION ROADMAP & AI CODING PLAN

> Lịch thực thi cập nhật 12/11/2026: Phase 01 đã nghiệm thu Windows và merge; Phase 02–09 dùng [kế hoạch chat song song v2](../roadmap/PARALLEL_EXECUTION.md), [dependency/checkpoint](../roadmap/PARALLEL_TASKS.json) và [PLAN](../../PLAN.md). Các milestone dưới đây giữ phạm vi nghiệm thu; task được mở sớm bằng checkpoint đã duyệt, không phải chờ toàn phase trước.

**Dự án:** `novel-translate-cli`  
**Phiên bản:** 2.0 — lịch thực thi song song  
**Ngày:** 12/11/2026  
**Trạng thái:** M0-r03 và M1-r01 đã B Đạt trong phạm vi Windows; Phase 01 đã merge. Phase 02–09 chưa triển khai; trạng thái review kế hoạch song song v2 theo [PLAN](../../PLAN.md).
**Tiền đề:** [Bước 1](ARCHITECTURE_OVERVIEW.md), [Bước 2](SUBSYSTEM_ARCHITECTURE.md), [Bước 3](RUST_PROJECT_ARCHITECTURE.md), [Bước 4](MODULE_DESIGN_STEP4.md), [Bước 5](TECHNICAL_FOUNDATION_STEP5.md), [Bước 6](DATA_ARCHITECTURE_STEP6.md), [Bước 7](WORKFLOW_STATE_MACHINE_STEP7.md), [Bước 8](UI_UX_DESIGN_STEP8.md).

> **Phạm vi:** Thiết kế cách chuyển hệ thống đặc tả thành phần mềm Rust qua những giai đoạn có thể kiểm thử. Các lựa chọn kỹ thuật còn mở ở Bước 4–8 không tự động trở thành quyết định đã chốt chỉ vì chúng được nêu trong roadmap.

---

## 1. Mục tiêu và chiến lược triển khai

Bước 9 không chỉ là một danh sách như “tạo database → backend → TUI → test”. Đây là hệ thống kế hoạch phát triển, giúp AI coding agent và developer biết phải làm gì, làm theo thứ tự nào, kiểm chứng ra sao và khi nào được chuyển bước.

### 1.1. Ba phương pháp kết hợp

| Phương pháp | Vai trò |
|---|---|
| **Architecture-First** | Bảo vệ các ranh giới kiến trúc, ownership và contract đã thiết kế |
| **Vertical Slice Development** | Triển khai từng luồng xuyên Domain → Application → Infrastructure → CLI/TUI → Tests |
| **Incremental Delivery** | Hoàn thiện có kiểm chứng theo từng phase, không viết toàn hệ thống trong một lần |

### 1.2. Vertical Slice thay vì Horizontal Big Bang

**Không nên:**

```text
Toàn bộ Domain
      ↓
Toàn bộ Database
      ↓
Toàn bộ Application
      ↓
Toàn bộ Infrastructure
      ↓
Toàn bộ TUI
      ↓
Lúc này mới kiểm thử
```

**Nên:**

```text
Một hành vi/use case nhỏ
        │
        ├── Domain
        ├── Application
        ├── Storage/Adapter
        ├── CLI/TUI tối thiểu
        └── Tests
              │
              ▼
     Chạy và kiểm chứng
              │
              ▼
       Slice tiếp theo
```

Vertical slice nhập EPUB ở [Phase 02](#phase-02):

**Tạo workspace → Import EPUB → Lưu SourceSnapshot → Đóng ứng dụng → Mở lại → Đọc lại đúng dữ liệu đã lưu.**

[Phase 04](#phase-04) mở rộng từ dữ liệu đó thành luồng Dịch–Review đầu tiên bằng model giả lập.

### 1.3. Nguyên tắc quản trị thay đổi

- **[ĐÃ CHỐT]** Hành vi sản phẩm và điều kiện nghiệm thu theo [36 quyết định và bảng đối chiếu đặc tả](../spec/DESIGN_DECISIONS_36.md).
- **[ĐỀ XUẤT]** Kiến trúc và hợp đồng ở Bước 1–8. Cách chia Phase 00–09, milestone và task ở tài liệu này là phương án triển khai.
- **[CÒN MỞ]** Ghi quyết định kỹ thuật vào ADR trước task phụ thuộc. Đối chiếu lại đề xuất với yêu cầu đã chốt khi hai tài liệu mô tả khác nhau.
- [AGENTS.md](../../AGENTS.md) là nguồn quy tắc thực thi trong repo, gồm nhánh đích `rewrite/rust`, giữ Go hoạt động đến khi bản Rust đạt feature parity và các lệnh kiểm tra trước PR Rust.

Bước 1–9 là các tài liệu thiết kế. Phase 00–09 là các giai đoạn triển khai được đề xuất trong Bước 9.

---

## 2. Cấu trúc dự án và các tài liệu điều phối

### 2.1. Cây thư mục đề xuất

```text
novel-translate-cli/
├── AGENTS.md
├── PLAN.md
├── Cargo.toml
│
├── apps/
│   └── novel-translate/
│
├── crates/
│   ├── nt-domain/
│   ├── nt-application/
│   ├── nt-storage/
│   ├── nt-epub/
│   ├── nt-model/
│   ├── nt-research/
│   ├── nt-export/
│   ├── nt-coordinator/
│   └── nt-tui/
│
├── docs/
│   ├── spec/              # Đặc tả và 36 quyết định hiện có
│   ├── desgin/            # Thiết kế Bước 1–9 và SQL nháp hiện có
│   ├── roadmap/
│   │   ├── ROADMAP.md
│   │   ├── REVIEW_WORKFLOW.md
│   │   ├── templates/
│   │   │   ├── TASK_HANDOFF.md
│   │   │   ├── TASK_REVIEW.md
│   │   │   └── PHASE_REVIEW.md
│   │   ├── phases/
│   │   │   ├── PHASE-00.md
│   │   │   ├── PHASE-01.md
│   │   │   ├── PHASE-02.md
│   │   │   └── ...
│   │   └── evidence/       # Tạo khi thực hiện; báo cáo riêng A/B theo phase và lượt
│   └── decisions/
│       ├── ADR-001.md
│       └── ...
│
└── tests/
    ├── fixtures/
    ├── integration/
    └── acceptance/
```

Các tài liệu nguồn nằm ở `docs/spec/` và `docs/desgin/`. Đã có [roadmap](../roadmap/ROADMAP.md), [ADR-001](../decisions/ADR-001.md), [ADR-008](../decisions/ADR-008.md) và [PLAN.md](../../PLAN.md); các ADR khác và cây ứng dụng Rust vẫn là mục tiêu triển khai. `apps/novel-translate/` là thư mục app đã thống nhất trong ADR-008, phát hành executable `novel-translate-cli` theo [Quyết định 34](../spec/DESIGN_DECISIONS_36.md#decision-34).

### 2.2. Phân chia trách nhiệm

| File | Vai trò | Đối tượng sử dụng |
|---|---|---|
| `AGENTS.md` | Quy tắc bất biến và quy trình AI coding agent phải tuân thủ | AI Agent |
| `PLAN.md` | Trạng thái, nhiệm vụ và kế hoạch thực thi hiện hành | AI Agent + developer |
| `docs/roadmap/ROADMAP.md` | Tổng quan phase, milestone, phụ thuộc và tiến độ | Developer |
| `docs/roadmap/REVIEW_WORKFLOW.md` | Quy trình triển khai, review độc lập, xử lý findings và nghiệm thu | Model A + model B + developer |
| `docs/roadmap/templates/` | Mẫu bàn giao A, review task B và review phase B | Model A + model B |
| `docs/roadmap/phases/PHASE-XX.md` | Phân rã công việc một giai đoạn | AI Agent |
| `docs/decisions/ADR-XXX.md` | Quyết định kỹ thuật, lý do, đánh đổi, hệ quả | Developer + AI Agent |
| `tests/` | Bằng chứng implementation đúng contract | Developer + CI |

**Không dùng `AGENTS.md` thay `PLAN.md`, hoặc ngược lại.** `AGENTS.md` là luật triển khai; `PLAN.md` là nhiệm vụ hiện hành.

AI nhận việc từ [AGENTS.md](../../AGENTS.md) → [PLAN.md](../../PLAN.md) và [quy trình hai model](../roadmap/REVIEW_WORKFLOW.md) → task/phase và tài liệu tham chiếu. Model A triển khai, kiểm thử, tự review và bàn giao Chờ review; model B khác kiểm tra độc lập, ghi kết luận riêng. A sửa, B kiểm lại; đủ bằng chứng và B Đạt mới đóng task. PLAN trung tâm chỉ do A điều phối cập nhật từ báo cáo từng chat phase. Mỗi chat A tự gọi reviewer gpt-6.1-sol / max khác model/phiên theo REVIEW_WORKFLOW. Các hướng đã thống nhất nằm trong ADR, không cần hỏi lại khi hoàn thiện lựa chọn kỹ thuật thông thường.

---

## 3. Roadmap tổng thể: Phase 00–09

| Phase | Tên | Mục tiêu | Tài liệu dùng để chia task |
|---|---|---|---|
| [**00**](#phase-00) | Architecture Decisions & Preparation | Giải quyết quyết định kỹ thuật thực sự chặn triển khai | [Nền tảng và các mục TD](TECHNICAL_FOUNDATION_STEP5.md#13-trạng-thái-các-quyết-định-kỹ-thuật) |
| [**01**](#phase-01) | Rust Workspace Foundation | Cargo Workspace, crate skeleton, CLI/TUI shell và CI | [Crate](RUST_PROJECT_ARCHITECTURE.md), [Module](MODULE_DESIGN_STEP4.md) |
| [**02**](#phase-02) | Persistence & EPUB Foundation | SQLite, immutable storage, import EPUB và mapping | [EPUB đầu vào](../spec/EPUB_INPUT_SPEC.md), [Schema và transaction](DATA_ARCHITECTURE_STEP6.md) |
| [**03**](#phase-03) | Domain State & Context | Revision, Generation, Dependency, Skills và approved decisions | [Dữ liệu/trạng thái](../spec/DATA_MODEL_STATE_SPEC.md), [Skills](../spec/SKILLS_GLOSSARY_SPEC.md), [Xưng hô](../spec/ADDRESSING_CONTINUITY_SPEC.md) |
| [**04**](#phase-04) | First End-to-End Translation & Review Slice | Luồng Dịch–Review tối thiểu bằng model giả lập | [Dịch](../spec/TRANSLATION_STAGE_SPEC.md), [Review](../spec/REVIEW_STAGE_SPEC.md), [Workflow](WORKFLOW_STATE_MACHINE_STEP7.md) |
| [**05**](#phase-05) | AI Execution & Coordination | Model Gateway, usage/budget, attempts, Coordinator, checkpoint | [Model](../spec/MODEL_STRATEGY_SPEC.md), [Điều phối](../spec/WORKFLOW_ORCHESTRATION_SPEC.md), [Workflow và recovery](WORKFLOW_STATE_MACHINE_STEP7.md) |
| [**06**](#phase-06) | Complete Translation & Review Workflow | Dịch–Review–Research–Repair và phục hồi đầy đủ | [Điều phối](../spec/WORKFLOW_ORCHESTRATION_SPEC.md), [Rubric](../spec/REVIEW_RUBRIC_SPEC.md), [Research](../spec/RESEARCH_STAGE_SPEC.md) |
| [**07**](#phase-07) | Images, Preview & EPUB Export | Ảnh, Prepared Export Bundle, Preview, EPUB 3.3 | [Ảnh](../spec/IMAGE_EDIT_SPEC.md), [Bảo mật](../spec/SECURITY_SPEC.md), [EPUB đầu ra](../spec/EPUB_OUTPUT_SPEC.md) |
| [**08**](#phase-08) | Complete TUI & CLI Experience | Hoàn thiện tám màn TUI và CLI theo đặc tả | [Hợp đồng TUI/CLI](../spec/TUI_CLI_SPEC.md), [Thiết kế UI/UX](UI_UX_DESIGN_STEP8.md) |
| [**09**](#phase-09) | Hardening, Acceptance & Release | Kiểm thử, bảo mật, recovery, tương thích và phát hành | [Quyết định 36](../spec/DESIGN_DECISIONS_36.md#decision-36), [Kiểm tra Rust](../../AGENTS.md#required-checks) |

CLI entrypoint và TUI shell bắt đầu ở Phase 01. Mỗi luồng từ Phase 02–07 bổ sung phần giao diện dùng được với Application Core của luồng đó. Phase 08 hoàn thiện hợp đồng TUI/CLI theo [các gói giao diện](#ui-work-packages).

### 3.1. Milestone/exit gate

| Milestone | Bằng chứng cần có |
|---|---|
| M0 | M0-01 đủ cho nền tảng Phase 01; M0-02 đủ cho storage/import Phase 02. Có thể bàn giao M0-01 trước khi hoàn tất toàn Phase 00. |
| M1 | Workspace build/test/lint, dependency graph hợp lệ; CLI/TUI shell UI-01 qua nghiệm thu khởi động, bàn phím, resize và cleanup terminal |
| M2 | Import EPUB và mở lại workspace giữ đúng snapshot/mapping |
| M3 | Revision/approval/dependency/generation persistence có kiểm thử |
| M4 | Luồng Dịch–Review bằng fake provider và kiểm thử workflow thành công |
| M5 | AI request ledger/permit/budget/recovery chịu được các trường hợp lỗi giả lập |
| M6 | Workflow nghiệp vụ đầy đủ xử lý các điểm chờ và sửa |
| M7 | EPUB 3.3 hoàn chỉnh từ dữ liệu đã Review, phiên bản xuất không ghi đè |
| M8 | Tám màn và CLI đáp ứng acceptance tests |
| M9 | Release criteria và security/recovery/compatibility tests đạt |

<a id="ui-work-packages"></a>

### 3.2. Gói giao diện theo phase

Đây là cách chia việc đề xuất theo [tám màn đã chốt](UI_UX_DESIGN_STEP8.md#2-tám-màn-hình-chính-đã-chốt). Một màn có thể xuất hiện qua vài phase. Mỗi gói dùng các lệnh hoặc truy vấn đã có ở Application Core. CLI và TUI dùng chung hợp đồng nghiệp vụ.

| Gói | Phase | Phần giao diện | Bằng chứng nghiệm thu |
|---|---|---|---|
| UI-01 | [01](#phase-01) | Khung TUI, điều hướng và focus | Chạy binary không tham số mở TUI. Dùng bàn phím để di chuyển focus, mở trợ giúp và thoát về terminal. |
| UI-02 | [02](#phase-02) | Tổng quan với thao tác tạo và mở workspace | Mở lại workspace mẫu hiển thị đúng bộ/tập. Lỗi mở workspace chỉ rõ vấn đề. |
| UI-03 | [02](#phase-02) | Tập và chương với các task nhập EPUB, xem cây, phân loại và chỉnh mapping | Nhập mẫu, đóng và mở lại vẫn xem đúng thứ tự cây. Chỉnh mapping hợp lệ cập nhật cây. EPUB hoặc mapping lỗi hiển thị chẩn đoán từ core. |
| UI-04 | [03](#phase-03) | Editor Nhật–Việt với thao tác xem, sửa, lưu và xem lịch sử | Với bản thảo mẫu, `Ctrl+S` tạo revision mới. Khi revision nền đã đổi, giữ bản đang soạn và hiển thị xung đột. |
| UI-05 | [04](#phase-04) | Reviewer với danh sách lỗi, thẻ bằng chứng và lối mở Editor | Kết quả từ luồng giả lập mở đúng đoạn nguồn và revision. Quay lại giữ đúng thẻ lỗi. |
| UI-06 | [05](#phase-05) | Tác vụ với hàng đợi, model thực tế, chi phí và tạm dừng/tiếp tục | Xem tác vụ B khi A đang chạy vẫn thấy A giữ quyền. Tiếp tục dùng checkpoint đã lưu và đi qua hàng đợi. |
| UI-07 | [05](#phase-05) | Cấu hình kết nối, model theo vai trò và ngân sách | Lỗi cấu hình có thông báo và hướng xử lý. Key được che và vắng mặt trong chẩn đoán. |
| UI-08 | [06](#phase-06) | Tập và chương với lựa chọn phạm vi và bắt đầu Dịch | Phân biệt phần chữ cần dịch với chế độ chạy chương đã chọn hoặc đến hết EPUB. Tác vụ nhận đúng cả hai lựa chọn. |
| UI-09 | [06](#phase-06) | Reviewer với thao tác Review lại và xem bằng chứng Research | Yêu cầu Review lại nhắm đúng revision. Bằng chứng hiện nguồn và phần còn cần xác minh. |
| UI-10 | [06](#phase-06) | Skills và ngữ cảnh với đề xuất, phạm vi và duyệt nhóm | Chỉ các mục đã chọn đúng phiên bản được duyệt. Mục còn lại tiếp tục chờ. |
| UI-11 | [07](#phase-07) | Ảnh nguồn, ảnh thay và chẩn đoán | Ảnh thay hỏng chỉ rõ file cần sửa và chặn xuất. |
| UI-12 | [07](#phase-07) | Kiểm tra sẵn sàng và Preview | Preview mở đúng bản đã chuẩn bị. Thay đổi dữ liệu làm giao diện báo bản xem trước đã cũ. |
| UI-13 | [07](#phase-07) | Xuất EPUB và xem lịch sử phiên bản | Xuất từ bản đã chuẩn bị. Lịch sử hiện phiên bản và đường dẫn mới, giữ được các bản trước. |
| UI-14 | [08](#phase-08) | Hoàn thiện Tổng quan, báo cáo và trợ giúp | Lối tắt mở đúng tác vụ/điểm chờ. `F1` mở trợ giúp theo màn hiện tại. |
| UI-15 | [08](#phase-08) | Kiểm thử bàn phím, bố cục và trạng thái qua tám màn | Luồng chính chạy bằng bàn phím ở 80×24 và bố cục rộng từ 120×24. Thu nhỏ rồi mở rộng terminal vẫn giữ đúng mục đang xem và bản đang soạn. |

Khi viết `PHASE-XX.md`, chia từng gói thành task theo thao tác. Ví dụ UI-05 tách thành hiển thị danh sách lỗi, mở thẻ bằng chứng, rồi mở Editor và quay lại đúng vị trí. Mỗi task ghi hành động, kết quả hiển thị và ca trống, lỗi hoặc xung đột phù hợp. Chạy kiểm thử offline với fixture và adapter giả lập, hoặc workspace tạm khi cần kiểm chứng dữ liệu đã lưu. Điều hướng, cuộn và resize giữ nguyên số yêu cầu model đã gửi.

---

## 4. Phạm vi từng Phase

<a id="phase-00"></a>

### Phase 00 — Architecture Decisions & Preparation

**Kế hoạch chi tiết:** [PHASE-00.md](../roadmap/phases/PHASE-00.md), theo cấu trúc 16 mục; [M0-r03 đã B Đạt](../roadmap/evidence/phase-00/M0.review-r03.md) trong phạm vi Windows. Các quyết định dành cho consumer tương lai giữ trạng thái mở/hoãn như ADR và ROADMAP.

**Mục tiêu:** Giải quyết các quyết định có ảnh hưởng đến tính đúng đắn hoặc khả năng triển khai các slice đầu tiên.

Các ADR chốt theo consumer. Nền ADR-001/008 cho M0-01 đã được nghiệm thu trong [M0-r03](../roadmap/evidence/phase-00/M0.review-r03.md), và consumer Phase 01 đã qua [M1-r01 trên Windows](../roadmap/evidence/phase-01/M1.review-r01.md). Bảng dưới giữ lịch sử mốc cần trước; những phần mở của storage, IPC, provider và export vẫn phải chốt tại task sử dụng:

| ADR | Quyết định | Căn cứ cần đối chiếu | Cần trước |
|---|---|---|---|
| [ADR-001](../decisions/ADR-001.md) | Rust toolchain và thư viện nền tảng | TD-01–04, TD-06–09 | Phần scaffold dùng lựa chọn đó ở Phase 01 |
| ADR-002 | SQLite driver và Database Worker | TD-05, DB-13 | Adapter SQLite ở Phase 02 |
| ADR-003 | Biểu diễn ID, bản ghi hash và ánh xạ `SourceRange` tới XHTML | DB-01–03, DB-05, [Quyết định 04](../spec/DESIGN_DECISIONS_36.md#decision-04) | Source mapping ở Phase 02 |
| ADR-004 | Generation bootstrap, CAS và commit nhất quán | DB-08, [Quyết định 07](../spec/DESIGN_DECISIONS_36.md#decision-07) | Ghi snapshot/mapping ở Phase 02, mở rộng cho revision ở Phase 03 |
| ADR-005 | Coordinator IPC và vòng đời tiến trình | TD-10–11, W7-07 | IPC liên tiến trình ở Phase 05 |
| ADR-006 | Bàn giao permit, fencing và đối chiếu sau crash | DB-10, W7-05–07 | Cấp permit liên tiến trình ở Phase 05 |
| ADR-007 | Workflow, phiên bản checkpoint và vòng đời attempt | W7-01–05 | Task ghi checkpoint hoặc gửi model ở Phase 04–05 |
| [ADR-008](../decisions/ADR-008.md) | Hợp đồng API và lỗi giữa các crate | TD-12–15, Bước 4 mục 6 | Task đầu tiên dùng mỗi hợp đồng |

Đối chiếu các ID tại [Bước 5](TECHNICAL_FOUNDATION_STEP5.md#13-trạng-thái-các-quyết-định-kỹ-thuật), [Bước 6](DATA_ARCHITECTURE_STEP6.md#12-những-quyết-định-còn-mở) và [Bước 7](WORKFLOW_STATE_MACHINE_STEP7.md#19-những-quyết-định-còn-mở). Ghi quyết định trước task sử dụng nó. DB-16 cần trước khi công bố Skills, DB-06 trước khi kiểm tra độ phủ Dịch–Review, DB-09 trước khi hạch toán chi phí và DB-15 trước khi công bố EPUB xuất.

**Exit Criteria:** Không còn vấn đề kỹ thuật chưa giải quyết nào chặn vertical slice sắp triển khai; ADR ghi rõ lựa chọn và đánh đổi.

<a id="phase-01"></a>

### Phase 01 — Rust Workspace Foundation

**Kế hoạch chi tiết:** [PHASE-01.md — bản mẫu theo cấu trúc 16 mục](../roadmap/phases/PHASE-01.md).

Bản mẫu đã được dùng để viết [đủ Phase 00–09 và bảng truy vết yêu cầu](../roadmap/ROADMAP.md). Phase 01 đã hoàn thành theo [review M1-r01 trên Windows](../roadmap/evidence/phase-01/M1.review-r01.md), PR đã merge; Phase 02–09 chưa triển khai. Việc nhận đúng merge tree và kiểm Linux/WSL còn theo phạm vi ghi trong [PLAN](../../PLAN.md).

**Công việc:**

- Lập Cargo Workspace từ đặc tả và đồ thị phụ thuộc của Bước 3–4. Theo yêu cầu ngày 06/11/2026, code hiện tại không là căn cứ thiết kế, đầu vào tái sử dụng hoặc cơ sở tính tiến độ của kế hoạch.
- Tạo crate/module và composition root đủ cho shell; chỉ tạo contract/port khi có consumer thực tế. Bản mẫu đề xuất bắt đầu với app, `nt-domain`, `nt-application` và `nt-tui`, bổ sung adapter trong phase cần dùng.
- Thiết lập formatting, lint, unit test framework.
- Logging/tracing với redaction.
- CLI entrypoint `novel-translate-cli` và TUI shell mở khi chạy không có tham số. Triển khai UI-01 theo đặc tả và Bước 8: tám route với trạng thái trống, focus, trợ giúp, resize và vòng đời terminal; chức năng nghiệp vụ được nối ở phase tương ứng.
- CI build/test và kiểm tra dependency boundaries.

**Exit Criteria:** CI đạt [bộ lệnh kiểm tra Rust trong AGENTS.md](../../AGENTS.md#required-checks) và kiểm tra ranh giới phụ thuộc. Các bài kiểm thử chạy offline. Shell và CLI đạt các tiêu chí trong bản mẫu Phase 01, có bằng chứng kiểm tra terminal thực. Việc lập kế hoạch từ đặc tả không thay đổi quy tắc giữ Go và dữ liệu người dùng.

<a id="phase-02"></a>

### Phase 02 — Persistence & EPUB Foundation

**Kế hoạch chi tiết:** [PHASE-02.md](../roadmap/phases/PHASE-02.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

Vertical slice:

```text
Create Workspace
      ↓
Import EPUB
      ↓
Parse Package / Spine / XHTML
      ↓
Create SourceSnapshot / SourceBlocks / Mapping
      ↓
Commit snapshot/mapping + immutable resource references
      ↓
Close Application
      ↓
Open Workspace
      ↓
Read Verified Source State
```

Khởi tạo workspace Rust schema 1 với phần migration được rà soát cho slice này. Hoàn tất file bất biến trước khi công bố snapshot, mapping và tham chiếu trong một giao dịch. Bootstrap generation và write path dùng hợp đồng đã đối chiếu ở ADR-004. Hai file `*_DRAFT.sql` ở Bước 6 là tài liệu rà soát schema.

**Exit Criteria:** Import cục bộ fixture EPUB 2/3, giữ nguyên BlockID và mapping khi mở lại. Kiểm thử ZIP hỏng, giới hạn giải nén và việc ngắt ở ranh giới commit theo [Quyết định 36](../spec/DESIGN_DECISIONS_36.md#decision-36).

<a id="phase-03"></a>

### Phase 03 — Domain State & Context

**Kế hoạch chi tiết:** [PHASE-03.md](../roadmap/phases/PHASE-03.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

**Công việc:**

- TranslationRevision, Revision lineage và current pointer.
- Mở rộng Generation Commit từ snapshot/mapping sang revision, dependency và công việc dựng lại Markdown. Lưu ba trục chất lượng, hiệu lực và thực thi riêng.
- Skills/glossary, candidates/approved decisions.
- Characters, aliases và addressing rules.
- Immutable object references, file sync, manual edits.
- Approval batch và đồng bộ Skill có recovery.

**Exit Criteria:** Sửa tạo revision mới, history được bảo toàn; đổi rule chỉ làm xét lại phần liên quan; approval không tự động; generation giao dịch nhất quán.

<a id="phase-04"></a>

### Phase 04 — First End-to-End Translation & Review Slice

**Kế hoạch chi tiết:** [PHASE-04.md](../roadmap/phases/PHASE-04.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

Dùng **Fake Model Gateway** để chứng minh luồng xuyên lớp:

```text
Import EPUB
    ↓
Select Chapter
    ↓
Prepare Translation Unit
    ↓
Fake Translation Model
    ↓
Structural Validation
    ↓
Commit TranslationRevision
    ↓
Fake Source-only Reviewer
    ↓
Fake Comparison Reviewer
    ↓
Review Result
    ↓
Review đạt: lặp lại cho lượt tiếp theo nếu còn
    ↓
Chapter-level Review
    ↓
Present Result
```

Phase 04 lưu checkpoint tối thiểu có phiên bản cho prepared work, response, revision/Review và điểm chờ để kiểm chứng restart luồng giả lập. Phase 05 mở rộng thành attempt ledger, reservation, send-gap recovery và Coordinator liên tiến trình trước khi gửi model thật.

**Exit Criteria:** Fixture offline có nhiều lượt. Review đúng revision ngay sau mỗi lượt, rồi kiểm tra tổng thể chương. Fake response sai được xử lý trong đúng giới hạn và trạng thái hiển thị khớp dữ liệu đã lưu; restart dùng lại response/revision/Review hợp lệ.

<a id="phase-05"></a>

### Phase 05 — AI Execution & Coordination

**Kế hoạch chi tiết:** [PHASE-05.md](../roadmap/phases/PHASE-05.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

**Công việc:**

- Model Gateway + adapter/provider đầu tiên.
- Capability, tokenizer/token budget, usage và pricing.
- Request/Attempt lifecycle, preflight, reservations.
- Timeout/retry/uncertain attempt/late response.
- Machine-wide Coordinator, OS lock, queue, permit.
- Checkpoint, handoff và crash reconciliation.

**Exit Criteria:** Kiểm thử offline chứng minh mỗi hồ sơ người dùng chỉ có một permit hợp lệ. Trước khi gửi, kiểm tra quyền chạy, dữ liệu và ngân sách, rồi lưu attempt intent và khoản dự phòng. Recovery dùng lại kết quả đã lưu còn hợp lệ và giữ các khoản chưa đối chiếu. Nếu cần chạy thử adapter với model thật, thực hiện thủ công theo ngân sách người dùng chọn.

<a id="phase-06"></a>

### Phase 06 — Complete Translation & Review Workflow

**Kế hoạch chi tiết:** [PHASE-06.md](../roadmap/phases/PHASE-06.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

**Công việc:**

- Dịch phần đã chọn hoặc tự động đến hết EPUB. Chế độ chạy và phạm vi nội dung là hai lựa chọn riêng. Lưu phạm vi thực tế của tác vụ, chuẩn bị ngữ cảnh và áp dụng Skills đã duyệt.
- Source-only Review, comparison, evidence/coverage.
- Review toàn chương, S0–S3, M1–M4.
- Research và spoiler boundary khi được yêu cầu.
- Repair rounds, waiting states và resume.
- Dependency propagation và validation.

**Exit Criteria:** Tập fixture đi qua luồng với các nhánh lỗi/cần duyệt; không tự chuyển chương khi Review chưa đủ; không auto-approve hoặc auto-model-fallback; retry và repair rounds đếm riêng.

<a id="phase-07"></a>

### Phase 07 — Images, Preview & EPUB Export

**Kế hoạch chi tiết:** [PHASE-07.md](../roadmap/phases/PHASE-07.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

**Công việc:**

- Image source/edit matching và safety validation.
- Prepared Export Bundle bất biến với nguồn và revision được ghim.
- Preview loopback/session security.
- EPUB 3.3 package/navigation/resources.
- Kiểm tra EPUBCheck 5.3.0, độ phủ và liên kết. Lưu lịch sử phiên bản xuất theo [Quyết định 32–33](../spec/DESIGN_DECISIONS_36.md#decision-32).
- Recovery khi build/publish bị ngắt.

**Exit Criteria:** Preview và EPUB hoàn chỉnh dùng cùng bộ dữ liệu đã chuẩn bị. Mỗi lần xuất thành công giữ các phiên bản trước. Ảnh thay khớp nhưng hỏng chặn xuất và chỉ rõ file cần sửa. Xuất chỉ ghép nội dung đã được chấp nhận.

<a id="phase-08"></a>

### Phase 08 — Complete TUI & CLI Experience

**Kế hoạch chi tiết:** [PHASE-08.md](../roadmap/phases/PHASE-08.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

**Công việc:**

- Hoàn thiện các hành vi còn lại của tám màn theo [bảng gói giao diện](#ui-work-packages), gồm các trạng thái lỗi hoặc chờ người dùng.
- Hoàn thành UI-14 và UI-15. Chia kiểm thử theo màn và các luồng qua nhiều màn, gồm Reviewer → Editor → Reviewer, xem tác vụ đang chạy và resize khi có bản đang soạn.
- Hoàn thiện CLI commands, flags, JSON và exit codes theo [TUI_CLI_SPEC.md](../spec/TUI_CLI_SPEC.md). Kiểm thử CLI và TUI dùng cùng quy tắc duyệt, ngân sách và quyền chạy.

**Exit Criteria:** 8 màn đáp ứng acceptance tests; CLI/TUI cùng Application Core; không tự gọi AI khi chuyển màn, resize, cuộn; không mất bản thảo.

<a id="phase-09"></a>

### Phase 09 — Hardening, Acceptance & Release

**Kế hoạch chi tiết:** [PHASE-09.md](../roadmap/phases/PHASE-09.md), theo cấu trúc 16 mục; trạng thái thực thi chưa bắt đầu.

**Công việc:**

- Crash/Recovery/Database integrity.
- Dependency regression và contract tests.
- EPUB 2/3 fixtures và EPUBCheck.
- Security/credential redaction/Preview isolation.
- TUI resize, CLI non-interactive và process coordination.
- Nâng cấp workspace Rust có sao lưu, kiểm tra toàn vẹn và phục hồi. Với workspace thử nghiệm Go, tạo workspace Rust mới và import lại EPUB theo [Quyết định 06](../spec/DESIGN_DECISIONS_36.md#decision-06). PR thay đổi schema mô tả đường chuyển tiếp và tác động tương thích.
- Nghiệm thu chất lượng bằng bộ đối chứng 30 tình huống Nhật–Việt và 3 chương ngắn tự soạn. Đánh giá model thật thực hiện thủ công theo ngân sách do người dùng chọn.
- Packaging, release checks, README và ghi nhận feature parity trước khi thay thế đường chạy Go.

**Exit Criteria:** Các ca nghiệm thu thuộc phạm vi phát hành và bộ lệnh Rust bắt buộc đều đạt. Có bằng chứng phục hồi tại các điểm ngắt được kiểm thử, kiểm chứng bộ đối chứng chất lượng, EPUBCheck đạt và mẫu EPUB đọc được trên các trình đọc theo đặc tả. Bàn giao kết quả kiểm thử và tài liệu sử dụng theo [Quyết định 36](../spec/DESIGN_DECISIONS_36.md#decision-36).

---

## 5. Template chuẩn cho `PHASE-XX.md`

Mỗi phase được viết chi tiết theo cấu trúc 16 mục đã thống nhất. [Bản mẫu Phase 01](../roadmap/phases/PHASE-01.md) áp dụng cấu trúc này; các lựa chọn triển khai trong bản mẫu vẫn cần được ghi nhận theo trạng thái ADR.

| Mục | Nội dung |
|---|---|
| 1. Thông tin phase | ID, trạng thái, milestone, vị trí và người thực hiện/rà soát |
| 2. Mục tiêu và demo cuối phase | Kết quả quan sát được và luồng chạy chứng minh |
| 3. Phạm vi | Phần làm đến mức nào; phần chuyển sang phase sau |
| 4. Điều kiện bắt đầu | Contract, quyết định, môi trường và dữ liệu đầu vào cần có |
| 5. Đặc tả và quy tắc phải giữ | Nguồn yêu cầu, bất biến và cách xử lý khác biệt |
| 6. ADR và điểm còn mở | Lựa chọn cần ghi nhận, trạng thái và task bị ảnh hưởng |
| 7. Thiết kế triển khai | Ownership, module/crate, API và luồng dữ liệu/sự kiện |
| 8. Dữ liệu, lỗi và phục hồi | Vòng đời dữ liệu, side effect, lỗi, cleanup/retry và tương thích |
| 9. Phần TUI/CLI | Thao tác, trạng thái hiển thị và gói UI liên quan |
| 10. Gói việc và thứ tự | Đầu ra từng gói, task và quan hệ phụ thuộc |
| 11. Thẻ task cho AI | Mục tiêu, phạm vi, đầu ra, ca lỗi, acceptance và lệnh kiểm chứng |
| 12. Tiêu chí nghiệm thu | AC có ID, hành vi quan sát được, task và bằng chứng |
| 13. Ma trận kiểm thử | AC → ca kiểm thử → fixture → kết quả → lệnh chạy |
| 14. Rà soát chất lượng code | A tự review; B review độc lập đúng candidate, contract, kiến trúc, lỗi, bảo mật, test và phạm vi diff |
| 15. Điều kiện đóng phase | Bằng chứng tích hợp/checks và review B Đạt trên candidate cuối; không còn lỗi chặn hoặc thiếu kiểm tra bắt buộc |
| 16. Bàn giao và bằng chứng | Báo cáo A/B riêng theo lượt, baseline/candidate, findings, trạng thái thực tế và đầu vào cho phase sau |

Ví dụ **Phase 02** không thể chỉ ghi “Implement EPUB Import”; cần chia: safe ZIP parsing, OPF/spine, XHTML/ruby/notes, content classification, stable SourceBlock IDs, storage, import diagnostics, recovery và fixtures.

---

## 6. Template Implementation Task cho AI coding agent

**Không** giao nhiệm vụ “hãy xây dựng toàn bộ Translation & Review” trong một lượt mà không phân rã.

### 6.1. Nội dung một task

| Thành phần | Mô tả |
|---|---|
| Task ID | Mã duy nhất |
| Goal | Kết quả cần đạt |
| Requirements | Business decisions/specs áp dụng |
| References | File/đoạn đặc tả cần đọc |
| Scope | Module/files được phép thay đổi |
| Non-goals | Không được triển khai |
| Dependencies | Task/đầu ra/phiên bản đã review; checkpoint đã công bố khi nhận ngoài phase |
| Shared Contract | Dữ liệu vào/ra, lỗi, invariant, version và owner |
| Ownership | File/module được sửa, worktree/branch riêng và người giữ shared files |
| Parallel Tasks | Task có thể chạy cùng và điều kiện mở |
| Isolated Checks | AC/test riêng cùng oracle và ca lỗi |
| Integration / Checkpoint | Bên nhận, ca chạy lại, checkpoint 1–3 task; B Đạt thì commit/push ngay |
| Implementation Notes | Ràng buộc kiến trúc |
| Acceptance Criteria | Kiểm tra hành vi |
| Required Tests | Unit/integration/contract/acceptance |
| Verification | Các lệnh kiểm tra |
| Handoff Report | A ghi baseline/candidate, kết quả, file mới/thay đổi, AC, test, vấn đề và gửi Chờ review |
| Independent Review | B ghi kiểm chứng riêng, findings có căn cứ, kết luận và phạm vi candidate |
| Rework / Closure | A xử lý findings; B kiểm lại; chỉ đóng khi đủ nghiệm thu và review B Đạt |

### 6.2. Ví dụ

**Task:** [P02-T04 — Safe ZIP và giới hạn tài nguyên](../roadmap/phases/PHASE-02.md#p02-t04).

**Goal:** Đọc ZIP trong giới hạn đã chốt, xử lý đường dẫn/entry độc hại an toàn theo đặc tả EPUB đầu vào.

**Scope:** EPUB ZIP boundary, kiểm giới hạn, diagnostic và fixture tự sinh.

**Non-goals:** Chưa triển khai Translation/Review hoặc parser OPF/XHTML của task kế tiếp.

**Acceptance:** P02-AC04/P02-TC04: fixture dưới/đúng/trên giới hạn và đường dẫn độc hại có diagnostic đúng; file sentinel ngoài vùng stage không thay đổi.

A bàn giao bản cụ thể và test thực chạy; B kiểm độc lập theo yêu cầu, kể cả ca biên/đường lỗi. Task chỉ hoàn thành khi đủ AC/bằng chứng và review B Đạt cho đúng candidate.

---

## 7. Quy tắc thực thi trong `AGENTS.md`

[AGENTS.md hiện có](../../AGENTS.md) quy định nhánh, tính tương thích workspace, đường chạy Go, kiểm tra Rust và quy trình A/B. Khi triển khai, áp dụng các ràng buộc sản phẩm và kiến trúc sau cùng task/ADR tương ứng:

1. Không thay đổi 36 quyết định đã chốt khi chưa được phép.
2. Không tạo dependency ngược Application → Infrastructure hoặc Domain → framework.
3. Không sửa schema đã áp dụng mà thiếu migration.
4. Không bỏ validation/Review để “test xanh”.
5. Không tự thêm model fallback, tự tăng ngân sách/call cap.
6. Không xóa revision/history hoặc sửa dữ liệu bất biến.
7. Không tự thêm màn TUI chính ngoài tám màn đã chốt.
8. Không mở rộng task ngoài phạm vi mà không nêu rõ và xin quyết định khi cần.
9. Khi đặc tả mâu thuẫn hoặc thiếu, ghi issue/ADR, dừng phần bị ảnh hưởng thay vì tự suy diễn.
10. A bàn giao phải báo candidate, file đã sửa/mới, test đã chạy/chưa chạy, contract changes và rủi ro còn lại; giữ Chờ review.
11. B kiểm độc lập và ghi báo cáo riêng; A sửa findings, B kiểm lại. Không dùng tự review thay review độc lập.
12. B không sửa source/test chính thức/PLAN trong lượt review; A cập nhật trạng thái theo kết luận có căn cứ của B. Candidate đổi thì đánh giá lại review theo ảnh hưởng.

`AGENTS.md` ghi **nguyên tắc bền vững**; không nên sao chép toàn bộ roadmap hoặc mọi chi tiết nghiệp vụ vào file này.

---

## 8. Definition of Done (DoD)

| Tiêu chí | Điều kiện |
|---|---|
| Implementation | Đúng phạm vi đã giao |
| Architecture | Không vi phạm dependency/ownership |
| Unit Tests | Kiểm thử quy tắc liên quan |
| Integration Tests | Kiểm thử tại storage/adapter boundary nếu thay đổi |
| Error Handling | Không bỏ đường lỗi quan trọng |
| Documentation | Cập nhật khi contract đổi |
| Compatibility | Không phá dữ liệu/workflow hiện tại |
| Verification | Lệnh phù hợp với phạm vi task đã chạy. Trước PR Rust, áp dụng đủ [bộ lệnh trong AGENTS.md](../../AGENTS.md#required-checks). |
| Review | B Đạt cho đúng candidate sau khi kiểm AC/code/test; không còn finding chặn hoặc thiếu bằng chứng bắt buộc |
| Phase Gate | Review B trên bản tích hợp cuối, gồm checks/demo/thủ công bắt buộc; mỗi cổng M0 chỉ đóng đúng phạm vi đã review |

**Compile thành công không đồng nghĩa task hoàn thành.** Các yêu cầu Recovery, Revision, Coordinator và Workflow cần chứng minh bằng test tương ứng.

---

## 9. Quản lý tiến độ ba cấp

```text
ROADMAP
    ├── PHASE 01
    │     ├── Task 01
    │     ├── Task 02
    │     └── Task 03
    ├── PHASE 02
    │     ├── Task 01
    │     ├── Task 02
    │     └── Task 03
    └── ...
```

- **ROADMAP:** Toàn bộ hành trình, milestone và phụ thuộc.
- **PHASE:** Mục tiêu/gói công việc trong một giai đoạn.
- **TASK:** Đơn vị nhỏ AI coding agent thực thi được.

`PLAN.md` cập nhật phase/task hoặc nhóm nhỏ đang hoạt động, vai A/B, candidate, trạng thái, link báo cáo và bước tiếp theo. Trạng thái: Chưa bắt đầu → Đang làm → Chờ review → Hoàn thành; nếu B yêu cầu sửa thì Cần sửa → Đang làm → Chờ review lượt mới. Thiếu kiểm tra bắt buộc ghi Chờ kiểm chứng/Bị chặn. Giữ lịch sử hai báo cáo theo lượt; không sao chép nguyên roadmap vào PLAN.

Có thể ánh xạ task sang GitHub Issues và Phase sang milestone, nhưng không bắt buộc khi đang quản lý bằng Markdown.

---

## 10. Định hướng mở rộng về sau

Mỗi Phase cần xem xét **Architecture Evolution**:

| Thay đổi tương lai | Kiến trúc cần hỗ trợ |
|---|---|
| Thêm AI provider | Adapter/config mới nếu contract hiện có đủ |
| Thêm search provider | Không viết lại workflow chính |
| Thêm GUI | Không sao chép Translation/Review logic |
| Thay SQLite driver | Không kéo thay đổi vào Domain |
| Thêm export format | Tái sử dụng prepared content khi phù hợp |
| Thêm Review strategy | Giữ revision/source/evidence invariants |
| Thay workflow | Versioning và checkpoint migration nếu cần |

Mỗi contract mới phải có consumer và nhu cầu thật. Không tạo abstraction/plugin framework vô căn cứ.

---

## 11. Hai mức độ chi tiết của kế hoạch

Không khóa sớm toàn bộ hàng trăm task cho cả Phase 00–09.

Theo yêu cầu ngày 06/11/2026, đã viết [kế hoạch chi tiết cho cả 10 phase](../roadmap/ROADMAP.md). Đây là phân rã ban đầu để rà soát và giao việc; trước mỗi task vẫn đối chiếu đầu ra dependency đã công bố, ADR và các thay đổi contract rồi cập nhật task bị ảnh hưởng theo lịch song song; cổng cuối phase giữ đủ điều kiện nghiệm thu. Viết chi tiết không đồng nghĩa khóa cứng cách triển khai hoặc đánh dấu task hoàn thành.

| Mức | Độ chi tiết |
|---|---|
| **Toàn dự án** | Phase, milestone, phụ thuộc và mục tiêu |
| **Phase sắp triển khai** | Task, acceptance, tests, phạm vi file/code |

Khi một vertical slice cho thấy giới hạn schema/contract, có thể cập nhật task của phase tương lai mà không phá mục tiêu dài hạn.

**Roadmap ổn định về đích, linh hoạt về phương án thực thi.**

---

## 12. Deliverables của Bước 9

| Tài liệu | Vai trò |
|---|---|
| `IMPLEMENTATION_STRATEGY.md` | Phương pháp triển khai/đánh đổi |
| `docs/roadmap/ROADMAP.md` | Phase, milestone, dependencies |
| `AGENTS.md` | Rà soát và bổ sung quy tắc triển khai vào file hiện có |
| `PLAN.md` | Kế hoạch và trạng thái hiện hành, vai tiếp theo và bản đang review |
| [REVIEW_WORKFLOW.md](../roadmap/REVIEW_WORKFLOW.md) | Phân vai A/B, candidate, vòng sửa/review và cổng nghiệm thu |
| [TASK_HANDOFF](../roadmap/templates/TASK_HANDOFF.md), [TASK_REVIEW](../roadmap/templates/TASK_REVIEW.md), [PHASE_REVIEW](../roadmap/templates/PHASE_REVIEW.md) | Mẫu báo cáo riêng cho bàn giao A, review task B và review phase B |
| `docs/roadmap/phases/PHASE-XX.md` | Kế hoạch phase và điều kiện nghiệm thu có review độc lập |
| `docs/decisions/ADR-XXX.md` | Quyết định kỹ thuật |
| `DEFINITION_OF_DONE.md` | Quy tắc nghiệm thu task/phase |

**Tài liệu này là bản thiết kế Bước 9.** Đã có [ROADMAP.md và kế hoạch Phase 00–09](../roadmap/ROADMAP.md), gồm dependency, UI, bảng truy vết 36 quyết định và task/AC/kiểm thử của từng phase. [PLAN.md](../../PLAN.md) đã nhận M0/M1 Đạt trên Windows và PR Phase 01 đã merge; kế hoạch tiếp theo là Phase 02–09 theo task/checkpoint v2. [AGENTS.md](../../AGENTS.md) và quy trình review đã đồng bộ vai chat/điều phối. ADR/spec tiếp tục giữ quyết định đã duyệt; code các phase sau chưa được thực hiện trong lượt kế hoạch này.

---

## 13. Trình tự thực hiện sau khi phê duyệt Bước 9

| Thứ tự | Công việc | Kết quả |
|---|---|---|
| 1 | Đối chiếu Bước 1–9 với đặc tả | Dùng tham chiếu đã đồng bộ và ghi các quyết định kỹ thuật còn mở |
| 2 | Chốt ADR chặn phase sắp triển khai | Có lựa chọn kỹ thuật và điều kiện trước cho từng task |
| 3 | Xây Roadmap | Có Phase/milestone rõ |
| 4 | Rà soát AGENTS.md hiện có | AI có quy tắc thực hiện và bộ lệnh kiểm tra |
| 5 | Viết và rà chi tiết phase sắp thực hiện | Đã có Phase 00–09; điều chỉnh task theo bằng chứng và contract mới |
| 6 | Tạo PLAN.md | Có kế hoạch hiện hành |
| 7 | Bắt đầu coding | Triển khai từng task với acceptance/DoD |

M1 đã Đạt và merge. [Lịch song song v2 và lộ trình bỏ Go đã B r04 Đạt](../roadmap/evidence/planning/P02-P09-parallel-v2.review-r04.md). Bước tiếp là người dùng giao chat P02 và P09 chuẩn bị; điều phối xác minh nền và cấp worktree/inputs/ownership trước code. Các phase còn lại mở theo task dependency. Mỗi checkpoint 1–3 task Đạt được commit/push ngay, sau đó điều phối merge và hậu kiểm theo [quy trình hiện hành](../roadmap/MERGE_WORKFLOW.md); cổng cuối vẫn giữ DoD đầy đủ.

---

## Kết luận

**Bước 9 = Implementation Strategy + Development Roadmap + AI Coding Governance.**

Nó phải trả lời:

1. **Xây gì, theo thứ tự nào?** — Roadmap và Phase.
2. **AI triển khai từng phần thế nào?** — AGENTS.md, PLAN.md, Task Specs.
3. **Làm sao chứng minh đúng và không phá kiến trúc?** — Acceptance Criteria, DoD, Contract/Regression Tests.

Định hướng là **Vertical Slice và kiểm chứng luồng cốt lõi từ sớm**, không triển khai tất cả bảng SQLite, Domain hoặc TUI trong một lần.

**Trạng thái hiện hành:** M0-r03 và M1-r01 đã B Đạt trên Windows; Phase 01 đã merge. Kế hoạch Phase 02–09 có lịch song song v2 và checkpoint công bố sớm; verdict kế hoạch và bước nhận việc nằm tại [PLAN](../../PLAN.md). Phase 02–09 chưa triển khai. Quyết định mở và kiểm chứng consumer tương lai vẫn phải hoàn thành trước task phụ thuộc.

## Bổ sung chuyển hết sang Rust — 12/11/2026

Mỗi checkpoint rà phần Go liên quan và dọn theo capability Rust đã kiểm, caller đã chuyển và allowlist được cấp. Theo [PLAN §8](../../PLAN.md#go-retirement-plan), lượt dọn nhỏ có kiểm chứng sau xóa, B review và commit/push ngay; phần còn cần có owner/task/mốc xử lý. Sau M9 còn GO-FINAL bắt buộc để build/test/chạy/đóng gói trên môi trường không cần Go, kèm artifact mới và review đúng candidate. Giữ lịch sử Go và dữ liệu người dùng.
