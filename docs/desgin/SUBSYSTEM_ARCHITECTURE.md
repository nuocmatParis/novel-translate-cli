# SUBSYSTEM ARCHITECTURE

## novel-translate-cli --- Bước 2: Phân rã kiến trúc subsystem

  -----------------------------------------------------------------------
  Thuộc tính                          Giá trị
  ----------------------------------- -----------------------------------
  Tài liệu                            Subsystem Architecture

  Giai đoạn                           Step 2 -- Subsystem Architecture

  Phiên bản                           0.1

  Ngày cập nhật                       04/11/2026

  Trạng thái                          Bản thiết kế đề xuất để rà soát

  Tài liệu nền                        `ARCHITECTURE_OVERVIEW.md`,
                                      `DESIGN_DECISIONS_36.md` và các đặc
                                      tả chuyên ngành

  Phạm vi                             Trách nhiệm, quyền sở hữu dữ liệu,
                                      contract và luồng xử lý cấp
                                      subsystem

  Ngoài phạm vi                       Rust crate/module, trait signature,
                                      schema SQL chi tiết và cây thư mục
  -----------------------------------------------------------------------

------------------------------------------------------------------------

## Mục lục

1.  [Mục đích và cách đọc](#1-mục-đích-và-cách-đọc)
2.  [Nguyên tắc phân rã](#2-nguyên-tắc-phân-rã)
3.  [Bản đồ subsystem](#3-bản-đồ-subsystem)
4.  [Quyền sở hữu dữ liệu](#4-quyền-sở-hữu-dữ-liệu)
5.  [Hợp đồng giao tiếp](#5-hợp-đồng-giao-tiếp)
6.  [Thiết kế mười subsystem](#6-thiết-kế-mười-subsystem)
7.  [Luồng liên subsystem](#7-luồng-liên-subsystem)
8.  [Trạng thái và consistency
    boundary](#8-trạng-thái-và-consistency-boundary)
9.  [Lỗi và recovery](#9-lỗi-và-recovery)
10. [Bảo mật](#10-bảo-mật)
11. [Kiểm thử và nghiệm thu](#11-kiểm-thử-và-nghiệm-thu)
12. [Các quyết định còn mở](#12-các-quyết-định-còn-mở)
13. [Ranh giới với Bước 3](#13-ranh-giới-với-bước-3)
14. [Ma trận truy xuất đặc tả](#14-ma-trận-truy-xuất-đặc-tả)

------------------------------------------------------------------------

# 1. Mục đích và cách đọc

## 1.1. Mục đích

Bước 1 đã xác định mô hình kiến trúc tổng thể:

**Modular Monolith + Layered Architecture + Hexagonal Architecture
(Ports & Adapters).**

Bước 2 phân rã mô hình đó thành các subsystem có trách nhiệm và ranh
giới dữ liệu rõ ràng. Tài liệu này phải trả lời:

-   Subsystem nào chịu trách nhiệm cho từng nhóm nghiệp vụ?
-   Subsystem nào sở hữu và được phép thay đổi dữ liệu?
-   Các subsystem giao tiếp thông qua contract nào?
-   Workflow nào phải được điều phối tập trung?
-   Transaction, version check và recovery nằm ở đâu?
-   Có thể kiểm thử từng subsystem độc lập và tích hợp như thế nào?

Đây **không phải** bước chia thư mục hoặc tạo danh sách crate Rust.

## 1.2. Quy ước mức độ xác nhận

  -----------------------------------------------------------------------
  Nhãn                                Ý nghĩa
  ----------------------------------- -----------------------------------
  **\[ĐÃ CHỐT\]**                     Hành vi/ràng buộc đã được xác nhận
                                      trong 36 quyết định và đặc tả
                                      chuyên ngành

  **\[ĐỀ XUẤT\]**                     Cách phân rã component hoặc
                                      contract để hiện thực yêu cầu đã
                                      chốt

  **\[CÒN MỞ\]**                      Cần quyết định hoặc thiết kế kỹ
                                      thuật sâu hơn
  -----------------------------------------------------------------------

Đề xuất không tự động trở thành quyết định sản phẩm. Khi xác nhận một đề
xuất, cần cập nhật tài liệu quyết định tương ứng.

## 1.3. Thuật ngữ phải phân biệt

  Khái niệm          Ý nghĩa
  ------------------ ------------------------------------------------
  Layer              Tầng trách nhiệm kỹ thuật
  Subsystem          Nhóm năng lực có ranh giới trách nhiệm
  Component          Trách nhiệm nhỏ hơn bên trong subsystem
  Contract           Hợp đồng input/output, điều kiện và lỗi
  Snapshot           Bản nguồn cố định để đối chiếu
  Mapping            Quan hệ nội dung trích xuất với cấu trúc nguồn
  Revision           Phiên bản nội dung hoặc quyết định
  Generation         Mốc dữ liệu workspace đã commit nhất quán
  Dependency         Căn cứ/version mà một kết quả đã sử dụng
  Task               Ý định công việc do người dùng yêu cầu
  Workflow Step      Bước nghiệp vụ bên trong task
  Translation Unit   Phạm vi nguồn giao cho một lượt dịch
  Model Request      Yêu cầu logic gửi tới model
  Attempt            Một lần thực sự gửi request tới provider
  Repair Round       Một vòng sửa chất lượng rồi Review lại

Một task có thể có nhiều workflow step; một step có thể có nhiều
request; một request có thể có nhiều attempt. Attempt không tự tạo
revision và không quyết định chất lượng.

------------------------------------------------------------------------

# 2. Nguyên tắc phân rã

## 2.1. Quyền sở hữu dữ liệu

**\[ĐỀ XUẤT\]** Mỗi nhóm dữ liệu nghiệp vụ có một subsystem chịu trách
nhiệm tạo và thay đổi dữ liệu đó.

Subsystem khác được đọc qua contract hoặc yêu cầu subsystem sở hữu thực
hiện thay đổi. Không được:

-   Ghi trực tiếp vào bảng nội bộ của subsystem khác.
-   Tạo một bản dữ liệu chính thức thứ hai.
-   Thay đổi trạng thái bỏ qua quy tắc của owner.
-   Dùng cache/file hiển thị làm nguồn sự thật thay cho dữ liệu nghiệp
    vụ đã lưu.

Persistence cung cấp khả năng lưu trữ và commit; nó không thay thế quyền
sở hữu nghiệp vụ.

## 2.2. Tách điều phối khỏi nghiệp vụ

Workflow & Task Management quyết định **khi nào** một bước được chạy và
bước tiếp theo là gì.

Subsystem nghiệp vụ quyết định **bước đó thực hiện nghiệp vụ như thế
nào**.

Ví dụ: Workflow quyết định sau khi Translation lưu nháp thì Review đúng
revision đó. Translation tạo và kiểm tra bản dịch; Review đánh giá chất
lượng; Workflow nhận kết quả rồi quyết định tiếp tục, sửa, Research hoặc
chờ người dùng.

## 2.3. Candidate không phải quyết định chính thức

Các kết quả sau chưa tự động trở thành dữ liệu chính thức:

-   Response model chưa qua kiểm tra.
-   Thuật ngữ do AI đề xuất.
-   Quy tắc xưng hô đang chờ duyệt.
-   Sự kiện do Reviewer ghi nhận nhưng chưa kiểm chứng.
-   Evidence Research chưa được xác minh.
-   Kết luận Review mâu thuẫn hoặc thiếu coverage.

Không dùng candidate làm context đã xác nhận chỉ để workflow tiếp tục.

## 2.4. Các bất biến xuyên subsystem

1.  Snapshot nguồn không bị sửa tại chỗ.
2.  Mọi revision truy được nguồn và dependency đã dùng.
3.  Review Result gắn với đúng revision được kiểm tra.
4.  Dependency thay đổi chỉ invalidation phạm vi bị ảnh hưởng.
5.  Sửa tay không bị response cũ ghi đè.
6.  Một thời điểm chỉ có một task AI giữ quyền chạy theo 13A.
7.  Không tự động đổi model khi lỗi.
8.  Không tự duyệt thuật ngữ/quy ước mới.
9.  Preview và EPUB Export dùng cùng Prepared Export Bundle.
10. File EPUB đã xuất không bị ghi đè.
11. Kết quả provider chưa rõ không được biểu diễn thành thành công hoặc
    usage/chi phí bằng 0.
12. Dữ liệu đã commit phải phục hồi được sau crash.

------------------------------------------------------------------------

# 3. Bản đồ subsystem

## 3.1. Danh sách

  -----------------------------------------------------------------------
  ID                      Subsystem               Trách nhiệm
  ----------------------- ----------------------- -----------------------
  SS-01                   Workspace Management    Workspace, series,
                                                  volume và cấu hình

  SS-02                   EPUB Processing &       Import, snapshot,
                          Source Mapping          mapping, block và
                                                  classification

  SS-03                   Context & Skills        Skills, glossary, nhân
                                                  vật, alias và xưng hô

  SS-04                   Translation             Request planning, dịch,
                                                  structural validation
                                                  và draft revision

  SS-05                   Review & Quality        Source reading,
                          Control                 comparison, evidence,
                                                  issue và repair scope

  SS-06                   Research                Tra cứu nội
                                                  bộ/Internet, evidence
                                                  và spoiler boundary

  SS-07                   Workflow & Task         Task workflow, queue,
                          Management              Machine-wide
                                                  Coordinator và recovery

  SS-08                   Model Integration       Provider/model,
                                                  capability, usage,
                                                  pricing và budget
                                                  preflight

  SS-09                   Image & EPUB Export     Ảnh, Prepared Export
                                                  Bundle, Preview và EPUB
                                                  3.3

  SS-10                   Persistence &           SQLite, filesystem,
                          Synchronization         generation, commit và
                                                  đồng bộ file
  -----------------------------------------------------------------------

TUI/CLI là adapter tương tác dùng chung Application Core, không phải
subsystem nghiệp vụ sở hữu dữ liệu dịch riêng.

## 3.2. Sơ đồ tương tác

``` mermaid
flowchart TB
    UI["TUI / CLI"] --> APP["Application Facade"]
    APP --> WF["SS-07 Workflow"]

    WS["SS-01 Workspace"]
    EP["SS-02 EPUB Processing"]
    CTX["SS-03 Context & Skills"]
    TR["SS-04 Translation"]
    RV["SS-05 Review"]
    RS["SS-06 Research"]
    MG["SS-08 Model Integration"]
    EX["SS-09 Image & Export"]
    PS["SS-10 Persistence"]

    APP --> WS
    APP --> EP
    APP --> CTX
    APP --> EX

    WF --> TR
    WF --> RV
    WF --> RS
    WF --> PS

    TR --> EP
    TR --> CTX
    TR --> MG
    TR --> PS

    RV --> EP
    RV --> CTX
    RV --> RS
    RV --> MG
    RV --> PS

    RS --> EP
    RS --> CTX
    RS --> MG
    RS --> PS

    EX --> EP
    EX --> CTX
    EX --> TR
    EX --> RV
    EX --> PS

    WS --> PS
    EP --> PS
    CTX --> PS
    MG --> PS
```

Sơ đồ là quan hệ sử dụng contract ở mức logic, không phải danh sách Rust
dependency đã chốt.

## 3.3. Phân nhóm theo tầng

  -----------------------------------------------------------------------
  Layer                               Thành phần chính
  ----------------------------------- -----------------------------------
  Presentation                        TUI Adapter, CLI Adapter, Local
                                      Preview UI

  Application                         Use Cases, Workflow Orchestration,
                                      Coordinator Integration,
                                      Progress/Event Delivery

  Domain/Core                         Workspace, Source Mapping, Context,
                                      Translation, Review, Research, Task
                                      State, Budget Rules, Export
                                      Readiness

  Infrastructure                      SQLite, File Store, Provider
                                      Adapters, Web Adapter, EPUB
                                      Parser/Builder, Credential Store,
                                      Preview Server
  -----------------------------------------------------------------------

Một subsystem có thể trải qua nhiều layer; không ánh xạ máy móc một
subsystem thành một crate.

------------------------------------------------------------------------

# 4. Quyền sở hữu dữ liệu

## 4.1. Logical owner và physical storage

-   **Logical owner:** subsystem có quyền tạo, xác nhận và thay đổi dữ
    liệu nghiệp vụ.
-   **Physical storage:** SQLite hoặc filesystem lưu record/byte.
-   **Reader:** subsystem chỉ đọc dữ liệu qua contract.

Ví dụ, Translation sở hữu logic tạo revision bản dịch; Persistence ghi
revision vào SQLite/file bất biến. Persistence không tự quyết định
revision nào đạt Review.

## 4.2. Ma trận ownership

  -----------------------------------------------------------------------------------
  Dữ liệu                       Logical owner     Lưu vật lý        Reader chính
  ----------------------------- ----------------- ----------------- -----------------
  Series/Volume identity, cấu   Workspace         Series SQLite     Các subsystem
  hình                          Management                          

  EPUB source/snapshot          EPUB Processing   Filesystem +      Translation,
                                                  SQLite metadata   Review, Export

  Package structure, spine,     EPUB Processing   SQLite            Export, UI
  navigation                                      mapping/source    
                                                  snapshot          

  Chapter/block/range mapping   EPUB Processing   Series SQLite     Translation,
                                                                    Review, Export

  Content scope/classification  EPUB Processing;  Series SQLite     Workflow,
                                config thuộc                        Translation,
                                Workspace                           Review, Export

  Common/Series Skill versions  Context & Skills  Markdown + SQLite Translation,
                                                  metadata          Review

  Glossary/approved decisions   Context & Skills  Series SQLite +   Translation,
                                                  Skill file        Review, Export

  Character/alias/addressing    Context & Skills  Series SQLite +   Translation,
  rules                                           Skill file        Review

  Translation                   Translation       Series SQLite +   Review, Export
  candidate/revision                              immutable         
                                                  files/Markdown    

  Review                        Review            Series SQLite     Workflow, UI,
  run/coverage/issue/evidence                                       Export readiness

  Repair plan/round             Review đề xuất;   Series SQLite     Translation,
                                Workflow theo dõi                   Workflow
                                thực thi                            

  Research                      Research          Series SQLite +   Translation,
  request/evidence/cache                          source metadata   Review, Context

  Task/step/checkpoint          Workflow          Series SQLite     TUI, CLI,
                                                                    subsystem được
                                                                    giao

  Machine-wide queue/permit     Coordinator       Coordinator       TUI/CLI qua
                                                  SQLite            Coordinator

  Provider/model                Model Integration Tool config +     Workflow,
  profile/capability                              Series references Translation,
                                                                    Review, Research

  API key                       Credential        OS Credential     Provider Adapter
                                Adapter           Store/env đã chọn 

  Request/attempt/usage/cost    Model Integration Series SQLite     Workflow, UI,
                                                                    report

  Task budget snapshot          Workflow sở hữu;  Series SQLite     Workflow, Model
                                Model Integration                   Integration
                                tính cost                           

  Image assets/selected hashes  Image & Export    Filesystem +      Preview, Export
                                                  SQLite metadata   

  Prepared bundle/manifest      Image & Export    Immutable files + Preview, Builder
                                                  SQLite metadata   

  Export version/history        Image & Export    Filesystem +      UI, Workspace
                                                  Series SQLite     

  Generation/current            Persistence       Series SQLite     Subsystems qua
  state/history                                                     repository
                                                                    contracts

  Markdown draft sync state     Persistence;      Filesystem +      Translation, UI
                                Translation sở    SQLite            
                                hữu semantics                       

  Skill sync state              Context & Skills  Filesystem +      Context & Skills
                                sở hữu semantics; SQLite            
                                Persistence hỗ                      
                                trợ publish                         
  -----------------------------------------------------------------------------------

## 4.3. Ba loại file có semantics khác nhau

### EPUB nguồn và file bất biến

-   Là dữ liệu nguồn/căn cứ.
-   Giữ nguyên, định danh bằng hash và snapshot.
-   Không sửa snapshot cũ tại chỗ.
-   Khi nguồn thay đổi, tạo snapshot/mapping phù hợp và đánh giá
    dependency.

### Translation Markdown

-   Là bản biểu diễn/chỉnh sửa của revision.
-   Khi dựng file, lưu revision nền, hash và marker từng đơn vị.
-   Sửa ngoài ứng dụng được nhập thành revision sửa tay nếu marker/nền
    hợp lệ.
-   Xung đột phải giữ base, user edit và AI candidate; không tự ghi đè.

### Skills Markdown

-   Là tài liệu người dùng đăng ký để đọc/chỉnh quy ước.
-   Common Skill Markdown linh hoạt; Series Skill theo mẫu sáu nhóm.
-   Thay đổi rõ ràng, hợp lệ do người dùng lưu được áp dụng từ lượt chưa
    gửi tiếp theo theo 2B.
-   Candidate AI không tự thành quy tắc.
-   Publish decision cần đối chiếu file và SQLite; xung đột phải giữ cả
    hai phía.

------------------------------------------------------------------------

# 5. Hợp đồng giao tiếp

## 5.1. Quy ước contract

**\[ĐỀ XUẤT\]** Mỗi contract cần mô tả:

1.  Mục đích nghiệp vụ.
2.  Subsystem gửi/nhận.
3.  Input có ID và version.
4.  Điều kiện tiên quyết.
5.  Output/trạng thái.
6.  Lỗi có thể trả về.
7.  Idempotency nếu thao tác có thể gửi lại.
8.  Ranh giới commit.
9.  Sự kiện công bố sau thành công.

Đây là contract logic, chưa phải chữ ký hàm, trait hoặc JSON API cố
định.

## 5.2. Contract chính

  --------------------------------------------------------------------------------------------------
  Contract logic    Gửi → nhận                    Input cốt lõi          Output
  ----------------- ----------------------------- ---------------------- ---------------------------
  Open Workspace    UI → Workspace                Workspace              Workspace identity, format
                                                  path/reference         version, status

  Register Volume   Workspace → EPUB              Volume ID, EPUB path   Snapshot, mapping,
  Source                                                                 diagnostics

  Resolve Content   Workflow → EPUB               Volume, scope choice,  Translate/keep/unresolved
  Scope                                           mapping version        ranges

  Resolve Context   Translation/Review → Context  Series, volume, source Approved rules, versions,
                                                  scope, position/time   conflicts

  Create Draft      Workflow → Translation        Task, source scope,    Candidate hoặc draft
                                                  repair plan            revision

  Execute Model     Translation/Review/Research → Role, model, request,  Stream/result, model
  Request           Model                         policy                 identity, usage/error

  Commit Draft      Translation → Persistence     Validated candidate,   Revision ID, generation ID
                                                  base generation,       
                                                  dependency             

  Review Draft      Workflow → Review             Snapshot, mapping,     Coverage, issues, evidence,
                                                  revision set, scope    conclusion

  Request Research  Review/Translation/User →     Question, source       Evidence bundle,
                    Research                      position, spoiler      uncertainty
                                                  boundary               

  Create Repair     Review → Workflow             Issues, S/M, M scope,  Repair plan/round
  Plan                                            revision base          

  Publish Rule      Context → Persistence         Selected decisions,    Published versions or
  Decision                                        skill base             conflict
                                                  hash/version           

  Invalidate        Context/EPUB/Review →         Changed                Affected references and
  Dependencies      Workflow/Persistence          source/rule/revision   recheck state

  Prepare Export    User/Workflow → Export        Volume, current        Immutable bundle/manifest
                                                  generation, scope      

  Open Preview      UI → Export                   Prepared bundle ID     Local preview session

  Publish EPUB      User/Workflow → Export        Bundle ID, export      Export
                                                  intent                 version/path/hash/report

  Request AI Permit Workflow → Coordinator        Task ID, workspace ID  Permit or queued state

  Release/Handoff   Workflow → Coordinator        Task ID, permit        Released/handoff state
  Permit                                          sequence, reason       

  Persist           Workflow → Persistence        Task, step, attempt,   Durable checkpoint
  Checkpoint                                      base generation        
  --------------------------------------------------------------------------------------------------

## 5.3. Event logic đề xuất

Tên event dưới đây chỉ là nhãn mô tả, chưa phải event schema cố định.

  --------------------------------------------------------------------------
  Event                      Ý nghĩa                 Consumer
  -------------------------- ----------------------- -----------------------
  `SourceImported`           Snapshot/mapping đã lưu Workspace, Workflow, UI

  `MappingRevised`           Mapping version mới đã  Translation, Review,
                             commit                  Dependency

  `ContentScopeChanged`      Scope/classification    Workflow, Export
                             đổi                     

  `RuleDecisionCommitted`    Rule đã duyệt được công Context, Translation,
                             bố                      Review

  `DraftRevisionCommitted`   Draft revision đã       Workflow, Review, UI
                             commit                  

  `ReviewResultCommitted`    Review result đã lưu    Workflow, Context,
                                                     Dependency

  `RepairPlanCreated`        Repair plan đã tạo      Workflow, Translation

  `ResearchEvidenceReady`    Evidence bundle đã lưu  Translation, Review

  `DependencyInvalidated`    Kết quả cần kiểm tra    Workflow, Review, UI
                             lại                     

  `TaskCheckpointSaved`      Checkpoint đã commit    Workflow, Coordinator
                                                     reconciliation

  `AIPermitGranted`          Coordinator cấp quyền   Workflow

  `PreparedBundleCreated`    Prepared manifest đã    Preview, EPUB Builder
                             tạo                     

  `PreviewMarkedStale`       Bundle không còn khớp   Preview, UI
                             dữ liệu hiện hành       

  `ExportPublished`          EPUB version mới đã     Workspace, UI
                             publish                 
  --------------------------------------------------------------------------

Không phát event thành công trước commit. Consumer có thể đọc lại
current state để xác minh; event không phải nguồn sự thật duy nhất.

------------------------------------------------------------------------

# 6. Thiết kế mười subsystem

## 6.1. SS-01 --- Workspace Management

### Mục tiêu và trách nhiệm

Quản lý vòng đời workspace của một bộ truyện, volume thuộc bộ, cấu hình
và việc mở/đóng workspace an toàn.

**\[ĐỀ XUẤT\] Components:**

-   Workspace Lifecycle: create/open/close và kiểm tra format.
-   Series/Volume Registry: identity và metadata của series/volume.
-   Configuration Resolver: xác định cấu hình có hiệu lực.
-   Scope Configuration: lưu lựa chọn all-text/story cho volume.
-   Workspace Access Coordinator: yêu cầu/giải phóng writer lock.
-   Workspace Status: cung cấp trạng thái cho TUI/CLI.

### Không chịu trách nhiệm

Không parse EPUB; không quản lý revision; không điều phối model; không
tự tính giá; không chuyển workspace Go test cũ; không tự migration
schema chi tiết.

### Dữ liệu sở hữu

Series/Volume identity, cấu hình cấp series, tham chiếu kết nối không bí
mật, content scope của volume, format version và trạng thái mở/đóng.

Task phải lưu snapshot cấu hình thực tế; thay mặc định không làm đổi
task đã tồn tại.

### Contract

  Gửi → nhận                 Nội dung
  -------------------------- --------------------------------------------
  UI → Workspace             Open/Create workspace
  Workspace → EPUB           Register source cho volume
  Workspace → Persistence    Open DB, validate format, acquire lock
  Workflow → Workspace       Validate workspace/volume/write permission
  Workspace → Model Config   Resolve connection/model references

### Quy tắc đã chốt

-   Một SQLite cho mỗi series; mọi volume trong series dùng chung dữ
    liệu cấp bộ.
-   Workspace Rust mới bắt đầu schema 1.
-   Không chuyển workspace test Go cũ.
-   Tiếp tục giữ Go hoạt động cho đến khi phần thay thế đạt yêu cầu.
-   Default series mới là **Dịch toàn bộ phần chữ**.
-   Có thể đổi sang **Chỉ dịch nội dung truyện và phần liên quan** cho
    volume hiện tại.
-   **Lưu làm mặc định của bộ** chỉ tác động volume tạo sau.
-   Lựa chọn thực tế phải được lưu theo volume/task và hiển thị trước
    lần dịch đầu.

### Lỗi, consistency và recovery

-   Workspace không hỗ trợ: báo cần tạo workspace mới, không tự sửa/xóa.
-   Workspace đang bị writer khác giữ: không cho ghi cạnh tranh.
-   Schema cần nâng cấp: khóa ghi, backup database/file sẽ sửa, migrate
    theo version, integrity check, rollback khi lỗi.
-   Khi mở lại, kiểm tra generation và file synchronization còn dang dở.

### Kiểm thử

Create/open, unsupported format, scope inheritance, config snapshot,
writer lock, schema validation và reopen recovery.

------------------------------------------------------------------------

## 6.2. SS-02 --- EPUB Processing & Source Mapping

### Mục tiêu và trách nhiệm

Chuyển EPUB nguồn thành mô hình nội dung có định danh ổn định, giữ cấu
trúc và mapping cho Translation, Review và Export.

**\[ĐỀ XUẤT\] Components:**

-   Package Validator: ZIP/container/package/path checks.
-   EPUB Structure Parser: OPF, manifest, spine, navigation/NCX.
-   XHTML Extractor: text, inline, ruby, note, link.
-   Resource Extractor: ảnh và tài nguyên.
-   Reading Order Builder: thứ tự đọc từ spine.
-   Chapter/Boundary Resolver: chương, part và ranh giới.
-   Block/Range Mapper: stable IDs và source ranges.
-   Content Classifier: all-text/story, keep-original, image,
    unresolved.
-   Mapping Editor Service: gộp/tách/dời ranh giới/đổi tên hiển thị.

### Không chịu trách nhiệm

Không dịch, Review, duyệt glossary, gọi AI khi Import, sửa EPUB nguồn,
OCR scan/manga hoặc đóng gói đầu ra.

### Dữ liệu sở hữu

Snapshot, parser version, package structure, navigation/spine mapping,
chapter tree, block IDs, subranges, protected token map, content
classification và mapping version.

### Quy tắc nhận diện cấu trúc đã chốt

1.  Spine xác định thứ tự đọc.
2.  Ưu tiên ranh giới người dùng đã xác nhận cho đúng nguồn.
3.  Tiếp theo là Nav/NCX hợp lệ.
4.  Sau đó là heading và dấu ngắt XHTML.
5.  Ranh giới file chỉ là gợi ý cuối.
6.  TOC hỏng/mâu thuẫn có ý nghĩa tạo phạm vi tạm để người dùng đối
    chiếu.
7.  Part vẫn là tiêu đề trong chương, chưa cần cấp quản lý riêng.
8.  Cho gộp, tách, dời ranh giới và đổi tên hiển thị.
9.  Mapping mới phải đúng thứ tự và phủ đủ nguồn.

### Stable ID và subrange

**\[ĐÃ CHỐT\]** Block ID là SHA-256 của bản ghi có version gồm hash
EPUB, parser version, normalized XHTML path và stable node position. Ưu
tiên ID XHTML duy nhất; nếu không có thì dùng node path và order. Hai
đoạn giống chữ vẫn có ID riêng.

Subrange mang block ID và khoảng nửa mở `[start, end)`, kèm mapping về
node/cụm ký tự gốc. Ruby, ảnh và marker inline có quy tắc ranh giới
riêng.

Tên chương và chunk index không thay đổi ID nguồn. Chia lại chunk không
tạo block mới. EPUB đổi tạo snapshot mới; chỉ tái dùng kết quả khi match
duy nhất vị trí/nội dung/protected assets và dependency.

### Scope classification

**\[ĐÃ CHỐT\]**

Mỗi phần phải xác định thuộc nhóm cần dịch, giữ nguyên theo scope, ảnh
hoặc chưa phân loại.

-   All-text: phần chữ đọc được ở cuối sách vẫn cần dịch dù chưa rõ là
    ngoại truyện hay quảng cáo.
-   Story-only: trường hợp phân loại có thể làm thay đổi kết quả phải
    vào Cần phân loại.
-   Không tự gán Giữ nguyên để bỏ qua.
-   Phần phụ trợ dùng chung với truyện phải giữ relation để không dịch
    lặp/bỏ sót.

### Import safety

**\[ĐÃ CHỐT -- MẶC ĐỊNH THỬ NGHIỆM\]**

-   Tối đa 10.000 mục ZIP.
-   128 MiB mỗi mục.
-   1 GiB tổng giải nén.
-   Kiểm tra byte thực tế, path traversal và xung đột tên trên hệ đích.
-   Nội dung ngoài phạm vi hỗ trợ phải báo vị trí/lý do, không âm thầm
    bỏ qua.

Phạm vi đầu vào ban đầu: EPUB 2/3 có XHTML đọc được, reflowable, có thể
xen bìa/trang minh họa fixed-layout. EPUB toàn scan, manga cần
OCR/reading-order, interactive content hoặc nội dung mã hóa không đọc
được nằm ngoài phạm vi giai đoạn đầu.

### Contract

-   Workspace → EPUB: import source vào volume.
-   EPUB → Translation/Review: source block/range + snapshot/mapping
    version.
-   EPUB → Export: reading order, navigation, notes, image/resource
    relation.
-   EPUB → Persistence: snapshot/mapping/classification commit.

### Recovery và test

Import chỉ được báo hoàn tất sau khi snapshot/mapping/resource metadata
đã commit. Test EPUB 2/3, multi-XHTML chapters, one-XHTML-many-chapters,
TOC conflicts, ruby, notes, images, fixed-layout interleaving, ZIP/path
attacks, stable IDs và range coverage.

------------------------------------------------------------------------

## 6.3. SS-03 --- Context & Skills

### Mục tiêu và trách nhiệm

Cung cấp hướng dẫn, glossary, nhân vật, alias, sự kiện và quy tắc xưng
hô đã được xác nhận, đúng phạm vi và thời điểm.

**\[ĐỀ XUẤT\] Components:**

-   Common Skill Registry.
-   Series Skill Registry.
-   Skill Parser/Normalizer.
-   Approved Decision Store.
-   Glossary Resolver.
-   Character/Alias Registry.
-   Addressing Timeline Resolver.
-   Candidate/Proposal Manager.
-   Batch Approval Coordinator.
-   Skill File Synchronizer.
-   Context Resolver.

### Không chịu trách nhiệm

Không dịch câu, không tự xác nhận sự kiện từ suy luận model, không tự
duyệt candidate, không điều phối task, không đánh giá chất lượng toàn
bản dịch.

### Hai tầng Skills đã chốt

  -----------------------------------------------------------------------
  Loại                                Đặc điểm
  ----------------------------------- -----------------------------------
  Common Skills                       Thư viện nhiều file Markdown dùng
                                      chung; nội dung linh hoạt

  Series Skills                       Quy ước riêng bộ truyện, theo mẫu
                                      sáu nhóm
  -----------------------------------------------------------------------

Mẫu Series Skill:

1.  Văn phong.
2.  Nhân vật và bí danh.
3.  Thuật ngữ và danh hiệu.
4.  Địa danh.
5.  Xưng hô.
6.  Ngoại lệ.

Skill riêng ưu tiên hơn hướng dẫn chung trong phạm vi phù hợp; nguyên
tác vẫn là căn cứ ý nghĩa.

### Context Resolver

Input: series/volume/chapter, snapshot/mapping, source range, vị trí
trong sách, thời điểm sự kiện, nhân vật/người nói/người nghe và role
request.

Output: Skills liên quan, glossary decisions, character/alias
resolution, addressing rules, version/hash, conflict/candidate đang chặn
và dependency manifest.

Không đưa candidate chưa duyệt vào context chính thức.

### Xưng hô/continuity đã chốt

-   Mỗi nhân vật có ID; alias đã xác nhận cùng trỏ tới ID.
-   Quy tắc có chiều A→B, cách tự xưng, cách gọi, hoàn cảnh và khoảng
    hiệu lực.
-   Lưu riêng vị trí cảnh trong sách và thời điểm sự kiện trong truyện.
-   Hồi tưởng dùng rule có hiệu lực ở thời điểm hồi tưởng, đồng thời lọc
    theo những gì độc giả được biết tại vị trí hiện tại.
-   Chưa rõ người nghe/mốc/phản ứng phải giữ Chưa xác định.
-   Ngoại lệ có thể chỉ áp dụng một cảnh.

### Candidate và approval

Candidate cần nguyên ngữ, vị trí, snapshot, ngữ cảnh, nghĩa/định danh,
phương án, phạm vi và lý do.

**\[ĐÃ CHỐT\]** Chỉ người dùng xác nhận mới thành quyết định chính thức.
Duyệt nhóm theo 7A: chọn đúng mục/version; kiểm tra conflict giữa các
mục và rule hiện hành; nếu một mục lỗi, giữ lựa chọn và cho phép bỏ mục
đó để chốt phần còn lại; nhóm đã chọn công bố cùng thành công.

### Skill sync 2B

**\[ĐÃ CHỐT\]**

1.  Watcher + hash check.
2.  Đọc file ổn định và parse toàn bộ.
3.  Đối chiếu ID, scope, version nền.
4.  Thay đổi rõ ràng/hợp lệ do người dùng lưu tự áp dụng từ request chưa
    gửi tiếp theo.
5.  Mơ hồ, invalid hoặc conflict thì chờ xử lý.
6.  Request đã gửi giữ version cũ.
7.  Request chưa gửi phải chuẩn bị lại nếu dependency đổi.
8.  Không reset model/budget/counters khi Skill đổi.
9.  Không tự resume task người dùng đã pause/cancel.

Khi tool publish decision, lưu intent/base hash/target hash, ghi file
tạm, kiểm tra base trước thay thế, xác minh file thực tế rồi mới kích
hoạt decision trong transaction hoàn tất.

### Dependency và test

Rule có thể scope theo series, nghĩa/định danh, tập/chương hoặc khoảng
hiệu lực. Cùng mặt chữ không đủ để coi là cùng nghĩa. Thay decision tạo
version mới. Test ưu tiên rule, 2B, approval, alias, pair direction,
flashback timeline, conflict và invalidation đúng phạm vi.

------------------------------------------------------------------------

## 6.4. SS-04 --- Translation

### Mục tiêu và trách nhiệm

Tạo bản dịch theo source scope, context và model đã chọn; validate
response và lưu draft revision có thể Review/phục hồi.

**\[ĐỀ XUẤT\] Components:**

-   Translation Intake.
-   Unit/Chunk Planner.
-   Context Request Builder.
-   JSON Request Assembler.
-   Model Invocation Client (qua SS-08).
-   Response Collector.
-   Structural Validator.
-   Candidate Gate.
-   Draft Revision Publisher.

### Không chịu trách nhiệm

Không tự chọn chương tiếp theo, tự Review, tự duyệt glossary, tự retry
vô hạn, đổi model dự phòng, điều phối queue toàn máy hoặc export EPUB.

### Pipeline năm bước đã chốt

1.  Nhận lệnh Dịch.
2.  Chuẩn bị dịch và ngữ cảnh.
3.  Tạo bản dịch.
4.  Kiểm tra cấu trúc.
5.  Lưu bản nháp.

Đầu ra là draft hợp lệ về cấu trúc, trạng thái chờ Review; chưa phải
chất lượng Đạt.

### Chunk planning

SS-02 cung cấp source blocks/ranges ổn định; Translation lập unit phù
hợp model capability. Mốc thử nghiệm khoảng 3.000 token nguồn/lượt. Tính
toàn bộ request gồm hướng dẫn, Skills/glossary, context, source, JSON
schema và adapter envelope. Dự trù output khoảng 2× source tokens cộng
JSON; reserve 5% khi tokenizer đúng, 10% khi ước lượng. Kiểm tra
input/output/context riêng.

Ưu tiên tokenizer đúng version. Nếu chỉ ước lượng phải hiển thị rõ;
thiếu limit đáng tin thì tạm dừng. Cắt context tùy chọn trước; không cắt
nguyên văn bắt buộc rồi khai báo đã giao đủ block.

### Request/response

**\[ĐÃ CHỐT\]** Request JSON có `schema_version`, units theo thứ tự,
ID/range, source text, protected markers, read-only context,
Skills/glossary/addressing đã duyệt, style instructions và output
contract.

Response gồm `units[{id,target}]`, `uncertainties[]`,
`continuity_candidates[]`, `rule_candidates[]`. Candidate/uncertainty
cần vị trí và source evidence. Hash/revision/task IDs do chương trình
quản lý.

### Structural validation

Kiểm tra:

-   Finish reason không phải truncated.
-   JSON/schema hợp lệ.
-   Đủ ID, đúng thứ tự, không duplicate/unknown.
-   Target không rỗng.
-   Inline marker/cặp token hợp lệ.
-   Ruby, image, note, link mapping còn đúng.
-   Subrange coverage đầy đủ, không overlap.
-   Candidate quotes tồn tại trong source.

Structural pass không chứng minh đúng nghĩa; Review chịu trách nhiệm đó.

### Candidate gate và commit

Nếu response dùng thuật ngữ/quy tắc chưa duyệt, giữ candidate nội bộ và
chặn phần phụ thuộc khỏi draft hiện hành. Sau khi user duyệt, dựng lại
phần phụ thuộc với rule version mới.

Commit qua SS-10, kiểm tra base generation, source/mapping, dependency,
writer permission và Markdown edit/hash. Sau commit trả
revision/generation; Workflow gọi Review đúng revision.

### Retry/timeout đã chốt

-   Network retry tối đa 2 lần ngoài lần đầu, cùng model.
-   Structural repair tối đa 2 lượt.
-   Timeout mặc định 10 phút/lần gọi.
-   Retry wait mặc định 10 giây rồi 30 giây.
-   Không tự đổi model.
-   Response bị cắt phải tính lại phạm vi/đầu ra.
-   Lỗi chưa rõ hoặc auth error chuyển người dùng.
-   Network/structural errors không tính thành quality repair round.

### Test

Token planning, schema, IDs, markers, ruby, candidate gate, stale
response, revision commit, Markdown recovery và thứ tự Review trước lượt
tiếp theo.

------------------------------------------------------------------------

## 6.5. SS-05 --- Review & Quality Control

### Mục tiêu và trách nhiệm

Đánh giá bản dịch theo nguyên tác, xác định issue có evidence, phạm vi
ảnh hưởng và repair scope phù hợp.

**\[ĐỀ XUẤT\] Components:**

-   Review Intake.
-   Source Reader.
-   Source Findings Store.
-   Bidirectional Comparator.
-   Coverage Tracker.
-   Evidence/Quote Validator.
-   Issue Builder.
-   Severity/Repair Evaluator.
-   Chapter Review Aggregator.
-   Repair Plan Builder.
-   User Exception Manager.

### Không chịu trách nhiệm

Không tạo revision bản dịch, không tự duyệt thuật ngữ/xưng hô, không tự
công bố sự kiện chưa xác minh, không tự điều phối AI, không chọn M4 theo
số lượng lỗi hoặc tổng điểm.

### Quy trình Review đã chốt

**Giai đoạn A -- đọc nguồn:** request chỉ chứa nguyên tác và context
nguồn đã kiểm chứng; không chứa bản dịch đang được đánh giá. Ghi chủ
thể, người nói/nghe, sự kiện, phủ định/giả định, thời điểm và điểm chưa
rõ.

**Giai đoạn B -- đối chiếu:** sau khi có source findings, gửi request có
cả nguyên tác và bản dịch. Kiểm tra hai chiều source→translation và
translation→source; kiểm tra quan hệ giữa đoạn/cảnh.

Không chỉ gửi cả nguồn và bản dịch trong một request rồi yêu cầu "đọc
nguồn trước".

### Coverage

Theo dõi riêng source đã đọc, source→translation, translation→source,
điểm nối/cảnh, phần chưa có kết luận và phần giữ nguyên theo story-only.
Ngữ cảnh chồng không được tính thành phần đã Review.

### Issue/evidence

Issue cần ID, loại, S0--S3, snapshot/source range, translation
revision/range, quote nguồn, quote bản dịch, giải thích, evidence
status, phạm vi ảnh hưởng, M scope, hướng xử lý và lịch sử.

-   Lỗi bỏ sót: quote nguồn và vị trí thiếu trong bản dịch.
-   Lỗi thêm: câu Việt và vùng nguồn đã kiểm tra để đánh giá sự vắng
    mặt.
-   Code kiểm tra ID/version/quote/coverage.
-   Reviewer đánh giá quote có thực sự hỗ trợ kết luận không.

### Severity và repair scope

**\[ĐÃ CHỐT\]**

  -----------------------------------------------------------------------
  Severity                            Ý nghĩa
  ----------------------------------- -----------------------------------
  S0                                  Gợi ý, đúng ý; không bắt buộc sửa

  S1                                  Nhỏ; lỗi rõ nhưng chưa đổi thông
                                      tin/cách hiểu

  S2                                  Lớn; mất/sai thông tin cần truyền
                                      đạt

  S3                                  Nghiêm trọng; thay đổi sự kiện,
                                      danh tính, quan hệ hoặc điều kiện
                                      then chốt
  -----------------------------------------------------------------------

  M    Hành động
  ---- ---------------------------------------------
  M1   Sửa trực tiếp vị trí chính xác
  M2   Dịch lại một đoạn
  M3   Dịch lại cảnh/nhóm đoạn liên quan
  M4   Dịch lại toàn chương sau kiểm chứng bổ sung

S và M là hai trục riêng. Không chọn M dựa trên số lượng lỗi hoặc tổng
điểm.

### M4 và repair rounds

M4 cần sai lệch cấp chương đã xác nhận, bằng chứng lan truyền, nguyên
nhân được kiểm tra và lượt verification bổ sung có thể xác nhận/bác/thu
hẹp. Nếu còn mâu thuẫn, chuyển Cần xác minh.

Tối đa năm vòng sửa--Review thất bại. Chỉ tính vòng khi bản sửa hợp lệ
được Review lại và lỗi của đợt đó vẫn còn. Review đầu tiên, network,
structural error và chờ duyệt có bộ đếm riêng. Hết năm vòng thì dừng tự
sửa và bàn giao bằng chứng/lịch sử.

User exception lưu người duyệt, lý do, issue, severity, phạm vi,
revision và căn cứ. Ngoại lệ không bỏ qua thiếu nội dung, Review chương,
decision bắt buộc hoặc lỗi kỹ thuật.

### Contract

-   Workflow → Review: snapshot/mapping/revision set/scope.
-   Review → Research: question/source position/context/spoiler
    boundary.
-   Review → Workflow: result/blocker/repair plan/user decision
    required.
-   Workflow → Translation: repair plan, source scope, revision base.
-   Translation → Review: revision mới đã commit.
-   Review → Context: candidate cần xác minh/duyệt.

### Test

Source-only request, bidirectional coverage, quote giả, quote không hỗ
trợ kết luận, M1--M4, M4 verification, five repair rounds, user
exception và review đúng revision.

------------------------------------------------------------------------

## 6.6. SS-06 --- Research

### Mục tiêu và trách nhiệm

Tra cứu thông tin nội bộ/Internet khi Translation, Review hoặc người
dùng cần căn cứ bổ sung.

**\[ĐỀ XUẤT\] Components:**

-   Research Request Intake.
-   Internal Evidence Resolver.
-   Web Search Adapter.
-   Page Reader.
-   Evidence Normalizer.
-   Spoiler Boundary Filter.
-   Research Cache.
-   Evidence Bundle Publisher.

### Không chịu trách nhiệm

Không duyệt glossary/xưng hô, không thay nguyên tác bằng wiki/anime,
không tự kết luận Review đạt, không chạy AI song song, không bịa
URL/quote, không đưa thông tin tương lai vào context hiện tại như sự
thật.

### Thứ tự tra cứu đã chốt

1.  Nguyên tác và ngữ cảnh tại vị trí cần xử lý.
2.  Skills/glossary/quyết định đã xác nhận còn hiệu lực.
3.  Internet và tài liệu tham khảo đã cấu hình.

Web adapter đầu tiên: **Brave Search API** do người dùng cấu hình. Cho
phép nguồn ưu tiên/chặn theo series.

### Giới hạn Research

**\[ĐÃ CHỐT -- MẶC ĐỊNH THỬ NGHIỆM\]** Mỗi câu hỏi tối đa 3 truy vấn
web, 5 trang đọc, 15 phút tổng cộng; cho phép chỉnh. Hết giới hạn thì
trả evidence đã có và phần còn thiếu. Search/page-read fees được ghi
riêng trong task cost. Model calls Research vẫn tính vào task
budget/call cap.

Cache theo câu hỏi, entity, mốc truyện, source scope, source config và
dependency version. Evidence nội bộ dùng lại khi còn hiệu lực; web
evidence kiểm tra lại sau 7 ngày hoặc sớm hơn khi có mâu thuẫn/nguồn
đổi.

### Evidence item

Cần nguồn, URL/vị trí nội bộ, excerpt đã đọc, hash excerpt, thời điểm
lấy, entity/event, thời điểm sự kiện, earliest reader disclosure và
verification status.

Evidence chưa rõ spoiler boundary được giữ trong hồ sơ nhưng loại khỏi
context cảnh hiện tại.

### Output và contract

Research trả câu hỏi, kết quả, evidence, suy luận có nhãn, phần chưa rõ,
rule liên quan, candidate nếu có và lịch sử model/usage/fee.

-   Translation/Review/User → Research: Research Request.
-   Research → Translation/Review: Evidence Bundle.
-   Research → Context: candidate cần duyệt.
-   Workflow giữ task gốc ở đúng step đang chờ.

Research result chưa tự thành approved glossary, verified context hoặc
Review pass.

### Test

Internal evidence reuse, Brave adapter contract, 3/5/15 limits,
URL/quote validation, source conflict, spoiler unknown, candidate
approval và recovery task gốc.

------------------------------------------------------------------------

## 6.7. SS-07 --- Workflow & Task Management

### Mục tiêu và hai phạm vi

Điều phối vòng đời task và quyền chạy AI toàn máy.

**A. Task Workflow Engine:** quản lý scope, chapter queue,
Translation→Review→Research→Repair, user wait, pause/resume/cancel,
checkpoint và next action.

**B. Machine-wide Coordinator:** quản lý queue, permit, task/workspace
ID, permit sequence, OS lock, handoff và crash recovery.

Hai phần không được nhập làm một. Workflow không tự cấp quyền AI toàn
máy; Coordinator không quản lý nội dung dịch.

### Dữ liệu sở hữu

Series SQLite: task record, user intent, scope, run mode, content scope
snapshot, current step, chapter/unit progress, checkpoints, waiting
reasons, next action, repair-round progress reference, model/budget
snapshot và task history.

Coordinator SQLite: queue order, task/workspace ID, requested action,
permit holder, permit sequence, process ownership và reconciliation
state.

### State

**\[ĐÃ CHỐT\]** Execution state: Chờ chạy, Đang chạy, Đang tạm dừng, Đã
tạm dừng, Chờ người dùng, Gặp lỗi vận hành, Hoàn tất, Đã hủy.

Chất lượng và validity là hai trục độc lập, không được gộp vào execution
state.

### Workflow chương

``` mermaid
flowchart TD
    A["Create / Resume task"] --> B["Check readiness"]
    B --> C["Prepare translation unit"]
    C --> D["Translation"]
    D --> E["Validate and commit"]
    E --> F["Review exact revision"]
    F --> G{"Outcome"}
    G -->|Issue| H["Repair plan"]
    H --> I["Translation repair"]
    I --> F
    G -->|Research| J["Research"]
    J --> F
    G -->|User decision| K["Wait for user"]
    K --> C
    G -->|Local pass| L{"More units?"}
    L -->|Yes| C
    L -->|No| M["Chapter-wide review"]
    M --> N{"Complete?"}
    N -->|No| H
    N -->|Yes| O["Next chapter / task complete"]
```

**\[ĐÃ CHỐT\]** Review ngay sau mỗi lượt; không dịch nháp tất cả lượt
trước. Review tổng thể cuối chương trước khi chuyển chương. Auto-to-end
giữ phạm vi toàn tập và tự chuyển chương đủ điều kiện.

### Một tác vụ AI hoạt động --- 13A

-   Một Coordinator theo user profile, dùng chung TUI/CLI/workspaces.
-   OS lock bảo đảm một Coordinator process.
-   Một task AI giữ permit tại một thời điểm.
-   Translation/Review/Research/verification/retry/repair là các bước
    tuần tự.
-   Retry wait không tự chuyển permit.
-   Khi task pause/chờ user, lưu state, xử lý xong request hiện hành ở
    phía tool rồi mới bàn giao quyền.
-   User có thể chủ động chọn task độc lập khác.
-   Task phụ thuộc decision chưa xác nhận vẫn chờ.
-   Local-only operations có thể chạy nếu không tranh chấp ghi.

### Queue và recovery

Chỉ task người dùng yêu cầu chạy/tiếp tục mới vào queue. Bấm lại cùng
task/scope mở hồ sơ cũ, không tạo bản sao. Trước khi cấp permit phải
kiểm tra source/edit/rules/config/budget.

Sau crash đối chiếu OS lock, permit sequence, queue, task/workspace IDs
và attempt ledger. Hai SQLite không có một transaction chung.

**\[ĐỀ XUẤT\]** Durable handoff intent/ack theo task ID và permit
sequence có thể hỗ trợ reconciliation.

**\[CÒN MỞ\]** Thứ tự ghi, acknowledgement format, timeout và xử lý
database hỏng cần thiết kế sâu hơn.

### Checkpoint

**\[ĐÃ CHỐT\]** Checkpoint sau prepare, trước send, khi nhận đủ
response, sau validate/commit, khi chuyển Review, chờ duyệt, retry và
hoàn tất. Trước send ghi ý định và reservation. Request đã gửi nhưng
chưa biết provider xử lý có trạng thái `Chưa rõ đã xử lý`.

### Test

One active AI task, queue, handoff, pause/resume, auto-to-end,
dependency blocker, crash reconciliation, no reset counters và
local-only operation.

------------------------------------------------------------------------

## 6.8. SS-08 --- Model Integration

### Mục tiêu và trách nhiệm

Cung cấp hợp đồng thống nhất cho Translation, Review và Research.

**\[ĐỀ XUẤT\] Components:**

-   Provider Registry.
-   Connection Resolver.
-   Model Catalog/Capability Resolver.
-   Provider Adapter.
-   Request/Streaming Transport.
-   Response Normalizer.
-   Usage Collector.
-   Price Catalog.
-   Cost Estimator/Reservation.
-   Budget Preflight.

### Không chịu trách nhiệm

Không chọn workflow step, không tự retry vô hạn, không đổi model
fallback, không tự thay task budget, không quyết định prompt nghiệp vụ
và không công bố revision.

### Ba role đã chốt

  Role          Mục đích
  ------------- ------------------------------------
  Translation   Tạo bản dịch/sửa theo Review
  Review        Đọc nguồn, đối chiếu và kiểm chứng
  Research      Tra cứu/hỗ trợ thuật ngữ

Research mặc định dùng model Review cùng task; có thể chọn riêng. Cùng
model không gộp request/context/history.

### Provider contract

Hỗ trợ one-shot và streaming với cùng một hồ sơ kết quả: request/attempt
IDs, requested alias, actual model ID nếu provider trả, finish reason,
content, usage, provider wait và error classification. Partial response
lưu riêng.

Capability/context/output limits phải có nguồn/version. Thiếu limit cần
thiết thì không đoán.

### Không tự đổi model

**\[ĐÃ CHỐT\]** Chỉ dùng model user chọn. Đổi model là thao tác chủ
động, áp dụng request chưa gửi; request đang chạy giữ model đã dùng. Khi
đổi, tính lại token/chunk/cost của phần chưa gửi, giữ kết quả đã hoàn
thành còn hợp lệ và toàn bộ lịch sử.

### Timeout/retry đã chốt

  Quy tắc                   Giá trị
  ------------------------- -----------------------------
  Timeout mỗi call          10 phút mặc định
  Retry service temporary   2 lần ngoài lần đầu
  Retry wait                10 giây, rồi 30 giây
  Structural repair         2 lượt
  Quality repair            5 vòng sửa--Review thất bại

Các bộ đếm tách biệt. Tôn trọng Retry-After dài hơn; lỗi chưa
rõ/auth/limit không retry liên tục.

### Request/attempt và late response

Một công việc logic có `request_id`; mỗi lần gửi có `attempt_id`,
deadline và input versions. Mỗi attempt có usage riêng. Response cũ đến
muộn được lưu trong attempt cũ; chỉ attempt hiện hành có quyền publish
trên nền version hợp lệ. Không áp kết quả hai lần hoặc ghi đè revision
mới.

### Budget/cost

Hai chế độ: đặt hạn mức hoặc không giới hạn chi phí. Lần thiết lập đầu
phải chọn rõ. Task mới kế thừa theo task → series → tool default; task
đã tồn tại giữ snapshot.

Trong cap mode:

`Recorded cost + outstanding reservations + next request estimate <= task limit`

Không đủ dữ liệu ước tính thì dừng và nêu phần thiếu; không tự chuyển
unlimited/model khác. Phân biệt provider usage, estimated cost,
reservation, unknown cost và Research tool fee. Unknown không được ghi
thành 0. Mỗi task dùng một currency; tỷ giá quy đổi do user nhập và lưu
ngày.

### Total model-call cap

Mặc định không đặt trần. User có thể đặt số nguyên dương theo task. Mọi
lần gửi model đều tính, kể cả retry/timeout/Research/repair. Lần N đã
gửi vẫn được nhận/lưu; chặn trước N+1. Resume, đổi model hoặc chuyển
chương không reset count.

### Credential

API key nằm trong OS Credential Store hoặc env source được chọn rõ.
Workspace/task chỉ giữ connection reference. Không tự quét `.env`, không
ghi key vào workspace/log/prompt/preview/EPUB. Không tự đổi
account/key/model khi auth fail.

### Test

Provider mock, streaming, capability source, no fallback,
timeout/retry/late response, usage/cost reconciliation, budget modes,
call cap, credential redaction và role separation.

------------------------------------------------------------------------

## 6.9. SS-09 --- Image & EPUB Export

### Mục tiêu và trách nhiệm

Quản lý ảnh source/edit, tạo Prepared Export Bundle bất biến, phục vụ
Preview và EPUB 3.3.

**\[ĐỀ XUẤT\] Components:**

-   Image Asset Registry.
-   Replacement Resolver.
-   Image Validator.
-   Prepared Bundle Builder.
-   Export Manifest Store.
-   Local Preview Session.
-   EPUB 3.3 Builder.
-   EPUB Validator.
-   Export Version Publisher.

### Image replacement đã chốt

-   `images/source/` và `images/edit/`.
-   Match chính xác relative path, case và extension.
-   JPEG, PNG, GIF, WebP, SVG tĩnh an toàn.
-   Edit match nhưng hỏng thì chặn export, không fallback âm thầm.
-   Edit bị xóa/di chuyển thì lần prepare mới dùng source.
-   Một source asset được tham chiếu nhiều nơi thì cùng dùng một ảnh đã
    chọn.
-   Giai đoạn đầu thay ở cấp asset, không theo occurrence.

**Mặc định thử nghiệm:** 64 MiB/file, 16.384 px mỗi chiều, 40 triệu
px/frame, 100 triệu px cộng dồn ảnh động. SVG phải tĩnh/allowlist, chặn
script và external resource.

### Prepared Export Bundle

Manifest bất biến theo hash gồm:

-   Snapshot/mapping.
-   Content scope và phần giữ nguyên.
-   Translation revisions.
-   Review results/user exceptions.
-   Approved rules.
-   Ruby representation decisions.
-   Metadata/layout.
-   Selected image source/edit hashes.
-   Resource references và validation inputs.

Thay chữ, ảnh, metadata, ruby, scope hoặc layout liên quan khiến bundle
cũ cần đánh giá/prepare lại.

### Export readiness

**\[ĐÃ CHỐT\]** Chỉ xuất EPUB hoàn chỉnh của volume trong giai đoạn đầu.

Cần đủ phần dịch theo scope, Review hiện hành, Review tổng thể chương,
dependency còn hiệu lực, không có decision nghĩa bắt buộc đang chờ,
metadata bắt buộc và kỹ thuật hợp lệ.

Story-only giữ trang phụ đúng vị trí/link, không gán Đã dịch/Đạt Review.
S0 không tự chặn. User exception chỉ miễn đúng issue/revision, không bỏ
qua các điều kiện khác.

### Preview

Preview và EPUB builder dùng cùng Prepared Export Bundle.

**\[ĐÃ CHỐT\]** Loopback only, temporary port, one-time 256-bit secret,
fragment-to-POST exchange, same-origin/Host checks, no token in
log/referrer, resource access by bundle IDs, 30-minute idle timeout mặc
định thử nghiệm. Không chạy script/event handler/form/iframe chủ động,
không tải network resources hoặc path tùy ý. Bundle đổi thì Preview
stale.

### EPUB 3.3

**\[ĐÃ CHỐT\]**

-   EPUB 2/3 đều xuất EPUB 3.3.
-   EPUB 2 NCX chuyển thành Navigation XHTML.
-   Dựng package/manifest/spine/metadata/link hợp lệ.
-   `mimetype` đầu gói, không nén.
-   Bản Việt đọc ngang trái sang phải.
-   Giữ chapter/part/paragraph/scene
    break/notes/anchors/images/emphasis.
-   Ruby thuần phát âm không hiện chú âm sau khi xác định; ruby có nghĩa
    riêng theo quyết định đã duyệt.
-   Readback, XML/resource/coverage checks và EPUBCheck 5.3.0.
-   Bước xuất không gọi model để dịch thêm.

### Version publish

Mỗi lần export mới tạo file version mới theo volume; giữ file cũ.
Version reservation trong transaction. Lệnh mới sau r003 thất bại lấy
r004; resume chính task r003 thì phục hồi r003. Build/validate temp
file, ghi hash/path/export ID rồi publish không overwrite. Recovery theo
ID/path/hash/report.

### Test

Exact asset matching, invalid edit blocks export, prepared bundle
immutable, Preview/export cùng bundle, stale preview, EPUB 2→3.3,
EPUBCheck, full-volume readiness, version monotonicity và recovery.

------------------------------------------------------------------------

## 6.10. SS-10 --- Persistence & Synchronization

### Mục tiêu và trách nhiệm

Cung cấp storage, commit, versioning, file sync và recovery nhất quán.

**\[ĐỀ XUẤT\] Components:**

-   Series SQLite Repository.
-   Coordinator SQLite Repository.
-   Immutable File Store.
-   Generation Manager.
-   Current-State/History Store.
-   Dependency Index.
-   Workspace Writer Lock/Version Guard.
-   Markdown Draft Synchronizer.
-   Skill Publish/Synchronizer.
-   Schema Migration/Recovery.

### Không chịu trách nhiệm

Không quyết định bản dịch đúng/sai, không duyệt rule, không tự chọn
revision, không điều phối workflow, không chứa API key, không coi mọi
Markdown là cùng semantics và không tự triển khai Event Sourcing thuần
túy.

### Hai SQLite đã chốt

  -----------------------------------------------------------------------
  Store                   Phạm vi                 Dữ liệu
  ----------------------- ----------------------- -----------------------
  Series SQLite           Một DB mỗi series       Mapping, revision,
                                                  decision, dependency,
                                                  Review, task, usage,
                                                  budget, export history

  Coordinator SQLite      Một DB theo user        Queue order,
                          profile                 task/workspace IDs,
                                                  permit holder/sequence
  -----------------------------------------------------------------------

Coordinator DB không lưu source text, translation hoặc Review evidence.

### Immutable File Store

Filesystem lưu EPUB nguồn, ảnh source/edit, Skills Markdown, Translation
Markdown, file bất biến và EPUB xuất. SQLite giữ metadata/reference.
File chưa được tham chiếu có thể GC sau theo chính sách an toàn.

### Generation/Atomic Commit

**\[ĐÃ CHỐT\]** Generation có ID cha, change references và checksum.
Revision, Review references, decision, dependency, progress, metadata và
current pointer liên quan được công bố trong một SQLite transaction.

File bất biến: ghi temp → hoàn tất/hash → đặt vào store → commit
reference trong SQLite. File orphan chỉ được GC khi chắc chắn không có
generation/manifest tham chiếu.

Current state được lưu để đọc nhanh cùng history để truy ngược; không
yêu cầu Event Sourcing thuần túy.

### Markdown draft sync

-   Lưu base revision/hash và marker mỗi unit.
-   Watch file và kiểm tra hash khi open/trước write/trước resume.
-   Edit hợp lệ được nhập thành revision sửa tay.
-   Marker thiếu/trùng: giữ file, chỉ vùng mất mapping và cho user gắn
    lại.
-   Xung đột AI/user: base/user/AI comparison.
-   Merge phần không overlap nếu base khớp; phần overlap cần user
    chọn/kết hợp.
-   Không ghi đè sửa tay bằng AI response cũ.

Nếu generation commit nhưng Markdown sync fail, báo
`Đã lưu bản dịch, chờ dựng Markdown`; phục hồi từ generation và bảo vệ
edit tay.

### Skill sync

Skill file có semantics khác Translation Markdown. External user edit
rõ/hợp lệ theo 2B; tool publish approved decision phải lưu intent/base
hash/target hash, temp write, base check, safe replace, reread/hash
verify và chỉ activate decision sau khi DB/file đối chiếu. Conflict giữ
cả hai phía.

### Lock/migration/recovery

-   Một writer hợp lệ cho workspace.
-   Version check trước commit.
-   Lock không thay thế optimistic version check.
-   Schema Rust mới bắt đầu 1; không migrate workspace Go test.
-   Migration tương lai khóa ghi, backup DB/file sẽ sửa, chuyển từng
    version, integrity check, rollback nếu lỗi.
-   Mở lại phải phục hồi task, revision, sửa tay, decision,
    usage/cost/counters và sync intent.

### Test

Atomic commit, orphan files, generation checksum/current pointer,
Markdown recovery, user edit revision, three-way conflict, Skill publish
recovery, writer lock, migration rollback và reopen.

------------------------------------------------------------------------

# 7. Luồng liên subsystem

## 7.1. Import EPUB

``` mermaid
sequenceDiagram
    participant UI as TUI/CLI
    participant WS as Workspace
    participant EP as EPUB Processing
    participant PS as Persistence

    UI->>WS: Open workspace / register volume
    WS->>PS: Open Series DB, check version and lock
    PS-->>WS: Workspace ready
    WS->>EP: Import EPUB
    EP->>EP: Validate, parse, map, classify
    EP->>PS: Persist snapshot/mapping/resources
    PS-->>EP: Commit result
    EP-->>WS: Import result
    WS-->>UI: Content tree + unresolved items
```

Import chạy cục bộ, không gọi model.

## 7.2. Translation → Review → Repair

``` mermaid
sequenceDiagram
    participant WF as Workflow
    participant CTX as Context
    participant TR as Translation
    participant MG as Model
    participant PS as Persistence
    participant RV as Review
    participant RS as Research

    WF->>TR: Create draft for unit
    TR->>CTX: Resolve approved context
    CTX-->>TR: Context + dependency
    TR->>MG: Translation request
    MG-->>TR: Candidate + usage/attempt
    TR->>TR: Structural validation
    TR->>PS: Commit draft revision
    PS-->>TR: Revision + generation
    TR-->>WF: Draft committed, pending Review
    WF->>RV: Review exact revision
    RV->>MG: Source-only request
    MG-->>RV: Source findings
    RV->>MG: Comparison request
    MG-->>RV: Issues/evidence candidates
    opt Research needed
        RV->>RS: Research request
        RS-->>RV: Evidence bundle
    end
    RV->>PS: Commit Review result
    RV-->>WF: Pass / repair / user wait
    opt Repair required
        WF->>TR: Repair plan + revision base
        TR->>PS: Commit new revision
        WF->>RV: Review new revision
    end
```

Các request có thể chia nhỏ theo giới hạn dung lượng; sơ đồ chỉ thể hiện
thứ tự trách nhiệm.

## 7.3. Skill change và dependency

``` mermaid
flowchart TD
    A["Skill file changed"] --> B["Watcher + hash check"]
    B --> C["Read stable file"]
    C --> D["Parse IDs/scope/base version"]
    D --> E{"Clear and valid?"}
    E -->|No| F["Keep valid version and pending conflict"]
    E -->|Yes| G["Publish new rule version"]
    G --> H["Use from next unsent request"]
    H --> I["Find reverse dependencies"]
    I --> J["Mark affected results for recheck"]
    J --> K["Keep unaffected revisions and user edits"]
```

Request đã gửi giữ version cũ. Không gắn nhãn version mới vào response
cũ.

## 7.4. Coordinator handoff

``` mermaid
sequenceDiagram
    participant UI as TUI/CLI
    participant CO as Coordinator
    participant CDB as Coordinator DB
    participant SDB as Series DB

    UI->>CO: Request task A
    CO->>CDB: Enqueue/grant permit
    CO->>SDB: Record start + permit sequence
    UI->>CO: Request task B
    CO->>CDB: Enqueue B
    CO-->>UI: B queued
    UI->>CO: Pause/handoff A
    CO->>SDB: Save checkpoint and attempt state
    CO->>CDB: Release/handoff permit
    CO->>SDB: Reconcile task end/handoff
    CO-->>UI: A paused, permit available
```

Chi tiết thứ tự ghi/ack giữa hai SQLite còn mở; không giả định
distributed transaction.

## 7.5. Prepare → Preview → Export

``` mermaid
flowchart TD
    A["Check volume readiness"] --> B["Resolve accepted revisions"]
    B --> C["Resolve source/mapping/scope"]
    C --> D["Resolve approved rules/ruby"]
    D --> E["Resolve image assets"]
    E --> F["Build immutable manifest"]
    F --> G{"Preview?"}
    G -->|Yes| H["Local Preview"]
    H --> I["Inspect bundle"]
    I --> J{"Stale?"}
    J -->|Yes| A
    J -->|No| K["EPUB Builder"]
    G -->|No| K
    K --> L["EPUB 3.3 validation"]
    L --> M{"Valid?"}
    M -->|No| N["Report blockers"]
    M -->|Yes| O["Publish new version"]
```

Export không gọi model để lấp phần thiếu.

------------------------------------------------------------------------

# 8. Trạng thái và consistency boundary

## 8.1. Ba trục trạng thái

  -----------------------------------------------------------------------
  Trục                                Ví dụ
  ----------------------------------- -----------------------------------
  Chất lượng                          Chưa dịch, Chờ Review, Cần sửa, Cần
                                      xác minh, Đạt, Người dùng giữ ---
                                      có ngoại lệ

  Hiệu lực                            Còn phù hợp, Cần kiểm tra lại

  Thực thi                            Chờ chạy, Đang chạy, Đang tạm dừng,
                                      Đã tạm dừng, Chờ người dùng, Gặp
                                      lỗi vận hành, Hoàn tất, Đã hủy
  -----------------------------------------------------------------------

Ví dụ: `Đạt ở r2`, `Cần kiểm tra lại do glossary đổi`,
`Task đã tạm dừng`.

## 8.2. Ranh giới commit

  -----------------------------------------------------------------------
  Hoạt động                           Dữ liệu cần commit cùng nhau
  ----------------------------------- -----------------------------------
  Import                              Snapshot, mapping, resource
                                      references, import state

  Mapping edit                        Mapping version và source coverage

  Translation                         Revision, dependency, generation,
                                      progress

  Review                              Review run, issue/evidence,
                                      coverage và conclusion đúng
                                      revision

  Rule approval                       Decision version, approval history,
                                      Skill publish state

  Research                            Query/evidence metadata và
                                      verification state

  Task checkpoint                     Task state, current step,
                                      request/attempt refs, next action

  Coordinator permit                  Queue/permit transition trong
                                      Coordinator DB và đối chiếu Series
                                      DB

  Prepare export                      Immutable manifest và asset hashes

  EPUB publish                        Version reservation, temp artifact,
                                      validation report, final publish
                                      record
  -----------------------------------------------------------------------

## 8.3. SQLite và filesystem

SQLite transaction không bao trùm filesystem. File bất biến phải được
ghi/hash xong trước khi DB commit reference. Markdown draft và Skill
publish có giao thức riêng vì user có thể sửa file bên ngoài.

## 8.4. Hai database độc lập

Series SQLite và Coordinator SQLite không có transaction chung. Hợp đồng
yêu cầu đối chiếu task ID/workspace ID/permit sequence/attempt ledger
sau crash.

**\[CÒN MỞ\]** Thứ tự ghi, durable acknowledgement, reconciliation
timeout và xử lý khi một database hỏng.

## 8.5. Dependency invalidation

Khi source/mapping/rule/addressing/translation đổi:

-   Tìm dependency ngược.
-   Đánh dấu đúng kết quả cần kiểm tra lại.
-   Giữ lịch sử và revision cũ.
-   Không tự dịch lại toàn series.
-   Nếu không chứng minh được phạm vi nhỏ hơn, giữ phạm vi kiểm tra của
    toàn lượt đã nhận context đó.
-   Review có thể xác nhận bản cũ vẫn phù hợp; nếu sai, chọn M1--M4.

------------------------------------------------------------------------

# 9. Lỗi và recovery

## 9.1. Error ownership

  Nhóm lỗi                                           Owner xử lý
  -------------------------------------------------- ------------------------------
  EPUB package/resource/path                         EPUB Processing
  Chapter/mapping/coverage                           EPUB Processing
  Skill parse/rule conflict/addressing unknown       Context & Skills
  Network/provider/timeout                           Model Integration + Workflow
  Invalid key/model/capability                       Model Integration
  JSON/ID/marker response                            Translation hoặc Review
  Sai nghĩa/thiếu/thêm nội dung                      Review
  No source/contradictory evidence/spoiler unknown   Research
  SQLite/filesystem/hash/lock                        Persistence
  Invalid image/EPUB/package/export                  Image & Export
  User approval/conflict/exception                   Owner nghiệp vụ + Workflow

## 9.2. Bộ đếm không được gộp

  -----------------------------------------------------------------------
  Bộ đếm                              Quy tắc
  ----------------------------------- -----------------------------------
  Network retry                       Tối đa 2 lần ngoài lần đầu

  Structural repair                   Tối đa 2 lượt

  Quality repair                      Tối đa 5 vòng sửa--Review thất bại

  Optional model-call cap             Mặc định không đặt; nếu bật, mọi
                                      model send đều tính

  Research cap                        Mặc định thử nghiệm 3 query, 5
                                      trang, 15 phút/câu hỏi
  -----------------------------------------------------------------------

## 9.3. Recovery matrix

  -----------------------------------------------------------------------
  Điểm dừng/crash                     Hành vi
  ----------------------------------- -----------------------------------
  Import chưa commit                  Không báo hoàn tất; có thể chạy lại
                                      từ source

  Snapshot/mapping đã commit          Dùng lại dữ liệu đã lưu

  Request gửi nhưng chưa rõ provider  `Chưa rõ đã xử lý`, đối chiếu nếu
  xử lý                               provider hỗ trợ

  Response đã lưu, chưa validate      Validate response đã lưu trước khi
                                      gọi lại

  Response hợp lệ, chưa commit        Check base version/lock rồi commit

  Generation commit, Markdown chưa    Dựng lại từ generation, bảo vệ user
  dựng                                edits

  Draft commit, chưa Review           Review đúng revision

  Review partial                      Giữ coverage đã có, tiếp tục phần
                                      thiếu

  Repair plan commit, chưa draft      Tiếp tục đúng plan/round

  Skill publish dở                    Đối chiếu intent/hash/file, tiếp
                                      tục hoặc conflict

  Task paused/waiting user            Giữ bước chờ, không tự chấp nhận

  Coordinator crash                   Đối chiếu OS lock, permit, queue và
                                      attempt ledger

  Prepared bundle đã tạo              Preview/Export nếu manifest còn
                                      hiện hành

  Preview stale                       Prepare lại

  EPUB temp đã tạo, chưa publish      Recovery theo export
                                      ID/hash/report, không báo thành
                                      công

  EPUB đã publish, history chưa hoàn  Reconcile theo ID/path/hash/report,
  tất                                 không overwrite
  -----------------------------------------------------------------------

------------------------------------------------------------------------

# 10. Bảo mật

## 10.1. Quyền theo subsystem

  -----------------------------------------------------------------------
  Tài nguyên                          Quyền
  ----------------------------------- -----------------------------------
  Workspace                           Read/write lock và identity

  Source EPUB                         Read-only sau import; snapshot/hash

  Translation revision                Translation/approved edit workflow
                                      tạo revision

  Review evidence                     Review sở hữu kết luận

  Approved rules                      Context & Skills publish sau
                                      approval

  API key                             Credential/Provider Adapter đọc khi
                                      gửi

  Coordinator permit                  Coordinator cấp/thu hồi

  Prepared bundle                     Export tạo; Preview chỉ đọc

  Export files                        Export publish version mới, không
                                      overwrite
  -----------------------------------------------------------------------

## 10.2. Credential

-   API key trong OS Credential Store hoặc env source đã chọn.
-   Workspace/task chỉ giữ connection reference.
-   Không tự quét `.env`.
-   Không ghi key vào log, prompt, task history, preview hoặc EPUB.
-   Auth failure không tự đổi account/key/model.

## 10.3. EPUB và Preview

EPUB là dữ liệu không đáng tin cậy:

-   Không thực thi script/event handler.
-   Chặn form/iframe chủ động.
-   Chặn network resource references.
-   Chặn file path tùy ý.
-   Chỉ phục vụ resource trong Prepared Export Bundle.
-   Kiểm tra Host/origin/session.
-   Không cho nội dung sách truy cập config, secret hoặc workspace tùy
    ý.

Preview loopback-only, cổng tạm, one-time 256-bit secret,
fragment-to-POST exchange, same-origin/Host checks, no token in
log/referrer, 30-minute idle timeout mặc định thử nghiệm và resource
access theo ID nội bộ.

------------------------------------------------------------------------

# 11. Kiểm thử và nghiệm thu

## 11.1. Test pyramid đề xuất

1.  Unit tests: domain rules, mapping, state transition, budget.
2.  Contract tests: input/output, errors, idempotency và version checks.
3.  Integration tests: SQLite/filesystem, provider mock, Coordinator,
    watcher, EPUB builder.
4.  Workflow tests: Import → Translation → Review → Repair → Export.
5.  Recovery tests: crash injection tại checkpoint/commit.
6.  Acceptance tests: EPUB fixtures và bộ chuẩn Nhật--Việt.

CI offline, deterministic, không gọi provider trả phí.

## 11.2. Ma trận subsystem

  -----------------------------------------------------------------------
  Subsystem                           Kiểm thử bắt buộc
  ----------------------------------- -----------------------------------
  Workspace                           create/open, unsupported format,
                                      config inheritance, lock, schema
                                      version

  EPUB                                EPUB 2/3, spine/nav, mapping, range
                                      coverage, scope, ZIP/path limits

  Context                             precedence, scope, 2B sync,
                                      candidate approval, addressing
                                      timeline

  Translation                         token planning, JSON schema,
                                      ID/marker validation, candidate
                                      gate, revision commit

  Review                              source-first, bidirectional
                                      coverage, evidence, S0--S3, M1--M4,
                                      chapter review

  Research                            Brave adapter, 3/5/15, evidence,
                                      spoiler boundary, cache

  Workflow                            one AI task, queue, pause/resume,
                                      handoff, checkpoints, auto-to-end

  Model                               provider mock, timeout, retry, late
                                      response, model identity,
                                      budget/cost

  Export                              image validation, bundle, preview
                                      security, EPUBCheck, version
                                      publish

  Persistence                         atomic commit, generation,
                                      Markdown/Skill sync, lock,
                                      migration, recovery
  -----------------------------------------------------------------------

## 11.3. End-to-end acceptance

1.  Tạo workspace mới và import EPUB 2.
2.  Import EPUB 3 có navigation, ruby và fixed-layout image xen kẽ.
3.  All-text đưa trang phụ vào phạm vi.
4.  Story-only giữ trang phụ đúng vị trí, không gắn nhãn đã dịch.
5.  Dịch một lượt và Review đúng revision trước lượt tiếp theo.
6.  Candidate thuật ngữ chờ người dùng.
7.  Duyệt nhiều thuật ngữ, chỉ publish đúng mục được chọn.
8.  Skill external edit áp dụng 2B.
9.  Reviewer chọn M1/M2 cho lỗi S3 cục bộ khi phạm vi sửa được khoanh.
10. M4 verification có thể bác/thu hẹp đề xuất.
11. Research giữ Cần xác minh khi nguồn mâu thuẫn.
12. Task B queue trong lúc A chạy.
13. Pause/handoff A rồi user chọn B độc lập.
14. Crash giữa Coordinator DB và Series DB rồi reconcile.
15. Crash sau generation commit trước Markdown sync.
16. User edit trong lúc model chạy không bị response cũ ghi đè.
17. Chỉ đổi ảnh không gọi model dịch lại.
18. Preview/EPUB cùng Prepared Export Bundle.
19. Metadata đổi làm bundle stale.
20. EPUB 3.3 EPUBCheck đạt và version mới không overwrite file cũ.

## 11.4. Bộ chuẩn chất lượng

**\[ĐÃ CHỐT\]** 30 tình huống Nhật--Việt và 3 chương ngắn tự soạn, có
đối chứng do người đọc nguyên tác Nhật và biên tập tiếng Việt xác nhận.
Bao phủ phủ định, chủ thể ẩn, thêm/bớt ý, hồi tưởng, xưng hô, đa nghĩa,
ruby, điểm nối, trích dẫn giả, ngoại lệ và cách diễn đạt khác nhưng
đúng.

Ca thiếu đồng thuận gắn nhãn chờ duyệt. Model thật được đánh giá thủ
công có ngân sách; CI dùng mock. Không tuyên bố AI hiểu nguyên tác 100%.

------------------------------------------------------------------------

# 12. Các quyết định còn mở

  ------------------------------------------------------------------------
  ID                      Vấn đề                  Cần quyết định
  ----------------------- ----------------------- ------------------------
  O-01                    Subsystem boundary      Ranh giới component
                                                  chính thức

  O-02                    Data ownership          Record/table nào thuộc
                                                  owner nào; quyền
                                                  read/write

  O-03                    Contract schema         Field, error code,
                                                  versioning, idempotency

  O-04                    Coordinator transport   Named Pipe, Unix Socket
                                                  hay IPC khác

  O-05                    Coordinator lifecycle   Daemon/service hay
                                                  process được host

  O-06                    Coordinator             Thứ tự ghi, durable
                          reconciliation          intent/ack, permit
                                                  recovery

  O-07                    Workspace lock          OS lock primitive và
                                                  stale lock handling

  O-08                    Local concurrency       Local-only operations
                                                  nào được chạy đồng thời

  O-09                    SQLite transactions     Transaction boundary,
                                                  isolation, busy timeout

  O-10                    Immutable file GC       Orphan
                                                  retention/reference
                                                  scan/delete safety

  O-11                    Markdown sync           Watcher debounce, stable
                                                  read, merge conflict

  O-12                    Skill schema evolution  Template versioning và
                                                  parser compatibility

  O-13                    Model catalog           Nguồn cập nhật
                                                  capability/model limits

  O-14                    Provider rollout        Provider adapters đầu
                                                  tiên và contract tests

  O-15                    Tokenizer fallback      Xử lý khi
                                                  tokenizer/capability
                                                  không đáng tin

  O-16                    Raw request retention   Thời gian lưu,
                                                  redaction, user deletion

  O-17                    Research retention      Lưu raw search/page
                                                  response theo điều khoản
                                                  dịch vụ

  O-18                    EPUB parser boundaries  Biến thể
                                                  XHTML/OPF/navigation cần
                                                  hỗ trợ

  O-19                    Fixed-layout            Giới hạn trang ảnh
                                                  fixed-layout xen
                                                  reflowable

  O-20                    Preview implementation  Server/sanitizer
                                                  allowlist/browser
                                                  behavior

  O-21                    Export naming           File naming và path
                                                  collision

  O-22                    Export cancellation     Cancel ở
                                                  build/validate/publish

  O-23                    Progress delivery       Polling hay IPC event
                                                  stream, reconnect/order

  O-24                    Error taxonomy          Error codes và CLI exit
                                                  mapping

  O-25                    Backup/restore          Chính sách
                                                  backup/restore vận hành

  O-26                    TUI runtime             Framework, event loop và
                                                  rendering

  O-27                    Architecture            CI ngăn dependency
                          enforcement             vòng/truy cập repository
                                                  trái quyền
  ------------------------------------------------------------------------

## 12.1. Không mở lại các quyết định đã chốt

Các điểm sau đã được xác nhận và không được coi là open chỉ vì chi tiết
implementation chưa hoàn tất:

-   Một SQLite mỗi series; Coordinator SQLite riêng theo user profile.
-   Workspace Rust schema 1; không chuyển workspace test Go.
-   Generation/Atomic Commit và immutable references.
-   Ba trục trạng thái.
-   Một task AI hoạt động tại một thời điểm.
-   Không tự đổi model.
-   Retry tối đa 2; structural repair tối đa 2; quality repair tối đa 5
    vòng.
-   Timeout 10 phút; retry wait 10/30 giây.
-   Hai chế độ budget; kế thừa task → series → tool default.
-   Translation JSON schema version và validation.
-   Review source-only trước comparison.
-   Severity S0--S3; repair M1--M4.
-   Research role, Brave Search API adapter đầu tiên, 3/5/15 mặc định
    thử nghiệm.
-   Skill approval và sync 2B.
-   Preview loopback, one-time 256-bit token.
-   Prepared Export Bundle dùng chung Preview/Export.
-   EPUB 3.3, EPUBCheck 5.3.0.
-   Export mỗi lần version mới, không overwrite.
-   TUI/CLI dùng chung core.

------------------------------------------------------------------------

# 13. Ranh giới với Bước 3

Bước 2 kết thúc khi subsystem responsibilities, data ownership, contract
logic và consistency boundary được rà soát/xác nhận.

[Bước 3](RUST_PROJECT_ARCHITECTURE.md) chuyển các ranh giới này sang Cargo workspace và đồ thị phụ thuộc crate. [Bước 4](MODULE_DESIGN_STEP4.md) chi tiết hóa module, ownership và các cổng giao tiếp. [Bước 5](TECHNICAL_FOUNDATION_STEP5.md) tiếp tục với nền tảng kỹ thuật và hợp đồng giữa các cổng. Schema và transaction thuộc [Bước 6](DATA_ARCHITECTURE_STEP6.md). IPC, checkpoint và cách phục hồi được đối chiếu với [Bước 7](WORKFLOW_STATE_MACHINE_STEP7.md).

Không tạo cây thư mục hoặc trait signature trước khi contract logic được
duyệt.

------------------------------------------------------------------------

# 14. Ma trận truy xuất đặc tả

  -----------------------------------------------------------------------------------
  Subsystem               Đặc tả nguồn chính                  Quyết định 36 mục
  ----------------------- ----------------------------------- -----------------------
  SS-01 Workspace         `DATA_MODEL_STATE_SPEC.md`          06--10

  SS-02 EPUB Processing   `EPUB_INPUT_SPEC.md`                01--05

  SS-03 Context & Skills  `SKILLS_GLOSSARY_SPEC.md`,          11--16
                          `ADDRESSING_CONTINUITY_SPEC.md`     

  SS-04 Translation       `TRANSLATION_STAGE_SPEC.md`,        05, 17--20
                          `MODEL_STRATEGY_SPEC.md`            

  SS-05 Review            `REVIEW_STAGE_SPEC.md`,             21--25
                          `REVIEW_RUBRIC_SPEC.md`             

  SS-06 Research          `RESEARCH_STAGE_SPEC.md`            26--27

  SS-07 Workflow          `WORKFLOW_ORCHESTRATION_SPEC.md`,   07, 09, 10, 28--29
                          `DATA_MODEL_STATE_SPEC.md`          

  SS-08 Model             `MODEL_STRATEGY_SPEC.md`,           17--19
                          `SECURITY_SPEC.md`                  

  SS-09 Image/Export      `IMAGE_EDIT_SPEC.md`,               30--33
                          `EPUB_OUTPUT_SPEC.md`,              
                          `SECURITY_SPEC.md`                  

  SS-10 Persistence       `DATA_MODEL_STATE_SPEC.md`,         06--10, 13, 28--29, 33
                          `SKILLS_GLOSSARY_SPEC.md`           
  -----------------------------------------------------------------------------------

## 14.1. Tài liệu nguồn

-   [ARCHITECTURE_OVERVIEW.md](ARCHITECTURE_OVERVIEW.md)
-   [DESIGN_DECISIONS_36.md](../spec/DESIGN_DECISIONS_36.md)
-   [EPUB_INPUT_SPEC.md](../spec/EPUB_INPUT_SPEC.md)
-   [DATA_MODEL_STATE_SPEC.md](../spec/DATA_MODEL_STATE_SPEC.md)
-   [SKILLS_GLOSSARY_SPEC.md](../spec/SKILLS_GLOSSARY_SPEC.md)
-   [ADDRESSING_CONTINUITY_SPEC.md](../spec/ADDRESSING_CONTINUITY_SPEC.md)
-   [MODEL_STRATEGY_SPEC.md](../spec/MODEL_STRATEGY_SPEC.md)
-   [TRANSLATION_STAGE_SPEC.md](../spec/TRANSLATION_STAGE_SPEC.md)
-   [REVIEW_STAGE_SPEC.md](../spec/REVIEW_STAGE_SPEC.md)
-   [REVIEW_RUBRIC_SPEC.md](../spec/REVIEW_RUBRIC_SPEC.md)
-   [RESEARCH_STAGE_SPEC.md](../spec/RESEARCH_STAGE_SPEC.md)
-   [WORKFLOW_ORCHESTRATION_SPEC.md](../spec/WORKFLOW_ORCHESTRATION_SPEC.md)
-   [IMAGE_EDIT_SPEC.md](../spec/IMAGE_EDIT_SPEC.md)
-   [SECURITY_SPEC.md](../spec/SECURITY_SPEC.md)
-   [EPUB_OUTPUT_SPEC.md](../spec/EPUB_OUTPUT_SPEC.md)
-   [TUI_CLI_SPEC.md](../spec/TUI_CLI_SPEC.md)
-   [Quyết định 36 — Kiểm thử và nghiệm thu](../spec/DESIGN_DECISIONS_36.md#decision-36)

------------------------------------------------------------------------

## Kết luận

Bước 2 đề xuất mười subsystem với trách nhiệm, quyền sở hữu dữ liệu và
contract riêng. Workflow điều phối thứ tự; Model Integration thực hiện
hợp đồng provider và usage/cost; Context & Skills quản lý quyết định đã
duyệt; Translation tạo revision; Review đánh giá đúng revision; Research
cung cấp evidence; Export chỉ đóng gói dữ liệu đã được chấp nhận;
Persistence bảo đảm commit và recovery.

Các quyết định trong 36 mục được giữ nguyên. Những phần chưa có hợp đồng
kỹ thuật đủ chi tiết được đánh dấu \[ĐỀ XUẤT\] hoặc \[CÒN MỞ\], không tự
biến thành yêu cầu đã xác nhận.

Tài liệu này là cơ sở rà soát subsystem boundary trước khi chuyển sang
thiết kế Rust crate/module ở Bước 3.
