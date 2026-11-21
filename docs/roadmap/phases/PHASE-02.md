# PHASE 02 — LƯU TRỮ WORKSPACE VÀ NHẬP EPUB

> Lịch thực thi v2 · 12/11/2026, đã nhận bổ sung T14–T18/CP08–CP09 tại §17 ngày 21/11/2026.

> Lịch task song song và checkpoint commit/push tại [mục 10](#p02-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Căn cứ lịch sử — kế hoạch v0.3 · 09/11/2026, trạng thái ban đầu Chưa bắt đầu.** Bản kế hoạch nhận hợp đồng từ [M0-02-r01](../evidence/phase-00/M0-02.review-r01.md) và chuẩn bị công việc sau M1. Trạng thái hiện hành ở §1, §16 và §17; dòng lịch sử này không thay kết quả công bố. Căn cứ là yêu cầu đã chốt và thiết kế. Thay đổi trong code cũ hoặc spike chỉ dùng để nhận diện phần cần bảo toàn và đối chiếu bằng chứng kỹ thuật.
>
> Giữ truy vết Q02 đã bổ sung ở P00-T01-r02 sau finding F02. T08/AC08/TC08 kiểm quyết định phân loại theo **vị trí, hash EPUB và phiên bản mapping**, gồm hai ca thay từng căn cứ riêng.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P02 / M2 |
| Tên | Lưu trữ workspace và nhập EPUB |
| Trạng thái | T01–T13 cùng bổ sung T11 đã công bố tại CP01–CP07. Đang tiếp nhận góp ý T14–T18 ở CP08/CP09; M2 chờ Master nghiệm thu. |
| Đầu vào chính | M0-02 và M1 đã Đạt trên Windows; M1 merge `9fb133078a126171186a0a12a1359443746363d8` nằm trong CP07 `8edd20965ef7c4c5d156b54fdf5c3e36b6bcd3a2` đã hậu kiểm. |
| Gói giao diện | UI-02 tạo/mở workspace và UI-03 nhập EPUB, cây nguồn, phân loại, chỉnh mapping |
| Phân rã | 18 task, 9 checkpoint: 5 gói/13 task ban đầu và 5 task bổ sung ở §17. Mỗi task có bước nhỏ và ca kiểm chứng riêng. |
| Nhánh khi triển khai | Nhánh theo phạm vi `feature/...`, PR Rust vào `rewrite/rust`. |
| Vai trò | Model A triển khai và bàn giao. Model B khác review độc lập trên đúng candidate. |

Xem [roadmap chung](../ROADMAP.md) và [phạm vi tại Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-02). Kế hoạch và bằng chứng nội bộ lưu ngoài Git theo [quy tắc repo](../../../AGENTS.md).

## 2. Mục tiêu và demo cuối phase

Sau M2, người dùng tạo hoặc mở một bộ, nhập EPUB cho một tập, xem cây nội dung và lưu chỉnh sửa phạm vi hoặc ranh giới chương. Mở lại bộ phải thấy đúng nguồn, phạm vi và lịch sử đã lưu. Phase 03 nhận bản đồ nguồn ổn định để gắn revision vào đúng đoạn.

1. Tạo workspace bằng TUI qua chọn thư mục, mở lại bằng TUI; kiểm CLI create/open với workspace thử và cùng dữ liệu.
2. Nhập hai tập từ EPUB 2 và EPUB 3 tự soạn. So thứ tự chương, một đoạn ruby, vị trí ảnh và chú thích với bảng kỳ vọng của bộ mẫu. Bản EPUB nguồn vẫn khớp hash ban đầu.
3. Chuyển tập 1 sang **Chỉ truyện**. Xem trang đang cần phân loại và kiểm phạm vi tập 2 vẫn đúng. Xem trước rồi xác nhận một chỉnh sửa ranh giới hoặc tên chương.
4. Thoát và mở lại. Đối chiếu cây, phạm vi, mapping và liên kết về nguồn. Chạy ca file nhập lỗi trên tập trống và kiểm hai tập đã công bố.
5. Dùng bộ thử của T13 để ngắt trước và sau commit. Mở lại phải đọc mốc cũ hoặc mới trọn vẹn. Thiếu hoặc hỏng object đã tham chiếu phải có chẩn đoán và chặn ghi.

Các bước người dùng thao tác ở [mục 16.4](#p02-user-test). Kiểm hash, transaction và điểm ngắt có bằng chứng riêng ở mục 13.

## 3. Phạm vi

| Phase 02 thực hiện | Phase nhận phần kế tiếp |
|---|---|
| `nt-storage`, Series DB schema 1, create/open, kho file bất biến, kiểm integrity, writer và generation | Phase 03 thêm lưu revision và dựng Markdown. Phase 05 thêm quyền chạy/liên tiến trình. Phase 09 kiểm nâng cấp và phục hồi cho release. |
| `nt-epub` đọc ZIP/XML, metadata, spine, Nav/NCX, cây nguồn, ảnh/ruby/link/chú thích | Phase 07 dùng inventory để chuẩn bị, xem và xuất EPUB. Phần chữ trong ảnh thuộc luồng ảnh. |
| Phân loại cục bộ, phạm vi tập, quyết định người dùng và phiên bản mapping | Phase 03 xử lý rebase/reuse revision. Phase 04 chia lượt. Phase 06 nối Research và xét lại phụ thuộc. |
| UI-02/03 và CLI workspace/import/scope gọi cùng use case | Editor bản dịch, Review, kết nối model và Preview xuất thuộc phase tương ứng. |

Giữ đường chạy Go. Workspace Phase 02 là định dạng Rust mới. Khi gặp định dạng Go cũ hoặc schema chưa hỗ trợ, báo cách xử lý và bảo toàn dữ liệu. Nâng cấp các schema **Rust** dùng cơ chế version, backup và kiểm tra theo ADR-004. Luồng nhập, xem nguồn và chỉnh mapping đều chạy cục bộ.

## 4. Điều kiện bắt đầu

P02-T01 nhận M0-02 và M1 đã Đạt. P02-CP05 công bố snapshot/mapping đã lưu và đọc lại để P03-T01 và P07-T02 nhận sớm; T10/T12 và kiểm M2 tiếp tục trong chat P02.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-MODULE | [Bước 4 — Module và ownership](../../desgin/MODULE_DESIGN_STEP4.md) |
| D-UI | [Bước 8 — UI/UX](../../desgin/UI_UX_DESIGN_STEP8.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-TEST | [Chiến lược kiểm thử](../TEST_STRATEGY.md) |
| R-REVIEW | [Quy trình hai model](../REVIEW_WORKFLOW.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Qxx là quyết định xx trong S-Q. Hành vi đã chốt là căn cứ nghiệm thu. Tên module, bảng, trait và SQL bên dưới là cấu trúc dự kiến, được xác định theo use case và phép kiểm của task.

- **Q01–02.** Bộ mới mặc định toàn bộ phần chữ. Tập mới nhận mặc định của bộ. Thay đổi tập là riêng cho tập đó. Hành động **Lưu làm mặc định của bộ** áp dụng cho tập tạo sau. Tách loại nội dung khỏi cách xử lý. Chữ đọc được ở trang phụ vẫn thuộc all-text. Story đưa phần mơ hồ vào **Cần phân loại** khi việc phân loại làm đổi cách xử lý.
- **Q02.** Lưu quyết định đã chốt cùng hash EPUB, phiên bản mapping và vị trí. Cùng căn cứ thì tái dùng đúng vị trí. Lệch bất kỳ căn cứ nào thì đối chiếu lại trước khi áp nhãn.
- **Q03–04.** Spine định thứ tự đọc. Ưu tiên ranh giới đã xác nhận → Nav/NCX hợp lệ → heading/ngắt → file. Part vẫn là tiêu đề trong chương. BlockID và SourceRange tuân ADR-003, giữ đủ ruby, ảnh, link và quan hệ chú thích. Nội dung giống nhau ở hai node giữ hai ID khác nhau.
- **Q06–07.** Mỗi Series một SQLite DB quản lý các Volume. Schema Rust bắt đầu từ 1. File bất biến hoàn tất và được kiểm hash trước khi công bố tham chiếu. Generation, lịch sử và con trỏ hiện hành cùng commit. Đọc thấy một mốc nhất quán.
- **Q36 và S-SEC.** Đọc ZIP/XML có giới hạn. Mặc định thử nghiệm có thể cấu hình là 10.000 entry, 128 MiB/entry và 1 GiB tổng byte giải nén thực tế. Kiểm đường dẫn và xung đột tên trên hệ đích. Chặn truy cập tài nguyên ngoài kho và nội dung chủ động. Bảo toàn chữ nguồn hợp lệ.
- **S-INPUT.** Hỗ trợ EPUB 2/3 có XHTML đọc được, kể cả trang ảnh xen trong sách reflowable. Chẩn đoán riêng phạm vi thiếu/mã hóa/scan/script phụ thuộc. Phân biệt font obfuscation với nội dung bị khóa.

## 6. ADR và điểm còn mở

| Hợp đồng nhận từ Phase 00/01 | Dùng ở Phase 02 |
|---|---|
| [ADR-001](../../decisions/ADR-001.md) và [ADR-008](../../decisions/ADR-008.md) | Toolchain, hướng phụ thuộc, event loop, lỗi/JSON/redaction theo M1. Bổ sung kết quả từng lệnh khi consumer có thật. |
| [ADR-002 v0.2](../../decisions/ADR-002.md), phần kỹ thuật v0.1 đã B Đạt | Rusqlite, một writer riêng cho mỗi Series DB, reader riêng chỉ đọc. WAL local, foreign keys từng connection, FULL và busy timeout ban đầu tối đa 2 giây. ACK sau commit, drain và join khi shutdown. |
| [ADR-003 v0.4](../../decisions/ADR-003.md), phần kỹ thuật v0.3 đã B Đạt | Canonical BlockID v1, chuẩn URI, vector hash độc lập, đơn vị hiển thị và range nửa mở. |
| [ADR-004 v0.5](../../decisions/ADR-004.md), phần kỹ thuật v0.4 đã B Đạt | Bootstrap nguyên tử, file hoàn tất bằng hard link không thay thế, CAS/replay, kiểm integrity và migration trên bản staging. Bằng chứng process crash trên Windows local. |

| ID | Phần cần chốt và phép kiểm | Owner / hạn |
|---|---|---|
| P02-D01 | Chọn kiểu/format ID, UTC/precision, format hash tổng quát, schema version, khóa và ràng buộc cùng Series/Volume/snapshot. Liệt kê bảng tối thiểu theo use case. Thử insert sai quan hệ và round-trip. DB-01/02/03/07/08/18, TD-15. | T01 trước migration đầu, T02 trước kho file, T03 trước write path. |
| P02-D02 | Chốt hỗ trợ đường dẫn workspace local, kiểm khả năng filesystem, giới hạn hàng đợi và cách caller nhận busy/ACK. Kiểm xung đột và đóng app có request đang chờ. TD-05, DB-13. | T01/T03 trước adapter và writer. T11/T12 kiểm qua terminal. |
| P02-D03 | Chọn thư viện ZIP/XML theo khả năng kiểm byte, namespace, giới hạn, chặn DTD/entity bên ngoài và Windows path. Chốt tập giá trị cấu hình cùng ca dưới/bằng/trên ngưỡng. | T04 trước ZIP, T05 trước parser XML. |
| P02-D04 | Chốt parser version, node path, cách ghép visible text và giữ span khi có entity/inline lồng. Dùng fixture EPUB thật và vector ADR-003 để kiểm. DB-05. | T06 trước source mapping. |
| P02-D05 | Chốt chẩn đoán inventory, trạng thái chưa hỗ trợ/thiếu nguồn, điều kiện publication, phép nhập lại cùng nội dung và trường JSON của import. Tách trạng thái công bố nguồn khỏi phần còn cần phân loại/chỉnh ranh giới. | T05 trước unsupported diagnostics. T09 trước publish/replay. T12 kiểm cách CLI/TUI trình bày. |
| P02-D06 | Chốt cách lấy/tạo Volume ID và ngữ nghĩa `import --volume`, gồm ID chưa có, trùng hoặc không thuộc bộ. T01 chỉ cần schema metadata. T09 nhận Volume ID đã được giải quyết. | T12 trước RegisterVolume và đường CLI/TUI nhập tập. |
| P02-D07 | Chốt biểu diễn thứ tự và coverage, bằng chứng ranh giới tạm, quyết định phân loại và vùng ảnh hưởng khi chỉnh. Kiểm round-trip và ca version/hash đổi riêng. | T07/T08 trước mapping/phân loại. T10 trước công bố chỉnh sửa. |
| P02-D08 | Chốt phạm vi orphan inventory, backup DB cùng file tham chiếu, kiểm lỗi migration trong staging và cách báo sửa dữ liệu. DB-12. | T02 trước thêm GC nếu có consumer. T03 trước migration ghi. T13 kiểm tích hợp. |

Đọc lịch [TD/DB](../ROADMAP.md#decision-schedule) và [mục còn mở trong ADR](../ROADMAP.md#adr-followups). A ghi lựa chọn, căn cứ, tác động và phép kiểm vào ADR liên quan trước code phụ thuộc. Phần đã chốt giữ theo ADR. Cách nâng cấp hai kho trong updater, durability khi mất điện, lease Coordinator và Markdown projection giữ owner ở phase nhận.

## 7. Thiết kế triển khai

### 7.1. Thành phần và chiều gọi

| Thành phần | Sở hữu | Kết nối |
|---|---|---|
| `nt-domain` | Kiểu ID/range, thứ tự, coverage, phân loại và quy tắc phạm vi thuần | Không làm IO. |
| `nt-application` | Use case tạo/mở, import, đọc cây, đổi phạm vi, chỉnh mapping. Xác nhận điều kiện để công bố. | Dùng domain và port tối thiểu cho importer/storage. |
| `nt-epub` | Đọc ZIP/XML, kiểm kê và bóc nguồn. Trả kết quả và chẩn đoán, kèm vị trí nguồn. | Cài port của application, không tự ghi Series DB. |
| `nt-storage` | Series DB, schema/migration, file bất biến, writer, generation và read model nhất quán | Cài port của application. Chi tiết SQLite ở adapter. |
| App `novel-translate` và `nt-tui` | App nối adapter, xử lý CLI. TUI giữ màn, focus, bản đang sửa và hiển thị lỗi. | Cùng gọi use case. App điều phối công việc IO ngoài luồng render. |

Giữ graph kiểm tra được của M1. Thêm đúng hai adapter khi task có consumer. Các adapter trao đổi qua application và app nối thành phần. Chỉ mở API công khai có caller trong phase.

### 7.2. Dữ liệu tối thiểu theo lát triển khai

| Cụm dữ liệu | Điểm đọc/ghi cần có | Owner |
|---|---|---|
| Series, Volume, schema và current generation | Tạo bộ, mở bộ và đọc tập theo đúng Series. Lưu mặc định bộ và lựa chọn của từng tập khi consumer cần. | T01/T03/T08 |
| Object và quan hệ tài nguyên | Hash EPUB/ảnh/bytes, đường dẫn lưu do adapter quản, quan hệ với entry archive và từng vị trí xuất hiện. File gốc ở kho nguồn theo S-INPUT. | T02/T05/T09 |
| Snapshot, block và mapping | Parser version, canonical path, source spans, metadata nguồn, thứ tự/chương, phạm vi, căn cứ phân loại và chẩn đoán. | T06–T10 |
| Generation/history và operation | Parent, current, tham chiếu object, thay đổi, kết quả để đối chiếu khi mất ACK. | T03/T09/T10 |
| Backup và migration | Bản DB, tập object đang được tham chiếu và manifest hash để kiểm trước khi đưa bản nâng cấp vào dùng. | T01/T03/T13 |

Schema 1 và bootstrap g0/current phải hoàn tất trong cùng transaction theo ADR-004. T01 xác định schema rỗng hợp lệ. T03 hoàn thiện writer và kiểm lại bootstrap ở các điểm ngắt. Ràng buộc DB và kiểm use case phải chặn tham chiếu chéo Volume/snapshot. Chỉ đưa bảng/cột vào migration khi có consumer và ca kiểm.

### 7.3. Luồng công bố nguồn

1. App nhận workspace và tập đích đã được nhận diện. Đọc generation nền và chuẩn bị operation ID.
2. Chụp byte nguồn vào vùng tạm và tính hash. Parser dùng chính byte đã chụp để inventory, bóc block và dựng mapping. Kiểm coverage, nguồn bắt buộc, đường dẫn và giới hạn trước khi xin công bố.
3. Hoàn tất object bất biến và kiểm hash đọc lại. Mọi công việc parse, ghi file và chờ người dùng diễn ra ngoài transaction DB.
4. Writer mở `BEGIN IMMEDIATE`, kiểm replay và parent theo ADR-004. Ghi snapshot/mapping/references/history, cập nhật current đúng một row rồi commit.
5. ACK sau commit. App trả dữ liệu đọc từ generation đã công bố. Nếu mất ACK, mở lại và đối chiếu operation ID trước retry.

Ghi scope và chỉnh mapping cũng đi qua write path có version/CAS. Khi TUI đợi IO, trạng thái đang xử lý thuộc màn. Kết quả gắn đúng workspace/tập và thao tác đã gửi, kể cả khi người dùng đổi màn.

## 8. Dữ liệu, lỗi và phục hồi

| Tình huống | Giữ dữ liệu thế nào | Điều người dùng cần thấy |
|---|---|---|
| Path, quyền hoặc schema không hợp lệ | Giữ dữ liệu sẵn có. Open kiểm format trước khi mở ghi. | Lý do và cách chọn workspace được hỗ trợ. |
| ZIP/XML vượt giới hạn, đường dẫn thoát root, thiếu nguồn bắt buộc | Giữ generation trước. Trạng thái inventory dở/không hỗ trợ xử lý theo D05. | Entry hoặc vùng bị ảnh hưởng và lý do chưa sẵn sàng. |
| Mơ hồ về chương hoặc phân loại | Giữ nguồn và dấu hiệu. Có thể lưu mapping tạm theo contract. | Đúng vị trí cần kiểm, hai dấu hiệu ranh giới hoặc lựa chọn loại nội dung. |
| Ghi temp hoặc hoàn tất object lỗi | Không công bố tham chiếu. Object hợp lệ chưa có ref vẫn có thể còn trong kho. | Lý do lỗi lưu và mốc đang đọc. |
| `StorageBusy` hoặc `StaleGeneration` | Rollback phần chưa commit. Giữ draft của người dùng. | Báo bận hoặc xung đột, cho xem lại mốc mới trước khi xác nhận lần khác. |
| Mất ACK hoặc worker dừng | Đối chiếu operation ID. Kết quả chưa rõ dùng `CommitOutcomeUnknown`. | Trạng thái chưa xác định và cách kiểm lại. |
| Object có ref thiếu hoặc sai hash | Giữ con trỏ/lịch sử. Chặn mở ghi và chẩn đoán lỗi integrity. | Object/phạm vi lỗi và hướng phục hồi theo backup. |
| Migration/backup thất bại | Dừng trước khi dùng bản staging lỗi. Bản đang dùng và file nguồn được bảo toàn theo ADR-004. | Lý do, version và tình trạng backup/phục hồi. |

Chẩn đoán có mã ổn định, phạm vi và bước tiếp theo. Lọc đường dẫn cá nhân, nội dung nhạy cảm và lỗi thư viện trước khi hiển thị hoặc ghi log. Phân biệt lỗi nhập với nguồn đã lưu nhưng còn cần người dùng kiểm. Quy tắc Q02 vẫn áp dụng sau khi mở lại hoặc đổi mapping. Việc thu gom file chưa được tham chiếu có lịch riêng ở mục 6.

## 9. Phần TUI/CLI

| Màn / lệnh | Hành vi cần bàn giao | Task |
|---|---|---|
| UI-02, `workspace create <path>`, `workspace open <path>` | TUI chọn root qua hộp thư mục Windows, hiện lựa chọn và xác nhận Tạo/Mở; CLI giữ path. Đọc metadata/trạng thái, báo lỗi rõ và bảo toàn dữ liệu. | T11 + bổ sung UI-02 |
| UI-03, `import <epub> --volume <id>` | Chọn tập theo D06, nhập nguồn, thấy tiến trình và kết quả theo tập. Cây cho xem đoạn nguồn, ảnh, chú thích và lý do cần kiểm. | T12 |
| UI-03, `scope set --volume <id> --content all-text` hoặc `--content story` | Lưu đúng tập. Thao tác lưu mặc định của bộ rõ ràng, riêng với đổi lựa chọn tập. | T12 |
| Chỉnh mapping ở UI-03 | Chọn gộp, tách, dời hoặc đổi tên. Xem trước phạm vi và xác nhận để lưu. Khi lỗi hoặc stale, giữ draft và chỉ vùng cần sửa. | T10/T12 |

Giữ envelope JSON v1, stdout/stderr, cờ `--workspace`, `--json`, `--non-interactive` và mã thoát theo [ADR-008 §5.1](../../decisions/ADR-008.md#p01-d04) và S-TUI. D05/D06 định nghĩa `result`, `issues` và bước tiếp theo cho từng tình huống import/scope. Chế độ không tương tác trả hữu hạn và dùng `task_id: null` khi chưa có task thật. Lỗi tham số được xử lý trước IO.

Màn hẹp từ 80×24 dùng danh sách rồi chi tiết. Màn rộng từ 120 cột dùng hai vùng cho cây tập/chương. Giữ mục đang chọn, cuộn, focus và bản đang sửa theo bộ/tập khi resize hoặc quay lại. Hiển thị hành động hợp lệ theo focus và trạng thái. Các phím điều hướng, xem nguồn và help giữ nguyên dữ liệu. Nút Research được nối ở Phase 06.

<a id="p02-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P02 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P02-T01 nhận M0-02 và M1 đã Đạt. P02-CP05 công bố snapshot/mapping đã lưu và đọc lại để P03-T01 và P07-T02 nhận sớm; T10/T12 và kiểm M2 tiếp tục trong chat P02. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p02-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p02-cp01"></a>P02-CP01 — Workspace và kho file | [P02-T01](#p02-t01), [P02-T02](#p02-t02) | M0-02 Đạt, M1 Đạt | P02-T01: V-01 trong phạm vi AC/TC của task; P02-T02: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |
| <a id="p02-cp02"></a>P02-CP02 — Writer và tạo/mở workspace | [P02-T03](#p02-t03), [P02-T11](#p02-t11) | [P02-CP01](#p02-cp01) | P02-T03: V-03 trong phạm vi AC/TC của task; P02-T11: V-06, V-07, V-10 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-10 chỉ phần tạo/mở workspace của T11; import/scope/mapping thuộc P02-CP06 và trọn bài ở CP07. V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |
| <a id="p02-cp03"></a>P02-CP03 — Đọc ZIP và inventory EPUB | [P02-T04](#p02-t04), [P02-T05](#p02-t05) | [P02-CP01](#p02-cp01) | P02-T04: V-04 trong phạm vi AC/TC của task; P02-T05: V-04 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |
| <a id="p02-cp04"></a>P02-CP04 — Source ranges và cây chương | [P02-T06](#p02-t06), [P02-T07](#p02-t07) | [P02-CP03](#p02-cp03) | P02-T06: V-05 trong phạm vi AC/TC của task; P02-T07: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |
| <a id="p02-cp05"></a>P02-CP05 — Phạm vi nội dung và import bền vững | [P02-T08](#p02-t08), [P02-T09](#p02-t09) | [P02-CP01](#p02-cp01), [P02-CP02](#p02-cp02), [P02-CP03](#p02-cp03), [P02-CP04](#p02-cp04) | P02-T08: V-09 trong phạm vi AC/TC của task; P02-T09: V-06 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |
| <a id="p02-cp06"></a>P02-CP06 — Sửa mapping và giao diện nhập tập | [P02-T10](#p02-t10), [P02-T12](#p02-t12) | [P02-CP02](#p02-cp02), [P02-CP05](#p02-cp05) | P02-T10: V-05, V-06 trong phạm vi AC/TC của task; P02-T12: V-06, V-07, V-10 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-10 phần import/scope/mapping của T12 cùng hồi quy tạo/mở đã có; trọn bài ở CP07. V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |
| <a id="p02-cp07"></a>P02-CP07 — Recovery, migration và M2 | [P02-T13](#p02-t13) | [P02-CP01](#p02-cp01), [P02-CP02](#p02-cp02), [P02-CP03](#p02-cp03), [P02-CP04](#p02-cp04), [P02-CP05](#p02-cp05), [P02-CP06](#p02-cp06) | P02-T13: V-08, V-10, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M2; V-00 cho code, B review đúng candidate. | feature/epub-import; A chat P02 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Đọc thẻ đang được giao cùng mục 5–8, AC ở mục 12 và ca tương ứng ở mục 13. Các bước bên trong mỗi thẻ giúp chia diff và kiểm từng đầu ra. Task đóng khi đủ AC, kiểm tra liên quan và B Đạt trên cùng candidate. Tên module là vị trí dự kiến theo trách nhiệm.

<a id="p02-t01"></a>

### P02-T01 — Workspace schema và create/open

- **Đầu ra.** Tạo và mở một workspace Rust schema 1, giữ định danh Series và metadata Volume cần cho phase.
- **Đầu vào.** Dependency thực thi: M0-02 Đạt, M1 Đạt. Căn cứ kỹ thuật: M0-02, M1, Q06, S-DATA, D-DATA và ADR-002/004. Chốt D01, phần đường dẫn của D02 và phạm vi migration của D08 trước DDL/IO tương ứng.
- **Vùng sửa.** `nt-storage` phần SQLite/migration/Series và use case workspace. TUI/CLI được nối tại T11.

1. Chốt layout root/DB/object, format ID, timestamp và schema version. Từ chối path không phù hợp theo D02. Tách thao tác create khỏi open để open không tự tạo workspace.
2. Tạo migration đầu và bootstrap g0/current trong cùng transaction, kể cả `user_version`. Giữ đủ khóa để Volume thuộc đúng Series. Mở connection theo ADR-002.
3. Khi mở, kiểm format/version và trạng thái schema/integrity. Chỉ mở đường ghi khi đúng workspace được hỗ trợ. Phân biệt chưa tồn tại, thiếu quyền, Go cũ, version mới hơn và DB hỏng.
4. Tạo cơ chế chạy migration theo thứ tự phiên bản và xác định điểm backup/staging. Chỉ áp dụng migration thật có consumer. T03/T13 kiểm lỗi và restore bằng fixture.
5. Đọc lại metadata qua use case. So byte của dữ liệu sẵn có trước/sau các đường bị từ chối.

**Kiểm chứng.** P02-AC01, P02-TC01, V-01. B ưu tiên thử create vào đường dẫn có sentinel, open sai schema và kiểm constraint từ connection thật.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T01 |
|---|---|
| Đầu vào bắt buộc | M0-02 Đạt, M1 Đạt. Nhận schema baseline, bootstrap và lifecycle CLI/TUI đã được duyệt ở M0/M1. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Create/Open nhận path và version, trả workspace identity hoặc lỗi giữ nguyên dữ liệu. DDL theo ADR-002/004. Owner đầu ra là A-P02-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T01 giữ `nt-storage` phần SQLite/migration/Series và use case workspace. TUI/CLI được nối tại T11. Phần chung do owner storage của T01 giữ schema, người tích hợp M2 giữ manifest/app wiring. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC01 / P02-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Metadata, version và bootstrap đọc đúng. Lỗi bảo toàn dữ liệu. |
| Điểm tích hợp | [P02-CP01](#p02-cp01) — Workspace và kho file; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t02"></a>

### P02-T02 — Kho file bất biến và tài nguyên nguồn

- **Đầu ra.** Lưu, xác minh và đọc object theo hash. File hoàn chỉnh có thể được T03/T09 dùng làm tham chiếu.
- **Đầu vào.** Dependency thực thi: [P02-T01](#p02-t01). Căn cứ kỹ thuật: T01, Q06–07, S-INPUT/S-SEC và ADR-003/004. Chốt format hash/path và khả năng filesystem trước adapter.
- **Vùng sửa.** `nt-storage` phần kho file và port có consumer. T09 nối inventory ảnh và vị trí dùng với `images/source/`.

1. Tạo temp trong cùng thư mục object store và volume với đích theo ADR-004. Kiểm path thực tế, link/junction/reparse point, giới hạn root và xung đột tên Windows. Tên archive không được dùng trực tiếp để ghi file.
2. Ghi temp bằng tên unique, tính hash byte thực, flush/`sync_all` và đóng file. Hoàn tất bằng hard link không thay đích theo ADR-004.
3. Nếu đích có sẵn hoặc xuất hiện đồng thời, verify bytes/hash. Dùng lại object đúng. Đích sai giữ nguyên và trả lỗi. Thiếu khả năng hard link thì dừng trước khi trả tham chiếu.
4. Đọc lại object đã hoàn tất để verify. Chuẩn bị đường lấy ảnh gốc đúng bytes và định danh để nối từng vị trí xuất hiện ở T09.
5. Phân biệt temp dang dở, object hợp lệ chưa tham chiếu và object đã tham chiếu. Ghi cách kiểm kê orphan. Cleanup chỉ xử lý vùng tạm xác định thuộc thao tác hiện tại.

**Kiểm chứng.** P02-AC02, P02-TC02, V-02. B kiểm tình huống tạo đích đồng thời, fault IO và sentinel ngoài root. Ca unsupported filesystem cần kết quả hữu hạn và giữ nguyên dữ liệu.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T02 |
|---|---|
| Đầu vào bắt buộc | [P02-T01](#p02-t01). Nhận workspace root/schema và error contract T01. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Nhận bytes và identity nguồn, trả object hash/path đã xác minh. Công bố file giữ dữ liệu đích và giới hạn root theo ADR-003/004. Owner đầu ra là A-P02-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T02 giữ `nt-storage` phần kho file và port có consumer. T09 nối inventory ảnh và vị trí dùng với `images/source/`. Phần chung do T01 giữ schema, T02 giữ object path/hash. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC02 / P02-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu Object nguyên vẹn, không ghi đè khác nội dung hoặc thoát root. |
| Điểm tích hợp | [P02-CP01](#p02-cp01) — Workspace và kho file; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P07-T03](PHASE-07.md#p07-t03). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t03"></a>

### P02-T03 — Generation bootstrap, writer và CAS

- **Đầu ra.** Đường ghi chung cho import, đổi scope và mapping, có replay và đọc trạng thái nhất quán.
- **Đầu vào.** Dependency thực thi: [P02-T01](#p02-t01), [P02-T02](#p02-t02). Căn cứ kỹ thuật: T01–T02, Q07, DB-07/08/12/13, ADR-002/004. Chốt D01/D02/D08 trước write path.
- **Vùng sửa.** `nt-storage` writer/generation/migration và port commit của application.

1. Bổ sung writer riêng cho từng Series DB và reader read-only. Kiểm PRAGMA trên từng connection. Chốt giới hạn queue, timeout, lỗi busy và shutdown có drain/ACK/join.
2. Định nghĩa một operation gồm ID ổn định, expected parent và fingerprint dữ liệu sẽ ghi. Hoàn tất file trước DB. Writer nhận một yêu cầu đủ dữ liệu để thực hiện cả transaction.
3. Dùng `BEGIN IMMEDIATE`. Kiểm replay và parent. Ghi generation, thay đổi, refs và current trong cùng transaction. Kiểm CAS tác động đúng một row rồi mới commit.
4. Phân biệt replay cùng payload, ID bị dùng lại với payload khác và parent đã cũ. Replay trả mốc đã ghi, kể cả khi current đã tiến tiếp.
5. ACK sau commit. Nếu mất ACK, tra operation ID sau reopen. Kiểm reader giữ snapshot cũ trong lúc writer commit và đọc snapshot mới ở lượt đọc sau.
6. Kiểm lại bootstrap atomic của T01 và phần backup/migration staging có liên quan. T13 chạy bộ fault trên đường tích hợp.

**Kiểm chứng.** P02-AC03, P02-TC03, V-03. B kiểm trạng thái DB sau rollback và replay. Với hai writer, đọc current/history thực tế để phân biệt một lần thắng, busy và stale.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T03 |
|---|---|
| Đầu vào bắt buộc | [P02-T01](#p02-t01), [P02-T02](#p02-t02). Nhận schema, immutable objects và file-before-DB protocol. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Writer nhận expected generation và write set, trả commit/ACK hoặc busy/stale. Replay giữ một kết quả logic. Owner đầu ra là A-P02-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T03 giữ `nt-storage` writer/generation/migration và port commit của application. Phần chung do T03 giữ generation/migration và port commit, T02 giữ kho file. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC03 / P02-TC03, gồm các ca chi tiết §13.3. V-03. Đối chiếu Current, history và ACK khớp kết quả transaction. |
| Điểm tích hợp | [P02-CP02](#p02-cp02) — Writer và tạo/mở workspace; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t04"></a>

### P02-T04 — Safe ZIP và giới hạn tài nguyên

- **Đầu ra.** Đọc entry ZIP đã xác thực qua API giới hạn, trả lỗi có vị trí.
- **Đầu vào.** Dependency thực thi: [P02-T02](#p02-t02). Căn cứ kỹ thuật: T02, Q36, S-INPUT/S-SEC. Chốt D03 trước khi thêm dependency.
- **Vùng sửa.** `nt-epub` phần container/path/limits. Dùng test fixture tự sinh.

1. Chọn thư viện có thể đọc giới hạn và báo lỗi dữ liệu. Ghi version/features, cách đếm entry, bytes từng mục và tổng bytes. Dùng bộ giới hạn nhỏ trong test để kiểm chính xác ngưỡng.
2. Lập chỉ mục entry trước khi giải nén theo nhu cầu. Kiểm traversal, tên tuyệt đối/ổ đĩa, tên trùng, tên Windows nguy hiểm và xung đột khi đưa xuống filesystem.
3. Tách tên entry canonical trong archive khỏi luật tên file đích. Giữ case và Unicode gốc theo ADR-003, phát hiện xung đột tại nơi materialize.
4. Giới hạn theo byte giải nén thực, kể cả đọc dang dở hoặc metadata khai sai. Kiểm lỗi CRC/truncated/mimetype/container theo phần trách nhiệm. Trả entry và lý do trước khi trả nội dung cho parser.
5. Giữ IO trong staging và chỉ đưa byte đã kiểm cho tầng OPF/XHTML. Ca lỗi phải chứng minh sentinel ngoài root và dữ liệu nguồn vẫn nguyên.

**Kiểm chứng.** P02-AC04, P02-TC04, V-04. B ưu tiên oracle dưới/bằng/trên ngưỡng, archive báo thiếu kích thước và kiểm path trên Windows.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T04 |
|---|---|
| Đầu vào bắt buộc | [P02-T02](#p02-t02). Nhận kho file/path policy đã duyệt và fixture ZIP tự sinh. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Đọc entry qua giới hạn bytes/count/path, trả dữ liệu đã kiểm hoặc diagnostic chỉ đúng entry. Owner đầu ra là A-P02-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T04 giữ `nt-epub` phần container/path/limits. Dùng test fixture tự sinh. Phần chung do T02 giữ store/path, T04 giữ container policy và dependency ZIP. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC04 / P02-TC04, gồm các ca chi tiết §13.3. V-04. Đối chiếu Lỗi đúng entry. Không trả bytes chưa kiểm hoặc ghi ngoài staging. |
| Điểm tích hợp | [P02-CP03](#p02-cp03) — Đọc ZIP và inventory EPUB; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t05"></a>

### P02-T05 — OPF, spine, Nav/NCX và phạm vi hỗ trợ

- **Đầu ra.** Inventory EPUB 2/3 gồm metadata, tài nguyên, thứ tự đọc, điều hướng và phần cần người dùng biết.
- **Đầu vào.** Dependency thực thi: [P02-T04](#p02-t04). Căn cứ kỹ thuật: T04, Q02–03, S-INPUT/S-SEC. Chốt D03 cho XML và phần diagnostics của D05.
- **Vùng sửa.** `nt-epub` parser package/spine/navigation/resource. T07 dùng các anchor để dựng chương.

1. Đọc container/package theo namespace và resolve href từ đúng file gốc. Chặn external entity, tải mạng, truy cập file ngoài archive và mở rộng entity vượt chính sách. Giữ dạng XHTML hợp lệ trong phạm vi S-INPUT.
2. Lấy metadata, manifest, spine và thứ tự linear/nonlinear. Giữ thông tin nguồn thiếu dưới dạng chẩn đoán. Kiểm resource media, sự tồn tại và quan hệ tham chiếu.
3. Đọc NCX cho EPUB 2 và Nav cho EPUB 3 theo fixture. Giữ tên mục, cấp, file đích và anchor. Phân biệt đích thiếu với dấu hiệu điều hướng mơ hồ để T07 xử lý.
4. Kiểm layout từng item. Giữ trang ảnh xen truyện và liên kết tới tài nguyên ảnh gốc. Kiểm phạm vi thực sự bị mã hóa. Phân biệt font obfuscation với nguồn bắt buộc không đọc được.
5. Trả inventory cho phần đọc được và chẩn đoán nguồn thiếu/không hỗ trợ. Quy định của D05 quyết định trường hợp đủ điều kiện công bố tại T09.

**Kiểm chứng.** P02-AC05, P02-TC05, V-04. B xem inventory thực của cả hai dạng EPUB, nhất là non-linear, hybrid và lỗi tài nguyên bắt buộc. Chẩn đoán phải có vị trí và không bỏ qua lỗi để báo hoàn tất.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T05 |
|---|---|
| Đầu vào bắt buộc | [P02-T04](#p02-t04). Nhận đường đọc entry đã kiểm và namespace/XML policy. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Inventory gồm package/spine/Nav/NCX/resources theo source order, diagnostics cho nguồn thiếu hoặc chưa hỗ trợ. Owner đầu ra là A-P02-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T05 giữ `nt-epub` parser package/spine/navigation/resource. T07 dùng các anchor để dựng chương. Phần chung do T04 giữ IO container, T05 giữ inventory schema. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC05 / P02-TC05, gồm các ca chi tiết §13.3. V-04. Đối chiếu Inventory đúng kỳ vọng. Chẩn đoán rõ và không đọc ngoài archive. |
| Điểm tích hợp | [P02-CP03](#p02-cp03) — Đọc ZIP và inventory EPUB; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t06"></a>

### P02-T06 — SourceBlock và SourceRange ổn định

- **Đầu ra.** Block, ID và range ánh xạ ngược được về node/byte span nguồn.
- **Đầu vào.** Dependency thực thi: [P02-T05](#p02-t05). Căn cứ kỹ thuật: T05, Q04, S-INPUT, ADR-003. Chốt D04 và DB-05 trước tích hợp parser.
- **Vùng sửa.** `nt-domain` kiểu/kiểm range và `nt-epub` extraction. T09 lưu và đọc lại bản đồ.

1. Chụp parser version, canonical XHTML path và locator. Dùng ID XHTML duy nhất hoặc node path/thứ tự. Tạo SHA-256 từ bản ghi versioned theo ADR-003. Lưu content/resource hash riêng.
2. Trích heading, đoạn, link, chú thích, ảnh inline, ruby, dấu ngắt và text được bảo vệ. Giữ metadata/quan hệ trỏ tới ảnh và note dùng chung.
3. Ghép run text liền nhau trước khi chia grapheme, giữ span theo từng node. Xử lý dấu kết hợp xuyên node và ký tự đã decode nhưng vẫn truy được về vị trí nguồn.
4. Xác định protected units cho ruby, ảnh và cặp marker. Kiểm range nửa mở, độ dài, biên hợp lệ và ánh xạ ngược. Giữ nguyên hai lớp ruby.
5. Đối chiếu ID, spans và unit count với oracle độc lập. Đổi EPUB hash/parser version phải nhận diện namespace mới. Đổi tên chương thuộc mapping nên giữ ID nguồn.

**Kiểm chứng.** P02-AC06, P02-TC06, V-05. B kiểm vector đã chốt và fixture parser thật, gồm `ab`/`cd`, dấu kết hợp xuyên node, ID trùng và vùng chứa ruby/ảnh.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T06 |
|---|---|
| Đầu vào bắt buộc | [P02-T05](#p02-t05). Nhận inventory, XHTML/ruby và source identity. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | SourceBlock/SourceRange có ID ổn định và vị trí chính xác, giữ vector theo ADR-003. Owner đầu ra là A-P02-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T06 giữ `nt-domain` kiểu/kiểm range và `nt-epub` extraction. T09 lưu và đọc lại bản đồ. Phần chung do T06 giữ range contract, T05 giữ inventory. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC06 / P02-TC06, gồm các ca chi tiết §13.3. V-05. Đối chiếu So ID/range với vector và span kỳ vọng độc lập. |
| Điểm tích hợp | [P02-CP04](#p02-cp04) — Source ranges và cây chương; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t07"></a>

### P02-T07 — Dựng chương logic và chẩn đoán ranh giới

- **Đầu ra.** Cây tập/chương/nội dung có thứ tự, phạm vi và căn cứ ranh giới.
- **Đầu vào.** Dependency thực thi: [P02-T05](#p02-t05), [P02-T06](#p02-t06). Căn cứ kỹ thuật: T05–T06, Q03, S-INPUT, D07.
- **Vùng sửa.** Domain kiểm mapping, tầng importer/application dựng mapping ứng viên và dữ liệu đọc cho UI.

1. Lập thứ tự đơn vị nguồn từ spine và giữ inventory phụ trợ. Giải anchor thành vị trí/range thật.
2. Chọn boundary theo thứ tự đã chốt. Ranh giới xác nhận phải thuộc đúng nguồn và còn hợp lệ. Nav/NCX hỏng hoặc mâu thuẫn có lý do và đường fallback.
3. Cho một chương qua nhiều file và nhiều chương trong một file. Giữ Part như nội dung trong chương. Bảo toàn thứ tự và phạm vi của nội dung ngoài chương.
4. Với ranh giới mơ hồ, lưu kết quả tạm kèm dấu hiệu và vị trí để người dùng đối chiếu. Tách phần chưa xác định với phạm vi đã chắc chắn.
5. Kiểm mapping đủ coverage theo nguồn áp dụng và không trùng/gap ngoài quy tắc đã chốt. Trả dạng có thể xem trước ở T10/T12.

**Kiểm chứng.** P02-AC07, P02-TC07, V-05. B đối chiếu cây với bảng kỳ vọng viết tay, kiểm dấu hiệu trong ca Nav/heading xung đột và trường hợp có boundary người dùng.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T07 |
|---|---|
| Đầu vào bắt buộc | [P02-T05](#p02-t05), [P02-T06](#p02-t06). Nhận anchors, blocks và ranges đã xác minh. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Mapping nhiều file/chương giữ thứ tự và coverage, trả dấu hiệu ranh giới cần người dùng đối chiếu. Owner đầu ra là A-P02-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T07 giữ Domain kiểm mapping, tầng importer/application dựng mapping ứng viên và dữ liệu đọc cho UI. Phần chung do T06 giữ range, T07 giữ mapping contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC07 / P02-TC07, gồm các ca chi tiết §13.3. V-05. Đối chiếu Cây đúng thứ tự và phạm vi, chỉ đúng dấu hiệu cần đối chiếu. |
| Điểm tích hợp | [P02-CP04](#p02-cp04) — Source ranges và cây chương; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t08"></a>

### P02-T08 — Phân loại nội dung và mặc định phạm vi

- **Đầu ra.** Loại nội dung, cách xử lý theo chế độ và quyết định đã xác nhận được lưu riêng, đọc lại được theo tập.
- **Đầu vào.** Dependency thực thi: [P02-T06](#p02-t06), [P02-T07](#p02-t07). Căn cứ kỹ thuật: T06–T07, Q01–02, S-INPUT, D07.
- **Vùng sửa.** Domain classification/scope, application và phần lưu metadata có consumer.

1. Kiểm kê tiêu đề, nhãn TOC, trang phụ, caption/alt, phụ trang và ghi chú. Giữ dấu hiệu phân loại từ thứ tự, heading, liên kết và vị trí.
2. Tính cách xử lý theo all-text/story. All-text vẫn chọn phần chữ đọc được dù chưa rõ loại. Story báo **Cần phân loại** nếu lựa chọn loại làm đổi kết quả. Note chung dùng một identity và được tính một lần nếu thuộc phần cần dịch.
3. Lưu mặc định bộ và giá trị hiệu lực theo tập. Đổi tập A không đổi B. Thao tác lưu mặc định bộ chỉ đổi căn cứ cho tập tạo sau.
4. Lưu quyết định người dùng theo hash EPUB, version mapping và vị trí. Đọc lại và áp đúng vị trí khi cả hai căn cứ còn khớp. Khi hash hoặc version đổi, đưa về đối chiếu trước khi áp nhãn.
5. Trả danh sách phần cần kiểm và căn cứ đọc được cho T09/T12. Ghi rõ cách thay đổi mapping ở T10 ảnh hưởng quyết định đã lưu.

**Kiểm chứng.** P02-AC08, P02-TC08, V-09. B phải xem riêng ca chỉ đổi hash EPUB và ca chỉ đổi mapping version. So chế độ thực tế của hai tập và đếm identity của note chung.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T08 |
|---|---|
| Đầu vào bắt buộc | [P02-T06](#p02-t06), [P02-T07](#p02-t07). Nhận source ranges, cây chương và mapping version. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Classification/scope theo tập, nhãn chỉ reuse khi hash nguồn, mapping và vị trí còn khớp. Owner đầu ra là A-P02-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T08 giữ Domain classification/scope, application và phần lưu metadata có consumer. Phần chung do T07 giữ mapping, T08 giữ scope, T01/T03 giữ schema/write path. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC08 / P02-TC08, gồm các ca chi tiết §13.3. V-09. Đối chiếu Phạm vi đúng theo tập. Chỉ tái dùng nhãn khi khớp cả hai căn cứ và vị trí. |
| Điểm tích hợp | [P02-CP05](#p02-cp05) — Phạm vi nội dung và import bền vững; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P07-T01](PHASE-07.md#p07-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t09"></a>

### P02-T09 — Use case import và publication

- **Đầu ra.** Công bố một snapshot, resources và mapping đã kiểm trong generation, đọc lại được qua app.
- **Đầu vào.** Dependency thực thi: [P02-T01](#p02-t01), [P02-T02](#p02-t02), [P02-T03](#p02-t03), [P02-T04](#p02-t04), [P02-T05](#p02-t05), [P02-T06](#p02-t06), [P02-T07](#p02-t07), [P02-T08](#p02-t08). Căn cứ kỹ thuật: T01–T08, Q06–07, ADR-002/003/004, D05. Tập đích là Volume đã xác định. Quy tắc nhận ID từ UI/CLI thuộc T12.
- **Vùng sửa.** Use case import, storage repository/commit và app nối importer với kho file.

1. Xác minh scope/generation nền trước thao tác. Chụp và hash EPUB; parser, snapshot và đối chiếu nguồn phải cùng một bản bytes. Trả lỗi rõ khi phát hiện nguồn đang đổi hoặc đọc không nhất quán.
2. Stage nguyên bản EPUB và tài nguyên. Hoàn tất object đã verify. Lưu ảnh gốc trong `images/source/` theo S-INPUT với quan hệ đến resource và mọi vị trí xuất hiện.
3. Kết hợp inventory, source units, mapping, phân loại và diagnostics. Kiểm nguồn bắt buộc, coverage và điều kiện công bố D05. Giữ kết quả rõ ràng với phần đọc được nhưng còn cần người dùng kiểm.
4. Gửi operation đầy đủ qua writer. CAS ghi snapshot, mapping, file refs và history cùng transaction. Map busy/stale/lỗi lưu/mất ACK theo mục 8.
5. Đọc lại trên generation vừa công bố. Kiểm thao tác lặp cùng ID khác với nhập lại bằng operation mới. Tái dùng dữ liệu hiệu lực theo đúng căn cứ Q02. Giữ snapshot/mapping cũ trong lịch sử theo contract.
6. Trả kết quả qua application cho CLI/TUI. Kiểm source không thay đổi và import chạy được với cấu hình không có model.

**Kiểm chứng.** P02-AC09, P02-TC09, V-06. B đối chiếu hash, tree, ranges và refs sau reopen với fixture độc lập. Kiểm lỗi thiếu nguồn bắt buộc và mất ACK để phân biệt chưa lưu với đã lưu.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T09 |
|---|---|
| Đầu vào bắt buộc | [P02-T01](#p02-t01), [P02-T02](#p02-t02), [P02-T03](#p02-t03), [P02-T04](#p02-t04), [P02-T05](#p02-t05), [P02-T06](#p02-t06), [P02-T07](#p02-t07), [P02-T08](#p02-t08). Nhận writer/CAS cùng inventory, snapshot, mapping và effective scope. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Import nhận Volume đã xác định, công bố snapshot và refs nhất quán, trả operation/result có thể đọc lại. Owner đầu ra là A-P02-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T09 giữ Use case import, storage repository/commit và app nối importer với kho file. Phần chung do T03 giữ commit port, T05–T08 giữ nguồn và scope, người tích hợp cấp quyền app composition. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC09 / P02-TC09, gồm các ca chi tiết §13.3. V-06. Đối chiếu Reopen đúng snapshot/hash/ID/cây/mapping. Tình trạng công bố đúng. |
| Điểm tích hợp | [P02-CP05](#p02-cp05) — Phạm vi nội dung và import bền vững; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P03-T01](PHASE-03.md#p03-t01), [P07-T02](PHASE-07.md#p07-t02). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t10"></a>

### P02-T10 — Gộp, tách, dời và đổi tên mapping

- **Đầu ra.** Xem trước và lưu phiên bản mapping mới hợp lệ, giữ lịch sử và ID nguồn.
- **Đầu vào.** Dependency thực thi: [P02-T09](#p02-t09). Căn cứ kỹ thuật: T09, Q03–04, S-INPUT/S-DATA và D07.
- **Vùng sửa.** Domain coverage/range, application chỉnh mapping và storage history. T12 gắn form.

1. Tạo đề xuất từ mapping/generation nền cho từng thao tác gộp, tách, dời hoặc đổi tên. Xem trước phạm vi và phần ảnh hưởng.
2. Kiểm đúng tập/snapshot, thứ tự nguồn, khoảng thiếu/trùng và biên protected units. Báo vị trí không hợp lệ cho người dùng sửa.
3. Chỉ khi đã xác nhận mới publish version mới qua CAS. Khi stale, giữ bản đang sửa, đọc mốc hiện hành để người dùng đối chiếu.
4. Giữ BlockID, spans, source hashes và version cũ. Lưu vùng nguồn, chương và điểm nối đã đổi để consumer sau xét lại. Quyết định phân loại theo mapping cũ được đối chiếu theo Q02.
5. Đọc lại cả hai phiên bản và chạy lại oracle coverage. Test lỗi cần kiểm không có generation/history phát sinh từ draft hoặc thao tác bị từ chối.

**Kiểm chứng.** P02-AC10, P02-TC10, V-05/V-06. B ưu tiên ca chỉnh đồng thời, ca biên ruby/ảnh và phép so ID trước/sau đổi tên.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T10 |
|---|---|
| Đầu vào bắt buộc | [P02-T09](#p02-t09). Nhận snapshot/mapping hiện hành và commit/replay đã kiểm. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Gộp/tách/dời/đổi tên nhận expected version và ranges, trả mapping mới hoặc conflict giữ bản cũ. Owner đầu ra là A-P02-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T10 giữ Domain coverage/range, application chỉnh mapping và storage history. T12 gắn form. Phần chung do T07 giữ mapping, T03 giữ generation, T10 giữ use case chỉnh cây. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T01](PHASE-03.md#p03-t01), [P03-T02](PHASE-03.md#p03-t02), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC10 / P02-TC10, gồm các ca chi tiết §13.3. V-05, V-06. Đối chiếu Coverage đúng. Conflict giữ draft và version trước. |
| Điểm tích hợp | [P02-CP06](#p02-cp06) — Sửa mapping và giao diện nhập tập; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P03-T14](PHASE-03.md#p03-t14). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t11"></a>

### P02-T11 — UI-02 và CLI tạo/mở workspace

- **Đầu ra.** Tạo/mở bộ bằng bàn phím hoặc lệnh, xem đúng metadata và lỗi.
- **Đầu vào.** Dependency thực thi: [P02-T01](#p02-t01), [P02-T03](#p02-t03). Căn cứ kỹ thuật: T01, T03, M1/UI-01, S-TUI/D-UI, ADR-008.
- **Vùng sửa.** App CLI/composition và TUI Tổng quan/form workspace.

**Bổ sung UI-02 được duyệt 19/11/2026, đã công bố tại CP07 ngày 21/11/2026:** [Hợp đồng chọn thư mục](../../spec/TUI_CLI_SPEC.md#workspace-folder-picker) đã được cấp assignment phiên bản và quyền sửa cụ thể, thực hiện và B review Đạt trên bản tích hợp M2; [biên nhận nội bộ CP07](../evidence/phase-02/P02-CP07.postmerge-r01.md). CP02 cũ chỉ chứng minh phạm vi cũ, không chứng minh picker. Phạm vi bổ sung không hồi tố review CP02 hoặc đổi input/candidate T12 r01 trước đây. Các quyền protected và patch được lưu trong assignment/receipt; tài liệu này không cấp quyền sửa core tùy ý.

1. Nối create/open qua cùng application use case. Chọn kiểu kết quả workspace và error mapping dựa trên ADR-008.
2. Parse lệnh/cờ trước IO. Kiểm Unicode/khoảng trắng, path trống và thiếu tham số. JSON một envelope và mã thoát đúng, chế độ không tương tác kết thúc hữu hạn.
3. Gắn form tạo/mở vào Tổng quan với focus, xác nhận và trạng thái trống, đang chạy, thành công, thất bại. Hiển thị dữ liệu của đúng workspace đã mở.
4. Chạy IO ngoài event/render loop. Đưa kết quả về đúng yêu cầu, giữ input khi lỗi. Khi gõ trong form, phím nhập không kích hoạt hành động phía sau.
5. Thử trên terminal Windows thật với workspace tạm: tạo/mở qua folder picker, xác nhận riêng, kiểm CLI cùng Series và hủy/lỗi khi workspace đang dùng.
6. Form Tạo chọn root cuối cùng mới/trống, form Mở chọn root hiện có; hiện lựa chọn, không tự thêm con hoặc bắt gõ/dán. Chọn không dispatch; chỉ Tạo/Mở xác nhận. Cho phép người dùng tự tạo thư mục trong dialog nếu OS hỗ trợ. Tab/Enter và chuột nếu hỗ trợ; hủy giữ input/focus/workspace, lỗi có thử lại/hủy. IO ngoài event/render; version/request loại kết quả muộn sau hủy/thay form, điều hướng hoặc đổi workspace; resize giữ state. Giữ workspace hiện hành tới khi thao tác hợp lệ thành công. Không đổi CLI/layout/API/schema hoặc validation core.

**Kiểm chứng.** P02-AC11, P02-TC11, V-06/V-07 và phần create/open của V-10. B đối chiếu CLI/TUI, trạng thái form và lỗi schema/quyền. Kết quả không phụ thuộc key hoặc model.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T11 |
|---|---|
| Đầu vào bắt buộc | [P02-T01](#p02-t01), [P02-T03](#p02-t03). Nhận Create/Open command/result, writer lỗi và shell M1. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | CLI/TUI cùng command với path Unicode, JSON/exit và lỗi có vị trí theo ADR-008. Owner đầu ra là A-P02-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T11 giữ App CLI/composition và TUI Tổng quan/form workspace. Phần chung do T01/T03 giữ nghiệp vụ, người tích hợp M2 giữ router/composition. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P09-T01](PHASE-09.md#p09-t01), [P09-T02](PHASE-09.md#p09-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC11 / P02-TC11, gồm các ca chi tiết §13.3. V-06, V-07, V-10. Đối chiếu CLI output/exit và TUI phản ánh cùng kết quả core. |
| Điểm tích hợp | [P02-CP02](#p02-cp02) đã công bố phạm vi T11 cũ. Bổ sung UI-02 được giao riêng và công bố tại [P02-CP07](#p02-cp07), có AC11/TC11e–i, V-10 Windows và B tích hợp M2 Đạt; không hồi tố verdict CP02. Xem [biên nhận nội bộ CP07](../evidence/phase-02/P02-CP07.postmerge-r01.md). |


<a id="p02-t12"></a>

### P02-T12 — UI-03, CLI import/scope và cây nguồn

- **Đầu ra.** Luồng đăng ký/nhập tập, xem cây, đổi phạm vi, xác nhận loại nội dung và chỉnh mapping.
- **Đầu vào.** Dependency thực thi: [P02-T08](#p02-t08), [P02-T09](#p02-t09), [P02-T10](#p02-t10), [P02-T11](#p02-t11). Căn cứ kỹ thuật: T08–T11, Q01–04, S-TUI/D-UI. Chốt D06 và kết quả CLI trong D05 trước khi nối consumer.
- **Vùng sửa.** App import/scope/CLI result, TUI Tập và chương, thao tác đăng ký tập theo contract.

**Cập nhật được người dùng duyệt 14/11/2026:** Cấp phiên bản tài liệu này cho T12 trước triển khai; không thay đổi candidate/review T05 đang đóng băng. Adapter/thư viện thuộc owner P02-D06/T12; không quyết định dependency hoặc cam kết Linux native.

1. Chốt cách tìm/chọn Volume ID, ID không có hoặc trùng, scope và hành vi CLI không tương tác. Nối import/scope qua application. Kiểm lỗi trước side effect và cùng dữ liệu với TUI.
2. Dựng tree/detail từ read model đã công bố. Hiển thị tình trạng trống, đang nhập, lỗi, cần phân loại hoặc ranh giới tạm. Mở nguồn/ảnh/chú thích và dấu hiệu tại đúng vị trí.
3. Cho đổi riêng phạm vi tập và xác nhận loại nội dung. Thao tác lưu làm mặc định bộ có xác nhận riêng. Lấy đúng căn cứ hiện tại trước khi lưu quyết định phân loại.
4. Nối preview và xác nhận bốn thao tác mapping. Draft gắn với workspace/tập/version. Giữ draft khi resize, đổi màn hoặc gặp stale. Tách thao tác xem/hủy khỏi ghi.
5. Kiểm bàn phím, hẹp/rộng và phản hồi IO chậm. Khi đổi tập trong lúc import, kết quả về đúng tập đã chọn lúc gửi.
5a. Trong workspace đã tạo/mở, đặt cùng nút **＋ Nhập tập truyện** ở Tổng quan và Tập và chương, cả khi trống/có tập. Tab/Enter hoặc chuột khi terminal hỗ trợ mở chooser gốc Windows; duyệt thư mục, chọn một `.epub`, Mở/OK import cục bộ vào workspace gốc. TUI tự xác định/đăng ký Volume ID và truyền path/ID vào core, không yêu cầu gõ chúng; CLI `import <epub> --volume <id>` giữ nguyên. Thành công hiện cây và focus tập mới. Esc/hủy hoặc lỗi quay lại đúng workspace với focus/dữ liệu phù hợp, chẩn đoán và thử lại/hủy khi chooser không khả dụng; không công bố nguồn dở dang. Đổi workspace hoặc resize khi chờ không áp kết quả vào workspace sai.
6. Chạy bài nhập/scope/chỉnh/mở lại với dữ liệu mẫu. So output CLI, cây TUI và phạm vi thực tế khi lỗi nguồn hoặc còn cần phân loại.

**Kiểm chứng.** P02-AC12, P02-TC12, V-06/V-07 và V-10. B xem ca đổi tập/resize trong lúc chỉnh, xác nhận rõ trước ghi và thao tác xem giữ nguyên generation.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T12 |
|---|---|
| Đầu vào bắt buộc | [P02-T08](#p02-t08), [P02-T09](#p02-t09), [P02-T10](#p02-t10), [P02-T11](#p02-t11). Nhận scope, import/chỉnh mapping và form workspace đã duyệt. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | UI cây và CLI import/scope truyền đúng Volume/version, refresh giữ lựa chọn và báo lỗi core. Owner đầu ra là A-P02-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T12 giữ App import/scope/CLI result, TUI Tập và chương, thao tác đăng ký tập theo contract. Phần chung do T08–T10 giữ commands, T11 giữ workspace route, người tích hợp giữ CLI output chung. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T01](PHASE-03.md#p03-t01), [P03-T02](PHASE-03.md#p03-t02), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC12 / P02-TC12, gồm các ca chi tiết §13.3. V-06, V-07, V-10. Đối chiếu Dữ liệu, scope, draft và trạng thái đọc đúng. |
| Điểm tích hợp | [P02-CP06](#p02-cp06) — Sửa mapping và giao diện nhập tập; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P06-T13](PHASE-06.md#p06-t13), [P08-T01](PHASE-08.md#p08-t01), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p02-t13"></a>

### P02-T13 — Recovery, migration và bàn giao M2

- **Đầu ra.** Bằng chứng tích hợp trên Windows cho save/reopen, fault, migration và bài tự test.
- **Đầu vào.** Dependency thực thi: [P02-T01](#p02-t01), [P02-T02](#p02-t02), [P02-T03](#p02-t03), [P02-T04](#p02-t04), [P02-T05](#p02-t05), [P02-T06](#p02-t06), [P02-T07](#p02-t07), [P02-T08](#p02-t08), [P02-T09](#p02-t09), [P02-T10](#p02-t10), [P02-T11](#p02-t11), [P02-T12](#p02-t12). Căn cứ kỹ thuật: T01–T12, Q06–07/36, ADR-002/004, D08 và R-REPO.
- **Vùng sửa.** Test recovery/integration, fixture, README theo thay đổi thực tế, hồ sơ M2 và hướng dẫn tự test.

1. Dùng fixture nhỏ và tiến trình con để ngắt trong khi ghi temp, sau hoàn tất file, trước commit và sau commit trước ACK. Child ghi marker xác nhận đã tới đúng điểm. Parent mở lại và kiểm dữ liệu ở từng điểm.
2. Kiểm current/history, nguồn, mapping, operation và mọi hash đã tham chiếu. Dùng hai writer và một reader để kiểm race trên đường thật. Tiêm disk full/lỗi IO, thiếu hoặc hỏng file đã tham chiếu.
3. Khóa ghi và dừng các thao tác có thể đổi tập refs trước khi tạo online backup. Kiểm restore set có DB, version, current và tất cả hash object đang tham chiếu. Nâng bản staging của fixture sang version thử, ngắt giữa migration và kiểm rollback. Phạm vi release updater giữ theo ADR-004.
4. Chạy toàn demo M2 và bài tự test mục 16.4 trên terminal thật. Ghi candidate, lệnh, input, kết quả, lỗi và kiểm tra chưa chạy. Kiểm các đường Go bị ảnh hưởng nếu sửa cấu hình chung.
5. A tạo handoff M2. B kiểm bản tích hợp và các cổng ở mục 15. Đồng bộ README, ADR và hồ sơ bàn giao theo kết quả thực tế.

**Kiểm chứng.** P02-AC13, P02-TC13, V-08/V-10/V-00 và V-11 khi đổi dependency. B so trạng thái logic và hash trước/sau reopen. Process crash trên môi trường đã ghi và durability khi mất điện là hai phạm vi kiểm khác nhau.

**Sáu trường giao việc**

| Trường | Nội dung cho P02-T13 |
|---|---|
| Đầu vào bắt buộc | [P02-T01](#p02-t01), [P02-T02](#p02-t02), [P02-T03](#p02-t03), [P02-T04](#p02-t04), [P02-T05](#p02-t05), [P02-T06](#p02-t06), [P02-T07](#p02-t07), [P02-T08](#p02-t08), [P02-T09](#p02-t09), [P02-T10](#p02-t10), [P02-T11](#p02-t11), [P02-T12](#p02-t12). Nhận toàn bộ candidate nền dữ liệu/import/UI, fixture và schema versions. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Hồ sơ M2 đối chiếu DB/file hash, crash markers, migration và thao tác thật trên cùng candidate. Owner đầu ra là A-P02-T13. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P02-T13 giữ Test recovery/integration, fixture, README theo thay đổi thực tế, hồ sơ M2 và hướng dẫn tự test. Phần chung do owners T01–T12 sửa phần mình, A-T13 tích hợp và cập nhật evidence. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P03-T01](PHASE-03.md#p03-t01), [P03-T02](PHASE-03.md#p03-t02), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P02-AC13 / P02-TC13, gồm các ca chi tiết §13.3. V-08, V-10, V-00. Đối chiếu So toàn bộ snapshot, hash và integrity. Bản lỗi dừng an toàn. |
| Điểm tích hợp | [P02-CP07](#p02-cp07) — Recovery, migration và M2; A chat P02 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P02-AC01 | Create/Open nhận đúng Series, schema và metadata. Đường dẫn/format/lỗi bị từ chối giữ nguyên dữ liệu đã có. | P02-T01 | P02-TC01 · V-01 |
| P02-AC02 | Chỉ trả tham chiếu khi file đã hoàn tất và verify được. Đích trùng sai bytes và dữ liệu ngoài root giữ nguyên. | P02-T02 | P02-TC02 · V-02 |
| P02-AC03 | Chỉ một commit từ cùng parent thắng. Reader thấy trọn mốc cũ hoặc mới. Replay không nhân bản và ACK đi sau commit. | P02-T03 | P02-TC03 · V-03 |
| P02-AC04 | Entry không an toàn hoặc vượt giới hạn bị chặn trước khi dùng nội dung. Không ghi ngoài staging. | P02-T04 | P02-TC04 · V-04 |
| P02-AC05 | Inventory EPUB 2/3 đủ metadata, spine và tài nguyên phụ. Giữ sách reflowable xen tranh. Phần bắt buộc thiếu/chưa hỗ trợ có chẩn đoán và không được báo hoàn tất. | P02-T05 | P02-TC05 · V-04 |
| P02-AC06 | Cùng snapshot/parser cho cùng ID. Hai node riêng giữ ID riêng. Range chỉ nhận biên hợp lệ và ánh xạ ngược về nguồn. | P02-T06 | P02-TC06 · V-05 |
| P02-AC07 | Cây đúng thứ tự spine và thứ tự ưu tiên boundary. Ranh giới tạm có các dấu hiệu để đối chiếu. | P02-T07 | P02-TC07 · V-05 |
| P02-AC08 | All-text giữ phần chữ đọc được. Story chờ phân loại khi ảnh hưởng cách xử lý. Đổi tập 1 không đổi tập 2. Quyết định đã chốt tái dùng đúng vị trí khi hash EPUB và version mapping khớp. Lệch một trong hai thì đối chiếu trước khi áp nhãn. | P02-T08 | P02-TC08 · V-09 |
| P02-AC09 | Mở lại đọc đủ nguồn/mapping của mốc công bố. Tham chiếu đều tới object hoàn chỉnh. Tình trạng công bố và việc còn cần kiểm phản ánh đúng kết quả. | P02-T09 | P02-TC09 · V-06 |
| P02-AC10 | Chỉ mapping đã xác nhận và hợp lệ được lưu. Bản cũ truy được, source IDs giữ nguyên. Xung đột giữ bản đang chỉnh. | P02-T10 | P02-TC10 · V-05, V-06 |
| P02-AC11 | CLI/TUI tạo hoặc mở cùng dữ liệu; TUI chọn root và xác nhận riêng theo [hợp đồng picker](../../spec/TUI_CLI_SPEC.md#workspace-folder-picker), giữ dữ liệu/workspace khi hủy, lỗi hoặc kết quả cũ. Bàn phím và chuột khi hỗ trợ sử dụng được. Không đọc key hoặc gọi model. | P02-T11 | P02-TC11 · V-06, V-07, V-10 |
| P02-AC12 | Mở lại có đúng cây, scope và mapping. Chỉ xác nhận rõ mới ghi. Xem/resize giữ dữ liệu và import chạy cục bộ. Cả hai vị trí trong workspace có nút ＋ nhập tập khi trống/có tập; chọn một EPUB qua chooser Windows, Mở/OK nhập đúng workspace, ứng dụng tự đăng ký ID; hủy/lỗi an toàn và thành công focus tập mới. | P02-T12 | P02-TC12 · V-06, V-07, V-10 |
| P02-AC13 | Khi lỗi còn mốc nhất quán hoặc có chẩn đoán chặn ghi. Backup/migration/reopen giữ nguồn, history và mapping theo hợp đồng. | P02-T13 | P02-TC13 · V-08, V-10, V-00 |

Mỗi AC có phép kiểm và dữ liệu kỳ vọng riêng. Khi bàn giao, A chỉ ra tên test, ca thủ công hoặc artifact nào chứng minh hành vi. B kiểm cả điều kiện khiến test phải thất bại.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P02-TC01 / P02-AC01 | Workspace rỗng, sentinel, DB hỏng/newer-schema | Metadata, version và bootstrap đọc đúng. Lỗi bảo toàn dữ liệu. | V-01 |
| P02-TC02 / P02-AC02 | Bytes tự sinh, fault IO, đích trùng, path/link | Object nguyên vẹn, không ghi đè khác nội dung hoặc thoát root. | V-02 |
| P02-TC03 / P02-AC03 | Hai writer, stale parent, replay và ngắt writer | Current, history và ACK khớp kết quả transaction. | V-03 |
| P02-TC04 / P02-AC04 | ZIP sinh tạm dưới/bằng/trên giới hạn, path độc hại | Lỗi đúng entry. Không trả bytes chưa kiểm hoặc ghi ngoài staging. | V-04 |
| P02-TC05 / P02-AC05 | EPUB 2 NCX, EPUB 3 Nav, hybrid, nguồn thiếu/chưa hỗ trợ | Inventory đúng kỳ vọng. Chẩn đoán rõ và không đọc ngoài archive. | V-04 |
| P02-TC06 / P02-AC06 | Node trùng text, ruby, dấu kết hợp, emoji và inline lồng | So ID/range với vector và span kỳ vọng độc lập. | V-05 |
| P02-TC07 / P02-AC07 | Chương/file nhiều-nhiều, TOC sai, heading và boundary đã xác nhận | Cây đúng thứ tự và phạm vi, chỉ đúng dấu hiệu cần đối chiếu. | V-05 |
| P02-TC08 / P02-AC08 | Hai tập, trang mơ hồ, note chung và alt text. Mở lại cùng nguồn/mapping, đổi riêng hash EPUB, đổi riêng mapping version | Phạm vi đúng theo tập. Chỉ tái dùng nhãn khi khớp cả hai căn cứ và vị trí. | V-09 |
| P02-TC09 / P02-AC09 | Nhập hai lần, nguồn đang đổi, lỗi commit | Reopen đúng snapshot/hash/ID/cây/mapping. Tình trạng công bố đúng. | V-06 |
| P02-TC10 / P02-AC10 | Bốn thao tác mapping, sửa đồng thời, range không hợp lệ | Coverage đúng. Conflict giữ draft và version trước. | V-05, V-06 |
| P02-TC11 / P02-AC11 | Workspace tạm, chooser Windows tạo/mở, Unicode/khoảng trắng, hủy, lỗi và kết quả muộn | CLI output/exit và TUI phản ánh cùng core; xác nhận riêng và bảo toàn dữ liệu. | V-06, V-07, V-10 |
| P02-TC12 / P02-AC12 | Tạo/nhập qua chooser/đổi scope/chỉnh/đóng/mở lại, Unicode/khoảng trắng, hủy, lỗi nguồn/chooser, đổi workspace và resize khi chờ | Dữ liệu, scope, draft và trạng thái đọc đúng. | V-06, V-07, V-10 |
| P02-TC13 / P02-AC13 | DB/file trước-sau, failpoint, migration và demo | So toàn bộ snapshot, hash và integrity. Bản lỗi dừng an toàn. | V-08, V-10, V-00 |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo ở task tương ứng. T11 có thể tạo phần create/open của V-06/V-07 trước, T09/T12 bổ sung các ca import theo consumer. Nếu đổi tên, cập nhật bảng cùng code. Ghi test thực sự được phát hiện và số ca liên quan đã chạy. Exit 0 với 0 ca liên quan chưa chứng minh AC.

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | `cargo test -p nt-storage --test workspace_contract --locked` |
| V-02 | `cargo test -p nt-storage --test immutable_store --locked` |
| V-03 | `cargo test -p nt-storage --test generation_contract --locked` |
| V-04 | `cargo test -p nt-epub --test import_contract --locked` |
| V-05 | `cargo test -p nt-epub --test source_mapping --locked` |
| V-06 | `cargo test -p novel-translate --test phase02_import --locked` |
| V-07 | `cargo test -p nt-tui --test workspace_import --locked` |
| V-08 | `cargo test -p nt-storage --test phase02_recovery --locked` |
| V-09 | `cargo test -p nt-application --test source_scope --locked` |
| V-10 | Bài tự test mục 16.4 trên terminal Windows thật. Bổ sung UI-02 chạy create/open qua chooser Windows thật, xác nhận, hủy/lỗi và CLI; T12 chạy import/scope/mapping, M2 chạy trọn. T12 kiểm chooser Windows thật: nút ở cả hai vị trí, duyệt thư mục, chọn EPUB, Mở/OK, hủy, lỗi và mở lại. Ghi phiên bản, kích thước, thao tác và kết quả. |
| V-11 | Khi thêm hoặc đổi dependency, kiểm MSRV theo ADR-001 trên graph sản phẩm. Với mốc hiện hành dùng `cargo +1.88.0 test --workspace --all-features --locked`. |

**V-00 — Bộ kiểm tra Rust trước PR và trước đóng M2:**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

Chuẩn bị dependency và toolchain trước khi chạy offline. Dùng cùng policy V-07 của Phase 01 về cô lập mạng trong ứng dụng. Test tự động dùng EPUB tự soạn, thư mục tạm, clock/ID/barrier kiểm soát được. Dùng đối tượng giả ở tầng cô lập, còn TC09/TC13 chạy đường parser/storage thật. Không dùng sách, key hoặc workspace cá nhân. Ghi riêng test terminal thật, test process và test buffer.

### 13.2. Cách dùng bằng chứng

- **Bộ mẫu dùng lại.** EPUB 2 có NCX, chương qua nhiều file và mục phụ ngoài luồng chính. EPUB 3 có Nav, nhiều chương trong một file, trang ảnh xen, ruby, dấu kết hợp và chú thích dùng chung. Ghi trước thứ tự, spans, vị trí ảnh/note và trường nào cần người dùng đối chiếu.
- **Kỳ vọng độc lập.** Tính hash từ bytes fixture hoặc vector ADR-003 bằng công cụ tham chiếu. Kỳ vọng về thứ tự/range phải viết từ XHTML và bảng đơn vị nguồn. So metadata, refs, content và bytes đọc lại. Hai lần cùng output chưa đủ nếu cùng sai.
- **Race và fault.** Dùng barrier/marker để điều khiển thứ tự. Với process abort, parent xác nhận child đến đúng điểm rồi so trạng thái sau reopen. Chỉ exit lỗi không đủ bằng chứng. Tách IO giả lập với process crash trên filesystem thật.
- **Kết quả.** Lưu lệnh, target, toolchain, filesystem, ca thực chạy, trạng thái và lý do ca chưa chạy. Nếu thiếu môi trường cho ca bắt buộc, giữ chưa xác minh. Power-loss và updater giữ phạm vi Phase 09.

### 13.3. Chi tiết ca thử theo task

Các mã hậu tố phân rã nhóm P02-TCxx. Mỗi dòng có thể cần nhiều test để kiểm cả biến thể. Khi bàn giao, nối mã ca với tên test hoặc kịch bản thủ công đã chạy.

| Ca | Thao tác trên fixture | Kết quả phải đối chiếu |
|---|---|---|
| P02-TC01a | Create tại root trống rồi Open bằng connection mới. | Đúng Series, schema 1, một g0/current và metadata đọc lại. |
| P02-TC01b | Create tại vị trí có file dữ liệu hoặc sentinel theo policy D02. | Báo xung đột hoặc từ chối theo contract. Hash của file ban đầu giữ nguyên. |
| P02-TC01c | Open root thiếu, không đủ quyền, Go cũ, DB hỏng hoặc version mới hơn. | Lỗi có loại và hướng xử lý. Open không tạo hoặc sửa dữ liệu của các ca bị từ chối. |
| P02-TC01d | Ghi metadata Volume với Series khác hoặc sai khóa qua connection thật. | Constraint từ chối, chỉ dữ liệu hợp lệ được đọc lại. PRAGMA theo ADR-002. |
| P02-TC02a | Lưu bytes mẫu rồi đọc lại. Lưu thêm lần nữa cùng bytes. | Hash độc lập và bytes khớp. Hai lần dùng cùng object hợp lệ. |
| P02-TC02b | Xen thao tác tạo đích cùng tên trước khi hoàn tất. Thử đích cùng bytes và sai bytes. | Đích đúng được dùng lại, đích sai giữ nguyên và không có tham chiếu được trả. |
| P02-TC02c | Tiêm lỗi write/sync/link hoặc filesystem không hỗ trợ hard link. | Báo IO/capability rõ. Không trả object chưa verify. Bản nguồn còn nguyên. |
| P02-TC02d | Dùng link/junction/reparse point hướng ra ngoài root thử. | Từ chối đường thoát. Sentinel ngoài root và kho trước đó giữ nguyên. |
| P02-TC03a | Ngắt bootstrap trước commit rồi thử tạo lại. | Không còn DDL/version/current nửa vời. Retry tạo đúng một g0/current và refs hợp lệ. |
| P02-TC03b | Hai writer cùng parent. Reader giữ lượt đọc trước commit và mở lượt đọc mới sau commit. | Một mốc thắng. Writer còn lại bận hoặc stale theo lịch thử. Reader thấy trọn mốc tương ứng. |
| P02-TC03c | Replay cùng operation sau khi current đã tiến. Thử dùng lại ID với payload khác và ID mới trên parent cũ. | Cùng operation trả mốc cũ đúng. Hai ca sai bị từ chối. History/current không lùi hoặc tăng sai. |
| P02-TC03d | Ngắt trước/sau ACK và shutdown với request đã nhận trong queue. | Không success trước commit. Có drain/join hoặc lỗi xác định. Mất ACK đối chiếu được bằng operation ID. |
| P02-TC04a | Với giới hạn test nhỏ, tạo ZIP dưới, bằng và vượt ngưỡng entry/bytes/total. | Dưới và đúng ngưỡng đọc được. Vượt một đơn vị bị chặn với đúng giới hạn và entry. |
| P02-TC04b | Entry traversal, ổ đĩa/đường tuyệt đối, backslash và tên Windows nguy hiểm. | Từ chối theo D03 trước khi đọc hoặc ghi đích. Sentinel ngoài root giữ nguyên. |
| P02-TC04c | Entry trùng, case collision tại nơi lưu, alias path hợp lệ. | Báo đúng xung đột trên filesystem. Alias hợp lệ resolve về cùng entry mà không gộp nhầm ID nguồn. |
| P02-TC04d | ZIP cắt cụt, CRC sai hoặc kích thước khai báo khác số byte đọc. | Lỗi đúng nguyên nhân hoặc giới hạn thực. Không đưa payload dở cho parser. |
| P02-TC05a | Đọc EPUB 2 có NCX, thiếu một metadata không bắt buộc và một mục nonlinear. | Metadata thiếu được ghi rõ, inventory/spine/anchors khớp bảng kỳ vọng. |
| P02-TC05b | Đọc EPUB 3 có Nav, trang ảnh xen và font obfuscation. | Giữ trang ảnh và tài nguyên đúng vị trí. Xác định đúng tài nguyên bị ảnh hưởng. |
| P02-TC05c | Thiếu OPF/resource bắt buộc, nguồn bị khóa, scan hoặc script phụ thuộc. | Chẩn đoán đúng file/vùng và khả năng hỗ trợ theo D05. Trạng thái không báo hoàn tất sai. |
| P02-TC05d | XML có entity ngoài, tham chiếu mạng hoặc mở rộng vượt giới hạn. | Chặn truy cập ngoài archive. Harness ghi không mở file ngoài hoặc mạng. Phần XHTML hợp lệ của fixture đối chứng vẫn đọc được. |
| P02-TC06a | Tính vector ADR-003 và parse lại EPUB fixture cùng parser. | Vector cố định khớp. ID/locator kỳ vọng từ fixture khớp trên hai lượt. |
| P02-TC06b | Hai đoạn cùng chữ, XHTML id trùng, alias canonical. Đổi EPUB hash hoặc parser version. | Các node khác giữ ID riêng. Alias cùng entry cho cùng path. Phiên bản mới có namespace ID mới. |
| P02-TC06c | Text node liền `ab`/`cd` và dấu kết hợp xuyên hai node. | Bốn đơn vị và spans đúng cho chữ thường. Một grapheme có hai spans cho dấu kết hợp. |
| P02-TC06d | Ruby, ảnh, marker pair, emoji và entity đã decode. Thử range hợp lệ và range ngoài biên. | Range hợp lệ map đúng node/span và giữ cả protected unit. Range rỗng/ngoài biên bị từ chối. |
| P02-TC07a | Một chương qua nhiều file, một file chứa nhiều chương và ảnh xen. | Chương/range đúng kỳ vọng theo spine, phủ đúng vùng nguồn áp dụng. |
| P02-TC07b | Nav/NCX trỏ thiếu hoặc xung đột heading. | Có kết quả tạm và các dấu hiệu đối chiếu ở đúng vị trí. |
| P02-TC07c | Có boundary đã xác nhận còn hợp lệ. Lặp với boundary không còn hợp lệ và nguồn thiếu TOC. | Boundary hợp lệ ưu tiên. Trường hợp còn lại dùng fallback heading/ngắt rồi file và giữ lý do. |
| P02-TC07d | Có Part và trang phụ ngoài luồng chính. | Part nằm trong chương, phụ trợ còn inventory và phạm vi. |
| P02-TC08a | Hai tập kế thừa mặc định, đổi riêng A rồi lưu mặc định bộ và tạo C. | A/B độc lập. C nhận mặc định mới. Tập đã có giữ lựa chọn hiệu lực. |
| P02-TC08b | Trang mơ hồ có alt/caption và note được gọi từ phần truyện lẫn phần phụ. | All-text giữ chữ. Story chỉ chờ phân loại nếu kết quả đổi. Note chung có một identity và đúng cách xử lý. |
| P02-TC08c | Lưu quyết định rồi mở lại cùng hash/version ở vị trí cũ và so vị trí khác. | Tái dùng đúng nhãn ở đúng vị trí. Không áp sang vị trí khác. |
| P02-TC08d | Hai ca độc lập. Đổi riêng hash EPUB. Đổi riêng mapping version. | Cả hai ca đưa về đối chiếu trước khi áp nhãn. Nhãn cũ còn căn cứ trong lịch sử. |
| P02-TC09a | Nhập EPUB 2/3 qua app nối parser/storage thật, rồi đóng mở lại. | EPUB và ảnh gốc đúng bytes, tree/mapping/ID/refs khớp bảng mẫu. |
| P02-TC09b | Replay cùng operation và nhập lại cùng EPUB bằng operation mới. | Replay giữ kết quả đã lưu. Nhập lại theo D05 và giữ căn cứ Q02, parser tạo mapping ổn định với cùng nguồn/version. |
| P02-TC09c | Đổi file trong lúc chụp theo điểm kiểm soát. Thử parent đã tiến trước khi publish. | Không lẫn hash và bytes đã parse. Phát hiện thay đổi thì báo lỗi. Stale giữ mốc hiện hành. |
| P02-TC09d | Lỗi file/commit hoặc thiếu nguồn bắt buộc. Dùng cấu hình không có model. | Không có refs dở. Generation trước còn đọc được. Báo đúng trạng thái và không phát yêu cầu model. |
| P02-TC10a | Xem trước rồi xác nhận lần lượt gộp, tách, dời, đổi tên trên các bản mẫu. | Preview giữ generation. Sau lưu, mapping mới đúng thứ tự/coverage và giữ ID. |
| P02-TC10b | Tạo gap, overlap, sai snapshot hoặc tách bên trong protected unit. | Chỉ vùng vi phạm. Không tạo version/generation mới. |
| P02-TC10c | Hai draft từ cùng base. Xác nhận A rồi xác nhận B. | A lưu. B nhận conflict và giữ draft. Lịch sử không ghi bản bị từ chối. |
| P02-TC10d | Đọc version cũ/mới và căn cứ phân loại sau đổi boundary. | Version cũ truy được. Vùng ảnh hưởng đúng điểm nối và nhãn cần đối chiếu theo Q02. |
| P02-TC11a | Create CLI bằng path Unicode có khoảng trắng rồi Open trên TUI. | Cùng Series/schema, trạng thái trống đúng. |
| P02-TC11b | Thiếu tham số, path trống hoặc cờ sai với JSON/non-interactive. | Exit 2 và envelope đúng ADR-008. Không có IO trước parse hợp lệ. |
| P02-TC11c | Giữ IO đang chờ qua barrier, nhập trong form và thao tác phím. | Vòng UI vẫn phản hồi. Input không kích hoạt control phía sau. Kết quả về đúng request. |
| P02-TC11d | Mở DB hỏng, version mới hoặc thiếu quyền bằng CLI và terminal thật. | Có lỗi và hướng xử lý, workspace đang mở không bị thay bởi kết quả lỗi. |
| P02-TC11e | Từ Tổng quan dùng Tab/Enter và chuột nếu hỗ trợ, chọn root mới/trống Unicode/có khoảng trắng qua picker Tạo; tùy OS tự tạo thư mục. | Form hiện root cuối cùng; chọn chưa khởi tạo, chỉ Tạo xác nhận mới gọi core; không thêm con. |
| P02-TC11f | Chọn root workspace hiện có qua picker Mở; đối chiếu CLI create/open. | Chỉ Mở xác nhận mới mở; cùng Series/schema; CLI và core giữ hợp đồng. |
| P02-TC11g | Hủy/Esc, chooser lỗi/không khả dụng hoặc thiếu quyền khi đã có input/workspace. | Giữ input/focus/workspace; lỗi hữu ích có thử lại/hủy, không ép dán path hay tạo/mở. |
| P02-TC11h | Chọn đích tạo không trống, mở schema không hỗ trợ hoặc bị từ chối quyền. | Core từ chối theo validation hiện có, giữ dữ liệu và workspace đang mở. |
| P02-TC11i | Trì hoãn picker, hủy/thay form, điều hướng hoặc đổi workspace; resize khi chờ. | UI phản hồi; resize giữ state; kết quả cũ không gây khởi tạo/chuyển hoặc áp nhầm workspace. |
| P02-TC12a | Đăng ký/nhập hai tập và đổi scope qua CLI/TUI theo D06. | ID tập đúng, kết quả CLI hữu hạn. TUI và CLI đọc cùng scope/cây sau reopen. |
| P02-TC12b | Xem phần mơ hồ, xác nhận loại, lưu một chỉnh sửa chương rồi mở lại. | Dấu hiệu/nhãn đúng vị trí. Chỉ thao tác xác nhận ghi version mới. |
| P02-TC12c | Đổi kích thước, đổi tập trong lúc nhập hoặc sửa, trở lại đúng tập. | Mục chọn/focus/draft còn đúng scope. Phản hồi chậm không ghi hoặc hiển thị nhầm tập. |
| P02-TC12d | Nhập file lỗi và ca còn cần phân loại trong chế độ không tương tác. | Trạng thái, JSON và bước tiếp theo phân biệt rõ. Xem/đóng form không tự Research hoặc ghi. |
| P02-TC12e | Từ Tổng quan và Tập và chương, lúc trống/có tập, dùng Tab/Enter và chuột nếu hỗ trợ mở chooser Windows; chọn một EPUB trong đường dẫn Unicode/có khoảng trắng và Mở/OK. | Import cục bộ đúng workspace gốc; tự đăng ký ID, cây hiện và focus tập mới, mở lại giữ đúng. |
| P02-TC12f | Esc/hủy; chọn file sai/đuôi sai; chooser không khả dụng; đổi workspace hoặc resize khi chooser/import đang chờ. | Trở lại với focus/dữ liệu đúng; lỗi hữu ích, chooser lỗi có thử lại/hủy; không công bố nguồn dở dang hoặc áp kết quả vào workspace sai. |
| P02-TC13a | Child ngắt trong ghi temp, sau hoàn tất file, trước commit và sau commit trước ACK. | Marker đúng. Trước commit giữ mốc cũ, temp hoặc orphan hoàn chỉnh còn lại đều chưa được tham chiếu. Sau commit có mốc mới đủ refs/hash. |
| P02-TC13b | Xóa/hỏng object đã tham chiếu trong bản sao. Tiêm lỗi IO hoặc hết chỗ ghi. | Fail closed và chặn ghi khi integrity sai. Giữ pointer/history, không tự quay về ancestor. |
| P02-TC13c | Backup staging có đủ refs. Thử migration fixture thành công, rollback giữa DDL và backup thiếu/hash sai. | Kiểm integrity, foreign keys, version/current và mọi hash. Bản lỗi chưa được dùng. Bản trước vẫn mở được. |
| P02-TC13d | Chạy toàn demo M2, V-00 và các ca UI/CLI trong mục 16.4 trên bản tích hợp. | Đủ kết quả tự động và thủ công, môi trường rõ. Các phần chung có kiểm Go bị ảnh hưởng. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md). A đọc đặc tả và thực hiện task đã giao, tự kiểm rồi tạo candidate cùng danh sách file, baseline, căn cứ và bằng chứng. B đọc AC trước, kiểm độc lập đúng candidate và ghi báo cáo riêng. Khi sửa ảnh hưởng yêu cầu hoặc dữ liệu đã kiểm, A gửi lượt mới và B kiểm lại phần ảnh hưởng.

| Phần | Điểm B cần đối chiếu |
|---|---|
| T01–T03 | Workspace cũ/sai schema giữ nguyên. File thật đã verify trước refs. Bootstrap/CAS/current/history nguyên tử trong DB, ACK đúng thời điểm và reader nhất quán. |
| T04–T07 | Kiểm byte thực, chặn path/DTD ngoài phạm vi và giữ nguồn hợp lệ. Fixture EPUB 2/3 cho IDs/ranges và thứ tự chương đúng oracle. |
| T08–T10 | Phạm vi theo tập, note chung, cả hai phép đổi căn cứ Q02, việc công bố đủ nguồn, conflict và lịch sử mapping. |
| T11–T12 | CLI/TUI gọi cùng use case. Lỗi và pending hiển thị đúng. Event/render không đợi IO. Xem hoặc resize không lưu thay người dùng. |
| T13/M2 | Marker chứng minh đúng điểm fault. Bản mở lại đúng current/snapshot/hash hoặc chặn ghi. Migration/staging và bài tự test chạy đúng bản bàn giao. |

Rà cả dependency graph, diff ngoài phạm vi, API chưa có consumer, diagnostic đã lọc và test thực sự kiểm AC. Với mỗi check bắt buộc, ghi B tự chạy, bằng chứng đã đối chiếu hoặc phần chưa xác minh. Kiểm tra trên terminal thật thuộc V-10.

A commit phần đã được B duyệt theo [Git workflow](../../../AGENTS.md#git-workflow). Khi cả phạm vi PR đủ kiểm tra và review tích hợp, push nhánh theo phần việc, ví dụ `feature/workspace` hoặc `feature/epub-import`, và mở/cập nhật Draft PR vào `rewrite/rust`. Commit và PR ghi rõ từng thay đổi bằng tiếng Anh đơn giản. Kế hoạch, spec và báo cáo nội bộ ở máy người dùng.

## 15. Điều kiện đóng phase

- [x] Bổ sung picker T11/UI-02 đã được điều phối cấp assignment/checkpoint, thực hiện và có AC11/TC11e–i, V-10 Windows cùng review B Đạt trên candidate tích hợp M2; CP02 cũ không chứng minh phạm vi này.
- [x] T01–T13 có AC đã kiểm, tài liệu/ADR đúng consumer và review B **Đạt** còn hiệu lực. Các điểm D01–D08 ảnh hưởng M2 đã có quyết định và bằng chứng.
- [x] B review **Đạt** bản tích hợp M2 theo [mẫu review phase](../templates/PHASE_REVIEW.md). Candidate gồm code/test/config/dependency và căn cứ nội bộ có định danh. B rà các thay đổi có thể làm bằng chứng task cũ mất hiệu lực.
- [x] CLI và TUI thực hiện trọn luồng tạo/mở, nhập, đổi scope, chỉnh mapping và mở lại. V-10 đã chạy trên terminal Windows thật. Lệnh và bộ mẫu của mục 16.4 khớp bản bàn giao.
- [x] Fixture EPUB 2/3, hybrid, ruby/shared notes, ZIP/XML lỗi và cả hai phép đổi căn cứ Q02 có bằng chứng. Đối chiếu đủ hash, thứ tự và range.
- [x] Hai writer, replay/mất ACK, fault trước/sau commit và missing/corrupt object có kết quả. Backup/migration staging và rollback được kiểm trên bản sao.
- [x] V-00 và các kiểm tra áp dụng ở mục 13 đạt trên candidate tích hợp, gồm V-11 khi có dependency mới/đổi. Kiểm phần Go bị ảnh hưởng nếu sửa cấu hình chung. Ca bắt buộc chưa chạy giữ trạng thái chưa xác minh.
- [x] Theo chỉ đạo Master ngày 21/11/2026, README giữ bản gốc; lệnh, layout, nền tảng và bài tự test bàn giao qua [guide nội bộ M2 r03](../evidence/phase-02/CP07-integration-r03/M2.user-test-r03.md), được B r03 kiểm Đạt. Dữ liệu Go, tài liệu riêng, EPUB, key và output được bảo toàn theo R-REPO.
- [x] Bàn giao mục 16 đủ để Phase 03 nhận dữ liệu nguồn, lịch sử và hợp đồng ghi.
- [ ] T14–T18 và CP08/CP09 tại §17 đủ AC/TC, B độc lập, checks Windows và công bố/hậu kiểm; bản ghép M2 mới có B Đạt cùng UT06–UT09. Review kỹ thuật CP07 chỉ bao phủ candidate cũ.
- [ ] Master hoàn tất lượt dùng thử và nghiệm thu Phase 02. Tiếp nhận góp ý thành task có phạm vi, sửa và kiểm/review phần ảnh hưởng trước khi chốt. Chưa có kết luận nghiệm thu của Master.

**Đang nghiệm thu với Master; chưa đóng Phase 02/M2.** Bảy checkpoint đã công bố, gồm [PR #19](https://github.com/MajoKyo/novel-translate-cli/pull/19), head `8ac38f85fc62bea489e7a318d283f67dd4fc4fbe`, merge `8edd20965ef7c4c5d156b54fdf5c3e36b6bcd3a2`, tree `cc4bcae4b01aa982147d08dfff5e3f56357c1333`. B M2 r03 Đạt trong phạm vi kỹ thuật đã kiểm; CI, CodeRabbit và hậu kiểm 105 tests/Windows V-07 Đạt. Master đang dùng thử và yêu cầu giữ cổng nghiệm thu để góp ý. [Biên nhận nội bộ hiện hành](../evidence/phase-02/P02-CP07.postmerge-r02.md) và [bàn giao nghiệm thu](../evidence/phase-02/M2.handoff-r04.md). Các liên kết evidence là hồ sơ cục bộ, được loại khỏi Git theo yêu cầu Master. Go vẫn còn chín caller và chưa đủ điều kiện xóa; mục tiêu Rust-only còn các cổng riêng.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md), B dùng [mẫu review task](../templates/TASK_REVIEW.md), và B dùng [mẫu review phase](../templates/PHASE_REVIEW.md) cho M2. Lưu báo cáo ở `docs/roadmap/evidence/phase-02/` theo lượt, ví dụ `P02-T01.handoff-r01.md`, `P02-T01.candidate-r01.sha256`, `P02-T01.review-r01.md` và bộ hồ sơ `M2.*-rNN`. Giữ bằng chứng và finding ID của lượt trước.

A phase cập nhật sổ dưới đây và gửi báo cáo cho điều phối cập nhật PLAN trung tâm khi bàn giao và sau verdict. B chỉ ghi báo cáo độc lập. Phần chưa có review hoặc kiểm tra bắt buộc giữ đúng trạng thái theo workflow.

### 16.1. Sổ trạng thái

Theo dõi 9 checkpoint: 7 checkpoint ban đầu tại [§10.2](#p02-checkpoints), từ [P02-CP01](#p02-cp01) đến [P02-CP07](#p02-cp07), cùng [P02-CP08](#p02-cp08) và [P02-CP09](#p02-cp09) theo §17. CP07 giữ kết quả công bố cũ; đích kỹ thuật cuối M2 sau bổ sung là T18/CP09. PLAN giữ commit, remote SHA, PR và cổng công bố theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task / cổng | Trạng thái | Candidate / báo cáo A–B |
|---|---|---|---|
| P02-A | [T01](#p02-t01) | Đạt, CP01 đã công bố | [Hậu kiểm CP01](../evidence/phase-02/P02-CP01.postmerge-r01.md) |
| P02-A | [T02](#p02-t02) | Đạt, CP01 đã công bố | [Hậu kiểm CP01](../evidence/phase-02/P02-CP01.postmerge-r01.md) |
| P02-A | [T03](#p02-t03) | Đạt, CP02 đã công bố | [Hậu kiểm CP02](../evidence/phase-02/P02-CP02.postmerge-r01.md) |
| P02-B | [T04](#p02-t04) | Đạt, CP03 đã công bố | [Hậu kiểm CP03](../evidence/phase-02/P02-CP03.postmerge-r01.md) |
| P02-B | [T05](#p02-t05) | Đạt, CP03 đã công bố | [Hậu kiểm CP03](../evidence/phase-02/P02-CP03.postmerge-r01.md) |
| P02-B | [T06](#p02-t06) | Đạt, CP04 đã công bố | [B r05](../evidence/phase-02/CP04-r08-evidence/b-r05/P02-CP04.review-r05.md), [hậu kiểm](../evidence/phase-02/P02-CP04.postmerge-r01.md) |
| P02-B | [T07](#p02-t07) | Đạt, CP04 đã công bố | [B r05](../evidence/phase-02/CP04-r08-evidence/b-r05/P02-CP04.review-r05.md), [hậu kiểm](../evidence/phase-02/P02-CP04.postmerge-r01.md) |
| P02-B | [T08](#p02-t08) | Đạt, CP05 đã công bố | [Hậu kiểm CP05](../evidence/phase-02/P02-CP05.postmerge-r01.md) |
| P02-C | [T09](#p02-t09) | Đạt, CP05 đã công bố | [Hậu kiểm CP05](../evidence/phase-02/P02-CP05.postmerge-r01.md) |
| P02-C | [T10](#p02-t10) | Đạt, CP06 đã công bố | [Hậu kiểm CP06](../evidence/phase-02/P02-CP06.postmerge-r01.md) |
| P02-D | [T11](#p02-t11) | Đạt, CP02 phạm vi cũ; bổ sung UI-02 đã công bố tại CP07 | [Hậu kiểm CP02](../evidence/phase-02/P02-CP02.postmerge-r01.md), [CP07](../evidence/phase-02/P02-CP07.postmerge-r01.md) |
| P02-D | [T12](#p02-t12) | Đạt, CP06 đã công bố | [Hậu kiểm CP06](../evidence/phase-02/P02-CP06.postmerge-r01.md) |
| P02-E | [T13](#p02-t13) | Đạt, CP07 đã công bố | [B T13 r02](../evidence/phase-02/CP07-integration-r02/b-t13-r02/P02-T13.review-r02.md), [hậu kiểm CP07](../evidence/phase-02/P02-CP07.postmerge-r01.md) |
| P02-F | [T14](#p02-t14) | Đang chuẩn bị/sửa theo phiếu CP08; chưa B nghiệm thu | [Phiếu T14](../evidence/phase-02/CP08-T14-r01/assignment-r01.md) |
| P02-F | [T15](#p02-t15) | Chờ T14 B Đạt, chưa triển khai | [Lịch CP08](#p02-cp08) |
| P02-G | [T16](#p02-t16) | Chờ CP08 công bố, chưa triển khai | [Lịch CP09](#p02-cp09) |
| P02-G | [T17](#p02-t17) | Chờ T16 B Đạt, chưa triển khai | [Lịch CP09](#p02-cp09) |
| P02-G | [T18](#p02-t18) | Chờ T17 B Đạt, chưa triển khai | [Lịch CP09](#p02-cp09) |
| P02-G | M2 | B kỹ thuật CP07 Đạt cho candidate cũ; đang làm T14–T18 và chờ Master nghiệm thu, chưa đóng | [B CP07 r03](../evidence/phase-02/CP07-integration-r03/b-m2-r03/P02-CP07.M2.review-r03.md), [§17](#p02-ux-followup), [bàn giao nghiệm thu](../evidence/phase-02/M2.handoff-r04.md) |

### 16.2. Báo cáo mỗi task

1. Ghi phạm vi, task/AC, hành vi đã đổi, ADR/version áp dụng và thay đổi đã có từ baseline.
2. Định danh candidate bằng commit nếu có, diff và danh sách SHA-256 của đầu vào liên quan. Gồm file untracked, dependency/config và phiên bản spec/task. Đối chiếu trước khi review và trước verdict.
3. Nối từng AC và ca ở mục 13 với test/fixture cụ thể. Ghi lệnh, số ca liên quan, môi trường, kết quả và phép kiểm thủ công. Ghi rõ phần chưa chạy.
4. Ghi finding, phần cần sửa hoặc chưa xác minh, tác động và việc tiếp theo. Khi đạt, thêm commit và link PR nếu đã đến mốc Git của phần việc.

### 16.3. Đầu ra cho bước tiếp theo

| Nơi nhận | Nội dung bàn giao |
|---|---|
| [Phase 03](PHASE-03.md) | Series/Volume IDs, schema, source snapshot, BlockID/range, mapping history và vùng ảnh hưởng khi đổi, immutable store và writer/CAS. |
| [Phase 04](PHASE-04.md) | Fixture chương nhiều file và nhiều chương/file. Bảng nguồn, thứ tự, protected ranges và tài nguyên phụ cho kiểm coverage. |
| [Phase 06](PHASE-06.md) | Trạng thái cần phân loại, dấu hiệu nguồn, quyết định theo hash/version/vị trí và phần cần đối chiếu lại. |
| [Phase 07](PHASE-07.md) | Inventory EPUB 2/3, Nav/NCX, ảnh gốc cùng từng vị trí dùng, ruby, link và shared notes. |
| [Phase 09](PHASE-09.md) | Giới hạn nền tảng/filesystem, backup format, case migration và process crash đã kiểm, phần durability/updater còn cần nghiệm thu. |

Khi PLAN xác nhận M1 và giao Phase 02, điểm bắt đầu là [P02-T01](#p02-t01). Đối chiếu mục 4, chốt phần D01/D02 cần cho schema và đường dẫn, thực hiện P02-AC01/P02-TC01 bằng V-01 rồi bàn giao B. Chuyển sang task phụ thuộc sau khi có kết luận hợp lệ.

<a id="p02-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử bản Rust bàn giao cho **M2** trên nền tảng ghi trong hồ sơ. Mục tiêu là tạo/mở bộ, nhập EPUB, kiểm cây nguồn, đổi phạm vi và lưu chỉnh sửa mapping. A ghi candidate, lệnh chạy và vị trí bộ mẫu cho các ca dưới đây, dựa trên TC01, TC08, TC11 và TC12 khi bàn giao. B dựng lại môi trường từ hướng dẫn và thử các ca chính.

#### 16.4.2. Chuẩn bị

Dành khoảng 15 phút. Dùng thư mục trống dành riêng cho bài thử và EPUB tự soạn từ bộ mẫu M2. Bộ mẫu cần bảng kỳ vọng về thứ tự chương, tên một đoạn dễ nhận, vị trí ruby/ảnh/chú thích và một trang phụ chưa rõ loại. Chuẩn bị EPUB hợp lệ cho hai tập và một file lỗi cho tập trống. A chỉ rõ file, ID tập và dữ liệu mong đợi của từng ca trong gói bàn giao.

#### 16.4.3. Mở đúng bản

Mở executable Rust theo lệnh của hồ sơ M2. Nếu nhận source, build trên candidate M2 theo [cách lấy executable Rust của Phase 01](PHASE-01.md#p01-user-test). Chạy `--version` từ chính executable đó rồi mở TUI. Tại **Tổng quan**, dùng mục tạo hoặc mở workspace của UI-02. Lệnh CLI tương ứng là `workspace create <path>` và `workspace open <path>` với đường dẫn thư mục thử.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P02-UT01 — Tạo và mở bộ | Tại Tổng quan, tạo qua picker Windows chọn root mới/trống (có thể tự tạo thư mục trong dialog), kiểm form hiện root rồi bấm Tạo. Mở lại qua picker chọn root đó và bấm Mở; thử hủy/Esc, lỗi quyền/schema và đích không trống với dữ liệu thử. Kiểm thêm CLI create/open bằng path. | Chọn chưa tạo/mở; xác nhận thành công hiển thị đúng bộ và danh sách tập trống; hủy/lỗi giữ input, focus, workspace và dữ liệu; CLI đọc cùng bộ. |
| P02-UT02 — Nhập và xem nguồn | Trong workspace đã mở, tại **Tổng quan** bấm **＋ Nhập tập truyện**, duyệt thư mục chọn một EPUB mẫu rồi bấm **Mở/OK**; tại **Tập và chương** lặp lại với EPUB thứ hai. Không gõ path hay Volume ID. Mở chương và vị trí ảnh/chú thích được chỉ trong bảng mẫu. | Hai tập xuất hiện trong đúng workspace, tập vừa nhập được focus. Thứ tự cây, đoạn nguồn, ruby, vị trí ảnh và tham chiếu chú thích khớp bảng mẫu. Trang phụ mơ hồ có dấu hiệu để đối chiếu. |
| P02-UT03 — Đổi phạm vi một tập | Đổi tập 1 từ **Toàn bộ phần chữ** sang **Chỉ truyện**. Mở tập 2 và đối chiếu phạm vi. Quay lại tập 1 và xem trang mơ hồ có ảnh hưởng đến phạm vi. | Tập 2 vẫn dùng lựa chọn ban đầu. Tập 1 hiển thị phạm vi đã lưu và **Cần phân loại** ở vị trí tương ứng. |
| P02-UT04 — Chỉnh cây chương | Chọn chương mẫu, đổi tên hiển thị hoặc dùng thao tác gộp/tách hợp lệ được chỉ trong gói mẫu. Đối chiếu phần xem trước rồi xác nhận. | Cây phản ánh chỉnh sửa đã xác nhận. Vùng nguồn gắn với chương vẫn phủ đúng dữ liệu mẫu. |

#### 16.4.5. Thử ca lỗi và mở lại

**P02-UT05 — File lỗi và mở lại.** Nhập file lỗi vào tập trống, rồi xem chẩn đoán. Ca đạt khi tập lỗi được báo đúng và hai tập đã nhập vẫn đọc được. Thoát, mở lại đúng bộ và đối chiếu cây, tên/ranh giới vừa lưu và phạm vi riêng của hai tập với kết quả UT02–UT04.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Nếu lỗi, ghi bản M2, ID tập, tên EPUB mẫu, bước, kết quả mong đợi và thực tế. Với CLI, kèm mã thoát và thông báo.

#### 16.4.7. Thử lại từ đầu

Tạo một workspace thử mới và đăng ký các tập mẫu từ đầu. Dùng lại EPUB gốc trong bộ mẫu.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-02. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P02 rà **Workspace, file store, nhập EPUB và cây nguồn** tại các mốc **P02-CP05/CP06; kiểm toàn luồng tại M2**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Chỉ xét importer/storage đã có luồng Rust tạo–nhập–mở lại tương ứng. Nếu dịch, Editor hoặc exporter Go còn gọi thì giữ phần đó, ghi caller và mốc consumer chuyển; không xóa workspace Go của người dùng.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.

<a id="p02-ux-followup"></a>

## 17. Sửa trải nghiệm nhập EPUB sau khi người dùng thử M2

**Bổ sung ngày 21/11/2026.** T01–T13 và CP01–CP07 là lịch đã thực hiện. Mốc kỹ thuật gần nhất là CP07 trên `rewrite/rust` tại `8edd20965ef7c4c5d156b54fdf5c3e36b6bcd3a2`. M2 đang chờ người dùng nghiệm thu. Năm task dưới đây tiếp nhận góp ý trên Windows. Tất cả bắt đầu ở trạng thái **Chưa thực hiện**. Kết quả cũ của B chỉ bao phủ candidate cũ.

Khi nhập xong, người đọc cần thấy tập vừa nhập, nhìn được các chương theo thứ tự, mở chữ Nhật và tìm từng ảnh ngay tại vị trí xuất hiện. Khi bấm phím hay gõ vào ô nhập, tiêu đề phải đứng yên và người dùng phải nhìn được mục đang chọn. Trong ảnh góp ý, tiêu đề “Chưa có nguồn” xuất hiện cùng một tập đã nhập. “3/12” đếm vị trí trong danh sách có cả phần phụ. “6997 cần kiểm” cộng nhiều loại vấn đề của cả tập. Những con số này là ví dụ quan sát được, không phải số chương, ngưỡng lỗi hay hằng số để đưa vào code.

### 17.1. Giao việc và hai checkpoint

| Checkpoint | Task theo thứ tự | Đầu vào và thời điểm công bố | Master thử được gì sau khi merge và hậu kiểm |
|---|---|---|---|
| <a id="p02-cp08"></a>**P02-CP08 — Vẽ màn hình và nhập liệu** | [T14](#p02-t14) → [T15](#p02-t15) | Nhận CP07 đã công bố. Chạy V-07, V-10 và V-00 trên bản ghép. B kiểm cả vòng đời terminal và form có chooser. Commit, push và mở/cập nhật draft PR ngay khi đủ cổng. | Giữ phím di chuyển và gõ vào form sau khi quay về từ hộp chọn tệp. |
| <a id="p02-cp09"></a>**P02-CP09 — Duyệt chương và ảnh sau khi nhập** | [T16](#p02-t16) → [T17](#p02-t17) → [T18](#p02-t18) | Nhận đúng SHA của CP08 sau merge và hậu kiểm. B kiểm bản ghép với EPUB 2/3 qua luồng nhập thật, các ca §17.7 và V-00. Commit, push và mở/cập nhật draft PR của CP09 ngay khi đủ cổng. | Từ tập vừa nhập tìm được Chương 1, ảnh trong chương, lý do cần xem lại và tệp ảnh trong Explorer. |

**Nguồn giao việc.** Điều phối Phase 02 đăng ký đúng năm task, hai checkpoint và quyền ghi trong PLAN cùng [graph thực thi](../PARALLEL_TASKS.json) trước khi giao code. Lịch bổ sung đặt đầu ra kỹ thuật cuối M2 ở T18/CP09. CP07 là mốc đã công bố của lịch ban đầu. Consumer đã nhận CP05/CP06 tiếp tục dùng đúng bản đã bàn giao. Consumer yêu cầu toàn bộ M2 nhận bản mới sau hậu kiểm và nghiệm thu của Master. Một người điều phối nhận kết quả và ghép tuần tự, mỗi lượt có phiếu riêng hoặc nhóm 2–3 task gắn chặt đã đăng ký trước.

Lịch v2 đang dùng vẫn trỏ M2 vào T13. Khi nhận việc, điều phối thêm năm node theo hai hàng trên, chuyển con trỏ M2 sang T18 và cờ checkpoint cuối từ CP07 sang CP09 trong phiên bản lịch mới. Với nền 112 task/64 checkpoint, lịch sau tiếp nhận có 117 task/66 checkpoint. Cập nhật các tổng hiện hành, kiểm chu trình và các cổng mở sớm. Giữ phiên bản mà các consumer đang làm đã ghim. Gói bàn giao nội bộ có `graph-r01.patch` áp được lên lịch v2 và phần §17 riêng để đưa vào worktree. Nếu nền đã đổi, đối chiếu bảng trên và tạo diff theo nền mới.

**Các phần đã merge.** Từ merge CP07, A đối chiếu [quy tắc protected](../MERGE_WORKFLOW.md) với M1, CP06 (`3155243be22768a3036c14ed8cf0e1e87bffdcbc`) và CP07, ghi merge sở hữu từng đoạn thực sự cần sửa. Với đoạn đã nghiệm thu, trình Master lỗi tái hiện, file/hành vi, diff tối thiểu và test trước khi áp dụng. Quyền đã có hiệu lực cho đúng diff thì dùng tiếp. Sự đồng ý về thiết kế và prompt dưới đây giao việc lập candidate, chưa phê duyệt trước một diff runtime chưa thấy. Giao một owner cho mỗi file dùng chung. Tài liệu publication và candidate mà B đang review có bản ghim riêng.

**Hợp đồng dùng lại.** [TUI và phím theo ngữ cảnh](../../spec/TUI_CLI_SPEC.md), [thiết kế màn Tập và chương](../../desgin/UI_UX_DESIGN_STEP8.md), [thứ tự đọc và ranh giới chương](../../spec/EPUB_INPUT_SPEC.md), [ảnh gốc và ảnh chỉnh](../../spec/IMAGE_EDIT_SPEC.md), [vị trí nguồn](../../decisions/ADR-003.md) và [ranh giới crate](../../decisions/ADR-008.md). Việc duyệt sách và lấy bản sao một ảnh dùng nguồn đã lưu. Dịch, chỉnh ảnh và xuất EPUB thuộc các phase tương ứng.

<a id="p02-t14"></a>

### 17.2. P02-T14 — Di chuyển trong terminal mà màn hình không nháy

1. Trên bản CP07, ghi Windows/terminal, kích thước và chuỗi phím tái hiện. Tại `nt-tui/src/terminal.rs`, `draw_buffer` hiện xóa màn hình rồi in từng dòng và đặt lại con trỏ ở góc trên. Ghi lệnh vẽ cho hai khung liên tiếp khi đứng yên và khi chỉ đổi một dòng chọn. Chụp hình buffer tĩnh chưa đo được nháy.
2. Để một nơi sở hữu khung trước và lần ghi ra terminal. Lần vẽ đầu tạo khung đầy đủ. Khi di chuyển thì so hai khung, ghi các ô thay đổi trong cùng lượt và flush khi hoàn tất. Tính cả kiểu hiển thị, phần cuối dòng vừa ngắn đi và ô tiếp nối ký tự rộng. Dựng lại khung khi kích thước/bề mặt terminal thực sự mất hiệu lực.
3. Kiểm chuyển màn, đóng lớp trợ giúp, thu/phóng, thoát và đường lỗi trong lúc vẽ. Khôi phục con trỏ, chế độ nhập và màn hình gốc đúng vòng đời hiện có. Giữ xử lý event và công việc chạy nền hoạt động khi khung nhìn đứng yên.

| Trường | Nội dung giao việc |
|---|---|
| Đầu vào bắt buộc | P02-T13 đã Đạt, receipt CP07 và SHA nêu ở §17. Phiếu ghim bản terminal, TUI spec và review M1/CP đã đụng. |
| Hợp đồng dùng chung | Nhận kích thước và trạng thái hiện tại, xuất khung đúng cả ký tự lẫn vùng chọn. Một lần vẽ lỗi đi qua cleanup hiện hành. T15 nhận quy tắc ai vẽ con trỏ. |
| Phạm vi sở hữu | A-P02 phụ trách `crates/nt-tui/src/terminal.rs`, các đoạn vẽ liên quan trong `rendering.rs` và test bộ ghi terminal. Điều phối cấp đoạn dùng chung với T15. |
| Task có thể chạy cùng | Các chat ngoài P02 đã có đầu vào riêng có thể tiếp tục. Trong P02 cho phép chuẩn bị kịch bản nhập liệu bằng fixture riêng trong lúc B xem T14. |
| Kiểm thử riêng | **P02-AC14 / TC14a–c.** (a) Hai khung giống nhau và di chuyển ↑/↓ không phát lệnh xóa toàn màn trong lượt bình thường. (b) Replay các lần vẽ cho kết quả đúng như vẽ đầy đủ với tiếng Việt, chữ Nhật rộng, dòng ngắn lại và resize. (c) Giữ ↓ rồi ↑ trên Windows Terminal thật, kiểm vùng chọn, nháy và khôi phục terminal sau khi thoát. V-07, V-10. |
| Điểm tích hợp | B review T14, A giao renderer cho T15. Ở CP08 chạy lại TC14 cùng ca nhập chữ và chooser. |

<a id="p02-t15"></a>

### 17.3. P02-T15 — Chữ nằm trong ô nhập và người dùng thấy rõ nút đang chọn

1. Tái hiện riêng lỗi gõ chữ đè lên “Novel Translate” khi nhập tên/đổi tên, khi vừa trở về từ chooser và sau resize. Kiểm chỗ nhận sự kiện phím, chế độ echo của console, con trỏ hệ điều hành và tọa độ ô nhập. Việc đặt con trỏ ở góc trên là dấu hiệu cần kiểm tra, chưa kết luận được nguyên nhân của lỗi nhập chữ.
2. Mỗi lần gõ chỉ form đang có focus nhận chữ. Vẽ dấu nhập theo chiều rộng ô hiển thị bên trong trường, cuộn trường khi đầy. Khi rời ô nhập, trở về kiểu con trỏ phù hợp. Sau chọn hoặc hủy chooser, đưa focus về đúng control. Tab/Shift+Tab đi theo thứ tự nhìn thấy. Enter và bấm chuột vào cùng nút kích hoạt cùng một hành động. Nút hoặc dòng đang chọn có dấu hiệu nhìn được cả khi terminal ít màu.
3. Kiểm bộ gõ tiếng Việt, chữ Nhật đã xác nhận nhập, khoảng trắng, phím giữ, Backspace, Esc và quay lại từ hộp chọn. Khi đang gõ, chữ trùng phím tắt vẫn vào ô. Đổi kích thước hoặc đóng lớp trên cùng phải trả người dùng về đúng chỗ cùng nội dung nháp.

| Trường | Nội dung giao việc |
|---|---|
| Đầu vào bắt buộc | T14 và hợp đồng khung/con trỏ của đúng candidate B duyệt, hoặc cùng nhóm đã được điều phối ghi rõ. Nhận lại ca chooser T11/T12. |
| Hợp đồng dùng chung | Focus, vị trí con trỏ và vùng bấm dùng cùng bố cục. Chọn/hủy hộp thoại và sự kiện đến muộn chỉ ảnh hưởng form/yêu cầu còn hiệu lực. |
| Phạm vi sở hữu | A-P02 phụ trách `crates/nt-tui/src/input.rs`, `screens/workspace_form.rs`, `screens/import_draft.rs` và đường hiển thị con trỏ cần thiết. `windows_input.rs` chỉ vào phiếu nếu ca tái hiện dẫn đến đó. |
| Task có thể chạy cùng | Chat khác dùng worktree và terminal/profile thử riêng. Hai người sửa đường nhận phím hoặc con trỏ được điều phối xếp nối tiếp. |
| Kiểm thử riêng | **P02-AC15 / TC15a–d.** (a) Gõ tên tiếng Việt/Nhật, khoảng trắng và Backspace, đối chiếu giá trị trường và tiêu đề. (b) Tab/Enter và chuột kích hoạt đúng một lần, cả sau khi chooser trả về. (c) Hủy, resize và kết quả cũ đến muộn giữ đúng nháp/focus. (d) Người thử dùng Windows Terminal thật với bộ gõ và hộp thoại thật, ghi phím và kết quả. V-07, V-10. |
| Điểm tích hợp | A chạy TC14–15, V-00 và gửi B bản ghép CP08. Khi hậu kiểm xong, bàn giao SHA cho T16 và gửi Master cách thử UT06. |

<a id="p02-t16"></a>

### 17.4. P02-T16 — Đọc được cả mục lục, vị trí ảnh và lý do cần xem lại

1. Bắt đầu từ `BookView`, `describe_tree` và `ReviewIssue`. Hiện đường đọc trả về chủ yếu dòng được chọn và tổng số vấn đề. Tạo phần đọc vừa đủ cho giao diện lấy danh sách các mục cùng con của mục đang mở, đoạn nguyên tác và từng vị trí ảnh. Tái dùng snapshot, mapping đã lưu và phạm vi theo tập. Dùng thứ tự đọc, tiêu đề nguồn và chỉnh sửa ranh giới đã xác nhận để lập danh sách.
2. Phân biệt mở đầu, các chương có tên, phần kết, mục phụ và vùng chưa xác định theo bằng chứng. Chương có thể trải qua nhiều XHTML, hoặc nhiều chương dùng chung một XHTML. Giữ vị trí ảnh xen giữa chữ. Đếm ảnh toàn tập theo tài nguyên riêng và đếm/lần theo từng lần xuất hiện trong chương. Ảnh chưa gắn được vào chương vẫn tìm thấy trong danh sách ảnh toàn tập, với lý do cụ thể.
3. Trả lý do cần xem lại theo loại và vị trí nếu đã biết. Phân biệt tổng số ghi nhận, số chương, số ảnh và số lần xuất hiện. Khi chưa xác định vị trí, trả lý do để UI hiển thị ở cấp tập. Đọc theo phần đang xem, dùng lại dữ liệu trong cùng phiên bản. Chuyển mục trong cây không giải nén EPUB hoặc đọc lại toàn tập cho từng phím.

| Trường | Nội dung giao việc |
|---|---|
| Đầu vào bắt buộc | T15 ở CP08 đã công bố, hợp đồng mapping T07/T10, công bố nguồn T09 và dịch vụ đọc T12. Ghim phiên bản fixture có đáp án độc lập. |
| Hợp đồng dùng chung | Yêu cầu mang workspace, volume, generation/snapshot và phiên bản mapping. Kết quả chứa định danh mục, tiêu đề, thứ tự, trạng thái tạm/đã xác nhận, vị trí nguồn và tham chiếu ảnh khi có. Phân biệt đang đọc, rỗng, lỗi đọc và kết quả cũ. Xem dữ liệu là thao tác chỉ đọc. |
| Phạm vi sở hữu | A-P02 phụ trách phần đọc trong `apps/novel-translate/src/import_service.rs` và dữ liệu UI ở `crates/nt-tui/src/import_commands.rs`. Dùng các reader ứng dụng sẵn có. Ghi riêng đề xuất nếu thật sự cần đổi hợp đồng parser, storage hay schema đã merge. |
| Task có thể chạy cùng | Chat P03/P07 tiếp tục trên hợp đồng đã nhận khi có quyền riêng. Có thể soạn dữ liệu kỳ vọng ở checkout thử, rồi T17 mới nối reader đúng bản đã review. |
| Kiểm thử riêng | **P02-AC16 / TC16a–d.** (a) EPUB 2 NCX với chương qua nhiều file và EPUB 3 Nav với hai chương cùng file cho đúng thứ tự và tên. (b) Một ảnh xuất hiện hai nơi, hai ảnh trùng tên khác thư mục, ảnh chưa gắn chương cho đúng số và vị trí. (c) Mục lục thiếu/mâu thuẫn và mục phụ cho nhãn có căn cứ. (d) Đổi tập/mapping khi đang đọc loại kết quả cũ, đọc lại dữ liệu đã lưu cho kết quả đúng và generation không đổi. V-05, V-06 cùng test mới trên reader thật. |
| Điểm tích hợp | B kiểm bảng đối chiếu mục lục/ảnh/lý do bằng fixture tự soạn. Giao cho T17/T18 định danh đã duyệt và cách tìm đúng phiên bản ảnh. |

<a id="p02-t17"></a>

### 17.5. P02-T17 — Hiện kết quả nhập, cây chương và nguyên tác có ảnh

1. Cả hai nút **＋ Nhập tập truyện** dùng cùng luồng. Khi đang nhập, hiện tên tệp và trạng thái thực sự biết. Khi đã lưu, chuyển về tập mới và mở sẵn cây. Nếu cần thời gian lấy mục lục thì hiện “Đã nhập tập · Đang đọc mục lục…”. Lỗi đọc cây sau khi lưu có **Đọc lại**, tránh người dùng nhập trùng. Hủy, lỗi trước khi lưu hoặc đã đổi workspace cần trả đúng màn và trạng thái. Chỉ hiện “Chưa có nguồn” khi tập được chọn thực sự trống.
2. Dòng trạng thái hiển thị số chương chính và số ảnh lấy từ T16. Các hàng đầu tiên cho thấy các chương cạnh nhau, giữ tên gốc và thứ tự. Mở chương để chọn **Đọc chương** hoặc **Ảnh trong chương (n)**. Ở nguyên tác, nhãn ảnh nằm giữa các đoạn đúng vị trí nguồn. Từ danh sách ảnh quay đến được lần xuất hiện đã chọn. Với ảnh ở nhiều chỗ, cho chọn vị trí. Đưa các cảnh báo vào **Cần xem lại → Xem lý do** có mô tả bằng tiếng Việt và đường đến mục liên quan. Với ranh giới tạm, diễn đạt rõ điểm bắt đầu/kết thúc chương còn cần đối chiếu.
3. ↑/↓ đổi hàng, Enter mở mục đang chọn, Esc quay lại một cấp, Tab chuyển vùng. F1 và thanh cuối giải thích các thao tác hợp lệ của mục hiện tại. Đặt các lệnh phạm vi, phân loại, đổi tên, gộp, tách và dời ranh giới trong **Thao tác khác** có nhãn rõ. Dùng lại luồng nháp/xem trước/xác nhận tương ứng của T10/T12. Bấm chuột vào hàng/nút làm cùng việc như bàn phím. Ở 80×24 mở chi tiết theo từng màn, từ 120 cột có cây cạnh phần đọc, dưới mức tối thiểu áp dụng trạng thái thu nhỏ của TUI. Resize và quay lại giữ đúng mục, vị trí đọc và nháp.

Ví dụ bố cục cho bộ mẫu có năm chương và 21 ảnh. Mỗi nhãn và số đếm thực tế phải lấy từ tập đang mở.

~~~text
Đã nhập Tập 1 · 5 chương chính · 21 ảnh             [＋ Nhập tập truyện]
▾ Tập 1
    Mở đầu
  ▾ Chương 1 · <tên trong mục lục>
      Đọc chương
      Ảnh trong chương (4)
  ▸ Chương 2 · <tên trong mục lục>
    …
    Phần kết
    Ảnh toàn tập (21)
↑ ↓ Chọn mục     Enter Mở     Esc Quay lại     [Cần xem lại] [Thao tác khác]
~~~

| Trường | Nội dung giao việc |
|---|---|
| Đầu vào bắt buộc | T16 được B duyệt cùng bộ dữ liệu có nhãn/mapping tạm. Nhận hành vi focus của CP08 và các action T12. |
| Hợp đồng dùng chung | Cây, vùng chi tiết và thông báo dùng cùng tập/phiên bản. Chọn dòng hoặc xem lý do chỉ điều hướng. Các chỉnh sửa lưu qua use case cũ khi có xác nhận. |
| Phạm vi sở hữu | A-P02 phụ trách `crates/nt-tui/src/screens/import_screen.rs`, `rendering.rs`, input/state liên quan và test `workspace_import.rs`. Dùng reader T16 và điểm mở tệp T18. |
| Task có thể chạy cùng | T18 có thể chuẩn bị fixture và bộ giả chooser trong vùng riêng. Ghép nút và reader sau khi hợp đồng T17 đã Đạt, hoặc theo nhóm ghép đã đăng ký. |
| Kiểm thử riêng | **P02-AC17 / TC17a–d.** (a) Từ cả Tổng quan và Tập và chương, nhập/hủy/lỗi/đã lưu nhưng đọc lỗi đều cho tiêu đề và nút tiếp theo đúng. (b) 80×24 và ≥120 cột thấy đúng cây, chọn Chương 2 ngay từ danh sách, click/Enter trùng kết quả. (c) Đọc qua đoạn–ảnh–đoạn, theo liên kết tới ảnh dùng lại rồi quay về đúng hàng. (d) Xem lý do khi có/thiếu vị trí, thử mở menu và hủy chỉnh sửa, đổi tập/resize/reopen. Nội dung cũ, generation và draft đúng với từng thao tác. V-06, V-07, V-10. |
| Điểm tích hợp | B đối chiếu màn hình với đáp án EPUB thật của fixture. A ghép cây và hành động ảnh với T18 ở CP09. |

<a id="p02-t18"></a>

### 17.6. P02-T18 — Mở hoặc lưu ảnh gốc bằng tên dễ tìm

1. Khi chọn ảnh, hiển thị vị trí dễ đọc như **Tập 1 › Chương 1 › Ảnh 2**, tên tệp gốc và các vị trí dùng. **Mở thư mục chứa ảnh** mở Explorer và chọn đúng tệp nguồn đã xác minh của volume/snapshot hiện tại, kể cả khi có hai ảnh trùng tên. Chuỗi ID/hash dành cho việc đối chiếu nội bộ. Tên thân thiện chỉ là nhãn trên màn hình.
2. **Lưu bản sao vào…** mở hộp chọn nơi lưu trên Windows, gợi ý tên ảnh gốc và cho người dùng chọn đường dẫn ngắn. Ghi đúng byte của ảnh đã chọn. Khi đích đã tồn tại, cho chọn tên khác hoặc xác nhận thay thế rõ ràng. Hủy hoặc gặp lỗi thì bảo toàn tệp đích cũ, thu hồi phần ghi dở và cho thử lại. Loại đích trùng nguồn hoặc đè vào DB, object, snapshot được quản lý. Bản sao người dùng lưu là tệp riêng với ảnh chỉnh dùng để xuất EPUB.
3. Giải địa chỉ ảnh bằng tham chiếu đã kiểm của snapshot. Kiểm đúng ảnh hiện có, đường dẫn Unicode/có dấu cách, ảnh thiếu/hỏng và yêu cầu đã cũ sau đổi tập. Dùng adapter hệ điều hành với đối số đường dẫn tách riêng. Hộp thoại và I/O chạy ngoài vòng vẽ. Ghi nhận lỗi Explorer/quyền bằng thông báo có hành động tiếp theo. Chỉ nhận thành công cho đúng yêu cầu.

| Trường | Nội dung giao việc |
|---|---|
| Đầu vào bắt buộc | T17 và các định danh nguồn do T16 bàn giao ở cùng phiên bản. Ghim đường đọc ảnh gốc T02/T09 và fixture trùng tên. |
| Hợp đồng dùng chung | UI gửi định danh tài nguyên/lần xuất hiện và phiên bản. Adapter trả đã mở/đã lưu/hủy/lỗi cho yêu cầu phù hợp. Mở Explorer chỉ đọc nguồn. Lưu bản sao chỉ ghi đích người dùng đã chọn. |
| Phạm vi sở hữu | A-P02 phụ trách hành động ảnh trong TUI, adapter Windows ở `apps/novel-translate/src`, đường nối app và test riêng. Đọc ảnh qua API kho hiện hành. Cần chạm đoạn runtime đã merge thì dùng phiếu diff tại §17.1. |
| Task có thể chạy cùng | Chat P07 tiếp tục phần ảnh chỉnh/xuất bằng phiên bản đã nhận. Hai owner xếp nối tiếp nếu cần cùng điểm nối app hoặc cùng file. |
| Kiểm thử riêng | **P02-AC18 / TC18a–d.** (a) Capture đúng đường dẫn được giao cho Explorer, kiểm chọn tệp thật trên Windows. (b) Lưu đích ngắn có Unicode/khoảng trắng, so byte/hash với ảnh nguồn. (c) Hủy, trùng tên, từ chối ghi đè, lỗi quyền/ghi giữa chừng, đích trùng nguồn và ảnh mất/hỏng đều bảo toàn dữ liệu. (d) Đổi tập khi chooser đang mở và mở lại workspace, đối chiếu file, cây, nguồn và generation. V-02, V-06, V-07, V-10 và V-00 bản ghép. |
| Điểm tích hợp | A nộp B bản ghép CP09 và lượt kiểm toàn M2 trên candidate mới. Sau merge/hậu kiểm gửi Master UT07–UT09. Ghi riêng verdict kỹ thuật, phản hồi của Master và kết quả nghiệm thu cuối. |

<a id="p02-ux-user-test"></a>

### 17.7. Master tự nghiệm thu trên Windows

CI dùng EPUB tự soạn, workspace tạm riêng và đáp án lập trước. B chạy lại với parser, lưu/mở lại thật và terminal Windows. Sách riêng chỉ dùng cho lượt Master tự thử tại máy mình. Ảnh chụp, EPUB, đường dẫn cá nhân và evidence để cục bộ. Ghi bản đã chạy, loại terminal, kích thước, kết quả **Đạt / Không đạt / Chưa thử** cho từng ca. Sau mỗi checkpoint A giao lệnh chạy từ đúng commit/binary và thư mục fixture.

| Ca | Thao tác của người dùng | Pass khi |
|---|---|---|
| **P02-UT06 · CP08** | Tạo/mở workspace thử. Giữ ↑/↓, bấm Tab, nhập tên có dấu. Mở rồi hủy hộp chọn thư mục/tệp. Gõ tiếp, thu nhỏ rồi mở rộng cửa sổ. | Tiêu đề đứng yên, chữ chỉ xuất hiện trong ô nhập, nút đang chọn dễ nhận ra. Không cần bấm lại để tìm focus. |
| **P02-UT07 · CP09** | Bấm **＋ Nhập tập truyện**, chọn EPUB, đợi thông báo rồi mở Tập 1. Chọn Chương 1, sang Chương 2 bằng bàn phím. Mở một mục **Cần xem lại** nếu sách có. | Thấy kết quả nhập và thứ tự các chương cạnh nhau. Đọc được nguyên tác và lý do phải kiểm. Với đúng ấn bản đã dùng góp ý, đối chiếu mở đầu, Chương 1–5 và phần kết với mục lục. |
| **P02-UT08 · CP09** | Mở **Ảnh trong chương**. Chọn một ảnh, xem vị trí giữa các đoạn, mở thư mục chứa ảnh rồi **Lưu bản sao vào…** một thư mục thử ngắn. Thử hủy lượt lưu kế tiếp. | Explorer chọn đúng tệp, bản sao mở được và khớp ảnh gốc, hủy không sinh tệp. Với đúng ấn bản trên đối chiếu 21 ảnh của tập và 4 ảnh trong Chương 1. Với sách khác dùng số thực trong EPUB đó. |
| **P02-UT09 · CP09** | Đóng app rồi mở lại workspace. Mở lại Chương 1, xem ảnh và vị trí nguồn, thử ở 80×24 và từ 120 cột. | Tìm lại được chương/ảnh mà không phải nhập lại EPUB hay tìm thư mục hash. Cây, mapping và nội dung đã lưu khớp trước khi đóng. |

<a id="p02-ux-prompt"></a>

### 17.8. Prompt cho chat điều phối Phase 02

Dán vào **chat đang phụ trách Phase 02**. Nếu worktree đó có tài liệu cũ, lấy §17 từ checkout tài liệu gốc rồi ghim SHA-256 vào phiếu. Mỗi checkpoint có lượt review, công bố và hướng dẫn Master thử riêng.

~~~text
Tiếp tục Phase 02 từ §17 của docs/roadmap/phases/PHASE-02.md. Bắt đầu P02-T14.
Đọc AGENTS.md, PLAN.md và các file trong docs/roadmap gồm REVIEW_WORKFLOW.md,
PARALLEL_EXECUTION.md, PARALLEL_TASKS.json, MERGE_WORKFLOW.md cùng spec/ADR của §17.1.
Xác minh CP07 đã merge và hậu kiểm tại 8edd2096.
Đối chiếu với rewrite/rust hiện tại và phần các chat khác đang làm. Bảo toàn các bản đang review.
Nhận graph-r01.patch trong gói nội bộ post-import-ux-plan-2026-11-21 ở evidence/phase-02
hoặc đối chiếu bảng §17.1 trên nền mới. Với vai điều phối, ghi T14 → T15 tại CP08 và
T16 → T17 → T18 tại CP09 vào PLAN/graph, giữ các cổng nhận CP05/CP06 đã có.
Ghim đầu vào, owner, file, baseline và bằng chứng trước khi sửa. Dùng worktree, nhánh feature/
và dữ liệu thử riêng. Làm từng task hoặc nhóm nhỏ đã đăng ký. Với phần đã merge cần đổi,
tái hiện lỗi, đưa đúng diff tối thiểu và test cho Master duyệt theo MERGE_WORKFLOW.
Thực hiện AC/TC của từng task và thử trên Windows Terminal thật. Tự gọi gpt-6.1-sol
reasoning max trong phiên độc lập để review. Sửa findings, cho B kiểm lại bản ghép.
Khi từng CP đủ V-00, manual và B Đạt, commit có body, push thường, cập nhật draft PR ngay.
Điều phối merge khi đủ cổng, hậu kiểm, dọn scratch theo quy tắc và báo Master cách thử.
Sau CP08 bàn giao UT06, rồi tiếp tục CP09 khi đủ đầu vào. Sau CP09 bàn giao UT07–UT09,
hậu kiểm M2 và chờ Master xác nhận nghiệm thu. Lịch sử T01–T13 và CP01–CP07 vẫn truy được.
~~~
