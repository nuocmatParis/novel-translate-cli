# PHASE 09 — KIỂM THỬ HỆ THỐNG, NGHIỆM THU VÀ CHUẨN BỊ PHÁT HÀNH

> Lịch task song song và checkpoint commit/push tại [mục 10](#p09-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Draft 0.4 — 09/11/2026 · Chưa triển khai.** Lập từ đặc tả và thiết kế theo yêu cầu của người dùng; code hiện tại không là chuẩn thiết kế, đầu vào tái sử dụng hoặc bằng chứng hoàn thành. Lịch thực thi v2 ngày 12/11/2026; thay lịch v0.1 chưa triển khai.
>
> Cấu trúc theo [Phase 01](PHASE-01.md). Task, đường dẫn module và tên test là kế hoạch ban đầu; rà lại trước khi giao việc khi contract của phase trước thay đổi. Không tự thay yêu cầu đã chốt để khớp implementation.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P09 / M9 |
| Tên | Kiểm thử hệ thống, nghiệm thu và chuẩn bị phát hành |
| Trạng thái | Chưa bắt đầu; mọi AC và kiểm thử bên dưới chưa có kết quả thực thi |
| Đầu vào chính | M1–M8 và hồ sơ quyết định/fixture/evidence; release scope xác định |
| Gói giao diện | Nghiệm thu UI-01–15 trong gói phát hành |
| Phân rã | 4 gói việc, 12 task |
| Nhánh khi triển khai | rewrite/rust; giữ Go theo R-REPO |
| Người thực hiện / rà soát | Model A triển khai / model B khác review độc lập; chưa phân công |

Xem [roadmap chung](../ROADMAP.md) và [phạm vi tại Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-09). Tài liệu và bằng chứng nội bộ ở trong `docs/`, ngoài Git.

## 2. Mục tiêu và demo cuối phase

Chứng minh bản Rust đáp ứng phạm vi phát hành bằng kiểm thử hệ thống, recovery/security/compatibility và nghiệm thu chất lượng có đối chứng; chuẩn bị artifact/hướng dẫn đủ kiểm tra lại. M9 là cổng nghiệm thu có bằng chứng, không phải suy từ việc tất cả task đã viết code.

1. Chạy corpus offline từ import đến export qua gói binary đã build.
2. Kill/restart tại các điểm quan trọng và kiểm generation, revision, money, permit, Skill, export.
3. Nâng workspace Rust các phiên bản đã hỗ trợ; thử lỗi nâng cấp và phục hồi backup.
4. Chạy bộ chuẩn 30 tình huống Nhật–Việt, 3 chương tự soạn; xem nhãn đã xác nhận/chờ duyệt và báo cáo model eval riêng.
5. Kiểm EPUBCheck/reader và tám màn/CLI trên nền tảng dự kiến phát hành.
6. Mở release dossier: artifact/version/checksum, kết quả test, hạn chế, compatibility và quyết định sẵn sàng/chưa sẵn sàng.

## 3. Phạm vi

Acceptance traceability; corpus/oracle; đánh giá model thật thủ công có ngân sách; cross-subsystem fault injection; migration/backup/integrity; security; performance/resource measurements; platform/UI/CLI; EPUB readers; packaging/docs/parity và hồ sơ release.

Không thêm tính năng sản phẩm lớn trong hardening; lỗi cần sửa về đúng owner và có regression. Tự động xuất bản release, gửi thông báo hoặc xóa Go không nằm trong việc viết kế hoạch này; khi triển khai chỉ thực hiện trong phạm vi người dùng đã giao. Bản chưa đủ bằng chứng được giữ ở trạng thái chưa sẵn sàng.

## 4. Điều kiện bắt đầu

P09-T01/T02 chuẩn bị sớm sau M1. Corpus T02 chỉ Đạt khi có người đủ khả năng rà nghĩa Nhật–Việt xác nhận. T10 nhận M8 rồi tạo artifact bất biến; T03–T09 nhận đúng artifact đã review. Đổi code hoặc inputs đóng gói phải tạo artifact mới và đánh giá lại bằng chứng bị ảnh hưởng. M9 vẫn nhận đầy đủ M1–M8 và mọi task P09.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-SKILL | [Skills và duyệt thuật ngữ](../../spec/SKILLS_GLOSSARY_SPEC.md) |
| S-ADDRESS | [Xưng hô và ngữ cảnh](../../spec/ADDRESSING_CONTINUITY_SPEC.md) |
| S-MODEL | [Model, retry và chi phí](../../spec/MODEL_STRATEGY_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| S-TRANS | [Giai đoạn Dịch](../../spec/TRANSLATION_STAGE_SPEC.md) |
| S-REVIEW | [Giai đoạn Review](../../spec/REVIEW_STAGE_SPEC.md) |
| S-RUBRIC | [Rubric S0–S3 và M1–M4](../../spec/REVIEW_RUBRIC_SPEC.md) |
| S-RESEARCH | [Research và bằng chứng](../../spec/RESEARCH_STAGE_SPEC.md) |
| S-WORKFLOW | [Điều phối Dịch–Review](../../spec/WORKFLOW_ORCHESTRATION_SPEC.md) |
| S-IMAGE | [Ảnh nguồn và ảnh thay](../../spec/IMAGE_EDIT_SPEC.md) |
| S-OUTPUT | [EPUB đầu ra](../../spec/EPUB_OUTPUT_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-WORKFLOW | [Bước 7 — State machine và recovery](../../desgin/WORKFLOW_STATE_MACHINE_STEP7.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Ký hiệu Qxx trong tài liệu là quyết định xx của S-Q. Hành vi đã chốt ưu tiên hơn phương án kỹ thuật còn đề xuất. Thư viện, tên bảng/trait và SQL nháp phải được rà theo consumer thực tế; không tự áp dụng DDL thiết kế vào workspace.

- Q36: EPUB fixture tạo trong thư mục tạm; Git chỉ giữ generators/nguồn tự soạn/kỳ vọng, không chứa EPUB/output/workspace/key thật.
- 30 tình huống và 3 chương có oracle ý nghĩa/quote/acceptable variants/expected errors; bất đồng ghi chờ duyệt.
- CI offline với fake model/tools kiểm protocol/data/recovery/money/export; live model quality là đợt thủ công tách riêng.
- Báo riêng lỗi nghĩa, coverage, consistency, hallucination, Reviewer detection và false positives; không gộp thành một điểm che lỗi.
- Module chất lượng phải phát hiện đủ các S3 cố ý cài, dẫn nguồn đúng, false positives được đối chiếu trước chấp nhận.
- Migration Rust khóa ghi/backup DB và file sẽ sửa/đi từng version/integrity/rollback; không phá dữ liệu đã tạo từ workspace mới.
- Không yêu cầu migrate workspace test Go cũ; giữ Go hoạt động tới khi replacement có feature parity kiểm chứng.
- EPUB 3.3 phải qua EPUBCheck 5.3.0 và mở đọc trên Calibre cùng reader mục tiêu; mock validation không đủ.
- Release chỉ công bố nền tảng/khả năng đã kiểm tra; build chéo không tự chứng minh chạy được.

## 6. ADR và điểm còn mở

| Vấn đề | Cần trước | Nội dung |
|---|---|---|
| Release scope/platform | T01/T08/T10 | OS/architectures, format artifact, support claims |
| Quality oracle/adjudication | T02/T03 | Người rà nghĩa, consensus, pending labels và tiêu chí chấp nhận Q36 |
| DB-11/12/14 | T04–T07 | Retention/redaction, migration recovery, đo index/resource theo workload |
| Performance budgets | T01/T07 | Dữ liệu, giới hạn đo và điều kiện chấp nhận trước chạy; không bịa SLA từ thiết kế |
| Release/compatibility policy | T10–T12 | Versioning, known limitations, rollback, parity matrix và trạng thái chưa đủ bằng chứng |

Trạng thái ban đầu của các quyết định trên là cần đối chiếu/ghi nhận, không phải đã được thông qua. Dùng ADR đã có nếu còn phù hợp; chỉ mở rộng phần mới. Ghi lựa chọn, đánh đổi, task phụ thuộc và phép kiểm chứng. Chỉ phần thật sự thay đổi yêu cầu sản phẩm mới cần quyết định sản phẩm bổ sung.

## 7. Thiết kế triển khai

Các suites nghiệm thu dùng entrypoint/ports công khai và artifacts thật khi cần, không can thiệp private state để bỏ preflight. Harness có failpoint/clock/transport xác định; process tests kiểm OS locks; DB/file readback là oracle dữ liệu. Model evaluation lưu input/versions/usage và adjudication tách khỏi unit tests.

Owner sửa lỗi vẫn thuộc crate/domain cũ; phase hardening chỉ điều phối kiểm chứng và thêm regression có ý nghĩa. Packaging lấy từ workspace lock/toolchain đã chốt, không build bản khác với commit đã test rồi ghi cùng bằng chứng.

Contract có consumer thực tế mới được công khai. Domain/Application giữ quy tắc và quyền công bố; adapter thực hiện IO qua port; TUI/CLI gọi cùng use case. Không nối hai adapter qua implementation nội bộ để né ownership hoặc tạo dependency vòng.

### 7.1. Từ ma trận yêu cầu tới artifact

1. T01 chốt scope, nền tảng, loại kiểm chứng và ngưỡng cần có trước khi chạy. Mỗi hàng có requirement, AC/TC, cách kiểm và điều kiện chặn.
2. T02 chốt bộ đối chứng và nhãn trước khi đánh giá. T03 dùng model thật khi người dùng đã chọn kết nối, dữ liệu, model và ngân sách. Ghi kết quả nghĩa riêng với bằng chứng offline của các subsystem.
3. T04–T07 lấy bằng chứng từ các điểm ngắt, migration, kiểm quyền và workload đã đặt. Khi sửa lỗi, xác định phần bằng chứng phải chạy lại và gắn lại candidate.
4. T10 dựng gói từ bản code và lock đã kiểm. T08/T09 lấy đúng artifact này để thử trên terminal và trình đọc thật. Lưu hash file EPUB xuất cùng hash executable đã tạo file.
5. T11/T12 đối chiếu kết quả, phạm vi hỗ trợ, giới hạn và parity với cùng bản release. Lượt dọn Go nhận đầu ra theo mục 16.3 và có review riêng.

Nếu candidate thay đổi, lập danh sách chênh lệch và đối chiếu bằng chứng theo phần bị ảnh hưởng trước khi B kết luận. Báo cáo ghi riêng ca offline, ca trên môi trường thật, ca cần người rà nghĩa và ca đánh giá trả phí.

## 8. Dữ liệu, lỗi và phục hồi

| Sự cố | Kết quả bắt buộc |
|---|---|
| Crash ở commit/file/Skill/handoff/export | Khôi phục trạng thái hợp lệ theo operation IDs/hash/versions; không mất lịch sử |
| Migration thất bại sau khi sửa file | Phục hồi DB và đúng các file liên quan từ backup đã kiểm |
| Artifact lỗi trên một target | Không quảng bá target đạt; sửa và chạy lại kiểm tra bị ảnh hưởng |
| Oracle bất đồng hoặc eval vượt budget | Giữ kết quả/hồ sơ, đánh dấu pending/stopped; không tự tăng tiền hoặc gán pass |
| Lỗi bảo mật/coverage/S3 seeded bị bỏ sót | Chặn acceptance liên quan, sửa đúng owner và regression trước đánh giá lại |
| Chưa đạt parity | Giữ đường Go và mô tả phạm vi Rust thực có |

Recovery được xác nhận bằng trạng thái trước/sau, không chỉ app khởi động lại không crash. Báo rõ phần đã thử và những loại sự cố không có cam kết xử lý.

Mỗi lỗi cần giữ định danh phạm vi và nguyên nhân đã redaction, cùng bước tiếp theo. Không dùng thông báo thành công thay cho trạng thái đang chờ. Mọi thay đổi schema đã áp dụng phải có migration/version và kiểm thử tương thích; dữ liệu, lịch sử và sửa tay được bảo toàn.

## 9. Phần TUI/CLI

Dùng đúng bộ binary/artifact dự kiến phát hành để kiểm tám màn và CLI: keyboard, resize, Unicode, exact revision, current active model, pending approval/sync/budget, errors, Preview và version history. Hướng dẫn quick start/help phải dẫn tới thao tác dùng được. Không đưa báo cáo nội bộ hoặc protocol IDs không cần thiết vào màn người dùng.

Thông báo sản phẩm dùng ngôn ngữ người dùng hiểu được; mã task/ADR/crate chỉ nằm trong tài liệu kỹ thuật. Điều hướng, cuộn, resize và mở trợ giúp không tự gửi model hoặc thay quyết định nghiệp vụ.

<a id="p09-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P09 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P09-T01/T02 chuẩn bị sớm sau M1. Corpus T02 chỉ Đạt khi có người đủ khả năng rà nghĩa Nhật–Việt xác nhận. T10 nhận M8 rồi tạo artifact bất biến; T03–T09 nhận đúng artifact đã review. Đổi code hoặc inputs đóng gói phải tạo artifact mới và đánh giá lại bằng chứng bị ảnh hưởng. M9 vẫn nhận đầy đủ M1–M8 và mọi task P09. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p09-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p09-cp01"></a>P09-CP01 — Ma trận nghiệm thu | [P09-T01](#p09-t01) | M1 Đạt | P09-T01: V-01 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp02"></a>P09-CP02 — Corpus có người rà nghĩa | [P09-T02](#p09-t02) | [P09-CP01](#p09-cp01) | P09-T02: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp03"></a>P09-CP03 — Artifact để nghiệm thu | [P09-T10](#p09-t10) | M8 Đạt, [P09-CP01](#p09-cp01) | P09-T10: V-10, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp04"></a>P09-CP04 — Crash và migration | [P09-T04](#p09-t04), [P09-T05](#p09-t05) | [P09-CP01](#p09-cp01), [P09-CP03](#p09-cp03) | P09-T04: V-04 trong phạm vi AC/TC của task; P09-T05: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp05"></a>P09-CP05 — Security và performance | [P09-T06](#p09-t06), [P09-T07](#p09-t07) | [P09-CP01](#p09-cp01), [P09-CP03](#p09-cp03) | P09-T06: V-06 trong phạm vi AC/TC của task; P09-T07: V-07 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp06"></a>P09-CP06 — Terminal và trình đọc thật | [P09-T08](#p09-t08), [P09-T09](#p09-t09) | [P09-CP01](#p09-cp01), [P09-CP02](#p09-cp02), [P09-CP03](#p09-cp03) | P09-T08: V-08 trong phạm vi AC/TC của task; P09-T09: V-09 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp07"></a>P09-CP07 — Đánh giá model được cấp ngân sách | [P09-T03](#p09-t03) | [P09-CP02](#p09-cp02), [P09-CP03](#p09-cp03) | P09-T03: V-03 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp08"></a>P09-CP08 — README và parity matrix | [P09-T11](#p09-t11) | [P09-CP03](#p09-cp03), [P09-CP04](#p09-cp04), [P09-CP05](#p09-cp05), [P09-CP06](#p09-cp06) | P09-T11: V-01, V-08 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |
| <a id="p09-cp09"></a>P09-CP09 — Kết luận M9 | [P09-T12](#p09-t12) | M8 Đạt, [P09-CP01](#p09-cp01), [P09-CP02](#p09-cp02), [P09-CP03](#p09-cp03), [P09-CP04](#p09-cp04), [P09-CP05](#p09-cp05), [P09-CP06](#p09-cp06), [P09-CP07](#p09-cp07), [P09-CP08](#p09-cp08) | P09-T12: V-01, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M9; V-00 cho code, B review đúng candidate. | feature/release-checks; A chat P09 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Các đường dẫn dưới đây là vị trí mục tiêu từ thiết kế. Không sửa phần ngoài phạm vi chỉ để tiện triển khai. Tất cả task khởi tạo ở trạng thái **Chưa bắt đầu**; lệnh V-* nằm ở mục 13.

<a id="p09-t01"></a>

### P09-T01 — Ma trận nghiệm thu release và phạm vi

- **Mục tiêu / đầu ra:** Danh sách Q01–Q36/AC/target/evidence và mức chặn.
- **Căn cứ / phụ thuộc:** Q36, R-REPO, M1 đã Đạt và yêu cầu của M2–M8. T01 lập ma trận sớm, ghi evidence còn thiếu, cập nhật theo candidate trước nghiệm thu release. Dependency thực thi: M1 Đạt.
- **Phạm vi:** Release acceptance matrix và baseline version.
- **Việc làm:** Chốt scope/platform/performance workloads; map tất cả release gates tới task/tests; phân automatic/manual/live.
- **Ca lỗi / biên:** Kết quả từ build khác; yêu cầu chưa có test; đổi ngưỡng sau thấy fail.
- **Nghiệm thu:** `P09-AC01` — Mỗi yêu cầu trong scope có oracle/cách kiểm; thiếu bằng chứng được hiện rõ, không ghi pass mặc định.
- **Kiểm chứng:** `P09-TC01`, V-01; dữ liệu kiểm tra: Roadmap AC và candidate commit/build IDs.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lập ma trận release theo Q01–Q36 và AC còn hiệu lực của M1–M8. Gắn task/fixture, target, cách kiểm và bằng chứng cần thu cho từng hàng.
2. Ghi phạm vi Windows x64 và các nền tảng chưa nhận, nhóm chức năng, data versions, điều kiện chất lượng và workload/ngưỡng trước khi đo.
3. Đánh dấu hàng bắt buộc, điều kiện người dùng cần chọn và cách xử lý bằng chứng cũ khi candidate thay đổi.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC01.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T01 |
|---|---|
| Đầu vào bắt buộc | M1 Đạt. Nhận M1 baseline, Q01–Q36 và phạm vi AC dự kiến của M2–M8. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Ma trận requirement/target/AC/test/evidence ghi pending khi chưa có kết quả, ngưỡng workload chốt trước đo. Owner đầu ra là A-P09-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T01 giữ Release acceptance matrix và baseline version. Phần chung do T01 giữ acceptance matrix, người điều phối release giữ scope/baseline. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P08-T01](PHASE-08.md#p08-t01), [P08-T02](PHASE-08.md#p08-t02), [P08-T03](PHASE-08.md#p08-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC01 / P09-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Trace từng requirement tới bằng chứng cùng version/platform. |
| Điểm tích hợp | [P09-CP01](#p09-cp01) — Ma trận nghiệm thu; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t02"></a>

### P09-T02 — Bộ chuẩn 30 tình huống và 3 chương

- **Mục tiêu / đầu ra:** Corpus tự soạn có nhãn/oracle và trạng thái xác nhận.
- **Căn cứ / phụ thuộc:** Q36, S-REVIEW/S-RUBRIC; T01, fixtures trước. Dependency thực thi: [P09-T01](#p09-t01).
- **Phạm vi:** Fixture source/generators/expected outputs; hồ sơ rà nghĩa nội bộ.
- **Việc làm:** Phủ phủ định/chủ thể ẩn/thêm-bớt/hồi tưởng/xưng hô/đa nghĩa/ruby/joins/false citation/exception/đúng khác lời.
- **Ca lỗi / biên:** Expected output chỉ một cách diễn đạt; S3 không có căn cứ; dữ liệu sao sách thật; thiếu người xác nhận.
- **Nghiệm thu:** `P09-AC02` — Đủ 30+3, có acceptable variants/quotes/errors; ca chưa đồng thuận pending và chưa tính nghiệm thu nghĩa.
- **Kiểm chứng:** `P09-TC02`, V-02; dữ liệu kiểm tra: Nhật–Việt tự soạn, seeded S3/S0 và phần đúng.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lập danh mục 30 tình huống và 3 chương tự soạn, bao phủ các nhóm nghĩa, nhất quán, ruby, điểm nối, quote và ngoại lệ của Q36.
2. Gắn từng ca với source span, cách hiểu, bản dịch chấp nhận được và lỗi/severity dự kiến. Nhờ người đọc Nhật và biên tập Việt xác nhận, lưu ý kiến chưa đồng thuận.
3. Tạo generators và kỳ vọng từ nguồn mẫu. Tách seed lỗi S3 và ca dịch đúng khác lời để đo khả năng phát hiện và báo oan.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC02.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T02 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01). Nhận scope/tiêu chí T01 cùng source/fixtures tự soạn. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Corpus 30+3 có ID/source spans/acceptable variants/seeded errors và trạng thái người rà xác nhận. Owner đầu ra là A-P09-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T02 giữ Fixture source/generators/expected outputs; hồ sơ rà nghĩa nội bộ. Phần chung do T01 giữ criteria, T02 giữ corpus/oracle và generator fixtures. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P08-T01](PHASE-08.md#p08-t01), [P08-T02](PHASE-08.md#p08-t02), [P08-T03](PHASE-08.md#p08-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC02 / P09-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu Validator IDs/count/coverage; người có năng lực rà nghĩa/Việt xác nhận riêng, không dùng AI tự chấm làm xác nhận đó. |
| Điểm tích hợp | [P09-CP02](#p09-cp02) — Corpus có người rà nghĩa; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t03"></a>

### P09-T03 — Đánh giá model thật có ngân sách

- **Mục tiêu / đầu ra:** Báo cáo quality/cost theo model/role/config đã chọn.
- **Căn cứ / phụ thuộc:** Q36, S-MODEL, M8 Đạt, T02 đã có oracle và candidate/artifact T10. Người dùng chọn phạm vi/ngân sách/kết nối. Dependency thực thi: [P09-T02](#p09-t02), [P09-T10](#p09-t10).
- **Phạm vi:** Manual evaluation runner/report; không thuộc CI tự động.
- **Việc làm:** Dùng executor thật, pin input/prompts/rules/model/pricing; đo từng chỉ số; adjudicate false positives và quote correctness.
- **Ca lỗi / biên:** Cap/budget chạm, unknown usage, model đổi alias, oracle pending, S3 seeded không phát hiện.
- **Nghiệm thu:** `P09-AC03` — Báo riêng nghĩa/coverage/consistency/hallucination/recall/false positives/chi phí; đủ S3 và dẫn nguồn đúng trước chấp nhận module chất lượng.
- **Kiểm chứng:** `P09-TC03`, V-03; dữ liệu kiểm tra: Corpus T02, profile đã chọn, budget hiệu lực.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chờ phạm vi, model/role, kết nối và ngân sách người dùng chọn. Ghim corpus, version prompt/rules, thông số và căn cứ giá vào run ID.
2. Chạy thủ công qua executor thật và theo dõi requests, chi phí đã biết/chưa biết, lý do chặn hoặc ca chưa chạy.
3. Đối chiếu lỗi nghĩa, coverage, consistency, lỗi bịa, detection và báo oan theo oracle. Ghi đủ kết quả S3 cài sẵn và quote validity, rà các false positives trước quyết định.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC03.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T03 |
|---|---|
| Đầu vào bắt buộc | [P09-T02](#p09-t02), [P09-T10](#p09-t10). Nhận oracle đã xác nhận, exact candidate cùng lựa chọn model/connection/budget được người dùng giao. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Run thật lưu model/role/config/prompt/rules/input versions/usage, báo riêng nghĩa/coverage/consistency/quote/báo oan. Owner đầu ra là A-P09-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T03 giữ Manual evaluation runner/report; không thuộc CI tự động. Phần chung do T02 giữ oracle, M5 giữ executor/budget, T03 giữ live evaluation report. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC03 / P09-TC03, gồm các ca chi tiết §13.3. V-03. Đối chiếu Live evidence có run ID/versions/usage; thiếu điều kiện hoặc chưa chạy giữ pending, không thay bằng fake pass. |
| Điểm tích hợp | [P09-CP07](#p09-cp07) — Đánh giá model được cấp ngân sách; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t04"></a>

### P09-T04 — Crash và phục hồi xuyên subsystem

- **Mục tiêu / đầu ra:** Fault matrix mọi publication/send/handoff quan trọng.
- **Căn cứ / phụ thuộc:** Q07/13/19/28–29/33; T01 và suites P02–P07. Dependency thực thi: [P09-T01](#p09-t01), [P09-T10](#p09-t10).
- **Phạm vi:** App system fault harness; regressions ở owner liên quan.
- **Việc làm:** Ngắt trước/sau file/DB commit, Skill activation, intent/send/response, grant/ack/release, export publish/history.
- **Ca lỗi / biên:** DB và file lệch, stale permit, repeat send, orphan references, mất reservation hoặc user edits.
- **Nghiệm thu:** `P09-AC04` — Generation/history/permit/usage/export đúng invariant sau reopen; không tự resend uncertain.
- **Kiểm chứng:** `P09-TC04`, V-04; dữ liệu kiểm tra: Failpoint IDs, DB/file snapshots, multi-process temp profiles.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lập bảng failpoint trước/sau file/DB commit, Skill publish, model intent/send/result, cấp/nhận/nhả permit và export publish. Mỗi hàng có checkpoint, state và side effects kỳ vọng.
2. Ngắt tiến trình có marker, mở lại cùng workspace và kiểm hashes, revisions, history, queue, counters và ledger. Xác minh replay/resume qua đường chạy thật.
3. Theo ADR-004, kiểm riêng durability của file và directory entry trên filesystem/OS đích trước release. Ghi phần chưa có phép kiểm như một mục còn chặn theo phạm vi.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC04.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T04 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), [P09-T10](#p09-t10). Nhận candidate T10, failpoint/IO/permission protocols từ M2–M7. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Crash campaign có marker/expected readback/counters, power-loss durability có bằng chứng riêng theo ADR-004. Owner đầu ra là A-P09-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T04 giữ App system fault harness; regressions ở owner liên quan. Phần chung do owner storage/workflow từ M2–M7 giữ protocol, T04 giữ fault campaign. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC04 / P09-TC04, gồm các ca chi tiết §13.3. V-04. Đối chiếu So dữ liệu/hash/counters/send trace và integrity, không chỉ process restart thành công. |
| Điểm tích hợp | [P09-CP04](#p09-cp04) — Crash và migration; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t05"></a>

### P09-T05 — Migration Rust, backup và rollback

- **Mục tiêu / đầu ra:** Nâng các schema được hỗ trợ bảo toàn dữ liệu.
- **Căn cứ / phụ thuộc:** Q06, S-DATA, DB-12; T01 và migrations đã áp dụng. Dependency thực thi: [P09-T01](#p09-t01), [P09-T10](#p09-t10).
- **Phạm vi:** nt-storage migration/recovery; compatibility fixtures.
- **Việc làm:** Đọc từng version, writer lock, backup DB/file trước thay, apply steps, integrity và restore khi fail.
- **Ca lỗi / biên:** Disk full, backup incomplete, file đổi trong upgrade, newer schema, crash giữa DB và file.
- **Nghiệm thu:** `P09-AC05` — Upgrade/rollback giữ source/revision/rules/attempts/exports; newer schema không bị mở ghi sai.
- **Kiểm chứng:** `P09-TC05`, V-05; dữ liệu kiểm tra: Mỗi schema Rust đã hỗ trợ với dữ liệu/history thật về cấu trúc nhưng tự soạn.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Liệt kê schema Rust được hỗ trợ cùng đường migration. Chuẩn bị workspace cũ có DB và objects thật do fixture sinh, kèm checksum và expected history.
2. Lấy quyền ghi, backup đủ DB và files phụ thuộc, kiểm bản backup rồi nâng qua từng phiên bản. Xác minh integrity, references và nội dung khi mở bằng bản mới.
3. Kiểm điểm ngắt và khôi phục backup bằng thao tác đã mô tả. Giữ nguyên workspace mới hơn hoặc workspace Go, hướng dẫn luồng tạo mới/import lại theo Q06.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC05.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T05 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), [P09-T10](#p09-t10). Nhận candidate T10, supported Rust schemas và migration/backup contract. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Migrate/restore đối chiếu DB+objects, writer lock và newer-schema fail closed, Go workspace dùng create/import theo Q06. Owner đầu ra là A-P09-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T05 giữ nt-storage migration/recovery; compatibility fixtures. Phần chung do storage owner giữ migration IDs/schema, T05 giữ compatibility fixture/report. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC05 / P09-TC05, gồm các ca chi tiết §13.3. V-05. Đối chiếu Trước/sau hash/count/link identity đúng; không đưa workspace Go cũ qua migration Rust. |
| Điểm tích hợp | [P09-CP04](#p09-cp04) — Crash và migration; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t06"></a>

### P09-T06 — Security và redaction toàn hệ thống

- **Mục tiêu / đầu ra:** Bằng chứng secret/source untrusted/Preview không vượt quyền.
- **Căn cứ / phụ thuộc:** S-SEC, Q30–31/36; T01, fixtures P02/P05/P07. Dependency thực thi: [P09-T01](#p09-t01), [P09-T10](#p09-t10).
- **Phạm vi:** Security integration suites và fixes tại parser/credential/preview owners.
- **Việc làm:** Quét output/log/history/temp reports với sentinel; malicious ZIP/XML/SVG/CSS/pages; session/Host/origin/assets; IPC profile access.
- **Ca lỗi / biên:** Secret trong nested error/panic, traversal/symlink, network refs, replay token, page điều khiển tool.
- **Nghiệm thu:** `P09-AC06` — Không leak sentinel hoặc đọc/ghi tài nguyên ngoài contract; input ác ý không trở thành lệnh/quyền.
- **Kiểm chứng:** `P09-TC06`, V-06; dữ liệu kiểm tra: Sentinel secrets và malicious sources tự sinh.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lấy fixture tấn công theo boundary từ M2/M5/M7, thêm đường qua import, Research, IPC, Preview và file xuất trên bản release.
2. Đặt sentinel bí mật giả ở credential path và lỗi provider. Quét stdout, stderr, TUI, log, lịch sử, báo cáo, temp, Preview và EPUB bằng kiểm chứng phù hợp.
3. Kiểm quyền profile/process/session, Host/origin và allowlist. Ghi kết quả theo nguồn vào, đích ra và tác động bị chặn.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC06.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T06 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), [P09-T10](#p09-t10). Nhận candidate T10 và fixture hostile/sentinel của M2/M5/M7. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Kiểm source→sink, IPC/profile/session/allowlist và redaction trên log/UI/temp/EPUB, chỉ dùng secrets giả. Owner đầu ra là A-P09-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T06 giữ Security integration suites và fixes tại parser/credential/preview owners. Phần chung do owners security boundaries giữ fixes, T06 giữ acceptance probes. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC06 / P09-TC06, gồm các ca chi tiết §13.3. V-06. Đối chiếu Quan sát filesystem/transport/output/authorization effects; không dùng key thật. |
| Điểm tích hợp | [P09-CP05](#p09-cp05) — Security và performance; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t07"></a>

### P09-T07 — Performance và giới hạn tài nguyên

- **Mục tiêu / đầu ra:** Số đo trên workload/gate đã xác định, có profiling.
- **Căn cứ / phụ thuộc:** Q36, D-DATA DB-14; T01; corpus/source generators. Dependency thực thi: [P09-T01](#p09-t01), [P09-T10](#p09-t10).
- **Phạm vi:** Bench/stress harness, indexes/layout/worker fixes có bằng chứng.
- **Việc làm:** Đo import/DB read/large history/UI responsiveness/image/ZIP limits; tách CPU/IO và nguồn block event loop.
- **Ca lỗi / biên:** Input near limit, nhiều revisions/tasks, animation memory, resize khi worker chạy.
- **Nghiệm thu:** `P09-AC07` — Đạt tiêu chí đã đặt trước; vượt giới hạn có diagnostic/cancellation hợp lệ, không báo support không có bằng chứng.
- **Kiểm chứng:** `P09-TC07`, V-07; dữ liệu kiểm tra: Dữ liệu tự sinh nhỏ/vừa/sát giới hạn, máy có thông số ghi lại.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chốt máy, toolchain, fixture size, trạng thái cache, cách lặp và ngưỡng ở T01. Ghi load/import, truy vấn, UI input và export resource theo nhóm workload.
2. Đo trên release build với thời gian, bộ nhớ, DB/IO và độ trễ thao tác. Lấy profiling khi vượt ngưỡng để xác định owner.
3. Chạy ca sát/vượt các caps import/ảnh và lịch sử lớn. Ghi chẩn đoán, phần cleanup và tác động tới workspace.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC07.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T07 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), [P09-T10](#p09-t10). Nhận candidate T10 cùng workload/machine/ngưỡng đã chốt ở T01. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Số đo latency/resources/limits ghi điều kiện lặp, caps giữ validation và integrity dưới tải. Owner đầu ra là A-P09-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T07 giữ Bench/stress harness, indexes/layout/worker fixes có bằng chứng. Phần chung do T01 giữ ngưỡng, owner subsystem giữ profiling/fix, T07 giữ measurements. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC07 / P09-TC07, gồm các ca chi tiết §13.3. V-07. Đối chiếu Lưu latency/resource/counters; không tối ưu dựa trên một run ngẫu nhiên hoặc bỏ validation. |
| Điểm tích hợp | [P09-CP05](#p09-cp05) — Security và performance; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t08"></a>

### P09-T08 — UI/CLI và nền tảng trên artifact

- **Mục tiêu / đầu ra:** Acceptance chạy với binary dự kiến phát hành.
- **Căn cứ / phụ thuộc:** Q34–36, S-TUI; T01, artifact T10 sau build. Dependency thực thi: [P09-T01](#p09-t01), [P09-T10](#p09-t10).
- **Phạm vi:** Platform smoke, PTY/manual terminal, CLI process tests.
- **Việc làm:** Chạy import→workflow fake/read saved→export, flags/JSON/exit, keyboard/resize/Unicode, credential adapter platform contract.
- **Ca lỗi / biên:** Build chạy được trên dev nhưng không ở máy sạch; path space/Unicode; CLI pipe treo.
- **Nghiệm thu:** `P09-AC08` — Mỗi target quảng bá có build và runtime evidence; supported commands/help đúng binary.
- **Kiểm chứng:** `P09-TC08`, V-08; dữ liệu kiểm tra: Artifact version/checksum và workspace tự soạn.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lấy package T10 đúng checksum trên target đã chọn. Chạy fixture qua đường CLI và TUI của bản phân phối, ghi terminal/OS và cách chuẩn bị.
2. Lặp các ca M8 về non-interactive, exit/JSON, Unicode, resize, bàn phím, ngữ cảnh và terminal cleanup. Kiểm adapter credential theo S-SEC trên hệ đích.
3. Báo riêng nền tảng chưa chạy native. Nếu phát hiện lỗi cần sửa, dựng artifact mới và xác định phép kiểm phải lặp.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC08.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T08 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), [P09-T10](#p09-t10). Nhận artifact/hash T10, M8 screen/command matrix và terminal target. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Process CLI/TUI trên executable phân phối có exit/JSON/keyboard/resize/cleanup theo đúng OS. Owner đầu ra là A-P09-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T08 giữ Platform smoke, PTY/manual terminal, CLI process tests. Phần chung do M8 giữ UI/CLI contract, T10 giữ package, T08 giữ artifact acceptance. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC08 / P09-TC08, gồm các ca chi tiết §13.3. V-08. Đối chiếu Không dùng cargo run thay cho tất cả artifact checks; target chưa test bị ghi rõ. |
| Điểm tích hợp | [P09-CP06](#p09-cp06) — Terminal và trình đọc thật; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t09"></a>

### P09-T09 — EPUBCheck và nghiệm thu trình đọc

- **Mục tiêu / đầu ra:** Corpus output đã kiểm XML/resources/content và visual reading.
- **Căn cứ / phụ thuộc:** Q32/36, S-OUTPUT; T02, exporter P07, artifact T10. Dependency thực thi: [P09-T02](#p09-t02), [P09-T10](#p09-t10).
- **Phạm vi:** Export acceptance và reader checklist/evidence.
- **Việc làm:** EPUB2/3 inputs; ruby/notes/Nav/cover/illustrations/semantic formatting; validate 5.3.0; mở Calibre + reader mục tiêu.
- **Ca lỗi / biên:** EPUBCheck pass nhưng mất chữ/ảnh/link; font/Unicode lỗi; wrong reading order.
- **Nghiệm thu:** `P09-AC09` — Validator đạt và nội dung được đối chiếu trong readers ghi version/OS; warning ảnh hưởng nội dung đã xử lý.
- **Kiểm chứng:** `P09-TC09`, V-09; dữ liệu kiểm tra: Outputs tự sinh từ các trường hợp Q36.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dùng T10 xuất bộ mẫu EPUB 2/3 với Nav lồng, bìa, ảnh, ruby, links và chú thích. Ghi bundle, artifact hash và expected semantic content.
2. Đọc lại XML/resources/spine/coverage và chạy EPUBCheck 5.3.0 trên file xuất. Ghi phân loại warning và xử lý trước khi báo đạt.
3. Mở chính các file đã kiểm trên Calibre và trình đọc mục tiêu. Duyệt chương, điểm nối, ảnh, ruby, định dạng và đường quay lại chú thích.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC09.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T09 |
|---|---|
| Đầu vào bắt buộc | [P09-T02](#p09-t02), [P09-T10](#p09-t10). Nhận corpus T02, exporter M7 và artifact/hash T10. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | EPUB outputs gắn executable/bundle/hash, readback/EPUBCheck 5.3.0 và reader real trên cùng file. Owner đầu ra là A-P09-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T09 giữ Export acceptance và reader checklist/evidence. Phần chung do M7 giữ exporter, T02 giữ oracle, T09 giữ reader/validator evidence. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC09 / P09-TC09, gồm các ca chi tiết §13.3. V-09. Đối chiếu Đối chiếu source/bundle/output; ảnh hợp lệ giữ byte; không coi validator pass là đủ visual QA. |
| Điểm tích hợp | [P09-CP06](#p09-cp06) — Terminal và trình đọc thật; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t10"></a>

### P09-T10 — Đóng gói artifact và smoke từ môi trường sạch

- **Mục tiêu / đầu ra:** Gói versioned, checksum và build provenance có thể rà soát.
- **Căn cứ / phụ thuộc:** R-REPO, T01 đã chốt scope và core regression baseline đạt. T08/T09 nhận artifact từ T10. Dependency thực thi: [P09-T01](#p09-t01), M8 Đạt.
- **Phạm vi:** Rust packaging/CI artifacts/install instructions; không tự publish.
- **Việc làm:** Build bằng lock/toolchain, đặt executable đúng tên, runtime deps/license cần thiết, checksum; giữ binary Go riêng.
- **Ca lỗi / biên:** Artifact khác build đã test, thiếu runtime, path install sai, secrets/docs nội bộ lọt package.
- **Nghiệm thu:** `P09-AC10` — Artifact nguồn gốc rõ và không chứa dữ liệu riêng tư; sẵn cho T08/T09 smoke, chưa gán release đạt.
- **Kiểm chứng:** `P09-TC10`, V-10, V-00; dữ liệu kiểm tra: Clean staging directory và artifact manifest.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dùng candidate xác định và toolchain/lock đã chốt để tạo release binary. Ghi commit, phần thay đổi nếu có, dependency/config và lệnh build.
2. Đóng gói executable đúng tên, hướng dẫn, giấy phép và runtime dependencies thuộc phạm vi. Tạo checksum package và binary.
3. Giải nén trên môi trường sạch, đối chiếu checksum và smoke help/version/TUI. Bàn giao đúng gói cho T08/T09.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC10.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T10 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), M8 Đạt. Nhận đầu ra chức năng M8, regression baseline và scope/platform T01. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Build/package theo toolchain/lock, manifest/checksum truy được candidate, clean smoke dùng đúng executable. Owner đầu ra là A-P09-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T10 giữ Rust packaging/CI artifacts/install instructions; không tự publish. Phần chung do người tích hợp giữ Cargo/CI/release inputs, T10 giữ artifact provenance. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC10 / P09-TC10, gồm các ca chi tiết §13.3. V-10, V-00. Đối chiếu List nội dung/checksum, giải nén/chạy help/version ở target; không tải sách/key để test. |
| Điểm tích hợp | [P09-CP03](#p09-cp03) — Artifact để nghiệm thu; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t11"></a>

### P09-T11 — README, compatibility và parity matrix

- **Mục tiêu / đầu ra:** Hướng dẫn khả năng thực tế, upgrade/recovery và giới hạn.
- **Căn cứ / phụ thuộc:** R-REPO, Q06/34–36; T04–T10 có kết quả tương ứng. Dependency thực thi: [P09-T04](#p09-t04), [P09-T05](#p09-t05), [P09-T06](#p09-t06), [P09-T07](#p09-t07), [P09-T08](#p09-t08), [P09-T09](#p09-t09), [P09-T10](#p09-t10).
- **Phạm vi:** README/release notes draft; parity/compatibility report nội bộ.
- **Việc làm:** Ghi commands/layout/platform thật, backup/rollback, incomplete features; map capability yêu cầu tới Rust evidence.
- **Ca lỗi / biên:** Tuyên bố parity chỉ vì compile; link docs riêng tư như public docs; hướng dẫn key plaintext.
- **Nghiệm thu:** `P09-AC11` — Người khác dùng gói và hiểu giới hạn; chưa đủ parity giữ Go, không xóa đường chạy.
- **Kiểm chứng:** `P09-TC11`, V-01, V-08; dữ liệu kiểm tra: Documentation walkthrough với artifact và evidence matrix.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Đối chiếu README với command/help, đường dẫn, platform và các bước backup/restore đã chạy trên artifact.
2. Lập parity matrix cho từng khả năng Go được yêu cầu thay thế. Ghi hành vi cần giữ, đường Rust, scope, phiên bản và bằng chứng B có thể kiểm.
3. Ghi caller, scripts, CI và đóng gói còn cần Go. Nối các điểm thiếu tới task xử lý và hồ sơ mục 16.3.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC11.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T11 |
|---|---|
| Đầu vào bắt buộc | [P09-T04](#p09-t04), [P09-T05](#p09-t05), [P09-T06](#p09-t06), [P09-T07](#p09-t07), [P09-T08](#p09-t08), [P09-T09](#p09-t09), [P09-T10](#p09-t10). Nhận reports từ candidate hiện hành, artifact commands và capability cần thay Go. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | README/compatibility/parity claims có evidence, liệt kê missing capability/caller và đường upgrade/restore. Owner đầu ra là A-P09-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T11 giữ README/release notes draft; parity/compatibility report nội bộ. Phần chung do owners các capability xác nhận phạm vi, T11 giữ public docs/parity matrix. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC11 / P09-TC11, gồm các ca chi tiết §13.3. V-01, V-08. Đối chiếu Lệnh hướng dẫn được kiểm chứng; claim có bằng chứng, không dựa hình dạng code cũ. |
| Điểm tích hợp | [P09-CP08](#p09-cp08) — README và parity matrix; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p09-t12"></a>

### P09-T12 — Tổng hợp release dossier và kết luận M9

- **Mục tiêu / đầu ra:** Bản bàn giao cụ thể để quyết định phát hành.
- **Căn cứ / phụ thuộc:** Q36, R-REPO; T01–T11. Dependency thực thi: [P09-T01](#p09-t01), [P09-T02](#p09-t02), [P09-T03](#p09-t03), [P09-T04](#p09-t04), [P09-T05](#p09-t05), [P09-T06](#p09-t06), [P09-T07](#p09-t07), [P09-T08](#p09-t08), [P09-T09](#p09-t09), [P09-T10](#p09-t10), [P09-T11](#p09-t11), M8 Đạt.
- **Phạm vi:** Release report, known issues, artifacts/evidence links và trạng thái.
- **Việc làm:** Đối chiếu tất cả gates cùng candidate; ghi automatic/manual/live results, open blockers, parity và follow-up.
- **Ca lỗi / biên:** Một gate pending bị ghi pass; sửa code sau test mà evidence không cập nhật; thiếu human adjudication.
- **Nghiệm thu:** `P09-AC12` — Chỉ sẵn sàng khi đủ criteria; thiếu điều kiện ghi chưa sẵn sàng với task cần xử lý, không tự publish hoặc xóa Go.
- **Kiểm chứng:** `P09-TC12`, V-01, V-00 theo candidate cuối; dữ liệu kiểm tra: Candidate ID/checksums, complete release checklist.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Đối chiếu ma trận với artifact hash/candidate, các vòng review và bằng chứng M1–M8 còn hiệu lực. Rà kết quả offline, manual, oracle và live eval riêng.
2. Lập dossier có support/limitations, compatibility, rollback, blockers và bước tiếp theo. Handoff cho B đánh giá bản tích hợp M9 và yêu cầu còn chờ.
3. Khi có kết luận của B, ghi tình trạng release/parity theo evidence. Nếu đủ điều kiện thay Go, bàn giao phạm vi lượt dọn riêng theo mục 16.3.

Ca thử chi tiết tại [mục 13.3](#p09-cases) theo mã P09-TC12.

**Sáu trường giao việc**

| Trường | Nội dung cho P09-T12 |
|---|---|
| Đầu vào bắt buộc | [P09-T01](#p09-t01), [P09-T02](#p09-t02), [P09-T03](#p09-t03), [P09-T04](#p09-t04), [P09-T05](#p09-t05), [P09-T06](#p09-t06), [P09-T07](#p09-t07), [P09-T08](#p09-t08), [P09-T09](#p09-t09), [P09-T10](#p09-t10), [P09-T11](#p09-t11), M8 Đạt. Nhận toàn bộ evidence/checksum/review và các lựa chọn scope release. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Dossier M9 nối từng claim tới candidate, thiếu mandatory giữ chờ, Go retirement nhận lượt review riêng. Owner đầu ra là A-P09-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P09-T12 giữ Release report, known issues, artifacts/evidence links và trạng thái. Phần chung do người điều phối release và A-T12 nhận toàn bộ, B kết luận độc lập. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Chưa có ví dụ độc lập trong lịch hiện tại; điều phối đối chiếu graph khi cấp thêm việc. Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P09-AC12 / P09-TC12, gồm các ca chi tiết §13.3. V-01, V-00 theo candidate cuối. Đối chiếu Audit từng claim tới bằng chứng; có artifact/notes cụ thể trước hành động phát hành được giao. |
| Điểm tích hợp | [P09-CP09](#p09-cp09) — Kết luận M9; A chat P09 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P09-AC01 | Mỗi yêu cầu trong scope có oracle/cách kiểm; thiếu bằng chứng được hiện rõ, không ghi pass mặc định. | P09-T01 | P09-TC01; V-01 |
| P09-AC02 | Đủ 30+3, có acceptable variants/quotes/errors; ca chưa đồng thuận pending và chưa tính nghiệm thu nghĩa. | P09-T02 | P09-TC02; V-02 |
| P09-AC03 | Báo riêng nghĩa/coverage/consistency/hallucination/recall/false positives/chi phí; đủ S3 và dẫn nguồn đúng trước chấp nhận module chất lượng. | P09-T03 | P09-TC03; V-03 |
| P09-AC04 | Generation/history/permit/usage/export đúng invariant sau reopen; không tự resend uncertain. | P09-T04 | P09-TC04; V-04 |
| P09-AC05 | Upgrade/rollback giữ source/revision/rules/attempts/exports; newer schema không bị mở ghi sai. | P09-T05 | P09-TC05; V-05 |
| P09-AC06 | Không leak sentinel hoặc đọc/ghi tài nguyên ngoài contract; input ác ý không trở thành lệnh/quyền. | P09-T06 | P09-TC06; V-06 |
| P09-AC07 | Đạt tiêu chí đã đặt trước; vượt giới hạn có diagnostic/cancellation hợp lệ, không báo support không có bằng chứng. | P09-T07 | P09-TC07; V-07 |
| P09-AC08 | Mỗi target quảng bá có build và runtime evidence; supported commands/help đúng binary. | P09-T08 | P09-TC08; V-08 |
| P09-AC09 | Validator đạt và nội dung được đối chiếu trong readers ghi version/OS; warning ảnh hưởng nội dung đã xử lý. | P09-T09 | P09-TC09; V-09 |
| P09-AC10 | Artifact nguồn gốc rõ và không chứa dữ liệu riêng tư; sẵn cho T08/T09 smoke, chưa gán release đạt. | P09-T10 | P09-TC10; V-10, V-00 |
| P09-AC11 | Người khác dùng gói và hiểu giới hạn; chưa đủ parity giữ Go, không xóa đường chạy. | P09-T11 | P09-TC11; V-01, V-08 |
| P09-AC12 | Chỉ sẵn sàng khi đủ criteria; thiếu điều kiện ghi chưa sẵn sàng với task cần xử lý, không tự publish hoặc xóa Go. | P09-T12 | P09-TC12; V-01, V-00 theo candidate cuối |

Các AC không thay thế nhau. Build thành công, fixture trả đúng kết quả định sẵn hoặc một màn hiển thị đẹp không chứng minh toàn bộ hành vi nghiệp vụ.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P09-TC01 / P09-AC01 | Roadmap AC và candidate commit/build IDs. | Trace từng requirement tới bằng chứng cùng version/platform. | V-01 |
| P09-TC02 / P09-AC02 | Nhật–Việt tự soạn, seeded S3/S0 và phần đúng. | Validator IDs/count/coverage; người có năng lực rà nghĩa/Việt xác nhận riêng, không dùng AI tự chấm làm xác nhận đó. | V-02 |
| P09-TC03 / P09-AC03 | Corpus T02, profile đã chọn, budget hiệu lực. | Live evidence có run ID/versions/usage; thiếu điều kiện hoặc chưa chạy giữ pending, không thay bằng fake pass. | V-03 |
| P09-TC04 / P09-AC04 | Failpoint IDs, DB/file snapshots, multi-process temp profiles. | So dữ liệu/hash/counters/send trace và integrity, không chỉ process restart thành công. | V-04 |
| P09-TC05 / P09-AC05 | Mỗi schema Rust đã hỗ trợ với dữ liệu/history thật về cấu trúc nhưng tự soạn. | Trước/sau hash/count/link identity đúng; không đưa workspace Go cũ qua migration Rust. | V-05 |
| P09-TC06 / P09-AC06 | Sentinel secrets và malicious sources tự sinh. | Quan sát filesystem/transport/output/authorization effects; không dùng key thật. | V-06 |
| P09-TC07 / P09-AC07 | Dữ liệu tự sinh nhỏ/vừa/sát giới hạn, máy có thông số ghi lại. | Lưu latency/resource/counters; không tối ưu dựa trên một run ngẫu nhiên hoặc bỏ validation. | V-07 |
| P09-TC08 / P09-AC08 | Artifact version/checksum và workspace tự soạn. | Không dùng cargo run thay cho tất cả artifact checks; target chưa test bị ghi rõ. | V-08 |
| P09-TC09 / P09-AC09 | Outputs tự sinh từ các trường hợp Q36. | Đối chiếu source/bundle/output; ảnh hợp lệ giữ byte; không coi validator pass là đủ visual QA. | V-09 |
| P09-TC10 / P09-AC10 | Clean staging directory và artifact manifest. | List nội dung/checksum, giải nén/chạy help/version ở target; không tải sách/key để test. | V-10, V-00 |
| P09-TC11 / P09-AC11 | Documentation walkthrough với artifact và evidence matrix. | Lệnh hướng dẫn được kiểm chứng; claim có bằng chứng, không dựa hình dạng code cũ. | V-01, V-08 |
| P09-TC12 / P09-AC12 | Candidate ID/checksums, complete release checklist. | Audit từng claim tới bằng chứng; có artifact/notes cụ thể trước hành động phát hành được giao. | V-01, V-00 theo candidate cuối |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo trong task tương ứng; nếu đổi phải cập nhật kế hoạch. Lệnh exit 0 với 0 ca liên quan không được ghi là đạt. 

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | Rà requirement→test→evidence→candidate/target; kiểm trạng thái pending/failed/pass và các link/artifacts. |
| V-02 | cargo test -p novel-translate --test quality_corpus_contract --locked; người rà Nhật–Việt xác nhận oracle và pending cases |
| V-03 | Đợt đánh giá thủ công dùng executor thật với phạm vi/model/ngân sách đã được người dùng chọn; lưu lệnh/config/run ID thực tế trong báo cáo, không chạy bằng CI |
| V-04 | cargo test -p novel-translate --test system_recovery --locked |
| V-05 | cargo test -p nt-storage --test migration_compatibility --locked |
| V-06 | cargo test -p novel-translate --test security_acceptance --locked |
| V-07 | Chạy workload/resource suite đã định ở T01 với lệnh và máy được ghi trước; lưu số đo/profiling và kết quả so ngưỡng |
| V-08 | cargo test -p novel-translate --test release_cli --locked; kiểm artifact/terminal thật theo target matrix |
| V-09 | cargo test -p nt-export --test epubcheck_integration --locked; manual corpus trong Calibre + reader mục tiêu |
| V-10 | Build/package bằng pipeline đã định; kiểm artifact manifest/checksum và smoke --help/--version trên máy sạch |

**V-00 — Bộ kiểm tra Rust bắt buộc trước PR Rust và trước đóng phase có thay đổi Rust:**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

CI chạy offline với model/search/read/credential doubles và localhost cho IPC/Preview được kiểm soát. Dependency/EPUBCheck/browser chuẩn bị trước test. Live model evaluation V-03 là đợt thủ công riêng, có ngân sách và quyền dùng dịch vụ rõ; không cài mặc định để tự chạy khi test/build. Lệnh đo/packaging/evaluation chưa cố định phải được ghi chính xác trước đợt chạy, không báo là đã thực hiện từ kế hoạch này.

### 13.2. Cách dùng bằng chứng

Test phải kiểm tra kết quả quan sát được ở boundary, cả ca thành công và lỗi làm sai dữ liệu/quyền. Không viết test chỉ nhắc lại implementation. Đối với fault injection, ghi chính xác điểm ngắt, trạng thái trước/sau reopen và side effect đã/không xảy ra. Lỗi CI/môi trường và lỗi sản phẩm được phân biệt; chưa chạy thì ghi chưa xác minh.

<a id="p09-cases"></a>

### 13.3. Ca thử chi tiết theo task

Các ca chi tiết dùng nhóm V của TC gốc. Ghi target, OS/filesystem/terminal/reader, toolchain, hash candidate và fixture; đánh dấu kết quả đạt, lỗi hoặc còn chờ có lý do. Fault test cần marker chứng minh tiến trình chạm điểm ngắt, trạng thái trước/sau và side effects. Ghi phép kiểm mất điện/độ bền theo ADR-004 riêng với phép kill tiến trình. Chỉ lưu dữ liệu mẫu tự soạn hoặc có quyền dùng đã được chọn cho đợt kiểm.

| Mã ca | Fixture và thao tác | Kết quả cần đối chiếu |
|---|---|---|
| P09-TC01a | Một yêu cầu có cả ca tự động và ca mở reader thật. | Ma trận có hai bằng chứng riêng trên đúng candidate, thiếu một ca thì hàng còn chờ. |
| P09-TC01b | Máy build Windows và một target Linux chỉ mới build chéo. | Phạm vi hỗ trợ chỉ gồm target có gate phù hợp, ghi Linux native hoãn. |
| P09-TC01c | Thay đổi ngưỡng hoặc phạm vi sau khi đo. | Bản ma trận mới ghi quyết định/version và những phép đo phải chạy lại. |
| P09-TC02a | Rà manifest của 30 tình huống và 3 chương. | Đủ ca có ID ổn định, nguồn tự soạn và nhóm yêu cầu Q36. |
| P09-TC02b | Một ca có hai bản Việt hợp nghĩa và một bản sai phủ định. | Oracle chấp nhận cả hai cách nói đúng và ghi lỗi cho bản sai. |
| P09-TC02c | Người rà còn bất đồng về chủ thể hoặc mức lỗi. | Ca mang nhãn chờ duyệt, bảng tổng hợp chưa tính là oracle đạt. |
| P09-TC02d | Tạo fixture EPUB 2/3, tài nguyên thiếu và path bất hợp lệ. | Chỉ generator/nguồn/kỳ vọng nằm trong Git, file sinh ở thư mục tạm. |
| P09-TC03a | Thiếu phê duyệt phạm vi/ngân sách hoặc chạm cap khi đang chạy. | Lượt chưa được phép không gửi. Báo đúng ca chạy/chưa chạy và chi phí. |
| P09-TC03b | Reviewer phát hiện đủ lỗi S3 cài nhưng quote sai một ca. | Report tách detection và lỗi dẫn nguồn, cổng chất lượng chưa đạt. |
| P09-TC03c | Bản dịch đúng diễn đạt khác bị gắn lỗi. | Ghi báo oan và kết quả người rà thay vì cộng vào lỗi nghĩa thật. |
| P09-TC03d | Usage chưa xác định hoặc phiên model thay đổi giữa run. | Báo phần chưa xác định và phân tách theo phiên bản thực dùng. |
| P09-TC04a | Kill trước/sau DB commit khi object đã ghim hoặc Skill đang publish. | Đọc lại được tập căn cứ nhất quán hoặc báo recovery, không dùng bộ ghép phiên. |
| P09-TC04b | Kill sau intent, sau gửi nhưng chưa response, và lúc A nhả permit cho B. | Hồ sơ phí/quyền được phục hồi đúng. Lượt có outcome chưa rõ không bị gửi lại tự động. |
| P09-TC04c | Kill sau publish EPUB, trước ghi history. | Hash và operation ID đối chiếu để hoàn tất một lần, file cũ giữ nguyên. |
| P09-TC04d | Mô phỏng file tham chiếu thiếu/sai hash và chạy phép thử durability đã chốt. | Dừng ghi an toàn, có bằng chứng restore. Kết quả kill process và durability được ghi riêng. |
| P09-TC05a | Migrate từng phiên bản Rust được hỗ trợ có revisions, task và export history. | Schema mới đọc đúng nội dung, hash và lịch sử. |
| P09-TC05b | Fail giữa lúc thay DB và objects hoặc backup thiếu asset. | Chưa coi upgrade thành công, recovery kiểm cùng bộ DB và file trước khi mở ghi. |
| P09-TC05c | Mở schema mới hơn hoặc workspace Go thử nghiệm. | Từ chối an toàn và checksum dữ liệu nguồn giữ nguyên. |
| P09-TC05d | Một tiến trình giữ khóa ghi hoặc upgrade đã hoàn tất bị chạy lại. | Không ghi cạnh tranh hay lặp migration đã áp dụng. |
| P09-TC06a | Fixture ZIP traversal/collision, XML entity, SVG/CSS active và trang Research chứa chỉ dẫn. | Không đọc/ghi ngoài phạm vi hoặc thực thi nội dung mẫu, có diagnostic. |
| P09-TC06b | Process khác profile hoặc asset request với Host/token sai. | Từ chối trước truy cập quyền chạy hoặc bytes đã ghim. |
| P09-TC06c | Provider trả lỗi kèm sentinel trong khi báo cáo và tạo output. | Sentinel không xuất hiện ở các đích công khai hoặc file đóng gói. |
| P09-TC06d | Đóng Preview hoặc hết hạn rồi replay token và asset URL. | Quyền bị thu hồi và asset không được phục vụ. |
| P09-TC07a | Import nhiều entry và byte giải nén sát/vượt cap Q36. | Ca sát giới hạn theo policy, ca vượt dừng trước sử dụng dữ liệu và không để publish dở. |
| P09-TC07b | Mở history lớn rồi lọc, cuộn và resize trong terminal. | Số đo theo workload đã chốt, focus/nháp giữ đúng. |
| P09-TC07c | Ảnh lớn, nhiều frames hoặc EPUB chứa fixed-layout. | Decode/export tuân caps, số đo và cách xử lý được ghi trước khi kết luận. |
| P09-TC08a | CLI đọc workspace có đường dẫn Nhật–Việt rồi chạy luồng fake đã kiểm soát. | Exit/envelope/next_action đúng, không gọi provider trả phí. |
| P09-TC08b | TUI 80×24, 120×24, thu nhỏ rồi mở rộng trong khi có nháp. | Giữ ngữ cảnh và cleanup đúng khi thoát trên terminal đích. |
| P09-TC08c | Nhập và dùng tham chiếu kết nối thử theo adapter platform. | Đúng kho/quyền OS và không lộ giá trị giả trong log. |
| P09-TC09a | EPUB 2 có NCX và EPUB 3 có fixed-layout, giữ trang phụ theo story. | Nav/spine và phần giữ nguyên đúng, EPUBCheck đạt cho đúng hash. |
| P09-TC09b | File hợp lệ cấu trúc nhưng note link hoặc ruby hiển thị sai trong reader. | Ghi reader/version và quan sát sai, chưa đánh dấu đạt phần mở đọc. |
| P09-TC09c | Xuất lại sau đổi ảnh/metadata. | Version mới dùng inputs mới, các bản trước và báo cáo hash còn truy được. |
| P09-TC10a | Cài hoặc giải nén theo README trên Windows sạch, Go không có trong PATH. | Binary Rust chạy help/version/TUI và có hướng dẫn runtime cần thiết. |
| P09-TC10b | Đổi một byte artifact sau khi tạo checksum. | Bước kiểm phát hiện sai, kết quả từ gói gốc không chuyển sang gói đã đổi. |
| P09-TC10c | Kiểm manifest gói và license/runtime files. | Đúng executable, phiên bản và phụ thuộc, không kèm dữ liệu, key hoặc tài liệu nội bộ. |
| P09-TC11a | Làm theo README trên môi trường sạch với workspace mẫu. | Lệnh và đường dẫn khớp bản phân phối, người dùng biết cách phục hồi. |
| P09-TC11b | Một capability Go còn được dùng nhưng Rust thiếu bằng chứng. | Hàng có trạng thái còn thiếu, caller và đầu mối xử lý. |
| P09-TC11c | Upgrade từ Rust schema cũ và trường hợp Go thử nghiệm. | Hướng dẫn phân biệt migration có backup với tạo mới/import lại theo Q06. |
| P09-TC12a | Có bằng chứng đã đạt nhưng hash artifact khác bản dự định phát hành. | Hàng cần kiểm lại được nêu rõ trước verdict. |
| P09-TC12b | Còn ca live eval, durability hoặc reader bắt buộc chưa chạy. | Dossier giữ gate tương ứng chờ và ghi cách hoàn tất. |
| P09-TC12c | Các gate M9 và từng capability thay Go có B Đạt. | Dossier ghi bằng chứng, mốc Go cuối và kế hoạch review lượt dọn riêng. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md): A thực hiện task, kiểm thử và tự review rồi bàn giao **Chờ review**; B đối chiếu yêu cầu, kiểm bản bàn giao và tự chạy kiểm chứng phù hợp trong phiên riêng. B ghi báo cáo độc lập, không sửa source/test chính thức hoặc trạng thái PLAN. A xử lý findings; B kiểm tra lại trên candidate mới. Quy tắc này áp dụng cả task tài liệu/ADR/thử nghiệm của phase.

- Mỗi acceptance claim có evidence cùng candidate/version/platform?
- Oracle có độc lập và đủ người xác nhận, hay mô hình vừa làm vừa tự chấm?
- Fault campaign có kiểm dữ liệu/tiền/quyền sau restart?
- Migration backup/restore bao cả file và DB, không chỉ copy DB?
- Gói phát hành có chứa docs riêng tư, secret, fixture output hoặc claim support chưa test?

Rà thêm diff ngoài phạm vi, API công khai không có consumer, dependency cấm, error bị nuốt và test không được phát hiện/chạy. Không xóa hoặc nới test chỉ để qua kiểm tra. Sửa lỗi ở owner của quy tắc thay vì thêm ngoại lệ trong UI.

## 15. Điều kiện đóng phase

- [ ] Mọi task/nhóm trong phạm vi cổng có review B **Đạt** còn hiệu lực; đủ AC, kiểm tra bắt buộc và không còn finding chặn. A không tự duyệt phần mình triển khai.
- [ ] B đã review bản tích hợp cuối theo [mẫu review phase](../templates/PHASE_REVIEW.md); candidate và căn cứ khớp, bằng chứng các task còn áp dụng hoặc đã được kiểm lại theo ảnh hưởng.

- [ ] Q36 corpus 30+3 có oracle được xác nhận; module chất lượng đáp ứng seeded S3/quote và false-positive adjudication.
- [ ] Recovery/migration/security/performance/platform/EPUB reader gates đã đạt theo scope.
- [ ] Artifact, hướng dẫn, known limitations và parity matrix khớp candidate cuối; live eval chưa đủ thì chưa đóng gate chất lượng tương ứng.
- [ ] Có release dossier cụ thể; hành động publish/thay Go chỉ thực hiện khi đã được giao và đủ điều kiện.
- [ ] Tất cả task và AC của phase có bằng chứng, điểm chặn đã được giải quyết; ca chưa kiểm tra được ghi đúng trạng thái.
- [ ] V-00 áp dụng cho thay đổi Rust đã đạt; checks riêng trong mục 13 và các kiểm tra Go bị ảnh hưởng đã có kết quả.
- [ ] README được cập nhật nếu command/layout/nền tảng thay đổi; tài liệu nội bộ, key, EPUB, workspace và đầu ra tiếp tục ngoài Git.
- [ ] Bàn giao mục 16 đủ cho quyết định phát hành. Nếu đề xuất dọn Go, parity đã có bằng chứng và lượt dọn có phạm vi riêng theo mục 16.3.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md); B dùng [mẫu review task](../templates/TASK_REVIEW.md), và [mẫu review phase](../templates/PHASE_REVIEW.md) khi nghiệm thu cổng. Báo cáo nội bộ đặt theo lượt trong `docs/roadmap/evidence/phase-09/`: `P09-TYY.handoff-r01.md` và `P09-TYY.review-r01.md`; giữ lượt trước khi sửa. Hồ sơ cổng dùng ID M0-01/M0-02/M0 hoặc M1…M9 theo phạm vi. Các mẫu chưa phải bằng chứng thực thi.

Sổ task/cổng khi thực hiện ghi trạng thái, candidate, link báo cáo A, review B, finding còn mở và bước tiếp theo. A phase cập nhật sổ này và gửi điều phối cập nhật PLAN trung tâm sau bàn giao/kết luận; B chỉ ghi báo cáo review. Thiếu review B hoặc kiểm tra bắt buộc thì chưa đánh dấu hoàn thành.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 9 checkpoint của lịch v2: [P09-CP01](#p09-cp01), [P09-CP02](#p09-cp02), [P09-CP03](#p09-cp03), [P09-CP04](#p09-cp04), [P09-CP05](#p09-cp05), [P09-CP06](#p09-cp06), [P09-CP07](#p09-cp07), [P09-CP08](#p09-cp08), [P09-CP09](#p09-cp09); xem [§10.2](#p09-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task | Trạng thái | Bằng chứng |
|---|---|---|---|
| P09-A | T01–T03 | Chưa bắt đầu | Chưa có |
| P09-B | T04–T06 | Chưa bắt đầu | Chưa có |
| P09-C | T07–T09 | Chưa bắt đầu | Chưa có |
| P09-D | T10–T12 | Chưa bắt đầu | Chưa có |

### 16.2. Báo cáo mỗi task

Ghi task/AC, thay đổi hành vi, file/module, quyết định và tác động migration/compatibility; lệnh đã chạy, toolchain/OS, tên/số ca liên quan, kết quả; test thủ công và ca chưa chạy; rủi ro còn lại và task có thể nhận tiếp. Nếu lưu log/báo cáo, dùng `docs/roadmap/evidence/phase-09/`, đã redaction, không chứa dữ liệu cá nhân hoặc nội dung sách thật.

### 16.3. Đầu ra cho bước tiếp theo

Bàn giao candidate/artifact checksums, release notes, support/platform matrix, schema/backup/recovery instructions, kết quả automated/manual/live eval, known issues và parity decision. Nếu còn thiếu bằng chứng, giữ phase/gate chưa hoàn thành và chỉ rõ task cần tiếp tục. Việc lập đầy đủ kế hoạch Phase 00–09 không thay thế kết quả nghiệm thu sản phẩm.

#### Bàn giao lượt dọn Go cuối

T11/T12 đối chiếu [bảng chuyển dần Go → Rust](PHASE-03.md#p03-go-retirement) với từng chức năng được cam kết thay thế và bằng chứng trên bản Rust dự kiến phát hành. Nếu còn đường Go người dùng cần, ghi rõ chức năng, caller, nền tảng và lý do để xử lý trước.

Khi M9 và phạm vi thay thế đã được B xác nhận Đạt, ghi lượt dọn cuối thành việc riêng trong PLAN. Liệt kê code Go, `go.mod`/`go.sum`, script, Docker, CI, release và hướng dẫn nào đã hết caller. Ghi mốc Go cuối có thể quay lại. Bản dọn cần build/test/chạy/đóng gói Rust trên môi trường sạch không cần Go, kiểm lại dữ liệu và có review riêng trước PR. Giữ lịch sử bản Go và dữ liệu người dùng. Merge vào rewrite/rust theo MERGE_WORKFLOW sau đủ cổng; người dùng quyết định chuyển đường phát hành theo R-REPO.

Trạng thái kế hoạch không phải trạng thái implementation. Chỉ cập nhật hoàn thành khi có bằng chứng của hành vi thực tế.

<a id="p09-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử gói Rust ứng viên phát hành của **M9** đúng với hệ điều hành và kiến trúc ghi trong hồ sơ. A cung cấp version, file gói, SHA-256, hướng dẫn cài/chạy, bộ mẫu và báo cáo tương thích. B kiểm lại với đúng gói và ghi kết quả theo mục 15.

#### 16.4.2. Chuẩn bị

Dành khoảng 20–30 phút sau khi có gói và bộ mẫu. Chuẩn bị thư mục thử sạch, bộ mẫu EPUB/workspace theo phiên bản schema được hỗ trợ và trình đọc ghi trong hồ sơ. Các ca dưới đây dùng luồng offline đã bàn giao. Đợt đánh giá model thật thực hiện riêng theo V-03 khi đã có phạm vi và ngân sách đã chọn.

#### 16.4.3. Mở đúng bản

Trên Windows, dùng `Get-FileHash -LiteralPath <file-gói> -Algorithm SHA256` với đường dẫn gói thực tế để đối chiếu SHA-256. Mở executable bên trong gói theo hướng dẫn cài, chạy `--help` và `--version` rồi mở TUI. Phiên bản phải khớp hồ sơ M9.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P09-UT01 — Gói chạy được trong môi trường sạch | Cài hoặc giải nén theo quick start. Chạy CLI và mở TUI từ đúng gói, rồi thoát. | Lệnh và đường dẫn trong quick start dùng được. Phiên bản và nền tảng khớp hồ sơ, terminal dùng bình thường sau khi thoát. |
| P09-UT02 — Đi từ nguồn tới EPUB | Dùng lệnh của bộ thử offline để chạy luồng mẫu từ nhập đến xuất trên ứng viên M9. Mở file xuất trong hai trình đọc ghi trong hồ sơ. | File xuất đúng phiên bản và khớp bảng mẫu ở mục lục, chữ, ảnh và chú thích. |
| P09-UT03 — Mở bản sao dữ liệu Rust cũ | Dùng bản sao workspace mẫu thuộc phiên bản được hỗ trợ. Mở hoặc nâng cấp theo quick start, rồi đối chiếu nguồn, bản sửa tay, quy tắc và lịch sử xuất. | Mở được bằng bản M9 và dữ liệu đã lưu khớp bảng trước/sau. Bộ mẫu gốc vẫn dùng được cho lần kiểm tiếp. |
| P09-UT04 — Đối chiếu chức năng cần thay Go | Mở bảng tương đương tính năng trong hồ sơ M9. Chọn các chức năng cậu đang dùng và xem bằng chứng trên bản Rust này. | Mỗi chức năng có kết quả và phạm vi đã kiểm rõ ràng. Ghi lại phần còn thiếu để dùng khi quyết định thời điểm dọn Go. |

#### 16.4.5. Thử ca lỗi và mở lại

**P09-UT05 — Từ chối schema mới hơn và mở lại bản hợp lệ.** Dùng mẫu schema mới hơn đã chuẩn bị để thử mở. Ca đạt khi app báo không hỗ trợ và bản mẫu vẫn nguyên theo phép đối chiếu trong hồ sơ. Sau đó mở lại workspace đã kiểm ở UT03 và đối chiếu dữ liệu. Ca khôi phục backup có thể chạy thêm bằng kịch bản V-05 trên bản sao riêng.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Kèm version/hash gói, OS, reader nếu dùng, file mẫu, thao tác và kết quả mong đợi/thực tế. Với chức năng còn thiếu bằng chứng, ghi tên chức năng và nền tảng đã thử.

#### 16.4.7. Thử lại từ đầu

Tạo thư mục sạch, giải nén lại đúng gói đã kiểm hash và chép mới các workspace mẫu. Dùng thư mục xuất của workspace mẫu mới khi chạy lại.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-09. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P09 rà **Phần Go còn lại, go.mod/go.sum, script, Docker, CI và build/release** tại các mốc **P09-CP08/CP09 và lượt riêng GO-FINAL sau M9**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). T11/T12 nhận sổ dọn và parity theo từng khả năng, không mở lại phần đã có bằng chứng còn hiệu lực. GO-FINAL là việc bắt buộc để kết thúc chuyển đổi: kiểm candidate sau dọn trên môi trường sạch không cần Go, cập nhật artifact/checksum và review lại bằng chứng bị ảnh hưởng; merge vào rewrite/rust theo MERGE_WORKFLOW sau đủ cổng; không tự publish.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
