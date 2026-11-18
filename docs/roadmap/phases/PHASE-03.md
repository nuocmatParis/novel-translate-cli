# PHASE 03 — REVISION, TRẠNG THÁI VÀ NGỮ CẢNH ĐÃ DUYỆT

> Lịch thực thi v2 · 12/11/2026.

> Lịch task song song và checkpoint commit/push tại [mục 10](#p03-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Kế hoạch triển khai v0.3 · 09/11/2026 · Chưa bắt đầu.** M3 nhận dữ liệu nguồn và đường lưu đã nghiệm thu ở M2. Đặc tả và ADR là căn cứ cho hành vi Rust. Rà code Go để biết còn đường nào sử dụng khi chuyển đổi.
>
> Giữ yêu cầu Q08 đã bổ sung ở P00-T01-r02 sau finding F03. T12/AC12/TC12 phải kiểm gắn lại đơn vị nguồn khi marker thiếu hoặc trùng, gồm nhiều đích ứng viên, giữ byte sửa tay và tạo revision chờ Review khi xác nhận.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P03 / M3 |
| Tên | Revision, trạng thái và ngữ cảnh đã duyệt |
| Trạng thái | Kế hoạch đã chi tiết. Chờ M2, chưa triển khai hoặc chạy test M3. |
| Đầu vào | Workspace Rust, snapshot/mapping, BlockID/SourceRange, kho file, migration và writer/CAS từ M2 |
| Giao diện | UI-04 Editor Nhật–Việt và CLI `terms list/confirm`. UI duyệt nhóm hoàn thiện ở Phase 06. |
| Phân rã | 5 gói, 15 task |
| Nhánh | Nhánh theo phần việc, ví dụ `feature/revisions` hoặc `feature/skills`. PR vào `rewrite/rust`. |
| Vai trò | A triển khai, kiểm và bàn giao. B khác review độc lập. |

Xem [roadmap chung](../ROADMAP.md), [phạm vi Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-03) và [quy tắc repo](../../../AGENTS.md). Hồ sơ chi tiết ở máy người dùng theo quy tắc tài liệu nội bộ.

## 2. Mục tiêu và demo cuối phase

Sau M3, người dùng sửa bản dịch, xem lịch sử và đồng bộ bản thảo bên ngoài mà không mất sửa tay. Quyết định đã duyệt dùng đúng phạm vi và phiên bản. Nếu căn cứ đổi, người dùng thấy vùng cần kiểm lại. Phase 04 có thể ghim đúng revision, nguồn và ngữ cảnh khi nối Dịch–Review.

1. Mở workspace mẫu M2 có bản nháp. Vào Editor ở đúng đoạn, sửa câu và nhấn `Ctrl+S`. Mở lịch sử thấy cả bản cũ và bản mới chờ Review.
2. Đợi bản thảo Markdown được cập nhật. Sửa ngoài một đơn vị rồi đồng bộ. Dùng mẫu xung đột để xem bản nền, bản sửa tay và ứng viên. Thử gắn lại một marker thiếu/trùng từ danh sách đích ứng viên.
3. Dùng bộ thử T05 để nạp thư viện chung và xem trước một Skill riêng. So heading/bảng, một mẫu sai–đúng và mục Kanji đã tra. Xác nhận phần rõ của Skill riêng và đối chiếu bytes file gốc.
4. Qua CLI, chọn đúng nhóm thuật ngữ. Dùng bộ thử giả lập lỗi ghi Skill và mở lại. Đối chiếu trạng thái chờ đồng bộ và lần kích hoạt cả nhóm.
5. Thay quy tắc có phạm vi và đối chiếu đơn vị bị ảnh hưởng. Kiểm ngữ cảnh cho hai cảnh có mốc truyện/vị trí đọc khác nhau. Xem được lý do chọn hoặc chờ làm rõ.
6. Đổi mapping hoặc nguồn trên bản sao. Kiểm đơn vị còn đủ căn cứ được dùng lại và trường hợp mơ hồ giữ lịch sử để đối chiếu.

Phần người dùng thao tác ở [mục 16.4](#p03-user-test). Case ngắt tiến trình, đối chiếu hash và migration dùng bộ kiểm ở mục 13.

## 3. Phạm vi

### 3.1. Phần triển khai và phần bàn giao

| Phase 03 thực hiện | Consumer tiếp theo |
|---|---|
| Revision bất biến, current/history, ba trục trạng thái, lưu sửa tay và đồng bộ Markdown | Phase 04 lưu nháp Dịch, ghim Review đúng revision và scope. |
| Căn cứ theo nguồn/version, dependency và occurrence, xác định vùng cần kiểm lại | Phase 04 đưa context vào request giả lập. Phase 06 nối luồng xử lý và truyền thay đổi đầy đủ. |
| Skill dùng chung/riêng, nhân vật/bí danh/xưng hô, đề xuất và duyệt có lịch sử | Phase 06 nối Research, màn duyệt nhóm và tiếp tục tác vụ theo quyền chạy. |
| Editor UI-04, CLI terms, phục hồi và nâng workspace Rust M2 | Phase 07 dùng revision và quy tắc hiệu lực cho kiểm sẵn sàng xuất. |

M3 dùng bản nháp, kết quả Review và nguồn đề xuất tự soạn để kiểm hành vi lưu/đọc. Gọi model, đo chất lượng Dịch, sửa tự động, Research và tiếp tục qua Coordinator thuộc các phase nhận. Định dạng workspace Rust nâng theo quy tắc M2. Xử lý workspace test Go cũ theo Q06.

<a id="p03-go-retirement"></a>

### 3.2. Thay dần đường Go

Mục tiêu là **bản cuối chỉ cần Rust để build, test, chạy và phát hành**. Với từng phần, đưa đường chạy Rust vào dùng khi đã có bằng chứng thay thế. Chỉ gỡ file hoặc package Go sau khi xác định các đường Go còn giữ không dùng nó và phép kiểm trên nhánh thay đổi xác nhận vẫn chạy được phần còn lại. Giữ bản Go đang phục vụ người dùng trên `main` cho đến khi bản Rust thay đủ phạm vi đã chốt.

| Phần còn ở Go | Mốc có thể đánh giá bản Rust | Điều kiện gỡ |
|---|---|---|
| Lệnh chạy, TUI, script và CI | M1 có shell. M8 có CLI/TUI nghiệp vụ, M9 kiểm bản phát hành. | Chuyển đúng entrypoint và người dùng. Kiểm lệnh, exit, nền tảng và các pipeline còn phục vụ Go. |
| Workspace, nhập EPUB và cây nguồn | M2 | Luồng tạo/nhập/mở lại đạt. Xác định editor/dịch/xuất Go nào còn dùng importer hoặc workspace Go trước khi gỡ. |
| Revision, Markdown, Skills và xưng hô | M3, rồi consumer Dịch–Review/Research ở M4–M6 | Kiểm sửa tay, đọc lại, duyệt, recovery và consumer còn gọi. Giữ đường Go vẫn cần cho chức năng dịch chưa chuyển. |
| Chạy model, hàng đợi, Research, repair | M4–M6, đối chiếu chất lượng và recovery ở M9 | Kiểm quyền chạy, ngân sách, điểm chờ, sử dụng thực và dữ liệu sau lỗi. |
| Ảnh, preview, xuất, đóng gói | M7–M9 | Kiểm reader và artifact Rust. Chuyển Docker/CI/release và hướng dẫn tương ứng trước khi bỏ Go toolchain. |

Khi một phần đủ điều kiện, ghi một việc dọn riêng theo phạm vi vào PLAN. Bằng chứng gồm chức năng đang thay, caller/script/config đã chuyển, ca nghiệm thu Rust, phép kiểm các đường Go còn giữ và cách quay lại bản cuối dùng Go nếu cần. A và B review phần dọn theo quy trình repo. Tránh tạo cầu nối hai ngôn ngữ chỉ để tách file. Dữ liệu và lịch sử người dùng có quy trình riêng.

**Trạng thái 12/11/2026.** M1 đã Đạt trên Windows và Phase 01 đã merge; M2/M3 chưa triển khai. Chưa có kiểm kê caller hoặc bằng chứng cho phép gỡ Go trong lượt viết tài liệu này. Mốc đánh giá sớm theo checkpoint, owner và sổ dọn hiện hành nằm tại [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Bảng trên chỉ phạm vi chức năng cần kiểm; không buộc chờ toàn milestone mới rà/dọn phần độc lập đủ điều kiện. M9 tổng hợp parity, GO-FINAL kiểm bản chỉ Rust sau dọn. [Phase 09](PHASE-09.md)

## 4. Điều kiện bắt đầu

P03-T01 nhận P02-T09 đã Đạt và checkpoint P02-CP05 đã công bố. P03-T14 còn cần P02-T10. P03-CP04 bàn giao resolver cho P04-T01; approval nhận riêng ở P03-CP05 và Editor ở P03-CP02. P03-T15/M3 vẫn chờ M2 và toàn bộ phần việc P03.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-SKILL | [Skills và duyệt thuật ngữ](../../spec/SKILLS_GLOSSARY_SPEC.md) |
| S-ADDRESS | [Xưng hô và ngữ cảnh](../../spec/ADDRESSING_CONTINUITY_SPEC.md) |
| S-TRANS | [Giai đoạn Dịch](../../spec/TRANSLATION_STAGE_SPEC.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-MODULE | [Bước 4 — Module và ownership](../../desgin/MODULE_DESIGN_STEP4.md) |
| D-UI | [Bước 8 — UI/UX](../../desgin/UI_UX_DESIGN_STEP8.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REVIEW | [Quy trình hai model](../REVIEW_WORKFLOW.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Qxx là quyết định xx trong S-Q. Chốt schema, kiểu và tên API theo use case. Giữ các bất biến sau khi chọn kỹ thuật.

- **Q07–08.** Revision bất biến, có base/author/lý do/range. Lưu sửa tạo revision mới chờ Review. DB công bố qua generation. Markdown có base hash và marker từng đơn vị. Sau lỗi dựng file, revision đã commit còn đọc được và công việc dựng file chờ tiếp.
- **Q09–10.** Chất lượng gắn revision, hiệu lực gắn căn cứ, thực thi gắn tác vụ. Đổi một trục giữ hai trục còn lại. Ghi toàn bộ context/quy tắc cung cấp. Nếu chưa chứng minh ảnh hưởng cục bộ, giữ phạm vi cả lượt. Quy tắc mới đối chiếu thêm occurrence index.
- **Q11–12.** Skill riêng có version và sáu mục. File hiện có được xem trước theo ba nhóm đọc được, cần làm rõ và ghi chú. Chỉ quy ước người dùng xác nhận và quyết định đủ căn cứ mới thành quy tắc chính thức. Lọc đúng scope và ưu tiên quy tắc riêng đã duyệt. Xung đột bắt buộc cùng hiệu lực có lý do và hai nguồn.
- **Q13–15.** Thay đổi ngoài rõ ràng có version mới và áp dụng cho phần chưa gửi. Lượt đã gửi giữ context đã dùng. Gom đề xuất theo nguyên ngữ tra cứu, nghĩa/thực thể và phạm vi. Duyệt đúng danh sách/version đã chọn. Chờ đồng bộ chưa kích hoạt. Cả nhóm chỉ active sau khi file và căn cứ được đối chiếu.
- **Q16.** Xưng hô có chiều, mốc truyện và vị trí độc giả biết là hai căn cứ riêng. Alias hoặc người nghe mơ hồ giữ chưa xác định. Chỉ chọn sự kiện được biết tại vị trí đang dịch.
- **Q03–04.** Đổi mapping không làm mất đơn vị còn phù hợp. Đổi snapshot chỉ tái dùng khi match duy nhất vị trí và kiểm đủ text/protected resources/dependencies. Giữ đơn vị nguyên hoặc đưa về xử lý lại khi bản Việt không tách an toàn.

## 6. ADR và điểm còn mở

Nhận writer, CAS, object verification và fail-closed từ M2 theo [ADR-002](../../decisions/ADR-002.md) và [ADR-004](../../decisions/ADR-004.md). Dùng [ADR-003](../../decisions/ADR-003.md) cho source identity/range và [ADR-008](../../decisions/ADR-008.md) cho ranh giới use case, lỗi và CLI. M3 mở rộng phần có consumer.

| ID | Lựa chọn cần ghi và kiểm | Owner / hạn |
|---|---|---|
| P03-D01 | DB-04. Chọn SQLite/file bất biến/hybrid cho nội dung revision. Chốt key/scope, migration, tham chiếu và state transitions. Kiểm round-trip và lịch sử. | T01 trước lưu revision. |
| P03-D02 | ADR-004 outbox. Chốt job ID, version/hash nền và đích, đối chiếu file khi dựng hoặc nhập lại. Chọn protocol bảo toàn file đã sửa và replay. | T02 trước dựng file, T03 trước đồng bộ hai chiều. |
| P03-D03 | DB-17. Chốt loại cạnh, ID hợp lệ và phạm vi, phiên bản căn cứ, occurrence lookup và điều kiện lan truyền. Kiểm edge sai, cycle và lượt chưa chứng minh cục bộ. | T04 trước schema/engine dependency. |
| P03-D04 | Chốt bộ đọc Markdown, phân vai theo đoạn và bản riêng v1 ở §7.4. Kiểm giới hạn bằng fixture thư viện lớn, scope/version và xác nhận lại hash. Mở TD-16 khi có consumer extension thật. | T05 trước template/import. |
| P03-D05 | Chốt ID/alias, story time, reader position, unknown, thứ tự ưu tiên và provenance. Kiểm hồ sơ đúng scope và conflict bắt buộc. | T06/T07 trước resolver. |
| P03-D06 | Chốt kho đề xuất chung, khóa gom, xác minh evidence, version khi sửa lựa chọn và replay occurrence. | T08 trước nhận bốn loại nguồn. |
| P03-D07 | DB-16. Chốt intent theo nhóm, base/target hash, cách giữ đúng bytes file bị thay trên Windows, đối chiếu/replay và activation CAS. Chốt trạng thái CLI phù hợp ADR-008. | T09 trước preflight/intent, T10 trước ghi file, T13 trước CLI result. |
| P03-D08 | Chốt reuse qua mapping/snapshot, nguyên đơn vị, xử lý target không tách được, bản ghi provenance và version kiểm hiệu lực. | T14 trước write path reuse. |

A ghi phương án, bằng chứng và giới hạn vào ADR tương ứng, theo [lịch DB/TD](../ROADMAP.md#decision-schedule) và [các mục ADR còn mở](../ROADMAP.md#adr-followups). Quy tắc giải quyết trạng thái/quyền sản phẩm theo đặc tả. Test fixture đối chiếu schema và IO trước khi thêm adapter mới. W7 event durable, điều phối và ngân sách giữ owner sau M3.

## 7. Thiết kế triển khai

### 7.1. Owner và dữ liệu

| Thành phần | Sở hữu |
|---|---|
| `nt-domain` | Revision/state, rule scope, cách ghép và kiểm range, điều kiện duyệt, quy tắc xưng hô/context thuần. |
| `nt-application` | Save/SyncDraft, đọc lịch sử, tạo/giải context, nhận đề xuất, preflight/confirm/sync Skill và xét lại hiệu lực. Định nghĩa port khi có caller. |
| `nt-storage` | Migration, row/file IO, revision store, index persistence, generation, job/intent và kiểm hash. Watcher là tín hiệu cho use case. |
| App và `nt-tui` | App nối adapter và CLI. TUI giữ draft/focus/lựa chọn, hiển thị trạng thái và so sánh. Cùng gọi application. |

Schema thêm từng cụm có consumer. Cụm revision giữ lineage, unit ranges, body/ref, author, events và current view. Cụm context giữ versions, provenance, cạnh phụ thuộc/ngược, occurrences và trạng thái cần kiểm. Cụm approval giữ candidate versions, danh sách đã chọn, intent/file fingerprints và kết quả activation. Dữ liệu read model nhất quán theo generation. Giữ chiều phụ thuộc M2.

### 7.2. Revision và bản thảo ngoài

1. Save nhận base revision/generation và thay đổi theo unit. Kiểm đúng source/version, author và transition. Hoàn tất object bất biến trước DB nếu D01 dùng file.
2. Writer công bố revision, current, history, dependency liên quan và job dựng Markdown trong một generation. ACK nói rõ revision đã lưu và tình trạng dựng file.
3. Job dùng base/target hash và marker để so file thật trước khi ghi. Nếu đã dựng đúng, hoàn tất job idempotent. Nếu file đã đổi, giữ bytes và chuyển qua đồng bộ. Parse/ghi file nằm ngoài transaction.
4. Đồng bộ đọc file ổn định, kiểm hash và so nền/current/bản người dùng. Nhập sửa xác định được thành revision sửa tay. Ghép phần tách biệt khi nền đúng. Marker mơ hồ hoặc sửa chồng có ca đối chiếu và lựa chọn rõ.

### 7.3. Duyệt và công bố Skill

1. Preflight danh sách ID/version/choice/scope đúng như người dùng chọn. Kiểm rule hiện hành, file nền, xung đột trong nhóm và dữ liệu bắt buộc.
2. Ghi intent bền vững cùng operation ID, base/target hashes và nhóm đã chốt. Khi đang chờ, các mục chưa active.
3. Dựng temp, kiểm nền, thay file theo cơ chế đã kiểm trên môi trường đích và giữ bản thực tế bị thay. So cả bản được giữ và file đích. Ca ghi ngoài chồng lấn giữ đủ ba phía.
4. Sau kiểm, revalidate version/scope và kích hoạt cả nhóm trong transaction. Ghi history và dependency ảnh hưởng. Nếu mất ACK/restart, đối chiếu intent/hash/version trước khi tiếp tục. Watcher nhận cùng operation để tránh kích hoạt lặp.

Context resolver đọc đúng phiên bản có hiệu lực và trả selection cùng provenance, vùng chưa rõ và danh sách cung cấp để lưu dependency. Kết quả dùng cho fixture M3 và Phase 04. Luồng tiếp tục tác vụ theo quyền chạy thuộc consumer sau.

<a id="p03-skill-format"></a>

### 7.4. Nạp file Skill và định dạng do app quản lý

#### Hai đường nhập

| Khi người dùng chọn | Đầu vào được giữ | Kết quả |
|---|---|---|
| **Thư viện chung** | Một file hoặc một thư mục Markdown do người dùng chọn. Giữ nguyên file, heading, bảng, danh sách và khối ví dụ. | Đăng ký file, lựa chọn bật/tắt, phạm vi áp dụng và phiên bản theo hash. Tạo chỉ mục theo đoạn để chọn khi cần. |
| **Skill riêng của bộ** | File Markdown hiện có hoặc khung trống. Người dùng chọn đúng bộ trước khi xem trước. | Chia nội dung thành **Đọc được**, **Cần làm rõ**, **Giữ làm ghi chú**. Sau xác nhận, tạo bản riêng theo mẫu sáu mục của app và giữ nguyên file người dùng đưa vào. |

Tên file `Ref_…` hay `Library_…` có thể giúp gợi ý khi xem trước. Vai trò, phạm vi và việc bật file lấy từ lựa chọn của người dùng. Tiêu đề `HOẠT ĐỘNG`, `Ưu tiên cao` hoặc `Master Prompt` trong nội dung chỉ là chữ của file. File không quyết định quyền duyệt, chạy tool, đọc đường dẫn khác hay thay hợp đồng lưu/đầu ra.

#### Cách đọc thư viện Markdown

Đọc UTF-8, hỗ trợ BOM đầu file và hai kiểu xuống dòng LF/CRLF. Tính hash trên bytes gốc. Đọc Markdown theo cấu trúc, nhận heading, đoạn văn, danh sách, bảng có header và khối code/quote. Giữ đoạn dẫn cần thiết cùng bảng hoặc ví dụ. Mỗi đoạn xem trước có tên file, heading và vị trí để mở lại. Link, HTML và khối code được đọc như nội dung. App không tự tải link hay chạy nội dung trong file.

| Dạng quan sát từ bộ mẫu người dùng | Xử lý trong app |
|---|---|
| File hướng dẫn định dạng, phiên âm nguyên âm dài, kính ngữ và văn phong có cả bảng lẫn ví dụ | Giữ nhóm điều kiện, quy tắc và ví dụ. Chỉ coi hướng dẫn có nhãn rõ là hướng dẫn chung trong scope đã chọn. Phần còn mơ hồ giữ tham khảo hoặc cần làm rõ. |
| Mẫu vàng có nguyên tác, bản Anh, bản Việt và giải thích | Lưu theo từng mẫu hoàn chỉnh, gắn vai trò/ngôn ngữ của từng phần. Chọn mẫu phù hợp ngôn ngữ yêu cầu khi dựng context. |
| Cặp bản dịch sai và bản sửa trong guardrails/critique | Giữ nhãn sai/đúng, hai vế và giải thích cùng một đơn vị. Nếu không tách chắc, đưa cả đoạn về xem lại. Không đưa vế sai ra như mẫu nên dùng. |
| Bảng Hán–Việt, primer xưng hô và kho Kanji theo heading từng chữ | Giữ bảng hoặc mục tra cứu. Phân biệt phương án tham khảo với lựa chọn chính thức của bộ. Lập chỉ mục cục bộ để truy mục cần dùng. |

Bộ file được khảo sát có 10 file Markdown. Kho Kanji có 12.559 heading mục, khoảng 2,4 MB. T05 dùng fixture tự sinh cùng dạng và cỡ để kiểm giới hạn, tra đầu/cuối và việc nạp file lớn. Khi dựng context, T07 chỉ lấy mục liên quan, giữ nhóm ví dụ trọn và trả phần bắt buộc/tùy chọn để Phase 04/05 áp giới hạn request. Chốt mức tài nguyên ở D04 bằng phép đo và ca vượt ngưỡng. File vượt mức có lỗi rõ, dữ liệu đã đăng ký trước đó vẫn đọc được.

#### Đề xuất mẫu v1 cho Skill riêng

Mẫu app tạo vẫn là Markdown dễ chỉnh. Dòng `skill-format` chỉ phiên bản cú pháp file. Mỗi mục có ID do app cấp, tiêu đề và các dòng trường theo nhãn. Scope trống hiển thị rõ **Toàn bộ bộ truyện**. Ví dụ tự soạn dưới đây dùng để kiểm format ở T05.

```markdown
<!-- skill-format: 1 -->
# Skill của bộ

## Văn phong

## Nhân vật và bí danh

## Thuật ngữ và danh hiệu

### Võ đường
- ID: term-001
- Nguyên ngữ: 道場
- Cách dịch: võ đường
- Nghĩa/định danh: nơi luyện võ
- Phạm vi:

## Địa danh

## Xưng hô

## Ngoại lệ
```

Mục xưng hô có người nói, người nghe, tự xưng, cách gọi và mốc hiệu lực. T06 chốt cách tham chiếu nhân vật, vị trí đọc và mốc truyện theo D05. Các mục có thể để trống. ID giữ khi đổi tiêu đề/thứ tự. Cùng ID với thay đổi rõ tạo version mới. Mục mới rõ nhận ID. ID trùng hoặc thiếu mà chưa rõ là mục mới hay sửa mục cũ thì hiển thị diff. Scope chưa xác định cần làm rõ. Version file theo hash, version quy tắc và phiên bản cú pháp là ba thông tin riêng. Người duyệt, lịch sử, scope đã resolve và provenance được lưu trong SQLite. Một dòng tự nhận đã duyệt trong Markdown không thay xác nhận.

#### Xem trước, xác nhận và cập nhật

1. Người dùng chọn đường nhập và scope. App đọc file được chọn, chụp hash và tạo preview. Mục **Đọc được** phải đủ trường và vị trí nguồn. Mục thiếu nghĩa, có nhiều cách dịch, không rõ scope hoặc xung đột nằm ở **Cần làm rõ**. Đoạn giải thích, mẫu và ghi chú giữ đúng vai trò.
2. Người dùng xem từng nhóm. Khi xác nhận các quy ước rõ, app đối chiếu lại hash đã preview. File đã đổi thì đọc lại và tạo preview mới. Hiển thị rõ scope đã chọn trước khi lưu bản theo mẫu vào đường dẫn riêng của workspace. Bản gốc và mục chưa chốt được bảo toàn.
3. Các file đã đăng ký tiếp tục theo Q13. Thay đổi rõ ràng và hợp lệ áp dụng theo 2B từ phần chưa gửi. Trường hợp thiếu ID, conflict hoặc file đang ghi giữ mốc hợp lệ và chẩn đoán. Quy trình ghi của tool dùng intent T09/T10.
4. M3 chứng minh đường đọc, preview và xác nhận qua use case với dữ liệu mẫu. Phase 06 nối chọn file/thư mục, bật/tắt, preview và xác nhận trong màn **Skills và ngữ cảnh**. Đường lệnh `terms list/confirm` giữ nghĩa duyệt đề xuất. [P06-T15](PHASE-06.md#p06-t15)

## 8. Dữ liệu, lỗi và phục hồi

| Tình huống | Kết quả cần giữ |
|---|---|
| Base revision/generation cũ | Từ chối ghi với lý do conflict, giữ draft. Đọc mốc hiện hành để so sánh. |
| DB đã lưu revision, dựng Markdown lỗi | Hiện **Đã lưu bản dịch, chờ dựng Markdown**. Giữ job và bản ngoài. Mở lại hoàn tất đúng job. |
| Marker thiếu/trùng hoặc gắn chưa rõ | Giữ nguyên byte sửa ngoài. Chỉ vùng mất liên kết và các đích ứng viên. Chỉ tạo revision sau xác nhận hợp lệ. |
| Sửa tay và ứng viên chồng nhau | Giữ nền, bản người dùng và ứng viên. Phần cùng đơn vị chờ người dùng chọn/kết hợp. |
| Skill đổi trong lúc ghi | Giữ file thực tế bị thay, đối chiếu base/user/target. Ghép phần riêng khi đủ căn cứ. Phần chồng giữ chờ xử lý. |
| Sau thay file nhưng chưa activation hoặc chưa ACK | Reopen tra operation/intent, kiểm hash và version. Kích hoạt cả nhóm đúng một lần hoặc báo conflict. |
| Rule hoặc nguồn đổi | Đánh dấu hiệu lực ở phạm vi phụ thuộc, giữ lịch sử chất lượng/revision. Chưa đủ căn cứ reuse thì báo vùng cần xử lý. |
| DB/file hỏng hoặc migration lỗi | Fail closed theo M2. Giữ mốc trước, backup và chẩn đoán đã lọc. |

Watcher không quyết định dữ liệu hợp lệ. Kiểm hash khi mở, trước ghi và khi dựng ngữ cảnh mới theo hợp đồng. Scope của lỗi/chẩn đoán đủ để biết đoạn, revision hoặc mục Skill cần xem. Bảo vệ key, đường dẫn cá nhân và nội dung ngoài fixture khi ghi log.

## 9. Phần TUI/CLI

| Phần | Hành vi cần thử | Task |
|---|---|---|
| UI-04 đọc nguồn và bản dịch | Hiện đúng đoạn Nhật–Việt, revision nền và ba trục trạng thái. Xem lịch sử giữ current. Giữ vị trí theo workspace/tập/document. | T11 |
| UI-04 sửa và lưu | `Enter` xuống dòng, `Ctrl+S` lưu theo base. Khi lỗi giữ draft, hiển thị ba phía và thao tác gắn lại vùng marker chưa rõ. | T12 |
| `terms list` | Liệt kê mục và phiên bản cần dùng để chọn. Đọc theo workspace hiện hành, trả kết quả phù hợp chế độ JSON. | T13 |
| `terms confirm --file <json>` | Parse ID/version/lựa chọn/phạm vi trước side effect. Gọi cùng preflight và publication. Trả đúng conflict, chờ sync hoặc đã kích hoạt. | T13 |

Màn rộng dùng hai vùng Nhật–Việt theo S-TUI, màn hẹp chuyển vùng giữ selection và draft. `Esc` quay lại theo focus và bảo toàn nội dung đang soạn. Khi yêu cầu lưu đang xử lý, event loop vẫn nhận phím và kết quả gắn đúng draft/base. Giao diện dùng ngôn ngữ dễ đọc. JSON dùng envelope của ADR-008 và trường của consumer D07. Chế độ không tương tác kết thúc hữu hạn và không tự duyệt thêm mục.

<a id="p03-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P03 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P03-T01 nhận P02-T09 đã Đạt và checkpoint P02-CP05 đã công bố. P03-T14 còn cần P02-T10. P03-CP04 bàn giao resolver cho P04-T01; approval nhận riêng ở P03-CP05 và Editor ở P03-CP02. P03-T15/M3 vẫn chờ M2 và toàn bộ phần việc P03. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p03-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p03-cp01"></a>P03-CP01 — Revision và outbox | [P03-T01](#p03-t01), [P03-T02](#p03-t02) | [P02-CP05](PHASE-02.md#p02-cp05) | P03-T01: V-01, V-02 trong phạm vi AC/TC của task; P03-T02: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp02"></a>P03-CP02 — Đồng bộ file và Editor | [P03-T03](#p03-t03), [P03-T11](#p03-t11), [P03-T12](#p03-t12) | [P03-CP01](#p03-cp01) | P03-T03: V-03 trong phạm vi AC/TC của task; P03-T11: V-07, V-11 trong phạm vi AC/TC của task; P03-T12: V-07, V-11 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-11 phần Editor/sync thuộc T11/T12; các thao tác CLI terms và reuse còn thuộc CP06/CP07, toàn M3 ở CP08. V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp03"></a>P03-CP03 — Dependency và nạp Skill | [P03-T04](#p03-t04), [P03-T05](#p03-t05) | [P03-CP01](#p03-cp01) | P03-T04: V-04 trong phạm vi AC/TC của task; P03-T05: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp04"></a>P03-CP04 — Xưng hô và resolver | [P03-T06](#p03-t06), [P03-T07](#p03-t07) | [P03-CP03](#p03-cp03) | P03-T06: V-05 trong phạm vi AC/TC của task; P03-T07: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp05"></a>P03-CP05 — Candidate, approval và công bố Skill | [P03-T08](#p03-t08), [P03-T09](#p03-t09), [P03-T10](#p03-t10) | [P03-CP03](#p03-cp03), [P03-CP04](#p03-cp04) | P03-T08: V-05 trong phạm vi AC/TC của task; P03-T09: V-06 trong phạm vi AC/TC của task; P03-T10: V-06, V-08 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp06"></a>P03-CP06 — CLI thuật ngữ | [P03-T13](#p03-t13) | [P03-CP05](#p03-cp05) | P03-T13: V-09 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp07"></a>P03-CP07 — Reuse theo nguồn và mapping | [P03-T14](#p03-t14) | [P02-CP06](PHASE-02.md#p02-cp06), [P03-CP01](#p03-cp01), [P03-CP02](#p03-cp02), [P03-CP03](#p03-cp03) | P03-T14: V-10 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |
| <a id="p03-cp08"></a>P03-CP08 — Nghiệm thu M3 | [P03-T15](#p03-t15) | M2 Đạt, [P03-CP01](#p03-cp01), [P03-CP02](#p03-cp02), [P03-CP03](#p03-cp03), [P03-CP04](#p03-cp04), [P03-CP05](#p03-cp05), [P03-CP06](#p03-cp06), [P03-CP07](#p03-cp07) | P03-T15: V-08, V-11, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M3; V-00 cho code, B review đúng candidate. | feature/context; A chat P03 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Nhận một thẻ cùng mục 5–8, AC ở mục 12 và ca kiểm ở mục 13. Chia thay đổi theo các bước trong thẻ. Chỉ dùng tên module/bảng theo trách nhiệm khi đã chốt consumer. Bàn giao với đúng phần đã kiểm và điều kiện còn thiếu.

<a id="p03-t01"></a>

### P03-T01 — Revision lineage và ba trục trạng thái

- **Đầu ra.** Revision bất biến, lịch sử và chuyển trạng thái có điều kiện.
- **Đầu vào.** Dependency thực thi: [P02-T09](PHASE-02.md#p02-t09). Căn cứ kỹ thuật: Q08/10, S-DATA, P03-D01/DB-04. Chốt cách lưu body và migration trước schema.
- **Vùng sửa.** Domain revision/state, application đọc/lưu và storage theo lựa chọn D01.

1. Xác định revision ID, parent/base, workspace/tập/snapshot, unit ranges, author và lý do. Chốt nội dung lưu trực tiếp hay tham chiếu object đã verify. Giữ metadata của nguồn M2.
2. Thêm migration cho dữ liệu cần ở lát revision. Kiểm quan hệ scope/version bằng ca sai và đọc lại bằng connection mới.
3. Định nghĩa sự kiện và điều kiện chuyển chất lượng, hiệu lực, thực thi. Giữ độc lập theo đúng đối tượng. Review được dùng trong fixture có ID/revision và căn cứ.
4. Lưu sửa tay thành revision mới chờ Review, giữ lineage và revision cũ. Dùng current của generation để đọc view. T02 bổ sung publish set và outbox.
5. Kiểm transition không đủ điều kiện và tổ hợp đã đạt trên r2, cần kiểm lại, tác vụ tạm dừng.

**Kiểm chứng.** P03-AC01, P03-TC01, V-01/V-02. B đối chiếu lịch sử trước/sau lỗi và quan sát từng trục.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T01 |
|---|---|
| Đầu vào bắt buộc | [P02-T09](PHASE-02.md#p02-t09). Nhận workspace/source map, writer/CAS và migration đã kiểm trong P02-T01–T09/P02-CP05; M2 còn chờ task tích hợp cuối. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Revision lineage/current/history và ba trục trạng thái dùng cùng source identity, transition lỗi giữ bản đã lưu. Owner đầu ra là A-P03-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T01 giữ Domain revision/state, application đọc/lưu và storage theo lựa chọn D01. Phần chung do T01 giữ DB-04/revision schema, storage M2 giữ đường commit. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P07-T02](PHASE-07.md#p07-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC01 / P03-TC01, gồm các ca chi tiết §13.3. V-01, V-02. Đối chiếu Lịch sử/current/state đúng oracle. Lỗi giữ nguyên bản đã lưu. |
| Điểm tích hợp | [P03-CP01](#p03-cp01) — Revision và outbox; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t02"></a>

### P03-T02 — Generation cho revision và Markdown outbox

- **Đầu ra.** Lưu revision và job dựng bản thảo trong một generation. Mở lại phân biệt đã lưu với chờ dựng file.
- **Đầu vào.** Dependency thực thi: [P03-T01](#p03-t01). Căn cứ kỹ thuật: T01, Q07–08, S-DATA, M2 writer/CAS, P03-D02 và ADR-004.
- **Vùng sửa.** Application save/projection, storage write set và IO file ngoài.

1. Chốt operation ID, expected base/generation và dữ liệu ghi cùng revision. Nếu D01 dùng object, hoàn tất và verify trước DB.
2. Ghi revision/history/current, dependencies đi kèm, state và job dựng Markdown trong transaction. ACK phân biệt revision đã commit và tiến độ file. Kiểm stale/replay theo M2.
3. Dựng Markdown theo đúng revision đã lưu, với marker từng đơn vị, base revision/hash và bytes đích ổn định. So hash file thật và giữ bản sửa ngoài theo D02 trước khi thay file.
4. Nếu IO lỗi, giữ job và báo đã lưu/chờ dựng. Khi reopen, xác minh file đích hoặc tiếp tục từ job. Chặn job cũ làm lùi file mới. T03 nhận trường hợp file có chỉnh sửa chưa nhập.
5. Chạy rollback và ngắt sau DB commit/trước file, sau file/trước xác nhận job. So số revision/current/job và hash thực.

**Kiểm chứng.** P03-AC02, P03-TC02, V-02. B kiểm mốc trước/sau bằng DB và file đọc lại.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T02 |
|---|---|
| Đầu vào bắt buộc | [P03-T01](#p03-t01). Nhận revision representation và writer/file-before-DB M2. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Save revision nhận base generation, ghi body/current/dependencies và job projection có thể replay. Owner đầu ra là A-P03-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T02 giữ Application save/projection, storage write set và IO file ngoài. Phần chung do T01 giữ revision, T02 giữ outbox/write set theo ADR-004. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P07-T02](PHASE-07.md#p07-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC02 / P03-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu Reopen đúng revision và chạy lại job không tạo revision trùng hoặc ghi lùi. |
| Điểm tích hợp | [P03-CP01](#p03-cp01) — Revision và outbox; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T05](PHASE-04.md#p04-t05), [P05-T01](PHASE-05.md#p05-t01), [P07-T04](PHASE-07.md#p07-t04). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t03"></a>

### P03-T03 — Đồng bộ Markdown và so sánh ba phía

- **Đầu ra.** Nhập bản sửa ngoài thành revision khi xác định được đơn vị, giữ đủ dữ liệu khi cần người dùng xử lý.
- **Đầu vào.** Dependency thực thi: [P03-T01](#p03-t01), [P03-T02](#p03-t02). Căn cứ kỹ thuật: T01–T02, Q08, S-DATA, D02. Marker/range nhận từ M2 và cách dựng T02.
- **Vùng sửa.** Use case sync, domain so/ghép đơn vị và adapter đọc file/kiểm hash.

1. Khi mở hoặc nhận tín hiệu file đổi, đọc bản hoàn chỉnh và kiểm lại hash/identity. Lấy base đã dựng, current và file người dùng. Chặn IO ngoài scope.
2. Kiểm marker, phạm vi và base. Thay đổi rõ gắn đúng unit được lưu là revision sửa tay chờ Review. Giữ source spans và author.
3. Ghép phần sửa khác đơn vị khi căn cứ khớp. Khi cùng đơn vị hoặc base không còn đối chiếu được, giữ cả ba phía và báo phạm vi. Preview không ghi.
4. Marker thiếu/trùng đưa vào trạng thái cần gắn lại và giữ byte gốc. Trả các vị trí/đích có thể gắn cho T12. Lựa chọn hợp lệ mới được tạo revision.
5. Đối chiếu lại file/generation trước commit, xử lý replay của lần sync hoặc outbox và giữ đủ bằng chứng sửa chồng.

**Kiểm chứng.** P03-AC03, P03-TC03, V-03. B thử sửa trong lúc đọc, conflict cùng đơn vị và marker mơ hồ bằng byte/hashes trước/sau.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T03 |
|---|---|
| Đầu vào bắt buộc | [P03-T01](#p03-t01), [P03-T02](#p03-t02). Nhận base revision, source markers và projection identity. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | So base/user/current theo đơn vị, trả merge hợp lệ hoặc conflict/marker cần người dùng gắn lại, bảo toàn bytes. Owner đầu ra là A-P03-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T03 giữ Use case sync, domain so/ghép đơn vị và adapter đọc file/kiểm hash. Phần chung do T02 giữ publication, T03 giữ sync/merge contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC03 / P03-TC03, gồm các ca chi tiết §13.3. V-03. Đối chiếu So bytes, history và source ranges. Trả đúng vùng cần người dùng xử lý. |
| Điểm tích hợp | [P03-CP02](#p03-cp02) — Đồng bộ file và Editor; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t04"></a>

### P03-T04 — Dependency graph và occurrence index

- **Đầu ra.** Ghi căn cứ và xác định chính xác tập cần kiểm lại, gồm khi thêm rule mới.
- **Đầu vào.** Dependency thực thi: [P03-T01](#p03-t01), [P03-T02](#p03-t02). Căn cứ kỹ thuật: T01–T02, Q09, S-DATA, D03/DB-17.
- **Vùng sửa.** Domain dependency, application xét hiệu lực và storage edge/index.

1. Chốt loại edge, ID, scope và version. Cho phép quan hệ giữa các tập đúng phạm vi trong cùng bộ. Từ chối đầu mút không có, sai loại hoặc sai bộ.
2. Ghi mọi context đã cung cấp, source ranges và cặp/mốc xưng hô. Trường model nói đã dùng chỉ là bằng chứng thêm. Ghi kết luận chương gắn đúng tập revision.
3. Tạo lookup ngược và index lần xuất hiện. Khi đổi một căn cứ, duyệt theo dependency, có visited set. Ghi lý do cho từng phạm vi bị ảnh hưởng.
4. Khi thêm rule mới, tra occurrences và lọc hiệu lực/scope. Nếu chưa chứng minh vùng hẹp, giữ phạm vi cả lượt được cung cấp context.
5. Cập nhật hiệu lực qua generation. Giữ revision/chất lượng và phần độc lập. Ghi đầu ra để Phase 06 nối truyền sự kiện và tiếp tục tác vụ.

**Kiểm chứng.** P03-AC04, P03-TC04, V-04. B so danh sách affected IDs với oracle có chuỗi nhiều cấp, cycle, rule mới và trường hợp không đủ bằng chứng thu hẹp.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T04 |
|---|---|
| Đầu vào bắt buộc | [P03-T01](#p03-t01), [P03-T02](#p03-t02). Nhận revision IDs, generation và scope của M2/M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Dependency edges và occurrence index trả affected IDs kèm căn cứ, chặn edge sai loại/bộ. Owner đầu ra là A-P03-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T04 giữ Domain dependency, application xét hiệu lực và storage edge/index. Phần chung do T01 giữ identity, T04 giữ DB-17/edge schema. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P07-T02](PHASE-07.md#p07-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC04 / P03-TC04, gồm các ca chi tiết §13.3. V-04. Đối chiếu Tập affected IDs đúng oracle và phần độc lập giữ nguyên. |
| Điểm tích hợp | [P03-CP03](#p03-cp03) — Dependency và nạp Skill; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T05](PHASE-04.md#p04-t05), [P05-T10](PHASE-05.md#p05-t10). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t05"></a>

### P03-T05 — Skill template, import và loại nội dung

- **Đầu ra.** Đăng ký thư viện Markdown chung và đọc theo đoạn. Preview và tạo Skill riêng v1 sáu mục theo xác nhận.
- **Đầu vào.** Dependency thực thi: [P03-T04](#p03-t04). Căn cứ kỹ thuật: T04, Q11–13, S-SKILL, D04. Chỉ mở TD-16 khi có consumer thật.
- **Vùng sửa.** Domain Skill/rule, application preview/import và storage file/version.

1. Chốt hai đường nạp, format v1 dự kiến và limits ở [§7.4](#p03-skill-format). Đọc UTF-8, giữ source file hash và vị trí. Xác định file do người dùng chọn và scope từ đầu vào, không suy quyền từ tên hoặc nội dung.
2. Đọc thư viện tự do theo heading/bảng/list/fence. Giữ cặp sai–đúng, nhóm ví dụ và nhãn ngôn ngữ. Tách hướng dẫn, ví dụ và tham khảo theo nhãn rõ. Dựng chỉ mục cho tra mục Kanji. Trả phiên bản và các đoạn cho T07.
3. Tạo khung v1 sáu mục và kiểm trường cho từng loại. Với Skill riêng có sẵn, preview ba nhóm, giữ bytes gốc và provenance từng mục. Phạm vi trống hiển thị toàn bộ bộ theo spec.
4. Xác nhận đúng nội dung đã preview và đối chiếu lại hash. Tạo bản theo mẫu ở đường dẫn riêng bằng luồng tạo mới có kiểm hash. Nếu đích đã có, giữ nguyên file và báo cần đường sync ở T10. Ghi người duyệt, scope và history khi công bố. Ví dụ/tham khảo giữ vai trò riêng.
5. Thử import lại, format chưa hỗ trợ, ID/phạm vi mơ hồ và file lớn. Bàn giao lệnh/bộ thử nạp, read model preview và trạng thái bật/tắt cho P06-T15.

**Kiểm chứng.** P03-AC05, P03-TC05, V-05. B kiểm preview/hash, file gốc, nội dung pha hướng dẫn/ví dụ, cặp sai–đúng và bộ tra cứu lớn tự sinh.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T05 |
|---|---|
| Đầu vào bắt buộc | [P03-T04](#p03-t04). Nhận dependency/version contract và hai đường nạp Skill ở §7.4. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Markdown chung giữ vai trò nội dung, Skill riêng trả preview theo D04 cùng source hash, confirm chỉ dùng preview còn khớp. Owner đầu ra là A-P03-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T05 giữ Domain Skill/rule, application preview/import và storage file/version. Phần chung do T04 giữ deps, T05 giữ Skill format/import, T10 nhận publication cập nhật file. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P07-T02](PHASE-07.md#p07-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC05 / P03-TC05, gồm các ca chi tiết §13.3. V-05. Đối chiếu Đúng phiên bản, vai trò, scope, đoạn được lấy và bytes gốc. |
| Điểm tích hợp | [P03-CP03](#p03-cp03) — Dependency và nạp Skill; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P06-T15](PHASE-06.md#p06-t15). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t06"></a>

### P03-T06 — Nhân vật, bí danh và xưng hô theo thời điểm

- **Đầu ra.** Entity và quy tắc xưng hô có chiều, hoàn cảnh và hai mốc đối chiếu.
- **Đầu vào.** Dependency thực thi: [P03-T04](#p03-t04), [P03-T05](#p03-t05). Căn cứ kỹ thuật: T04–T05, Q16, S-ADDRESS và D05.
- **Vùng sửa.** Domain identity/addressing và read model/căn cứ tối thiểu.

1. Lưu nhân vật theo ID. Alias đã xác nhận trỏ đúng entity. Alias mơ hồ giữ danh sách căn cứ và trạng thái chưa xác định.
2. Lưu người nói, người nghe, tự xưng/cách gọi, hoàn cảnh, đoạn áp dụng và phiên bản. Tách cặp A→B khỏi B→A và A→C.
3. So thời điểm trong truyện với khoảng hiệu lực. So vị trí đọc với bằng chứng độc giả được biết. Trong hồi tưởng dùng cả hai phép lọc.
4. Chọn ngoại lệ đúng cảnh khi đủ căn cứ. Hội thoại, lời kể và trích dẫn giữ thông tin để phân biệt. Thiếu người nghe hoặc mốc không suy từ bản dịch chưa duyệt.
5. Ghi kết quả và rule version cho T07/T04. Đổi một chiều hoặc mốc chỉ ảnh hưởng phạm vi có quan hệ.

**Kiểm chứng.** P03-AC06, P03-TC06, V-05. B kiểm cùng cặp ở hồi tưởng và hiện tại, đảo chiều và alias mơ hồ.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T06 |
|---|---|
| Đầu vào bắt buộc | [P03-T04](#p03-t04), [P03-T05](#p03-t05). Nhận Skill entries, identity và occurrence provenance. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Nhân vật/alias/xưng hô được chọn theo người nói/nghe và mốc truyện, mơ hồ trả lý do chờ. Owner đầu ra là A-P03-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T06 giữ Domain identity/addressing và read model/căn cứ tối thiểu. Phần chung do T05 giữ entry format, T06 giữ addressing contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P07-T02](PHASE-07.md#p07-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC06 / P03-TC06, gồm các ca chi tiết §13.3. V-05. Đối chiếu Chọn đúng rule ID/version hoặc lý do chưa xác định. |
| Điểm tích hợp | [P03-CP04](#p03-cp04) — Xưng hô và resolver; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t07"></a>

### P03-T07 — Context resolver và ưu tiên quy tắc

- **Đầu ra.** Context selection theo phạm vi với provenance, version và lý do chờ.
- **Đầu vào.** Dependency thực thi: [P03-T04](#p03-t04), [P03-T05](#p03-t05), [P03-T06](#p03-t06). Căn cứ kỹ thuật: T04–T06, Q09/12/16, S-SKILL/S-ADDRESS và D05.
- **Vùng sửa.** Domain/app resolver và hợp đồng read cho consumer Dịch–Review.

1. Nhận scope, source ranges, ngôn ngữ, loại nội dung, entity và hai mốc. Chọn file đang bật và dữ liệu đã đủ căn cứ.
2. Áp ưu tiên quy tắc riêng đã duyệt đúng phạm vi trước hướng dẫn chung. Lấy đúng mục tra cứu hoặc mẫu phù hợp từ chỉ mục T05. Giữ nhãn vai trò, ngôn ngữ và cặp sai–đúng. Chỉ đưa dữ kiện độc giả được biết tại vị trí tương ứng.
3. Nếu hai chỉ dẫn bắt buộc cùng hiệu lực mâu thuẫn, trả lý do và cả hai nguồn. Mục mơ hồ ảnh hưởng cách dịch đưa về làm rõ.
4. Trả context có version/provenance và toàn bộ item được cung cấp để lưu dependency. Tách phần bắt buộc và tùy chọn để planner sau kiểm dung lượng.
5. Kiểm kết quả ổn định khi thứ tự file thay đổi. Scope không phù hợp, disabled/candidate/tương lai không trở thành quyết định chính thức.

**Kiểm chứng.** P03-AC07, P03-TC07, V-05. B đổi thứ tự input và đối chiếu tập selected/blocked, lý do và hai nguồn xung đột.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T07 |
|---|---|
| Đầu vào bắt buộc | [P03-T04](#p03-t04), [P03-T05](#p03-t05), [P03-T06](#p03-t06). Nhận entry versions, dependency graph và addressing rules. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Resolver trả tập mục thực dùng/provenance, ưu tiên quy tắc đúng scope, giữ conflict và lọc fact vượt mốc. Owner đầu ra là A-P03-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T07 giữ Domain/app resolver và hợp đồng read cho consumer Dịch–Review. Phần chung do T04–T06 giữ đầu vào, T07 giữ context read contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P07-T02](PHASE-07.md#p07-t02). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC07 / P03-TC07, gồm các ca chi tiết §13.3. V-05. Đối chiếu Selection/provenance đúng oracle, conflict nêu hai nguồn. |
| Điểm tích hợp | [P03-CP04](#p03-cp04) — Xưng hô và resolver; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T01](PHASE-04.md#p04-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t08"></a>

### P03-T08 — Gom candidates và lần xuất hiện

- **Đầu ra.** Một danh sách đề xuất theo bộ, giữ vị trí, căn cứ và các lựa chọn.
- **Đầu vào.** Dependency thực thi: [P03-T05](#p03-t05), [P03-T06](#p03-t06), [P03-T07](#p03-t07). Căn cứ kỹ thuật: T05–T07, Q14, S-SKILL và D06.
- **Vùng sửa.** Domain candidate/occurrence, application nhận đề xuất và persistence có consumer.

1. Nhận dữ liệu qua cùng contract từ chuẩn bị context, Dịch, Review và Research. Phase 03 dùng fixture mô phỏng bốn nguồn.
2. Kiểm đoạn trích/range/revision và scope. Đề xuất lưu nguyên ngữ, vị trí, nghĩa/thực thể nếu xác định được, lựa chọn, lý do và phạm vi.
3. Gom theo nguyên ngữ tra, nghĩa/entity và phạm vi. Một nghĩa với 20 vị trí giữ một đề xuất có 20 occurrences. Nghĩa khác giữ riêng. Chưa rõ nghĩa giữ ví dụ để phân biệt.
4. Dedup replay bằng định danh nguồn/vị trí/căn cứ. Khi lựa chọn đổi, ghi version mới để T09 chặn xác nhận stale.
5. Trả danh sách chờ và phần bị ảnh hưởng. Giữ lựa chọn thử trong đề xuất tới khi qua duyệt và công bố.

**Kiểm chứng.** P03-AC08, P03-TC08, V-05. B kiểm cardinality, quote sai và replay độc lập. Context đang dùng không đổi khi chỉ thêm candidate.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T08 |
|---|---|
| Đầu vào bắt buộc | [P03-T05](#p03-t05), [P03-T06](#p03-t06), [P03-T07](#p03-t07). Nhận resolver và Skill/occurrence IDs. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Gom đề xuất có nghĩa/phạm vi/evidence, dedup theo định danh và replay, active set chỉ đổi sau approval. Owner đầu ra là A-P03-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T08 giữ Domain candidate/occurrence, application nhận đề xuất và persistence có consumer. Phần chung do T05/T07 giữ entries/resolver, T08 giữ candidate schema. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC08 / P03-TC08, gồm các ca chi tiết §13.3. V-05. Đối chiếu Đúng số đề xuất/vị trí, giữ provenance, replay không nhân bản. |
| Điểm tích hợp | [P03-CP05](#p03-cp05) — Candidate, approval và công bố Skill; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T04](PHASE-04.md#p04-t04). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t09"></a>

### P03-T09 — Approval batch theo danh sách/version

- **Đầu ra.** Kiểm cả nhóm và lưu intent đúng danh sách/version người dùng đã chọn.
- **Đầu vào.** Dependency thực thi: [P03-T04](#p03-t04), [P03-T05](#p03-t05), [P03-T06](#p03-t06), [P03-T07](#p03-t07), [P03-T08](#p03-t08). Căn cứ kỹ thuật: T04–T08, Q15, S-SKILL, P03-D07/DB-16.
- **Vùng sửa.** Domain approval, application preflight/confirm và storage intent.

1. Nhận ID, version, lựa chọn và scope cho từng mục. Từ chối trùng/không tồn tại/sai bộ/stale. Giữ danh sách nguyên vẹn để người dùng sửa.
2. Kiểm xung đột trong nhóm, với rule đang có hiệu lực và với file Skill nền đã kiểm hash. Trả lỗi đúng mục và nguồn, gồm hai nguồn mâu thuẫn.
3. Nếu bất kỳ mục nào lỗi, chưa thay active rules. Cho nhận lại nhóm đã sửa, ví dụ bỏ mục lỗi để chốt 9 trong 10 mục.
4. Khi preflight đủ và có xác nhận, ghi một intent bền vững cho đúng nhóm, base/target hash, versions và operation ID. Kết quả ở trạng thái chờ sync. T10 thực hiện file và activation.
5. Thử replay cùng yêu cầu và ID bị dùng cho payload khác. Giữ trạng thái và lựa chọn nếu có xung đột trước khi công bố.

**Kiểm chứng.** P03-AC09, P03-TC09, V-06. B kiểm danh sách thực trong DB trước/sau một mục stale, nhóm đã sửa và replay.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T09 |
|---|---|
| Đầu vào bắt buộc | [P03-T04](#p03-t04), [P03-T05](#p03-t05), [P03-T06](#p03-t06), [P03-T07](#p03-t07), [P03-T08](#p03-t08). Nhận candidates cùng version, scope và điều kiện xung đột. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Confirm nhận danh sách ID/version/phương án, ghi cả nhóm hoặc trả conflict, active set chờ publication. Owner đầu ra là A-P03-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T09 giữ Domain approval, application preflight/confirm và storage intent. Phần chung do T04 giữ deps, T09 giữ DB-16 intent, T10 giữ activation. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC09 / P03-TC09, gồm các ca chi tiết §13.3. V-06. Đối chiếu Intent đúng nhóm đã chốt. Active set giữ nguyên trước publication. |
| Điểm tích hợp | [P03-CP05](#p03-cp05) — Candidate, approval và công bố Skill; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T04](PHASE-04.md#p04-t04), [P06-T15](PHASE-06.md#p06-t15). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t10"></a>

### P03-T10 — Skill publish và đồng bộ file ngoài có recovery

- **Đầu ra.** File Skill và quyết định được công bố theo intent có thể đối chiếu, giữ bản sửa ngoài và kích hoạt đúng một lần.
- **Đầu vào.** Dependency thực thi: [P03-T09](#p03-t09). Căn cứ kỹ thuật: T09, Q13/15, S-SKILL, DB-16, D07 và CAS của M2.
- **Vùng sửa.** Application publish/reconcile, storage file adapter và watcher. Ghi căn cứ/hiệu lực qua T04.

1. Nhận intent và dựng temp theo đích đã chốt. Verify hash và nền thực tế. Thay file bằng cơ chế đã thử trên Windows, giữ đúng bản bị thay tại thời điểm thay.
2. Đọc lại file đích và bản được giữ. So base/target hash và versions. Nếu file ngoài đổi, giữ cả ba phía. Ghép thay đổi ở các mục riêng khi đủ căn cứ, đưa phần chồng về chờ xử lý.
3. Sau verify, trong một transaction revalidate intent/generation/rule scope và kích hoạt cả nhóm cùng lịch sử, vùng cần kiểm lại và trạng thái. Khi IO lỗi, giữ **Đã nhận xác nhận, chờ đồng bộ Skill**.
4. Watcher nhận diện operation đang chờ và báo cho cùng use case. Reopen đối chiếu trước khi hoàn tất. Replay trả kết quả đã lưu hoặc conflict, không kích hoạt thêm lần.
5. Với chỉnh sửa rõ ràng ngoài tool theo Q13, kiểm file ổn định, ID, scope và loại đã đăng ký. Thư viện chung vẫn là hướng dẫn/ví dụ/tra cứu. Cùng ID tạo version mới, mục mới nhận ID mới. ID mất/mơ hồ hiển thị diff. Phần đã gửi sau này giữ đúng context version, phần phụ thuộc chưa gửi đợi dữ liệu hợp lệ.
6. Thử race writer ngoài, lỗi ghi và ngắt sau thay file/trước activation, sau activation/trước ACK. So bytes file, backup, intent và active set.

**Kiểm chứng.** P03-AC10, P03-TC10, V-06/V-08. B ưu tiên ca file thay sau lần check nền và ca mở lại khi file đích đúng nhưng DB chưa active.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T10 |
|---|---|
| Đầu vào bắt buộc | [P03-T09](#p03-t09). Nhận approval intent và expected file hashes, CAS M2. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Publish/reconcile gắn decision với bản file đã xác minh, chỉ active khi đồng bộ và replay giữ history. Owner đầu ra là A-P03-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T10 giữ Application publish/reconcile, storage file adapter và watcher. Ghi căn cứ/hiệu lực qua T04. Phần chung do T09 giữ intent, T10 giữ file activation/watcher, T04 giữ hiệu lực. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC10 / P03-TC10, gồm các ca chi tiết §13.3. V-06, V-08. Đối chiếu So bytes file thực tế, backup, DB/history và kết quả replay. |
| Điểm tích hợp | [P03-CP05](#p03-cp05) — Candidate, approval và công bố Skill; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T04](PHASE-04.md#p04-t04), [P05-T10](PHASE-05.md#p05-t10), [P06-T01](PHASE-06.md#p06-t01), [P06-T12](PHASE-06.md#p06-t12), [P06-T15](PHASE-06.md#p06-t15). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t11"></a>

### P03-T11 — UI-04 đọc nguồn, nháp và lịch sử

- **Đầu ra.** Editor mở đúng nguồn, bản nháp và revision lịch sử, với trạng thái rõ.
- **Đầu vào.** Dependency thực thi: [P03-T01](#p03-t01), [P03-T02](#p03-t02), [P03-T03](#p03-t03). Căn cứ kỹ thuật: T01–T03, M2, S-TUI/D-UI và event loop M1.
- **Vùng sửa.** TUI Editor read state, app wiring và read model có consumer.

1. Đọc document/range theo workspace, tập và source snapshot. Ghép nguyên tác, ruby và bản Việt với đúng revision. Hiển thị ba trục theo read model.
2. Thêm xem lịch sử và base/author khi có. View lịch sử là chỉ đọc và giữ current. Khi phần nguồn không còn đối chiếu, hiển thị phạm vi chưa rõ.
3. Giữ selection, cuộn, focus theo workspace/tập/document. Chuyển giữa hai tập hoặc resize rồi trở lại đúng chỗ. Render dùng dữ liệu đã tải.
4. Thử bản mẫu có hai tập cùng tên chương và nhiều revision. Kiểm route, ID và text theo oracle trước/sau chuyển màn.

**Kiểm chứng.** P03-AC11, P03-TC11, V-07 và phần đọc của V-11. B kiểm gắn đúng nguồn/bản Việt và history view không tạo generation.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T11 |
|---|---|
| Đầu vào bắt buộc | [P03-T01](#p03-t01), [P03-T02](#p03-t02), [P03-T03](#p03-t03). Nhận source/current/history read model và sync states. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Editor read theo workspace/tập/tài liệu, xem history giữ current, giữ focus khi resize. Owner đầu ra là A-P03-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T11 giữ TUI Editor read state, app wiring và read model có consumer. Phần chung do T01–T03 giữ read/save, người tích hợp M3 giữ router/composition. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC11 / P03-TC11, gồm các ca chi tiết §13.3. V-07, V-11. Đối chiếu Đúng IDs/text/focus. View chỉ đọc giữ generation. |
| Điểm tích hợp | [P03-CP02](#p03-cp02) — Đồng bộ file và Editor; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t12"></a>

### P03-T12 — UI-04 sửa, lưu và xử lý conflict

- **Đầu ra.** Editor lưu qua core, giữ draft và hỗ trợ gắn lại vùng mất liên kết theo lựa chọn người dùng.
- **Đầu vào.** Dependency thực thi: [P03-T03](#p03-t03), [P03-T11](#p03-t11). Căn cứ kỹ thuật: T03, T11, Q08, S-DATA/S-TUI. Giữ phần P00-T01-r02 đã bổ sung.
- **Vùng sửa.** TUI editor input/draft/conflict, application SaveRevision và gắn lại unit.

1. Gắn draft với workspace/tập/document/base revision. `Enter` xuống dòng, `Ctrl+S` gửi save. Giữ draft khi resize, quay màn hoặc IO lỗi.
2. Hiện lưu thành công theo generation mới và trạng thái dựng Markdown riêng. Nếu stale hoặc sửa chồng, đọc bản nền/người dùng/ứng viên, giữ draft để chọn hoặc kết hợp.
3. Với marker thiếu hoặc trùng, hiển thị đúng vùng mất liên kết và **các đích ứng viên**, gồm trường hợp nhiều đích. Preview từng đích/source unit. Giữ byte của file sửa tay và draft cho đến khi người dùng xác nhận.
4. Kiểm đích còn hợp lệ trong đúng snapshot, range và base. Nếu chọn sai, hủy hoặc có cập nhật cạnh tranh thì giữ nguyên. Xác nhận hợp lệ tạo revision sửa tay mới **chờ Review**, gắn đúng source unit và giữ lịch sử.
5. Chạy ca Nhật–Việt nhiều dòng, hai bản sửa, projection pending và mở lại. So nội dung từng bên và current trước/sau save.

**Kiểm chứng.** P03-AC12, P03-TC12, V-07/V-11. B kiểm cả ca thiếu/trùng marker với nhiều lựa chọn, byte trước xác nhận, ID được gắn và revision chờ Review.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T12 |
|---|---|
| Đầu vào bắt buộc | [P03-T03](#p03-t03), [P03-T11](#p03-t11). Nhận Editor target/base và sync/conflict use case. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Lưu bản sửa tay theo expected revision, gắn lại marker cần xác nhận người dùng, giữ draft khi conflict. Owner đầu ra là A-P03-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T12 giữ TUI editor input/draft/conflict, application SaveRevision và gắn lại unit. Phần chung do T03 giữ merge/save, T11 giữ route state, A-T12 giữ input/draft. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC12 / P03-TC12, gồm các ca chi tiết §13.3. V-07, V-11. Đối chiếu Giữ draft và file ở ca chưa xác nhận. Đối chiếu source unit/revision sau xác nhận. |
| Điểm tích hợp | [P03-CP02](#p03-cp02) — Đồng bộ file và Editor; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P04-T12](PHASE-04.md#p04-t12), [P08-T04](PHASE-08.md#p08-t04). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t13"></a>

### P03-T13 — CLI terms list/confirm

- **Đầu ra.** Script xem và xác nhận đúng nhóm, cùng điều kiện và trạng thái với core.
- **Đầu vào.** Dependency thực thi: [P03-T09](#p03-t09), [P03-T10](#p03-t10). Căn cứ kỹ thuật: T09–T10, Q15/34, S-TUI, ADR-008 và D07.
- **Vùng sửa.** App terms commands, kết quả CLI/JSON và kiểm tiến trình.

1. Nối `terms list` với danh sách/version. Nối `terms confirm --file <json>` qua preflight và publication. File chọn đúng ID/version/choice/scope.
2. Parse cờ và dữ liệu trước ghi. Từ chối file sai, ID lạ/trùng và version stale. Dùng đúng workspace, hạn chế chẩn đoán theo hợp đồng redaction.
3. Chốt trường result/issue/next_action và exit theo ADR-008 cho đã lưu, xung đột, chờ đồng bộ và lỗi vận hành. JSON một envelope. `task_id` chỉ có khi thật.
4. Chạy non-interactive với file hợp lệ và các file lỗi. Kiểm nhóm pending/active đúng với DB/file và mục mới đến sau lần list không bị xác nhận.
5. So lần đọc lại qua CLI sau reopen. Thử thiếu tham số và lỗi file publication, giữ lựa chọn và chỉ hành động tiếp theo.

**Kiểm chứng.** P03-AC13, P03-TC13, V-09. B kiểm exit/output thực, đối chiếu đúng tập ID đã chọn và số lần kích hoạt.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T13 |
|---|---|
| Đầu vào bắt buộc | [P03-T09](#p03-t09), [P03-T10](#p03-t10). Nhận atomic confirm/result và publication pending. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | terms list/confirm dùng đúng core batch với ID/version, JSON/exit khớp trạng thái DB/file. Owner đầu ra là A-P03-T13. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T13 giữ App terms commands, kết quả CLI/JSON và kiểm tiến trình. Phần chung do T09/T10 giữ nghiệp vụ, người tích hợp giữ CLI registration và output. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P02-T10](PHASE-02.md#p02-t10), [P02-T12](PHASE-02.md#p02-t12), [P04-T01](PHASE-04.md#p04-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC13 / P03-TC13, gồm các ca chi tiết §13.3. V-09. Đối chiếu Output process khớp DB/file. Không tự duyệt mục mới. |
| Điểm tích hợp | [P03-CP06](#p03-cp06) — CLI thuật ngữ; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t14"></a>

### P03-T14 — Tái sử dụng đơn vị khi mapping hoặc nguồn đổi

- **Đầu ra.** Tập đơn vị còn đủ căn cứ để dùng lại và phần phải đối chiếu, có provenance.
- **Đầu vào.** Dependency thực thi: [P03-T01](#p03-t01), [P03-T02](#p03-t02), [P03-T03](#p03-t03), [P03-T04](#p03-t04), [P02-T10](PHASE-02.md#p02-t10). Căn cứ kỹ thuật: T01–T04, M2 mapping/snapshot, Q03–04/09, S-INPUT/S-DATA, ADR-003 và D08.
- **Vùng sửa.** Domain kiểm source ranges, application reuse/rebase và persistence quan hệ.

1. Đổi mapping trong cùng nguồn thì đối chiếu BlockID/range, bảo toàn đơn vị nguyên còn hợp lệ. Xét lại ngữ cảnh ở điểm nối và kết luận chương phụ thuộc.
2. Đổi snapshot thì tìm đúng một ứng viên theo canonical path + locator. Kiểm nội dung, ruby/inline/ảnh được bảo vệ, parser/range và dependencies. Khác căn cứ hoặc nhiều ứng viên thì trả lý do chưa tái dùng.
3. Với chia lại range, chỉ giữ phần đối chiếu được trên bản Việt. Nếu đảo trật tự từ làm không thể tách, giữ nguyên đơn vị cũ khi còn hợp lệ hoặc đưa đúng vùng về xử lý lại.
4. Lưu provenance source/revision cũ và mốc mới, cùng vùng ảnh hưởng và hiệu lực. Giữ nguyên lịch sử revision gốc.
5. Kiểm trên snapshot/mapping thật của M2 với bảng expected reuse, mismatch và ambiguity. Bàn giao vùng cần lập lượt lại cho Phase 04/06.

**Kiểm chứng.** P03-AC14, P03-TC14, V-10. B thay riêng từng căn cứ và kiểm trường hợp cùng chữ khác vị trí hoặc bản Việt không thể cắt an toàn.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T14 |
|---|---|
| Đầu vào bắt buộc | [P03-T01](#p03-t01), [P03-T02](#p03-t02), [P03-T03](#p03-t03), [P03-T04](#p03-t04), [P02-T10](PHASE-02.md#p02-t10). Nhận snapshot/mapping M2, revisions và dependency IDs. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Rebase/reuse đối chiếu ranges, source hashes và căn cứ, trả tập dùng lại/kiểm lại cùng provenance. Owner đầu ra là A-P03-T14. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T14 giữ Domain kiểm source ranges, application reuse/rebase và persistence quan hệ. Phần chung do T04 giữ affected set, T01/T02 giữ revision writes, T14 giữ reuse policy. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T01](PHASE-04.md#p04-t01), [P04-T02](PHASE-04.md#p04-t02), [P04-T03](PHASE-04.md#p04-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC14 / P03-TC14, gồm các ca chi tiết §13.3. V-10. Đối chiếu Tập reuse/non-reuse đúng oracle và giữ provenance. |
| Điểm tích hợp | [P03-CP07](#p03-cp07) — Reuse theo nguồn và mapping; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p03-t15"></a>

### P03-T15 — Recovery và bàn giao M3

- **Đầu ra.** Bản tích hợp M3 có bằng chứng nâng cấp, lưu/sync, recovery và bài người dùng tự test.
- **Đầu vào.** Dependency thực thi: [P03-T01](#p03-t01), [P03-T02](#p03-t02), [P03-T03](#p03-t03), [P03-T04](#p03-t04), [P03-T05](#p03-t05), [P03-T06](#p03-t06), [P03-T07](#p03-t07), [P03-T08](#p03-t08), [P03-T09](#p03-t09), [P03-T10](#p03-t10), [P03-T11](#p03-t11), [P03-T12](#p03-t12), [P03-T13](#p03-t13), [P03-T14](#p03-t14), M2 Đạt. Căn cứ kỹ thuật: T01–T14, M2, Q07–16/36, R-REPO.
- **Vùng sửa.** Recovery/integration tests, fixture, README, evidence và tài liệu bàn giao.

1. Mở bản sao workspace do M2 tạo, sao lưu theo ADR-004, chạy migration M3 và kiểm source/mapping cũ, version/current và refs. Tiêm lỗi migration/staging và kiểm bản trước.
2. Chạy chuỗi sửa tay → dựng/nhập Markdown → xác nhận Skill → xét lại hiệu lực → reopen. So base/history, current, số lần approval và hashes. Dùng child/marker ở ranh giới DB/file/activation/ACK.
3. Chạy V-11 trên terminal thật và CLI. A điền bản chạy/lệnh/fixture vào hướng dẫn mục 16.4. B tự thử lại các ca chính và kiểm mẫu lỗi.
4. Chạy V-00 và V-12 khi đổi dependency. Rà đường Go liên quan theo mục 3.2. Chỉ đề xuất việc gỡ có bằng chứng thay thế, đã kiểm caller và đường build còn dùng.
5. Hoàn thiện README theo hành vi thực. Tạo candidate/handoff M3, B review bản tích hợp và tiêu chí mục 15.

**Kiểm chứng.** P03-AC15, P03-TC15, V-08/V-11/V-00. B so full state và byte file qua recovery. Ghi rõ ca chưa chạy và phần Go còn cần consumer.

**Sáu trường giao việc**

| Trường | Nội dung cho P03-T15 |
|---|---|
| Đầu vào bắt buộc | [P03-T01](#p03-t01), [P03-T02](#p03-t02), [P03-T03](#p03-t03), [P03-T04](#p03-t04), [P03-T05](#p03-t05), [P03-T06](#p03-t06), [P03-T07](#p03-t07), [P03-T08](#p03-t08), [P03-T09](#p03-t09), [P03-T10](#p03-t10), [P03-T11](#p03-t11), [P03-T12](#p03-t12), [P03-T13](#p03-t13), [P03-T14](#p03-t14), M2 Đạt. Nhận candidate các nhánh Editor, context, approval và reuse cùng schema M2→M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Hồ sơ M3 chứng minh DB/file/Skill versions nhất quán qua restart và thao tác người dùng. Owner đầu ra là A-P03-T15. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P03-T15 giữ Recovery/integration tests, fixture, README, evidence và tài liệu bàn giao. Phần chung do owners T01–T14 giữ lỗi theo module, A-T15 nhận bản tích hợp. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T01](PHASE-04.md#p04-t01), [P04-T02](PHASE-04.md#p04-t02), [P04-T03](PHASE-04.md#p04-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P03-AC15 / P03-TC15, gồm các ca chi tiết §13.3. V-08, V-11, V-00. Đối chiếu Kiểm integrity/version/refs, file hashes và UI/CLI thực. |
| Điểm tích hợp | [P03-CP08](#p03-cp08) — Nghiệm thu M3; A chat P03 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P03-AC01 | Sửa hợp lệ tạo ID mới, đọc được lineage và revision cũ. Chất lượng, hiệu lực và thực thi giữ độc lập. | P03-T01 | P03-TC01 · V-01, V-02 |
| P03-AC02 | Revision/current/lịch sử/job cùng công bố. Lỗi dựng file sau DB commit vẫn báo revision đã lưu và chờ dựng. | P03-T02 | P03-TC02 · V-02 |
| P03-AC03 | Chỉ thay đổi xác định được mới tạo revision sửa tay chờ Review. File và bản đang sửa còn nguyên khi chưa giải quyết. | P03-T03 | P03-TC03 · V-03 |
| P03-AC04 | Đánh dấu đúng đơn vị phụ thuộc, gồm rule mới. Chưa chứng minh ảnh hưởng hẹp thì giữ cả lượt. | P03-T04 | P03-TC04 · V-04 |
| P03-AC05 | Thư viện chung giữ Markdown, vai trò và tra đúng phần. Skill riêng có preview ba nhóm và mẫu sáu mục. Xác nhận nền tạo bản riêng có lịch sử. File gốc nguyên vẹn. | P03-T05 | P03-TC05 · V-05 |
| P03-AC06 | Quy tắc xưng hô đúng chiều, story time và reader position. Căn cứ thiếu giữ chưa xác định. | P03-T06 | P03-TC06 · V-05 |
| P03-AC07 | Ngữ cảnh đúng scope, thứ tự ưu tiên và version. Quy tắc chưa duyệt hoặc dữ kiện tương lai không thành hiện hành. | P03-T07 | P03-TC07 · V-05 |
| P03-AC08 | Hai mươi vị trí cùng nghĩa gom một đề xuất. Nghĩa khác giữ riêng. Chưa duyệt vẫn là đề xuất. | P03-T08 | P03-TC08 · V-05 |
| P03-AC09 | Chỉ đúng danh sách/version người dùng chọn được xác nhận. Nhóm lỗi giữ lựa chọn và không kích hoạt một phần. | P03-T09 | P03-TC09 · V-06 |
| P03-AC10 | Chờ đồng bộ tách khỏi active. File ngoài được bảo toàn và cả nhóm chỉ kích hoạt một lần sau đối chiếu. | P03-T10 | P03-TC10 · V-06, V-08 |
| P03-AC11 | Editor mở đúng nguồn/bản Việt/revision. Xem lịch sử giữ nguyên current và quay về đúng vị trí. | P03-T11 | P03-TC11 · V-07, V-11 |
| P03-AC12 | Save hợp lệ tạo revision chờ Review. Thiếu/trùng marker có các đích ứng viên để gắn lại. Giữ byte sửa tay và draft trước khi xác nhận. Gắn đúng unit tạo revision chờ Review. | P03-T12 | P03-TC12 · V-07, V-11 |
| P03-AC13 | CLI áp cùng preflight/Skill sync. Exit/state đúng nguyên nhân và chỉ nhóm đã chọn thay đổi. | P03-T13 | P03-TC13 · V-09 |
| P03-AC14 | Chỉ reuse đơn vị khớp chắc chắn, còn hiệu lực. Giữ revision gốc. Phần mơ hồ hoặc không tách an toàn được chỉ rõ. | P03-T14 | P03-TC14 · V-10 |
| P03-AC15 | Migration, reopen và replay giữ source, history, sửa tay và active set đúng một lần. Ca hỏng giữ mốc hợp lệ hoặc chặn ghi. | P03-T15 | P03-TC15 · V-08, V-11, V-00 |

A nối từng AC với test hoặc ca thủ công thực chạy. B kiểm oracle và ca âm có khả năng phát hiện lỗi tương ứng.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P03-TC01 / P03-AC01 | Chuỗi r1/r2, cùng revision ở ba trục, transition sai | Lịch sử/current/state đúng oracle. Lỗi giữ nguyên bản đã lưu. | V-01, V-02 |
| P03-TC02 / P03-AC02 | Race, failpoint DB/file, lỗi IO | Reopen đúng revision và chạy lại job không tạo revision trùng hoặc ghi lùi. | V-02 |
| P03-TC03 / P03-AC03 | Base/user/candidate, marker và sửa cùng/khác đơn vị | So bytes, history và source ranges. Trả đúng vùng cần người dùng xử lý. | V-03 |
| P03-TC04 / P03-AC04 | Nhiều cấp, cycle, rule mới, dữ liệu khác scope | Tập affected IDs đúng oracle và phần độc lập giữ nguyên. | V-04 |
| P03-TC05 / P03-AC05 | Skill chuẩn/tự do, bảng và ví dụ sai–đúng, kho tra cứu lớn tự sinh, mục mơ hồ | Đúng phiên bản, vai trò, scope, đoạn được lấy và bytes gốc. | V-05 |
| P03-TC06 / P03-AC06 | Cặp nhân vật, hồi tưởng, ngoại lệ và alias mơ hồ | Chọn đúng rule ID/version hoặc lý do chưa xác định. | V-05 |
| P03-TC07 / P03-AC07 | Rules khác nguồn, scope, mốc và chỉ dẫn bắt buộc xung đột | Selection/provenance đúng oracle, conflict nêu hai nguồn. | V-05 |
| P03-TC08 / P03-AC08 | Bốn nguồn, 20 occurrences, hai nghĩa và evidence sai | Đúng số đề xuất/vị trí, giữ provenance, replay không nhân bản. | V-05 |
| P03-TC09 / P03-AC09 | Nhóm 10 có một stale, sửa lại 9, conflict và replay | Intent đúng nhóm đã chốt. Active set giữ nguyên trước publication. | V-06 |
| P03-TC10 / P03-AC10 | Race file, hash sai và failpoints trước/sau activation | So bytes file thực tế, backup, DB/history và kết quả replay. | V-06, V-08 |
| P03-TC11 / P03-AC11 | Hai tập cùng tên chương, nhiều revision, ruby và resize | Đúng IDs/text/focus. View chỉ đọc giữ generation. | V-07, V-11 |
| P03-TC12 / P03-AC12 | Text nhiều dòng, competing update, nhiều đích cho marker thiếu/trùng | Giữ draft và file ở ca chưa xác nhận. Đối chiếu source unit/revision sau xác nhận. | V-07, V-11 |
| P03-TC13 / P03-AC13 | File xác nhận hợp lệ, lỗi cú pháp, stale và publication pending | Output process khớp DB/file. Không tự duyệt mục mới. | V-09 |
| P03-TC14 / P03-AC14 | Hai snapshots, nội dung/ảnh/rule đổi, range chia lại, đảo từ Việt | Tập reuse/non-reuse đúng oracle và giữ provenance. | V-10 |
| P03-TC15 / P03-AC15 | Workspace M2, chuỗi revision/Skill, failpoints và demo | Kiểm integrity/version/refs, file hashes và UI/CLI thực. | V-08, V-11, V-00 |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo ở task tương ứng. T01 tạo phần lưu/đọc của V-02 và T02 thêm ca commit/outbox. Ghi test được phát hiện và ca thực chạy. Exit 0 với 0 ca liên quan chưa chứng minh AC.

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | `cargo test -p nt-domain --test revision_state --locked` |
| V-02 | `cargo test -p nt-storage --test revision_commit --locked` |
| V-03 | `cargo test -p nt-storage --test markdown_sync --locked` |
| V-04 | `cargo test -p nt-application --test dependency_invalidation --locked` |
| V-05 | `cargo test -p nt-application --test context_rules --locked` |
| V-06 | `cargo test -p nt-storage --test skill_publication --locked` |
| V-07 | `cargo test -p nt-tui --test editor_revision --locked` |
| V-08 | `cargo test -p novel-translate --test phase03_recovery --locked` |
| V-09 | `cargo test -p novel-translate --test terms_contract --locked` |
| V-10 | `cargo test -p nt-application --test source_reuse_contract --locked` |
| V-11 | Kiểm Editor bằng bàn phím trên terminal thật ở T11/T12. M3 chạy trọn [bài tự test](#p03-user-test) với bộ mẫu, gồm CLI terms. |
| V-12 | Khi thêm/đổi dependency, kiểm MSRV theo ADR-001. Dùng `cargo +1.88.0 test --workspace --all-features --locked` khi mốc MSRV vẫn là 1.88.0. |

**V-00 — Bộ kiểm tra Rust trước PR và trước đóng M3.**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

Chuẩn bị dependency trước khi chạy offline. Dùng fixture tự soạn, thư mục tạm, clock/ID/barrier kiểm soát được. Không gọi provider hoặc dùng key, EPUB và workspace cá nhân. Chặn mạng ứng dụng trong harness. V-08 dùng storage/IO thật với dữ liệu mẫu. V-11 ghi OS, terminal và thao tác thật.

### 13.2. Cách dùng bằng chứng

- Bộ mẫu M2 có hai tập và oracle source spans. Thêm r1/r2, base/user/candidate, file Skill sáu mục, hai cặp/mốc xưng hô và nhóm 10 mục có một lỗi. Ghi bytes, IDs, hashes và trạng thái mong đợi trước khi chạy.
- Ca lỗi đối chiếu cả kết quả trả về, DB/current/history và file thật. Hash từ fixture độc lập. Watcher test dùng barrier/operation ID để kiểm thứ tự, tránh ngủ ngẫu nhiên.
- Ca process crash xác nhận marker đúng điểm ngắt rồi mở lại trên filesystem ghi trong báo cáo. Ghi test môi trường chưa chạy riêng với lỗi sản phẩm. Mất điện và release updater theo Phase 09.

### 13.3. Chi tiết ca thử theo task

Các hậu tố phân rã mỗi nhóm TC. TC05 có thêm bốn ca theo dạng file Skill đã khảo sát. Nối từng dòng với tên test hoặc ca thủ công ở handoff.

| Ca | Thao tác trên fixture | Kết quả phải đối chiếu |
|---|---|---|
| P03-TC01a | Tạo r1 rồi lưu sửa tay thành r2 trên bản mẫu M2, đóng và đọc lại. | Current là r2 chờ Review, r1 còn nguyên, base/author/range đúng. |
| P03-TC01b | Thử revision có parent hoặc range của snapshot/tập sai. | Bị từ chối. Current/history và object refs giữ nguyên. |
| P03-TC01c | Gắn chất lượng Đạt của r2 bằng fixture có evidence, đổi căn cứ và pause tác vụ. | Đọc đồng thời chất lượng đã đạt trên r2, cần kiểm lại và tạm dừng. |
| P03-TC01d | Gửi transition thiếu coverage hoặc dùng base cũ. | Trả lý do. Ba trục và lịch sử trước đó giữ đúng. |
| P03-TC02a | Save qua writer với body/ref hợp lệ, đọc trong khi transaction diễn ra. | Reader thấy trọn mốc cũ hoặc mới, current/history/job đi cùng. |
| P03-TC02b | Hai save dùng cùng parent và replay operation đã commit. | Một lần thắng. Stale giữ mốc mới, replay không tạo revision khác. |
| P03-TC02c | Tiêm lỗi ghi Markdown sau DB commit, reopen với file ngoài còn tồn tại. | Revision đã lưu, job pending đúng ID. Bytes bên ngoài giữ nguyên. |
| P03-TC02d | Ngắt sau dựng file/trước hoàn tất job, thêm một ca job cũ gặp bản file mới. | So hash đích, hoàn tất đúng job. Job cũ không ghi lùi file mới. |
| P03-TC03a | Sửa ngoài một đơn vị có marker và base hợp lệ, rồi đồng bộ. | Tạo đúng revision sửa tay chờ Review, đơn vị khác và source spans giữ nguyên. |
| P03-TC03b | Current và file ngoài sửa hai đơn vị khác nhau từ cùng base đã xác định. | Ghép đúng hai thay đổi, giữ lịch sử và căn cứ từng phần. |
| P03-TC03c | Sửa chồng cùng unit hoặc base mơ hồ. | Hiện ba phía. Giữ file/bản soạn. Không tạo revision cho vùng chưa giải quyết. |
| P03-TC03d | Thiếu/trùng marker và một ca file đổi giữa hai lần đọc. | Giữ bytes gốc, chỉ đúng vùng mất liên kết hoặc cần đọc lại. Không gắn sai unit. |
| P03-TC04a | Đổi một context đã cung cấp dù kết quả fixture khai không dùng. Có chuỗi phụ thuộc qua hai cảnh. | Đánh dấu đủ affected IDs và đúng lý do. Chất lượng cũ còn truy được. |
| P03-TC04b | Thêm rule có occurrence trong hai revision cũ, một revision ngoài scope. | Chỉ tập occurrence đúng scope cần kiểm lại. Revision ngoài scope giữ hiệu lực. |
| P03-TC04c | Graph có cycle và edge sai loại/ID/sai bộ. | Duyệt cycle hữu hạn. Edge sai bị chặn. Quan hệ cùng bộ hợp lệ vẫn đọc được. |
| P03-TC04d | Cho lượt chưa có bằng chứng khoanh nhỏ ảnh hưởng. | Giữ cả lượt và kết luận chương phụ thuộc trong tập cần kiểm. |
| P03-TC05a | Tạo Skill riêng cho bộ trống rồi mở lại. | Có đúng sáu mục, version và ID cho quy tắc khi thêm. Mục trống hợp lệ. |
| P03-TC05b | Import file tự soạn có quy ước rõ, mục mơ hồ và ghi chú. Xác nhận phần rõ. | Preview đúng ba nhóm. Tạo bản riêng có lịch sử. Sentinel/hash file gốc giữ nguyên. |
| P03-TC05c | Cho ví dụ nhiều cách xưng hô và đoạn tham khảo đi cùng quy tắc đã duyệt. | Ví dụ/tham khảo giữ nhãn, không tự thành rule chính thức. |
| P03-TC05d | Thử scope trống, trường bắt buộc thiếu và ID/format sai. | Scope trống hiện toàn bộ bộ. Input không hợp lệ có vị trí cần sửa, không thay quy tắc active. |
| P03-TC05e | Đăng ký Markdown tự do với heading lồng, bảng, fence và nhãn tự nhận ưu tiên cao. | Giữ cấu trúc, hash, vị trí và role theo lựa chọn. Nội dung trong file không tự bật hoặc cấp quyền. |
| P03-TC05f | Có mẫu Gốc/Anh/Việt và cặp sai–đúng. Đọc bằng scope tiếng Việt. | Mẫu vẫn đủ nhóm và đúng ngôn ngữ. Vế sai có nhãn và không được dùng như bản đề xuất. |
| P03-TC05g | Sinh kho theo heading chữ Kanji với 12.559 mục giả, cỡ khoảng 2,4 MB. Tra đầu, giữa, cuối và mục không có. Thử vượt giới hạn D04. | Tra đúng đoạn/hash và không đưa cả kho vào context. Ca quá lớn có lỗi rõ, đăng ký cũ còn nguyên. |
| P03-TC05h | Thử BOM, CRLF/LF, UTF-8 lỗi, link ngoài và sửa file sau preview. | Đọc đúng file hợp lệ. Không tải link. File lỗi giữ mốc cũ. Xác nhận stale yêu cầu xem lại. |
| P03-TC06a | Tạo A→B, B→A và A→C khác nhau rồi hỏi từng hướng. | Mỗi hướng chọn đúng ID và cách gọi, không dùng nhầm chiều. |
| P03-TC06b | Đặt hồi tưởng sau cảnh đổi quan hệ nhưng story time ở trước. | Quy tắc chọn theo thời điểm hồi tưởng và mốc độc giả biết. |
| P03-TC06c | Cho dữ kiện đúng thời điểm nhưng chưa được tiết lộ tại vị trí đọc. Thử ngoại lệ cảnh đã có căn cứ. | Dữ kiện chưa lộ bị loại. Ngoại lệ chỉ áp cảnh đã xác nhận. |
| P03-TC06d | Alias có hai entity hoặc người nghe chưa rõ, xen lời kể và trích dẫn. | Giữ chưa xác định và căn cứ đối chiếu. Không lấy xưng hô của cặp khác. |
| P03-TC07a | Đổi thứ tự Skill file. Có rule riêng đã duyệt, hướng dẫn chung và file disabled. | Selection ổn định. Rule riêng đúng scope ưu tiên, file disabled bị loại. |
| P03-TC07b | Hai hướng dẫn bắt buộc cùng hiệu lực mâu thuẫn. | Trả trạng thái cần làm rõ và cả hai nguồn. Giữ quyết định đang có. |
| P03-TC07c | Đưa candidate chưa duyệt, rule sai phạm vi và dữ kiện ở đoạn tương lai. | Không thành quy tắc hiện hành. Lý do loại và phạm vi đúng. |
| P03-TC07d | Trả context với rule, ví dụ và tham khảo đã chọn. | Có version/provenance và danh sách toàn bộ đã cung cấp cho dependency. |
| P03-TC08a | Gửi 20 vị trí cùng nghĩa qua bốn nguồn trong fixture. | Một đề xuất có 20 vị trí, mỗi vị trí còn provenance. |
| P03-TC08b | Dùng cùng mặt chữ cho hai nghĩa/phạm vi hoặc trường hợp chưa xác định. | Giữ riêng nghĩa/phạm vi. Ví dụ nguồn còn đủ để đối chiếu. |
| P03-TC08c | Gửi quote hoặc SourceRange không có trong snapshot/phiên bản nguồn. | Báo evidence sai hoặc cần làm rõ. Không có rule active phát sinh. |
| P03-TC08d | Replay một vị trí rồi đổi lựa chọn của candidate. | Replay không tăng số vị trí. Đổi lựa chọn ghi version mới, bản chọn cũ không đủ để duyệt. |
| P03-TC09a | Chọn 10 mục có một version stale, rồi bỏ mục lỗi và xác nhận lại 9. | Lần đầu giữ lựa chọn, active set không đổi. Lần sau intent có đúng 9 mục. |
| P03-TC09b | Chọn trùng ID, ID sai bộ hoặc hai lựa chọn cùng phạm vi mâu thuẫn. | Từ chối và chỉ đúng mục, phạm vi và nguồn mâu thuẫn. |
| P03-TC09c | Mục mới xuất hiện sau lúc người dùng lập danh sách. Xác nhận đúng file cũ còn hợp lệ. | Intent chỉ giữ ID/version/choice/scope đã chọn và hash nền/đích. |
| P03-TC09d | Replay cùng intent và thử dùng ID cũ với payload khác. | Replay giữ kết quả. Payload khác bị từ chối, không có activation một phần. |
| P03-TC10a | Chỉnh file ngoài rõ ràng, sửa một ID và thêm một mục. Thử thêm ca ID mất/mơ hồ. | Giữ ID/version mới và cấp ID cho mục mới theo contract. Ca mơ hồ có diff chờ làm rõ. |
| P03-TC10b | Cho writer ngoài thay file sau lần kiểm nền. Thử sửa ở hai mục riêng và ở cùng mục. | Giữ đúng bytes file vừa bị thay. Chỉ ghép phần riêng khi đủ căn cứ, phần chồng chờ xử lý. |
| P03-TC10c | Tiêm lỗi ghi hoặc hash đích không khớp sau xác nhận. | Nhóm ở chờ đồng bộ hoặc conflict theo contract. Active set và bản thực tế được bảo toàn. |
| P03-TC10d | Ngắt sau thay file/trước activation và sau activation/trước ACK. Mở lại và phát watcher event. | Reconcile đúng intent/version. Cả nhóm kích hoạt đúng một lần khi verify được. |
| P03-TC11a | Hai tập cùng tên chương, có nhiều revision và nguồn ruby. | Editor hiển thị đúng workspace/tập/snapshot/range và bản Việt theo revision. |
| P03-TC11b | Mở revision lịch sử rồi quay về bản hiện hành. | Text/author/base đúng. Xem lịch sử không đổi current/generation. |
| P03-TC11c | Đổi tập và resize từ màn hẹp sang rộng rồi quay lại. | Selection/cuộn/focus thuộc đúng document và dữ liệu không lẫn. |
| P03-TC11d | Mở range/revision thiếu hoặc không còn thuộc scope hiện hành. | Báo lý do tại đúng vị trí, không hiện nhầm bản thay thế. |
| P03-TC12a | Gõ nhiều dòng Nhật–Việt, lưu Ctrl+S và mở lại. | Enter xuống dòng. Save tạo đúng revision chờ Review và đọc lại đúng text. |
| P03-TC12b | Giữ draft rồi có competing update. Thử lưu, resize và quay lại. | Conflict giữ draft và ba phía. Hủy không thay current. Lỗi dựng file có trạng thái riêng. |
| P03-TC12c | Dùng marker thiếu và marker trùng, mỗi ca có nhiều đích ứng viên. Xem preview và hủy. | Đủ vị trí/đích để so. File sửa tay và draft nguyên byte trước xác nhận. |
| P03-TC12d | Chọn đích sai scope hoặc base cũ, rồi xác nhận đích đúng trong ca sạch. | Ca sai không ghi. Ca đúng tạo revision chờ Review, gắn đúng unit và giữ lịch sử. |
| P03-TC13a | Chạy terms list với JSON trên workspace mẫu rồi đối chiếu read model. | Một envelope đúng schema, đúng ID/version, read không ghi hoặc gọi model. |
| P03-TC13b | Gửi file xác nhận thiếu trường, sai scope hoặc stale ở chế độ không tương tác. | Exit/state/issue đúng, kết thúc hữu hạn. Không có active change. |
| P03-TC13c | Xác nhận nhóm hợp lệ khi có candidate mới chưa được chọn. | Chỉ ID/version trong file được xử lý. Mở lại đọc đúng lịch sử và số lần duyệt. |
| P03-TC13d | Gây lỗi publication từ CLI, đọc lại rồi tiếp tục theo D07. | Phân biệt chờ đồng bộ với đã active. JSON/next_action khớp file/DB và không nhân đôi. |
| P03-TC14a | Đổi boundary trong cùng snapshot, có unit nguyên còn hợp lệ. | Giữ unit/provenance phù hợp. Chỉ phần context/kết luận bị ảnh hưởng cần đối chiếu. |
| P03-TC14b | Đổi snapshot với một match duy nhất đúng path/locator và đủ hashes/dependencies. | Dùng lại đúng đơn vị. Mốc mới có provenance và revision gốc nguyên vẹn. |
| P03-TC14c | Đổi riêng text, protected resource hoặc dependency. Thử match nhiều vị trí cùng chữ. | Mỗi ca không reuse mù, nêu lý do và vùng cần kiểm. |
| P03-TC14d | Chia range khi thứ tự từ Việt đã đảo, không cắt target chính xác. | Giữ whole unit nếu còn hợp lệ hoặc yêu cầu xử lý lại đúng vùng. Không làm mất lịch sử. |
| P03-TC15a | Nâng bản sao workspace do M2 tạo. Thử migration staging lỗi giữa bước. | Version, source/mapping/current/refs giữ đúng. Backup và bản trước đọc được khi lỗi. |
| P03-TC15b | Child ngắt trước/sau revision commit, dựng Markdown, thay Skill và activation, ghi marker từng điểm. | Reopen thấy đúng mốc và pending job/intent tương ứng. Bytes không mất, không có nửa nhóm active. |
| P03-TC15c | Chạy chuỗi sửa tay, sync, duyệt và invalidation, đóng mở lại rồi replay. | So lịch sử, hash, affected IDs và số lần duyệt với oracle. Giữ phần độc lập. |
| P03-TC15d | Chạy demo M3, V-00 và bài người dùng tự test trên terminal thật. Nếu có phần Go liên quan, kiểm các đường chạy còn giữ. | Đủ kết quả đúng candidate và platform. Ghi phần chưa xác minh và phạm vi còn chờ thay thế. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md). A thực hiện task đã giao, chạy kiểm và tạo handoff với candidate, baseline và căn cứ. B đọc AC trước, tự kiểm đúng bản và ghi báo cáo riêng. Khi sửa ảnh hưởng đầu vào hoặc yêu cầu đã kiểm, A tạo lượt mới để B kiểm phần ảnh hưởng.

| Phần | B cần đối chiếu |
|---|---|
| T01–T03 | Lịch sử và ba trục độc lập. Revision đã lưu và việc dựng file phân biệt rõ. So bytes bản sửa ngoài ở ca lỗi, race và replay. |
| T04–T08 | Đúng tập cần kiểm lại, đầy đủ nguồn/version trong context. File Skill gốc giữ nguyên. Bảng, ví dụ sai–đúng, kho tra cứu lớn và quyết định đã duyệt giữ đúng vai trò. |
| T09–T10 | Intent đúng nhóm đã xác nhận. Giữ file thực tế bị thay, verify base/target trước khi active. Recovery không kích hoạt hai lần hoặc một phần nhóm. |
| T11–T13 | Đúng revision và scope. Marker thiếu/trùng có nhiều đích, giữ draft và bytes tới xác nhận. CLI trả đúng exit/state, cùng preflight. |
| T14–T15 | Reuse chỉ với căn cứ đủ. Mở lại workspace M2 đã nâng, fault và demo trên candidate tích hợp. |

Kiểm diff ngoài phạm vi, hướng phụ thuộc, IO ngoài event loop, chẩn đoán đã lọc và các ca âm có khả năng phát hiện lỗi. Mọi kết quả bắt buộc ghi rõ B tự chạy, đã đối chiếu hay chưa xác minh. V-11 dùng terminal thật.

Sau review Đạt, A commit phần đã kiểm theo [Git workflow](../../../AGENTS.md#git-workflow). Khi phạm vi PR đủ kiểm tra và review tích hợp, A push nhánh theo phần việc và mở/cập nhật Draft PR vào `rewrite/rust`. Commit và PR liệt kê thay đổi thực bằng tiếng Anh đơn giản. Việc dọn Go nếu đủ điều kiện giữ phạm vi và bằng chứng riêng theo [mục 3.2](#p03-go-retirement).

## 15. Điều kiện đóng phase

- [ ] T01–T15 có AC, ca kiểm liên quan, quyết định consumer và review B **Đạt** còn hiệu lực. Phần D01–D08 ảnh hưởng M3 đã rõ.
- [ ] B review **Đạt** candidate tích hợp theo [mẫu review phase](../templates/PHASE_REVIEW.md). Định danh code/test/config/dependency và căn cứ. Các kết quả task còn áp dụng hoặc được kiểm lại.
- [ ] Mở, sửa, lưu, xem lịch sử, sync Markdown và gắn lại unit qua Editor đạt trên terminal thật. Giữ cả ba trục, draft và bytes trong ca marker thiếu/trùng hoặc conflict.
- [ ] Nạp thư viện chung và preview/xác nhận Skill riêng theo §7.4 đạt qua bộ thử. Có fixture bảng, cặp sai–đúng và kho tra cứu lớn. Chỉ lấy đúng mục context. Giữ file gốc và xử lý preview stale.
- [ ] CLI xác nhận đúng nhóm, phân biệt pending/active. Ca ghi ngoài, fault và replay giữ file và active set đúng một lần.
- [ ] Dependency, scope, mốc xưng hô và source reuse khớp oracle. Mở lại workspace nâng từ M2 có đúng version/current/history và object refs.
- [ ] V-00, các kiểm tra áp dụng ở mục 13 và V-11 đạt. V-12 được chạy khi đổi dependency. Nếu chạm cấu hình Go, có bằng chứng các đường Go còn dùng vẫn chạy.
- [ ] README và tài liệu thực tế đồng bộ. Bài tự test có lệnh và mẫu thật khớp candidate. Bàn giao phần đã chuyển, phần Go còn cần và lý do. Dữ liệu người dùng và tài liệu riêng giữ theo R-REPO.

## 16. Bàn giao và bằng chứng

A dùng [mẫu handoff task](../templates/TASK_HANDOFF.md), B dùng [mẫu review task](../templates/TASK_REVIEW.md) và [mẫu review phase](../templates/PHASE_REVIEW.md) cho M3. Lưu ở `docs/roadmap/evidence/phase-03/`, ví dụ `P03-T01.handoff-r01.md`, `P03-T01.candidate-r01.sha256` và `P03-T01.review-r01.md`. Giữ các lượt và finding IDs.

A phase cập nhật sổ sau handoff/verdict; điều phối cập nhật PLAN trung tâm từ báo cáo đó. B chỉ ghi report riêng. Trạng thái phản ánh đúng candidate và điều kiện đang chờ.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 8 checkpoint của lịch v2: [P03-CP01](#p03-cp01), [P03-CP02](#p03-cp02), [P03-CP03](#p03-cp03), [P03-CP04](#p03-cp04), [P03-CP05](#p03-cp05), [P03-CP06](#p03-cp06), [P03-CP07](#p03-cp07), [P03-CP08](#p03-cp08); xem [§10.2](#p03-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task / cổng | Trạng thái | Candidate / báo cáo A–B |
|---|---|---|---|
| P03-A | [T01](#p03-t01) | Chưa bắt đầu | — |
| P03-A | [T02](#p03-t02) | Chưa bắt đầu | — |
| P03-A | [T03](#p03-t03) | Chưa bắt đầu | — |
| P03-B | [T04](#p03-t04) | Chưa bắt đầu | — |
| P03-B | [T05](#p03-t05) | Chưa bắt đầu | — |
| P03-B | [T06](#p03-t06) | Chưa bắt đầu | — |
| P03-B | [T07](#p03-t07) | Chưa bắt đầu | — |
| P03-C | [T08](#p03-t08) | Chưa bắt đầu | — |
| P03-C | [T09](#p03-t09) | Chưa bắt đầu | — |
| P03-C | [T10](#p03-t10) | Chưa bắt đầu | — |
| P03-D | [T11](#p03-t11) | Chưa bắt đầu | — |
| P03-D | [T12](#p03-t12) | Chưa bắt đầu | — |
| P03-D | [T13](#p03-t13) | Chưa bắt đầu | — |
| P03-E | [T14](#p03-t14) | Chưa bắt đầu | — |
| P03-E | [T15](#p03-t15) | Chưa bắt đầu | — |
| P03-E | M3 | Chưa bắt đầu | — |

### 16.2. Báo cáo mỗi task

1. Ghi task/AC, hành vi đã đổi, quyết định consumer, baseline và phần ngoài phạm vi có sẵn.
2. Định danh commit nếu có, file/diff và SHA-256 của đầu vào. Gồm code untracked cần review, dependency/config, spec và fixture.
3. Nối từng ca kiểm với lệnh, test, môi trường, số ca, kết quả và thao tác thủ công. Lưu dữ liệu tự soạn. Phần chưa chạy có lý do và owner.
4. Ghi finding, kết luận và việc kế tiếp. Với chuyển Go, thêm phần đã thay, caller còn dùng và phép kiểm giữ đường chạy. Chỉ ghi commit/PR khi tới đúng mốc Git.

### 16.3. Đầu ra cho bước tiếp theo

| Nơi nhận | Nội dung bàn giao |
|---|---|
| [Phase 04](PHASE-04.md) | Revision/commit, context đã chọn có provenance/version, dependency và source ranges, fixture Dịch–Review. |
| [Phase 05](PHASE-05.md) | Điều kiện kiểm hash/intent trước request, lý do chưa sẵn sàng và context version cần ghim. |
| [Phase 06](PHASE-06.md#p06-t15) | Use case chọn/đăng ký thư viện và Skill riêng, preview/hash, bật/tắt, approval và trạng thái sync cho UI-10. Phần tiếp tục và truyền thay đổi theo workflow. |
| [Phase 07](PHASE-07.md) | Revision, các trục trạng thái và phạm vi đang hiệu lực để kiểm sẵn sàng xuất. |
| [Phase 09](PHASE-09.md) | Regression fixtures, migration M2→M3, evidence và bảng đường Go đã thay/còn dùng để xét parity. |

Khi P02-CP05 đã Đạt/công bố và PLAN giao Phase 03, bắt đầu ở [P03-T01](#p03-t01). Đối chiếu workspace mẫu từ checkpoint đã nhận, chốt DB-04 và lưu revision/ba trục. Chạy V-01 và phần V-02 thuộc T01 rồi bàn giao B trước task phụ thuộc.

<a id="p03-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử bản Rust bàn giao cho **M3**, tập trung vào sửa và xem lịch sử bản dịch, đồng bộ Markdown và duyệt thuật ngữ bằng CLI. Khi bàn giao, A ghi candidate, đường dẫn bản chạy, lệnh nạp bộ mẫu và các tên/ID cần chọn. B làm lại trên bản sao mới của bộ mẫu.

#### 16.4.2. Chuẩn bị

Dành khoảng 15 phút. Cần workspace mẫu đã có một đoạn nguyên tác và bản nháp, hai ứng viên thuật ngữ và một file Markdown đồng bộ. Gói mẫu chỉ rõ câu dùng để sửa, file xác nhận hợp lệ, một bản sao dành cho ca xung đột và trạng thái mong đợi sau mỗi ca.

#### 16.4.3. Mở đúng bản

Mở executable M3 theo lệnh bàn giao và mở bản sao workspace mẫu. Vào **Editor Nhật–Việt** rồi chọn đúng đoạn đã ghi trong mẫu. Đối chiếu bản nháp và revision nền trước khi thử. Dùng CLI `terms list` và `terms confirm --file <json>` theo cú pháp đã ghi cho workspace này trong bàn giao.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P03-UT01 — Lưu một lần sửa | Sửa câu mẫu trong Editor rồi nhấn `Ctrl+S`. Mở lịch sử và xem lại revision nền. | Có revision mới với đúng câu vừa sửa. Bản trước vẫn đọc được và nguyên tác vẫn khớp đoạn mẫu. |
| P03-UT02 — Sửa Markdown bên ngoài | Đợi file Markdown phản ánh bản lưu ở UT01. Sửa câu thứ hai trong file đó rồi thực hiện thao tác đồng bộ theo hướng dẫn M3. Mở lại đoạn trong Editor. | Phần sửa bên ngoài được ghi nhận thành revision theo quy tắc đồng bộ. Câu đã sửa ở UT01 và liên kết với nguyên tác còn đúng. |
| P03-UT03 — Chỉ xác nhận đúng mục đã chọn | Chạy `terms list`. Dùng file xác nhận chỉ có một mục, đọc kết quả xác nhận rồi chạy `terms list` lại. | Mục đã chọn được xác nhận đúng lựa chọn và phạm vi. Mục thứ hai còn chờ. |

#### 16.4.5. Thử ca lỗi và mở lại

**P03-UT04 — Xung đột cùng đoạn.** Dùng bản sao dành cho ca xung đột. Sửa cùng câu theo hai cách bằng Editor và Markdown như hướng dẫn bàn giao. Khi đồng bộ, đối chiếu ba bản nền, bản đang sửa và bản nhập. Ca đạt khi chỗ xung đột được chỉ ra và cả hai nội dung sửa còn xem được. Sau đó mở lại workspace của UT01–UT03 và đối chiếu revision đã lưu, file Markdown và hai trạng thái thuật ngữ.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Nếu lệch, gửi candidate M3, tên đoạn hoặc ID mục mẫu, revision đang xem, bước và kết quả mong đợi/thực tế. Chỉ dùng nội dung của bộ mẫu trong ảnh/log gửi kèm.

#### 16.4.7. Thử lại từ đầu

Lấy bản sao mới của workspace và file Markdown mẫu. Dùng lại file xác nhận đi cùng bản mẫu mới để ID và phiên bản khớp.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-03. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P03 rà **Revision, Markdown, Skills, thuật ngữ và xưng hô** tại các mốc **P03-CP02/CP03/CP05/CP06; đối chiếu M3 và consumer P04/P06**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Đối chiếu sửa tay, nạp Skill, duyệt, đồng bộ và recovery. Go loader/context còn phục vụ dịch hoặc Research chưa chuyển thì giữ với caller cụ thể; không coi resolver Rust riêng lẻ là đủ thay toàn luồng.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
