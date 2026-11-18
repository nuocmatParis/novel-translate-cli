# RUST PROJECT ARCHITECTURE

**Dự án:** `novel-translate-cli`\
**Bước:** 03 --- Rust Project Architecture\
**Phiên bản:** Draft 0.1\
**Trạng thái:** Đề xuất kiến trúc --- chờ rà soát và xác nhận

------------------------------------------------------------------------

## 1. Mục tiêu và phạm vi

Bước 2 đã xác định 10 subsystem với trách nhiệm và ranh giới dữ liệu
riêng. Bước 3 chuyển các ranh giới logic đó thành kiến trúc Rust ở cấp
Cargo Workspace và crate.

Tài liệu trả lời: 1. Cargo workspace gồm những crate nào? 2. Mỗi crate
chịu trách nhiệm gì? 3. Subsystem được phân bổ vào crate nào? 4. Các
crate được phép phụ thuộc lẫn nhau ra sao? 5. Bên trong mỗi crate dự
kiến có module nào? 6. Trait/interface được định nghĩa ở đâu? 7. Làm sao
tránh circular dependency? 8. Cấu trúc thư mục toàn dự án ra sao? 9. CI
cần kiểm tra quy tắc kiến trúc nào?

**Phạm vi:** Thiết kế ở cấp workspace/crate và module breakdown sơ bộ.
Chưa phải thiết kế module chi tiết, chữ ký trait cuối cùng, SQLite
schema hay code triển khai.

### Quy ước trạng thái

  -----------------------------------------------------------------------
  Nhãn                                Ý nghĩa
  ----------------------------------- -----------------------------------
  `[ĐÃ CHỐT]`                         Yêu cầu/ràng buộc đã được xác nhận
                                      trong đặc tả hoặc quyết định dự án

  `[ĐỀ XUẤT]`                         Phương án kiến trúc đưa ra để
                                      review

  `[CÒN MỞ]`                          Cần quyết định ở bước thiết kế kỹ
                                      thuật
  -----------------------------------------------------------------------

Không tự động chuyển đề xuất thành quyết định đã chốt.

------------------------------------------------------------------------

## 2. Kiến trúc Rust tổng thể

**\[ĐÃ CHỐT\]** Dự án theo định hướng Modular Monolith, kết hợp Layered
Architecture và Hexagonal Architecture (Ports & Adapters).

**\[ĐỀ XUẤT\]** Tổ chức mã nguồn bằng Cargo Workspace, chia thành bốn
nhóm:

  -----------------------------------------------------------------------
  Nhóm                                Trách nhiệm
  ----------------------------------- -----------------------------------
  Domain                              Mô hình dữ liệu và quy tắc nghiệp
                                      vụ

  Application                         Use case, workflow orchestration và
                                      ports

  Infrastructure                      Hiện thực storage, provider, EPUB,
                                      research, export và coordinator

  Presentation                        CLI/TUI, tiếp nhận thao tác và
                                      trình bày kết quả
  -----------------------------------------------------------------------

Một subsystem có thể trải qua nhiều layer và nhiều crate. Không ánh xạ
máy móc một subsystem thành một crate.

### 2.1. Dependency direction

-   Domain không phụ thuộc Infrastructure.
-   Application phụ thuộc Domain.
-   Infrastructure hiện thực các contract/port mà Application cần.
-   Presentation gọi Application, không truy cập trực tiếp database.
-   Adapter không gọi trực tiếp implementation nội bộ của adapter khác.
-   Composition Root khởi tạo và kết nối implementation.

Ví dụ: Translation không tự mở SQLite; Review không gọi trực tiếp
provider SDK; TUI không tự sửa bản ghi Translation; EPUB Export không tự
quyết định chương đã đạt Review hay chưa.

------------------------------------------------------------------------

## 3. Đề xuất Cargo Workspace

**\[ĐỀ XUẤT\]** Bắt đầu với 9 library crates và 1 binary crate.

  ----------------------------------------------------------------------------
                     \# Crate               Loại             Trách nhiệm
  --------------------- ------------------- ---------------- -----------------
                      1 `nt-domain`         Library          Domain model và
                                                             business rules

                      2 `nt-application`    Library          Use cases,
                                                             workflow và ports

                      3 `nt-storage`        Library          SQLite,
                                                             filesystem,
                                                             synchronization
                                                             và persistence

                      4 `nt-epub`           Library          EPUB input,
                                                             parsing và source
                                                             mapping

                      5 `nt-model`          Library          Provider
                                                             adapters,
                                                             capability, usage
                                                             và pricing

                      6 `nt-research`       Library          Web search, page
                                                             reading và
                                                             evidence
                                                             collection

                      7 `nt-export`         Library          Image processing,
                                                             Prepared Export
                                                             Bundle, Preview
                                                             và EPUB output

                      8 `nt-coordinator`    Library          Machine-wide
                                                             queue, permit,
                                                             handoff và
                                                             recovery

                      9 `nt-tui`            Library          Terminal User
                                                             Interface

                     10 `novel-translate`   Binary           Entry point, CLI
                                                             và composition
                                                             root
  ----------------------------------------------------------------------------

Đây là ranh giới khởi đầu để review, không phải yêu cầu bất biến rằng dự
án phải giữ đúng 10 crate.

### 3.1. Lý do phân chia

-   `nt-domain` và `nt-application` tách riêng: Domain giữ quy tắc
    nghiệp vụ; Application tổ chức chúng thành use case.
-   `nt-storage` gom SQLite và filesystem ban đầu, nhưng module,
    repository ownership và transaction boundary phải rõ ràng.
-   `nt-model` cô lập provider SDK, HTTP transport, streaming,
    capability và response normalization.
-   `nt-epub` và `nt-export` tách input/mapping khỏi bundle, Preview và
    output.
-   `nt-coordinator` quản lý quyền chạy AI toàn máy, không sở hữu nội
    dung dịch.
-   `nt-tui` chỉ trình bày và tiếp nhận tương tác; CLI/TUI dùng chung
    Application Core.

------------------------------------------------------------------------

## 4. Sơ đồ phụ thuộc crate

Quy ước: mũi tên `A → B` nghĩa là A phụ thuộc B.

``` mermaid
flowchart TB
    APP["novel-translate<br/>Binary / Composition Root"]
    TUI["nt-tui"]
    CO["nt-coordinator"]
    EX["nt-export"]
    RS["nt-research"]
    MG["nt-model"]
    EP["nt-epub"]
    ST["nt-storage"]
    AP["nt-application"]
    DM["nt-domain"]

    APP --> TUI
    APP --> CO
    APP --> EX
    APP --> RS
    APP --> MG
    APP --> EP
    APP --> ST
    APP --> AP

    TUI --> AP
    CO --> AP
    EX --> AP
    RS --> AP
    MG --> AP
    EP --> AP
    ST --> AP

    AP --> DM
    ST --> DM
    EP --> DM
    MG --> DM
    RS --> DM
    EX --> DM
    CO --> DM
```

Sơ đồ thể hiện hướng phụ thuộc logic ở mức crate. Việc infrastructure
crate phụ thuộc `nt-application` để hiện thực port cần được xem xét kỹ
khi thiết kế public API; tuyệt đối không tạo dependency cycle hoặc kéo
implementation ngược vào Application.

### 4.1. Dependency bị cấm

  -----------------------------------------------------------------------
  Dependency              Trạng thái              Lý do
  ----------------------- ----------------------- -----------------------
  `nt-domain` →           Cấm                     Domain không biết cách
  `nt-storage`                                    lưu trữ

  `nt-domain` →           Cấm                     Domain không biết
  `nt-model`                                      provider

  `nt-application` →      Cấm theo hướng thiết kế Application phụ thuộc
  `nt-storage`                                    abstraction/port

  `nt-application` →      Cấm                     Application không phụ
  `nt-tui`                                        thuộc UI

  `nt-model` → `nt-tui`   Cấm                     Provider adapter không
                                                  liên quan presentation

  Review implementation → Cấm                     Hai nghiệp vụ giao tiếp
  Translation                                     qua Application
  implementation                                  contract

  TUI screen → SQLite     Cấm                     UI không truy cập
  repository                                      persistence trực tiếp
  -----------------------------------------------------------------------

Không được có circular dependency giữa các crate.

------------------------------------------------------------------------

## 5. Module breakdown sơ bộ

Các cây thư mục sau là phân rã trách nhiệm sơ bộ. Bước 4 sẽ xác định
module boundary, interface, trait, visibility và dependency chi tiết
hơn.

### 5.1. `nt-domain`

``` text
nt-domain/
└── src/
    ├── lib.rs
    ├── workspace/
    │   ├── mod.rs
    │   ├── series.rs
    │   ├── volume.rs
    │   └── configuration.rs
    ├── source/
    │   ├── mod.rs
    │   ├── snapshot.rs
    │   ├── block.rs
    │   ├── mapping.rs
    │   └── content_scope.rs
    ├── context/
    │   ├── mod.rs
    │   ├── skill.rs
    │   ├── glossary.rs
    │   ├── character.rs
    │   └── addressing.rs
    ├── translation/
    │   ├── mod.rs
    │   ├── unit.rs
    │   └── revision.rs
    ├── review/
    │   ├── mod.rs
    │   ├── issue.rs
    │   ├── evidence.rs
    │   ├── severity.rs
    │   └── repair.rs
    ├── research/
    │   ├── mod.rs
    │   └── evidence.rs
    ├── task/
    │   ├── mod.rs
    │   ├── task.rs
    │   ├── state.rs
    │   └── checkpoint.rs
    ├── model/
    │   ├── mod.rs
    │   ├── request.rs
    │   ├── attempt.rs
    │   └── usage.rs
    └── export/
        ├── mod.rs
        ├── bundle.rs
        └── version.rs
```

Domain chứa entity, value object, enum và quy tắc nghiệp vụ thuần. Ví
dụ: `revision.rs` định nghĩa bản sửa bản dịch; `severity.rs` định nghĩa
S0--S3; `repair.rs` định nghĩa M1--M4; `mapping.rs` định nghĩa mapping
version và source range; `state.rs` định nghĩa trạng thái task. Domain
không chứa logic HTTP, SQLite hoặc terminal.

### 5.2. `nt-application`

``` text
nt-application/
└── src/
    ├── lib.rs
    ├── ports/
    │   ├── mod.rs
    │   ├── workspace_repository.rs
    │   ├── source_importer.rs
    │   ├── context_resolver.rs
    │   ├── draft_repository.rs
    │   ├── review_repository.rs
    │   ├── model_gateway.rs
    │   ├── research_gateway.rs
    │   ├── coordinator_gateway.rs
    │   ├── file_store.rs
    │   └── export_gateway.rs
    ├── use_cases/
    │   ├── workspace/
    │   ├── import/
    │   ├── context/
    │   ├── translation/
    │   ├── review/
    │   ├── research/
    │   ├── task/
    │   └── export/
    ├── workflow/
    │   ├── mod.rs
    │   ├── engine.rs
    │   ├── transitions.rs
    │   ├── checkpoint.rs
    │   └── recovery.rs
    ├── events/
    └── errors.rs
```

`ports/` định nghĩa những khả năng Application cần từ bên ngoài. Ví dụ
`ModelGateway` là contract logic để gửi model request; Application không
biết request được gửi bằng SDK provider nào hay HTTP client nào. Chữ ký
trait, async strategy, cancellation và object-safety chưa được chốt ở
bước này.

### 5.3. `nt-storage`

``` text
nt-storage/
└── src/
    ├── lib.rs
    ├── sqlite/
    │   ├── mod.rs
    │   ├── connection.rs
    │   ├── series/
    │   └── coordinator/
    ├── filesystem/
    │   ├── mod.rs
    │   ├── immutable_store.rs
    │   └── workspace_files.rs
    ├── generation/
    │   ├── mod.rs
    │   └── commit.rs
    ├── synchronization/
    │   ├── mod.rs
    │   ├── markdown.rs
    │   └── skills.rs
    ├── migration/
    └── recovery/
```

Không để SQLite type hoặc filesystem implementation rò rỉ vào Domain.

### 5.4. `nt-epub`

``` text
nt-epub/
└── src/
    ├── lib.rs
    ├── package/
    │   ├── container.rs
    │   ├── opf.rs
    │   └── manifest.rs
    ├── navigation/
    │   ├── nav.rs
    │   └── ncx.rs
    ├── xhtml/
    │   ├── extractor.rs
    │   ├── ruby.rs
    │   ├── inline.rs
    │   └── notes.rs
    ├── mapping/
    │   ├── block.rs
    │   ├── range.rs
    │   └── stable_id.rs
    ├── classification/
    └── security/
```

Chỉ xử lý đầu vào, cấu trúc nguồn và mapping; không chứa logic dịch.

### 5.5. `nt-model`

``` text
nt-model/
└── src/
    ├── lib.rs
    ├── providers/
    │   ├── mod.rs
    │   ├── openai/
    │   ├── anthropic/
    │   └── custom/
    ├── connection/
    ├── catalog/
    ├── capability/
    ├── request/
    ├── streaming/
    ├── response/
    ├── usage/
    ├── pricing/
    └── credential/
```

Provider adapter được cô lập để bổ sung provider mới không buộc thay đổi
Translation hoặc Review.

### 5.6. `nt-research`

``` text
nt-research/
└── src/
    ├── lib.rs
    ├── search/
    │   └── brave.rs
    ├── page_reader/
    ├── extraction/
    ├── evidence/
    ├── spoiler/
    └── cache/
```

Không tự gọi model để dịch. Các yêu cầu model thuộc contract của
`nt-model`, được Application điều phối.

### 5.7. `nt-export`

``` text
nt-export/
└── src/
    ├── lib.rs
    ├── images/
    │   ├── resolver.rs
    │   └── validator.rs
    ├── bundle/
    │   ├── builder.rs
    │   └── manifest.rs
    ├── preview/
    │   ├── server.rs
    │   ├── security.rs
    │   └── session.rs
    ├── epub33/
    │   ├── package.rs
    │   ├── navigation.rs
    │   ├── content.rs
    │   └── resources.rs
    ├── validation/
    └── publisher/
```

`Prepared Export Bundle` là đầu vào chung cho Preview và EPUB Builder.

### 5.8. `nt-coordinator`

``` text
nt-coordinator/
└── src/
    ├── lib.rs
    ├── queue/
    ├── permit/
    ├── lock/
    ├── handoff/
    ├── recovery/
    └── reconciliation/
```

Chứa logic cấp quyền, queue, handoff và recovery. Chưa quyết định daemon
hay process host, cũng chưa khóa IPC transport.

### 5.9. `nt-tui`

``` text
nt-tui/
└── src/
    ├── lib.rs
    ├── app.rs
    ├── state/
    ├── events/
    ├── navigation/
    ├── screens/
    │   ├── dashboard/
    │   ├── workspace/
    │   ├── volume/
    │   ├── translation/
    │   ├── review/
    │   ├── context/
    │   ├── research/
    │   ├── tasks/
    │   ├── settings/
    │   └── export/
    ├── components/
    └── rendering/
```

Screen không gọi repository trực tiếp. Mọi thao tác nghiệp vụ đi qua
Application.

------------------------------------------------------------------------

## 6. Cấu trúc thư mục toàn dự án

**\[ĐỀ XUẤT\]**

``` text
novel-translate-cli/
│
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── .gitignore
│
├── apps/
│   └── novel-translate/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── bootstrap.rs
│           ├── cli/
│           └── commands/
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
├── tests/
│   ├── fixtures/
│   │   ├── epub2/
│   │   ├── epub3/
│   │   ├── malformed/
│   │   └── japanese/
│   ├── integration/
│   ├── acceptance/
│   └── recovery/
│
├── docs/
│   ├── spec/            # Đặc tả hiện có
│   ├── desgin/          # Thiết kế Bước 1–9 và SQL nháp hiện có
│   ├── decisions/       # Đề xuất cho ADR triển khai
│   └── roadmap/         # Đề xuất cho kế hoạch theo phase
│
└── scripts/
    ├── check.sh
    └── test.sh
```

Tên thư mục và vị trí crate được rà soát khi chuyển từ package Rust hiện có sang workspace theo [Phase 01 của Bước 9](IMPLEMENTATION_ROADMAP_STEP9.md#phase-01). `apps/novel-translate/` là tên thư mục đề xuất. Tên executable đã chốt là `novel-translate-cli` theo [Quyết định 34](../spec/DESIGN_DECISIONS_36.md#decision-34).

------------------------------------------------------------------------

## 7. Quy tắc thiết kế interface và module

  -----------------------------------------------------------------------
  Quy tắc                             Yêu cầu
  ----------------------------------- -----------------------------------
  Domain isolation                    Domain không import adapter

  Dependency inversion                Application định nghĩa ports

  Repository ownership                Dữ liệu chỉ thay đổi qua owner
                                      nghiệp vụ

  No direct DB access                 TUI/use case không tự truy cập
                                      SQLite

  No provider leakage                 Provider SDK không xuất hiện trong
                                      Application API

  No UI leakage                       Terminal types không xuất hiện
                                      trong Domain

  Explicit transaction                Commit có generation/base version

  Explicit errors                     Không chuyển mọi lỗi thành `String`

  Narrow public API                   Chỉ export type/function thực sự
                                      cần thiết

  No circular dependency              Không có dependency cycle giữa
                                      crate
  -----------------------------------------------------------------------

### 7.1. Public API

Mỗi crate giới hạn những gì được phép sử dụng từ bên ngoài. Ví dụ,
`nt-domain` có thể công khai `TranslationRevision`, `SourceRange`,
`ReviewIssue`, `TaskState`, `RepairScope`; không nên công khai helper
nội bộ chỉ vì crate khác thấy tiện.

### 7.2. Error handling

Phân biệt ba tầng: 1. **Domain Error:** Vi phạm quy tắc nghiệp vụ. 2.
**Application Error:** Use case không thể hoàn thành. 3.
**Infrastructure Error:** SQLite, filesystem, HTTP, provider hoặc
parser.

Không để lỗi provider trở thành loại lỗi nghiệp vụ trong Domain.

------------------------------------------------------------------------

## 8. Application Composition Root

Binary crate trong `apps/novel-translate/` chịu trách nhiệm khởi tạo và kết nối các crate. Tên executable là `novel-translate-cli`.

Nhiệm vụ: - Đọc CLI arguments. - Đọc cấu hình. - Khởi tạo storage. -
Khởi tạo provider adapters. - Khởi tạo Research adapters. - Khởi tạo
Coordinator. - Khởi tạo Application services. - Khởi tạo TUI khi chạy không có tham số. - Dispatch command tương ứng.

``` mermaid
flowchart TD
    A["main.rs"] --> B["Parse CLI Arguments"]
    B --> C["Load Configuration"]
    C --> D["Initialize Storage"]
    D --> E["Initialize Infrastructure Adapters"]
    E --> F["Build Application Services"]
    F --> G{"Command"}
    G -->|CLI| H["Execute CLI Use Case"]
    G -->|TUI| I["Start TUI"]
    H --> J["Application Core"]
    I --> J
```

TUI và CLI dùng chung use case, không xây dựng hai hệ thống xử lý nghiệp
vụ riêng.

------------------------------------------------------------------------

## 9. Các quyết định kỹ thuật cần giải quyết

  -----------------------------------------------------------------------
  ID                      Vấn đề                  Nội dung cần xác định
  ----------------------- ----------------------- -----------------------
  R3-01                   Rust toolchain          Rust stable version và
                                                  chính sách nâng cấp

  R3-02                   Async runtime           Tokio hay runtime khác

  R3-03                   Async trait             Native async trait hay
                                                  `async-trait`

  R3-04                   TUI framework           Ratatui hay framework
                                                  khác

  R3-05                   CLI framework           Clap hay giải pháp khác

  R3-06                   SQLite driver           SQLx, rusqlite hoặc lựa
                                                  chọn khác

  R3-07                   Serialization           Serde và boundary giữa
                                                  Domain/Storage

  R3-08                   Error handling          `thiserror`, `anyhow`
                                                  và error mapping

  R3-09                   IPC                     Named Pipe, Unix Socket
                                                  hoặc phương án khác

  R3-10                   HTTP client             Client dùng chung cho
                                                  Model và Research đến
                                                  mức nào

  R3-11                   Logging                 `tracing`, structured
                                                  logging và redaction

  R3-12                   Dependency policy       Kiểm soát phiên bản,
                                                  feature và crate
                                                  dependency

  R3-13                   Test strategy           Unit, integration,
                                                  contract và recovery
                                                  tests

  R3-14                   Build configuration     Feature flags và cấu
                                                  hình theo nền tảng
  -----------------------------------------------------------------------

Không chọn thư viện chỉ vì phổ biến. Mỗi lựa chọn cần dựa trên yêu cầu,
khả năng tương thích và chi phí bảo trì.

------------------------------------------------------------------------

## 10. Tiêu chí nghiệm thu Bước 3

-   [ ] Cargo workspace structure được xác định.
-   [ ] Danh sách crate và trách nhiệm từng crate được rà soát.
-   [ ] Dependency graph được rà soát.
-   [ ] Module breakdown sơ bộ được xác định.
-   [ ] Quy tắc public API được xác định.
-   [ ] Application ports/interfaces được phân loại.
-   [ ] Quy tắc error handling được xác định.
-   [ ] Composition root được xác định.
-   [ ] Quy tắc dependency enforcement được xác định.
-   [ ] Test architecture được xác định.
-   [ ] Các quyết định kỹ thuật đã chốt và còn mở được ghi nhận riêng.

------------------------------------------------------------------------

## 11. Ranh giới với Bước 4

Bước 3 tập trung vào Cargo Workspace, crate boundaries, dependency
direction và module breakdown sơ bộ.

[Bước 4 — Rust Module Design](MODULE_DESIGN_STEP4.md) đi sâu vào: - Xác định trách
nhiệm chính xác của từng module. - Xác định module public/private và
ranh giới `pub`/`pub(crate)`. - Thiết kế interface, trait và type giữa
các module. - Xác định module nào sở hữu logic và module nào chỉ điều
phối. - Xác định dependency giữa các module, tránh vòng phụ thuộc. - Xác
định contract giữa Domain, Application và Infrastructure. - Kiểm tra khả
năng test độc lập. - Chuẩn bị nền tảng cho thiết kế SQLite schema và
triển khai.

Không tạo toàn bộ file module hoặc code triển khai trước khi ranh giới
được rà soát.

------------------------------------------------------------------------

## 12. Trạng thái tài liệu

  Hạng mục                                 Trạng thái
  ---------------------------------------- ----------------------------------
  Mô hình Modular Monolith                 Đã xác định ở mức định hướng
  Bốn nhóm layer                           Đề xuất
  9 library crates + 1 binary crate        Đề xuất
  Dependency graph                         Đề xuất
  Module breakdown                         Sơ bộ, cần chi tiết hóa ở Bước 4
  Cấu trúc thư mục                         Đề xuất
  Composition root                         Đề xuất
  14 quyết định kỹ thuật R3-01 đến R3-14   Còn cần giải quyết
  Code triển khai                          Chưa thực hiện

**Kết luận:** Bước 3 đưa ra phương án tổ chức Rust Cargo Workspace và
ranh giới crate cho `novel-translate-cli`. Đây là cơ sở cho Bước 4 ---
Rust Module Design, không phải tuyên bố rằng mọi crate, module và thư
viện đã được chốt hoàn toàn.
