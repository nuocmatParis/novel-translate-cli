# BƯỚC 5 — TECHNICAL FOUNDATION & INTERFACE CONTRACTS

**Dự án:** `novel-translate-cli`  
**Phiên bản:** Draft 0.2  
**Ngày:** 07/11/2026  
**Trạng thái:** ĐỀ XUẤT — chưa phê duyệt toàn bộ quyết định kỹ thuật  
**Tiền đề:** Bước 1 — Architecture Overview; Bước 2 — Subsystem Architecture; Bước 3 — Rust Project Architecture; Bước 4 — Rust Module Design (Extensibility, Maintainability & Evolvability).  
**Nguồn yêu cầu đã chốt:** [DESIGN_DECISIONS_36.md](../spec/DESIGN_DECISIONS_36.md) và các đặc tả liên quan. Những thư viện và tên trait bên dưới khởi tạo là **phương án kỹ thuật đề xuất**; phần hướng nền tảng đã thống nhất và chi tiết còn mở được phân biệt tại mục 13 và ADR tương ứng.

---

## 1. Mục tiêu

Sau Bước 4, kiến trúc đã có crate và module boundaries. Bước 5 xác định những lựa chọn nền tảng và các hợp đồng giao tiếp đủ rõ để chuẩn bị triển khai mà vẫn bảo vệ khả năng thay đổi về sau.

**Phần A — Technical Foundation:** Rust toolchain, async runtime, framework TUI/CLI, SQLite driver, serialization, logging, HTTP client, IPC, credential và cơ chế xử lý tác vụ blocking.  
**Phần B — Interface Contracts:** các port, kiểu dữ liệu trao đổi, typed errors, transaction/version guard, cancellation, timeout, idempotency và contract testing.

Điều kiện xuyên suốt: một tác vụ AI hoạt động trên máy theo 13A, không tự đổi model, không tự duyệt glossary, Revision/Review/generation có lịch sử và có thể phục hồi.

## 2. Technical Foundation — phương án đề xuất

| Thành phần | Công nghệ đề xuất | Mục đích/lý do |
|---|---|---|
| Rust | Stable + toolchain được pin trong repo | Tái lập build, xác định MSRV |
| Async runtime | **Tokio** | HTTP streaming, timer, IPC, event processing |
| TUI | **Ratatui + Crossterm** | Terminal UI keyboard-first |
| CLI | **Clap** | Subcommands và help/argument parsing |
| Database | **SQLite + rusqlite** | Kiểm soát chặt transaction, kết nối, migration |
| Serialization | **Serde + serde_json** | Request JSON, CLI JSON và metadata |
| Domain/application errors | **thiserror** | Typed and structured errors |
| Bootstrap/binary errors | **anyhow** | Lỗi có ngữ cảnh tại ranh giới entrypoint |
| HTTP | **reqwest** | Provider/Research transport |
| Tracing | **tracing + tracing-subscriber** | Structured logs, request/attempt correlation và redaction |
| Windows Coordinator IPC | **Named Pipes** | Giao tiếp giữa CLI/TUI và coordinator trên máy |
| Credentials | OS Credential Store adapter; env được user chọn | Không ghi key vào DB/Markdown/log |
| Async interfaces | Native async trait hoặc boxed future/`async-trait` tùy consumer | Không trừu tượng hóa quá mức |

**Chưa chốt:** phiên bản crate/MSRV; Windows-only hay đa nền tảng trong release đầu; lifecycle Coordinator; lựa chọn rusqlite thay SQLx; hình thức async trait, IPC framing/auth; các chi tiết này phải được ghi ADR.

### 2.1. Tokio không thay thế chính sách Coordinator

Tokio giúp không chặn event loop trong lúc chờ HTTP, retry, timer và IPC; nhưng việc runtime cho phép concurrency **không** cấp quyền chạy nhiều AI task cùng lúc. Coordinator vẫn cấp đúng **một AI permit** trên user profile.

Các thao tác SQLite và xử lý ZIP/EPUB có tính blocking/CPU-bound phải chạy ngoài async event loop; một blocking job đã bắt đầu không bảo đảm có thể abort như async task. Cancellation cần được kiểm tra qua checkpoint và quyền công bố dữ liệu, không chỉ qua `JoinHandle::abort`.

### 2.2. Vì sao nghiêng về rusqlite

**[ĐỀ XUẤT]** Tokio + rusqlite + database writer worker cho mỗi Series SQLite và worker Coordinator riêng.

```text
Application/Workflow
    │
    ▼
Repository Port (async-facing where necessary)
    │
    ▼
Storage Adapter
    │
    ▼
Dedicated SQLite writer worker
    │
    ▼
Series SQLite
```

Ưu điểm: giao dịch nhiều bước dễ kiểm soát, quyền ghi tập trung, có điểm dừng để kiểm tra generation. Đánh đổi: cần hàng đợi lệnh giữa async và database worker, xử lý lỗi worker và shutdown/recovery. Reader connection/pool có thể tách khỏi writer nếu đo/test chứng minh cần.

**[CÒN MỞ]** so sánh với SQLx qua prototype đo transaction semantics, migration, cancellation và độ phức tạp; không quyết định chỉ dựa vào tốc độ benchmark.

### 2.3. Đồng bộ và bất đồng bộ

| Hoạt động | Ranh giới dự kiến |
|---|---|
| Domain rules, severity, budget calculations | Đồng bộ |
| Dependency graph evaluation thuần | Đồng bộ; có thể đưa ra worker nếu tốn CPU |
| Repository calls từ use case | Async-facing khi cần; adapter có worker SQLite blocking |
| Model request/streaming | Bất đồng bộ |
| Research Internet | Bất đồng bộ |
| Coordinator IPC | Bất đồng bộ |
| EPUB parse/ZIP packaging | Blocking/CPU worker, giới hạn tài nguyên |
| TUI render | Đồng bộ trong event loop, không gọi API trực tiếp |

**Nguyên tắc trait:** dùng native async trait cho static dispatch khi phù hợp; boxed futures/`async-trait` nếu cần `dyn Trait`, cancellation và object safety. Domain không phụ thuộc Tokio.

## 3. Interface Catalog — hợp đồng logic

| Port | Consumer | Implementation đề xuất |
|---|---|---|
| `WorkspaceRepository` | Workspace use cases | `nt-storage` |
| `SourceRepository` | Translation/Review/Export | `nt-storage` |
| `ContextRepository` | Context/Skills | `nt-storage` |
| `TranslationRepository` | Translation/Review/Export | `nt-storage` |
| `ReviewRepository` | Review/Workflow/Export | `nt-storage` |
| `TaskRepository` | Workflow | `nt-storage` |
| `ResearchRepository` | Research | `nt-storage` |
| `GenerationCommitPort` | Application | `nt-storage` |
| `ImmutableFileStore` | Import/Translation/Export | `nt-storage` |
| `ModelGateway` | Translation/Review/Research | `nt-model` |
| `SearchGateway` | Research | `nt-research` |
| `EpubImporter` | Import use case | `nt-epub` |
| `CoordinatorGateway` | Workflow | `nt-coordinator` |
| `ExportGateway` | Export use cases | `nt-export` |
| `Clock`, `IdGenerator` | Time/ID-dependent use cases | Production/test adapters |

Đây là **catalog logic**, chưa bắt buộc mỗi tên thành một Rust trait. Port chỉ được hình thành khi có consumer và ranh giới thay đổi/kiểm thử thực tế.

### 3.1. Mọi port có hợp đồng tối thiểu

- Input/output là typed DTO/domain references, không phải raw SQLite row hay HTTP response.
- Các ID và version nền phải rõ; ghi dữ liệu cần check base generation.
- Điều kiện trước/sau, lỗi, idempotency key khi có retry, và giới hạn scope.
- Cách ghi request/attempt/usage trong trường hợp timeout hoặc response muộn.
- Sự kiện trạng thái chính thức chỉ được phát sau commit.

## 4. ModelGateway

**Input logic:** `request_id`, `attempt_id`, `role` (Translation/Review/Research), provider/model đã chọn, structured content parts, output contract, timeout, capability/pricing snapshot và dependency version.

**Output logic:** normalized response/stream completion, actual model identity nếu có, finish reason, usage, classified error, provider retry-after, trạng thái complete/truncated/unknown và metadata được phép lưu.

- Streaming partial không phải Revision.
- Adapter không tự fallback hoặc tự chọn model mới.
- Network retry (tối đa 2 ngoài lần đầu), structural repair (tối đa 2) và quality repair (tối đa 5 vòng thất bại) là bộ đếm khác nhau.
- Request đã gửi nhưng chưa biết provider xử lý được giữ `Chưa rõ đã xử lý`; usage/cost chưa rõ không thành số 0.
- Attempt đến muộn phải giữ lịch sử nhưng không ghi đè revision mới.
- Credential do adapter lấy từ OS Credential Store hoặc env do user chỉ định; không lưu secret trong DB/task/markdown.

**Extension:** provider mới hiện thực cùng contract và chạy mock/contract tests; không sửa Translation/Review Domain.

## 5. GenerationCommitPort

```text
CommitGeneration
  Input:
    base_generation
    validated_domain_changes
    dependency_changes
    immutable_file_references
    state_transitions
  Output:
    committed_generation
    OR stale_generation / conflict / storage_error
```

- Workspace/Domain owner quyết định nội dung thay đổi; Persistence thực hiện giao dịch.
- File bất biến phải được hoàn tất và hash-verified **trước** khi SQLite tham chiếu.
- Revision, Review references, decision, dependency, progress, metadata và current pointer liên quan công bố trong **một SQLite transaction**.
- Markdown projection sau commit có thể thất bại và phục hồi; không vì thế rollback thành quả đã commit.
- Batch approval Skill cần giao thức intent/hash/file publish riêng; không tự kích hoạt nếu file chưa đồng bộ.
- Tránh cho nhiều repository tự commit từng phần khi cần atomicity toàn nghiệp vụ.

## 6. CoordinatorGateway

**Các operation logic đề xuất:** `enqueue`, `request_permit`, `get_status`, `request_pause`, `release_permit`, `resume`, `reconcile`.

Hợp đồng mang `task_id`, `workspace_id`, `permit_sequence`, command/request ID và trạng thái kết quả. Chỉ một task giữ quyền AI; task B không được gửi khi A chưa bàn giao hợp lệ. Retry wait của A không tự giải phóng permit. Response muộn của A không ghi đè state hiện hành.

### 6.1. Mô hình tiến trình đề xuất

```text
TUI ──────┐
CLI ──────┤───> Coordinator process (IPC + OS lock)
GUI (sau)┘             │
                    Coordinator SQLite
                         │
                    Series task ledgers
```

**[ĐỀ XUẤT]** Coordinator là tiến trình cục bộ riêng, khởi động khi cần; Windows Named Pipes làm transport đầu tiên. Cần định nghĩa IPC auth/ACL, framing, protocol version, idempotency, reconnect và crash recovery.

**[CÒN MỞ]** daemon/hosted lifecycle, shutdown behavior, Windows-first vs cross-platform, durable ack/thứ tự ghi giữa Series và Coordinator SQLite. Không giả định distributed transaction.

## 7. EpubImporter

**Input:** EPUB source reference, volume ID, parser version, import options và giới hạn tài nguyên.  
**Output:** SourceSnapshot candidate, package/reading order, source blocks/ranges, ruby/inline/note/image/link mappings, resource references, diagnostics và unresolved classification.

Không rút về plain text làm mất semantics. Không gọi AI trong Import. Source IDs giữ ổn định theo snapshot/parser/path/node; chia chunk không làm đổi block IDs. Mở rộng TXT/DOCX/PDF tương lai có thể buộc mở rộng source model — không cam kết chỉ thêm parser.

## 8. ExportGateway

Ba operation logic:

1. **Prepare:** kiểm tra full-volume readiness, đóng gói snapshot/revision/Review/scope/metadata/ruby/ảnh thành `PreparedExportBundle` bất biến.
2. **Preview:** phục vụ bản chuẩn bị trên loopback theo session/token/resource allowlist; không truy cập file tùy ý.
3. **Publish:** dựng EPUB3.3 từ bundle, kiểm tra EPUBCheck 5.3.0, cấp version mới và publish không overwrite.

Preview và Export của cùng lần chuẩn bị phải đọc **cùng bundle**; bundle stale khi dependency/metadata/ảnh đổi. Export không được gọi model để tự dịch hoặc vượt qua Review.

## 9. Error Handling

| Loại | Ví dụ | Biện pháp |
|---|---|---|
| Domain | InvalidTransition, RuleConflict, MissingCoverage | Giữ invariants; typed reason |
| Application | UserDecisionRequired, BudgetBlocked, Queued, StaleGeneration | Trả next action, không đoán kết quả |
| Infrastructure | SQLite, filesystem, provider HTTP, parser, IPC | Normalize error và redaction |
| Presentation | CLI JSON/exit code, TUI message | Không rò raw provider/DB/secret |

Không gom tất cả vào `String`. Application mapping từ lỗi cụ thể sang trạng thái task; CLI giữ exit codes/JSON fields đã chốt trong `TUI_CLI_SPEC`.

## 10. Compatibility và Versioning

| Boundary | Version/identity |
|---|---|
| Workspace | `schema_version` (Rust mới bắt đầu 1) |
| Source | EPUB hash, parser version |
| Mapping | Mapping version |
| Translation | Revision ID và dependency versions |
| Review | Review run và exact revision set |
| Skills | Rule ID/version/scope |
| Model | Capability/pricing version, requested/actual model |
| Workflow | Task/checkpoint/attempt identity |
| Coordinator | IPC protocol version **[ĐỀ XUẤT]** |
| Export | Immutable PreparedBundle/manifest version |
| CLI | JSON `schema_version` |

Thay implementation có thể giữ nguyên contract. Đổi ý nghĩa field/state/ID là thay đổi breaking và cần plan compatibility/migration. Dữ liệu Rust workspace mới phải được bảo toàn qua các lần nâng cấp tương lai; **không** yêu cầu migrate workspace test Go cũ.

## 11. Extensibility — tác động thay đổi

| Kịch bản | Nơi nên thay đổi | Không mong muốn kéo theo |
|---|---|---|
| Thêm provider | `nt-model` adapters/capability tests | Translation/Review Domain |
| Thêm search engine | `nt-research` | Workflow chính |
| Đổi SQLite driver | `nt-storage` | Domain/use case semantics |
| Thêm GUI | Presentation adapter mới | Business logic và Coordinator policy |
| Thêm output format | Export adapter và có thể prepared-content contract | Dịch lại khi content không đổi |
| Thêm input format | Import adapter **có thể** cùng Source model changes | Provider logic không liên quan |
| Thêm ngôn ngữ | Domain/Context/Translation/Review có thể thay đổi | Không cam kết plug-and-play |
| Thay IPC | Coordinator transport | Workflow Domain |
| Thêm Review strategy | Review policy/version | Bỏ qua source-first/evidence không được phép |

Không xây generic plugin framework/dynamic loading hoặc workflow DSL khi chưa có nhu cầu đã xác nhận.

## 12. Testing kiến trúc

- Domain unit tests không phụ thuộc framework.
- Application tests với mock ports, không gọi provider trả phí.
- Contract tests cho repository/model/search/coordinator/export adapters.
- SQLite atomic-commit, write-lock, stale-generation và rollback tests.
- Coordinator IPC/multiple-process/crash reconciliation tests.
- Timeout, cancellation, unknown provider state và late-response tests.
- PreparedBundle/Preview/EPUBCheck tests.
- Migration/schema/CLI JSON compatibility tests.
- Architecture dependency tests (cấm cycle, cấm Domain→adapter, UI→DB).
- Extension regression: thêm adapter mới không làm thay use case cũ.

## 13. Trạng thái các quyết định kỹ thuật

Ngày 07/11/2026, [ADR-001](../decisions/ADR-001.md) ghi hướng Rust stable có pin, Ratatui/Crossterm, Clap, tracing/subscriber và kiểm thử Windows/Linux đã thống nhất; [ADR-008](../decisions/ADR-008.md) ghi ranh giới bốn crate và hướng shell. Phiên bản/features, API cụ thể, runtime/CLI/terminal contract và bằng chứng còn được hoàn thiện ở M0-01. Bảng dưới vẫn là danh mục quyết định theo consumer; không có nghĩa toàn bộ đề xuất Bước 5 đã được thông qua hoặc kiểm chứng.

| ID | Đề xuất | Độ ưu tiên |
|---|---|---|
| TD-01 | Rust Stable + pin toolchain | Cao |
| TD-02 | Tokio runtime | Cao |
| TD-03 | Ratatui + Crossterm | Cao |
| TD-04 | Clap | Cao |
| TD-05 | Rusqlite + controlled database worker | **Rất cao** |
| TD-06 | Serde + serde_json | Cao |
| TD-07 | thiserror; anyhow tại bootstrap | Trung bình |
| TD-08 | reqwest | Cao |
| TD-09 | tracing + subscriber | Cao |
| TD-10 | Windows Named Pipes IPC | **Rất cao** |
| TD-11 | Coordinator tiến trình riêng, on-demand | **Rất cao** |
| TD-12 | Application định nghĩa ports, adapters hiện thực | Rất cao |
| TD-13 | Không kéo async/framework vào Domain | Rất cao |
| TD-14 | Typed contracts, errors, version checks | Rất cao |
| TD-15 | Version contracts/persistence/IPC khi cần | Cao |
| TD-16 | Không dynamic plugin framework trong v1 | Cao |

**Lưu ý:** TD-05, TD-10 và TD-11 cần prototype và recovery tests trước khi khóa ADR cuối.

## 14. Tiêu chí nghiệm thu Bước 5

- [ ] Bộ công nghệ nền tảng được đánh giá/phê duyệt.
- [ ] Có quyết định có căn cứ về rusqlite/SQLx và DB worker.
- [ ] Có quyết định Coordinator lifecycle/IPC.
- [ ] Port Catalog được rà soát theo consumer.
- [ ] Hợp đồng chính có input/output, typed errors, version guard.
- [ ] Có quy tắc đồng bộ/async, timeout và cancellation.
- [ ] Có versioning/compatibility và migration strategy.
- [ ] Có contract tests độc lập cho adapter.
- [ ] Không vi phạm 36 quyết định đã chốt.
- [ ] Không có cycle hoặc direct UI→DB/provider dependencies.

## 15. Lộ trình tiếp theo (đề xuất)

| Bước | Chủ đề |
|---|---|
| **Bước 5** | Technical Foundation & Interface Contracts |
| [**Bước 6**](DATA_ARCHITECTURE_STEP6.md) | Data Architecture & SQLite Schema Design |
| [**Bước 7**](WORKFLOW_STATE_MACHINE_STEP7.md) | Workflow State Machine & Recovery Design |
| [**Bước 8**](UI_UX_DESIGN_STEP8.md) | TUI/CLI UI & UX Design |
| [**Bước 9**](IMPLEMENTATION_ROADMAP_STEP9.md) | Implementation Roadmap & AI Coding Plan |

Bước 9 gắn các mục TD với ADR và phase cần dùng quyết định đó. Trạng thái của từng lựa chọn kỹ thuật theo mục 13 và ADR tương ứng; nhiệm vụ thực thi hiện hành ở [PLAN.md](../../PLAN.md).

---

## Tài liệu tham chiếu

**Tài liệu dự án:** [DESIGN_DECISIONS_36.md](../spec/DESIGN_DECISIONS_36.md), [DATA_MODEL_STATE_SPEC.md](../spec/DATA_MODEL_STATE_SPEC.md), [WORKFLOW_ORCHESTRATION_SPEC.md](../spec/WORKFLOW_ORCHESTRATION_SPEC.md), [MODEL_STRATEGY_SPEC.md](../spec/MODEL_STRATEGY_SPEC.md), [TUI_CLI_SPEC.md](../spec/TUI_CLI_SPEC.md), [EPUB_INPUT_SPEC.md](../spec/EPUB_INPUT_SPEC.md), [EPUB_OUTPUT_SPEC.md](../spec/EPUB_OUTPUT_SPEC.md), [SECURITY_SPEC.md](../spec/SECURITY_SPEC.md).

**Tài liệu kỹ thuật:** [Tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [Tokio Named Pipes](https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/), [rusqlite Transaction](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html), [SQLite CREATE TABLE](https://www.sqlite.org/lang_createtable.html), [SQLite WAL](https://www.sqlite.org/wal.html).

**Kết luận:** Đây là bản thiết kế Bước 5 để thẩm định, không phải quyết định đã chốt toàn bộ công nghệ. Bước 6 cần xây dựng danh mục thực thể, bảng, FK/index, generation/transaction boundaries và migration theo đúng các bất biến đã xác nhận.
