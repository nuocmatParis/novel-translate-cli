# PHASE 07 — ẢNH, PREVIEW VÀ XUẤT EPUB 3.3

> Lịch task song song và checkpoint commit/push tại [mục 10](#p07-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Draft 0.4 — 09/11/2026 · Chưa triển khai.** Lập từ đặc tả và thiết kế theo yêu cầu của người dùng; code hiện tại không là chuẩn thiết kế, đầu vào tái sử dụng hoặc bằng chứng hoàn thành. Lịch thực thi v2 ngày 12/11/2026; thay lịch v0.1 chưa triển khai.
>
> Cấu trúc theo [Phase 01](PHASE-01.md). Task, đường dẫn module và tên test là kế hoạch ban đầu; rà lại trước khi giao việc khi contract của phase trước thay đổi. Không tự thay yêu cầu đã chốt để khớp implementation.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P07 / M7 |
| Tên | Ảnh, Preview và xuất EPUB 3.3 |
| Trạng thái | Chưa bắt đầu; mọi AC và kiểm thử bên dưới chưa có kết quả thực thi |
| Đầu vào chính | M6 readiness/Review/exception; M2 source/resources; M3 generation/immutable store |
| Gói giao diện | UI-11, UI-12, UI-13 |
| Phân rã | 6 gói việc, 15 task |
| Nhánh khi triển khai | rewrite/rust; giữ Go theo R-REPO |
| Người thực hiện / rà soát | Model A triển khai / model B khác review độc lập; chưa phân công |

Xem [roadmap chung](../ROADMAP.md) và [phạm vi tại Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-07). Tài liệu và bằng chứng nội bộ ở trong `docs/`, ngoài Git.

## 2. Mục tiêu và demo cuối phase

Chuẩn bị một bundle bất biến từ nội dung đã đủ điều kiện, xem Preview an toàn và xuất EPUB 3.3 phiên bản mới bằng cùng bộ dữ liệu đó. Mọi lỗi ảnh/độ phủ/package hoặc publication được báo rõ và không làm mất bản xuất trước.

1. Dùng tập fixture đã Review, đặt một ảnh thay hợp lệ và xem ảnh gốc/ảnh chọn.
2. Thử ảnh thay khớp nhưng hỏng: thấy đúng file gây chặn; sửa file rồi chuẩn bị lại.
3. Mở Preview, đổi ảnh/metadata; thấy bản xem đã cũ và nạp bundle mới.
4. Xuất r001, r002; thử r003 thất bại rồi lệnh mới lấy r004; resume chính r003 đối chiếu r003.
5. Ngắt sau publish trước history, mở lại và khôi phục đúng bản theo ID/hash.
6. Readback, EPUBCheck 5.3.0 và mở fixture trên Calibre cùng một trình đọc mục tiêu.

## 3. Phạm vi

Readiness/coverage gate; image matching/validation/stable capture; immutable PreparedBundle; safe XHTML/CSS; EPUB 3.3 conversion/package/readback/validation; version reservation/no-clobber publication/recovery; Preview sessions/asset isolation; UI-11/12/13 và check/preview/export.

OCR/dịch/chỉnh ảnh tự động, xuất sách dở dang, web UI quản lý ứng dụng và dịch nội dung còn thiếu trong lúc packaging đều ngoài phạm vi. Metadata/ảnh thay đổi chỉ dựng lại phần phụ thuộc, không buộc dịch lại chữ còn hiệu lực.

## 4. Điều kiện bắt đầu

P07-T02 nhận inventory/source store đã import ở P02-T09 qua P02-CP05; T03 tiếp nối để công bố checkpoint ảnh sớm. P07-T01 chỉ mở khi nhận P06-T08/T12, P02-T08 và P04-T09. Bundle/render/export không dùng readiness giả để nghiệm thu. P07-T15/M7 vẫn cần M6; các giao diện đã review/công bố được P08 nhận trước M7.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-IMAGE | [Ảnh nguồn và ảnh thay](../../spec/IMAGE_EDIT_SPEC.md) |
| S-OUTPUT | [EPUB đầu ra](../../spec/EPUB_OUTPUT_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-REVIEW | [Giai đoạn Review](../../spec/REVIEW_STAGE_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-MODULE | [Bước 4 — Module và ownership](../../desgin/MODULE_DESIGN_STEP4.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Ký hiệu Qxx trong tài liệu là quyết định xx của S-Q. Hành vi đã chốt ưu tiên hơn phương án kỹ thuật còn đề xuất. Thư viện, tên bảng/trait và SQL nháp phải được rà theo consumer thực tế; không tự áp dụng DDL thiết kế vào workspace.

- Chỉ xuất tập hoàn chỉnh theo content mode: đủ phần cần dịch/Review và đủ phần giữ nguyên/tài nguyên; ngoại lệ không miễn technical gates.
- Ảnh edit khớp chính xác relative path/case/extension; có bản khớp hỏng phải chặn, không fallback source im lặng.
- JPEG/PNG/GIF/WebP/SVG tĩnh an toàn; file hợp lệ giữ nguyên byte; không tự crop/convert để sửa bố cục.
- Mặc định thử nghiệm: 64 MiB/ảnh, 16.384 px/chiều bitmap, 40M px/frame, 100M px tổng animation; kiểm resource thực, cấu hình được.
- Bundle manifest ghim snapshot/mapping/content mode/revisions/Reviews/exceptions/rules/ruby/metadata/layout và ảnh source/edit hashes.
- Quét cả đường dẫn edit có thể xuất hiện mới; stable read trước pin; file đổi giữa đọc cần chuẩn bị lại.
- EPUB 3.3, zip và quick-xml theo Q32; mimetype đầu gói và không nén; Nav/NCX/links/metadata đúng; tiếng Việt ngang trái sang phải.
- Export không gọi model; title/TOC/notes/ruby dùng kết quả/quyết định đã chấp nhận.
- Preview loopback, cổng tạm, secret ngẫu nhiên 256 bit dùng một lần qua URL fragment rồi POST cùng origin; không đưa token vào log/referrer.
- Phiên quyền giữ trong memory trang tin cậy, tài nguyên theo ID/allowlist; đóng/tiến trình dừng/30 phút idle kết thúc.
- Lệnh mới sau r003 thất bại lấy r004; resume r003 dùng lại hồ sơ r003; file hiện có khác hash luôn conflict.

## 6. ADR và điểm còn mở

| Vấn đề | Cần trước | Nội dung |
|---|---|---|
| DB-15 và ADR-008 export | T04/T08/T09 | Manifest/version identity, reserve, validation proof, publish/reconcile |
| Image decoder/SVG policy | T02/T03 | Byte/dimension/frame/memory budgets và stable capture |
| Sanitizer và preview isolation | T05/T10/T11 | Allowlist XML/CSS, frame/CSP, Host/origin/auth, session lifecycle |
| Metadata/layout/ruby | T04/T06 | Rule→render mapping, source giữ nguyên và approved semantic ruby |
| EPUBCheck/readers | T07/T15 | Tool version pin 5.3.0, warning classification và reader/platform evidence |

Trạng thái ban đầu của các quyết định trên là cần đối chiếu/ghi nhận, không phải đã được thông qua. Dùng ADR đã có nếu còn phù hợp; chỉ mở rộng phần mới. Ghi lựa chọn, đánh đổi, task phụ thuộc và phép kiểm chứng. Chỉ phần thật sự thay đổi yêu cầu sản phẩm mới cần quyết định sản phẩm bổ sung.

## 7. Thiết kế triển khai

`nt-export` hiện thực image/bundle/render/package/Preview IO qua Application ports. Application quyết định readiness, chọn inputs đã được chấp nhận, cấp PreparedBundle/ExportVersion và quyền publication. Storage giữ immutable objects/operations/history; importer cung cấp source mapping, không bị exporter sửa lại để vừa package.

Luồng: readiness → stable capture → immutable manifest → sanitized render model → Preview hoặc EPUB temp → readback/EPUBCheck → durable export record → no-clobber publish → mark completed. Preview và EPUB dùng cùng bundle/layout semantics; không cam kết pixel/page count giống mọi reader.

Contract có consumer thực tế mới được công khai. Domain/Application giữ quy tắc và quyền công bố; adapter thực hiện IO qua port; TUI/CLI gọi cùng use case. Không nối hai adapter qua implementation nội bộ để né ownership hoặc tạo dependency vòng.

### 7.1. Một bộ dữ liệu dùng cho Preview và xuất

1. Tính readiness cho tập và scope yêu cầu. Khi đủ điều kiện, ghim source map, revisions, Review, quyết định, metadata, layout và byte ảnh đã kiểm vào một PreparedBundle theo hash. Đọc lại nguồn phụ thuộc khi dùng bộ đã chuẩn bị.
2. Từ bundle đã ghim, Preview phục vụ đúng nội dung và ảnh của bộ đó. Nếu nguồn phụ thuộc đổi, hiện Stale. Nạp lại tạo một bundle mới và chạy lại phần kiểm bị ảnh hưởng.
3. Lệnh xuất mới giữ số phiên bản theo tập và lưu operation ID. Render vào file tạm, đọc lại và gắn proof với hash file đó. Trước khi công bố, đối chiếu readiness và căn cứ theo DB-15. Sau khi công bố, hoàn tất lịch sử theo đúng operation ID.
4. Khi mở lại tác vụ, kiểm cả hồ sơ lẫn file đích. Nếu công bố đã xảy ra và đúng file đã kiểm, đối chiếu để hoàn tất cùng lần xuất. Nếu file đích thuộc lần khác hoặc sai hash, giữ file đó và báo xung đột.

Bàn giao cho M8 trạng thái và lỗi có thể hiển thị nhất quán trong TUI/CLI. Bàn giao cho M9 bundle ID, artifact hash, report EPUBCheck, reader/version và các điểm ngắt đã kiểm để chạy lại trên bản phát hành.

## 8. Dữ liệu, lỗi và phục hồi

| Trường hợp | Hành vi |
|---|---|
| Readiness thiếu/exception stale | Chặn đúng phần; không packaging nội dung thiếu |
| Ảnh đổi giữa read/validate | Bỏ candidate capture đó, chuẩn bị lại; không pin hash/bytes lệch |
| Dữ liệu đổi sau bundle | Bundle cũ vẫn bất biến nhưng đánh dấu stale; chuẩn bị bản mới theo phụ thuộc |
| Validate EPUB lỗi/cảnh báo ảnh hưởng nội dung | Giữ hồ sơ failed và diagnostic, không công bố |
| Reserve version xong rồi lỗi | Không dùng lại số cho lệnh mới; resume chính operation đối chiếu version cũ |
| Publish xong, history chưa done | Match export ID/destination/hash/report rồi reconcile; không overwrite |
| Destination có file khác hash | Conflict; giữ nguyên file đó và bản trước |
| Preview session mất quyền/idle | Chặn tài nguyên; không tự mở toàn workspace để fallback |

Trước publish phải kiểm bundle vẫn thỏa contract hiệu lực theo quyết định; không thay bytes của bundle giữa validate và publish. Preview request chỉ đọc, không có quyền tạo task/duyệt/export/ghi file tùy ý.

Mỗi lỗi cần giữ định danh phạm vi và nguyên nhân đã redaction, cùng bước tiếp theo. Không dùng thông báo thành công thay cho trạng thái đang chờ. Mọi thay đổi schema đã áp dụng phải có migration/version và kiểm thử tương thích; dữ liệu, lịch sử và sửa tay được bảo toàn.

## 9. Phần TUI/CLI

UI-11: ảnh source/edit, vị trí dùng, lỗi và chênh lệch kích thước/tỷ lệ. UI-12: readiness theo phạm vi, phần giữ nguyên, exceptions, Preview/current vs stale và nạp lại. UI-13: xuất, trạng thái kiểm tra/lỗi, lịch sử phiên bản/đường dẫn.

CLI `check --volume <id>`, `preview --volume <id>`, `export --volume <id>` qua cùng core. Preview là lựa chọn; người dùng có thể xuất trực tiếp khi đủ điều kiện. CLI non-interactive trả kết quả/lý do chặn, không tự mở lựa chọn hoặc dịch thêm.

Thông báo sản phẩm dùng ngôn ngữ người dùng hiểu được; mã task/ADR/crate chỉ nằm trong tài liệu kỹ thuật. Điều hướng, cuộn, resize và mở trợ giúp không tự gửi model hoặc thay quyết định nghiệp vụ.

<a id="p07-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P07 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P07-T02 nhận inventory/source store đã import ở P02-T09 qua P02-CP05; T03 tiếp nối để công bố checkpoint ảnh sớm. P07-T01 chỉ mở khi nhận P06-T08/T12, P02-T08 và P04-T09. Bundle/render/export không dùng readiness giả để nghiệm thu. P07-T15/M7 vẫn cần M6; các giao diện đã review/công bố được P08 nhận trước M7. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p07-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p07-cp01"></a>P07-CP01 — Kiểm và ghim ảnh | [P07-T02](#p07-t02), [P07-T03](#p07-t03) | [P02-CP01](PHASE-02.md#p02-cp01), [P02-CP05](PHASE-02.md#p02-cp05) | P07-T02: V-02 trong phạm vi AC/TC của task; P07-T03: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp02"></a>P07-CP02 — Readiness, bundle và render | [P07-T01](#p07-t01), [P07-T04](#p07-t04), [P07-T05](#p07-t05) | [P02-CP05](PHASE-02.md#p02-cp05), [P03-CP01](PHASE-03.md#p03-cp01), [P04-CP04](PHASE-04.md#p04-cp04), [P06-CP03](PHASE-06.md#p06-cp03), [P06-CP05](PHASE-06.md#p06-cp05), [P07-CP01](#p07-cp01) | P07-T01: V-01 trong phạm vi AC/TC của task; P07-T04: V-03 trong phạm vi AC/TC của task; P07-T05: V-04 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp03"></a>P07-CP03 — Dựng và kiểm EPUB | [P07-T06](#p07-t06), [P07-T07](#p07-t07) | [P07-CP02](#p07-cp02) | P07-T06: V-05 trong phạm vi AC/TC của task; P07-T07: V-05, V-06 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp04"></a>P07-CP04 — Reserve, publish và recovery | [P07-T08](#p07-t08), [P07-T09](#p07-t09) | [P07-CP02](#p07-cp02), [P07-CP03](#p07-cp03) | P07-T08: V-07 trong phạm vi AC/TC của task; P07-T09: V-07 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp05"></a>P07-CP05 — Preview server và viewer | [P07-T10](#p07-t10), [P07-T11](#p07-t11) | [P07-CP02](#p07-cp02) | P07-T10: V-08 trong phạm vi AC/TC của task; P07-T11: V-08 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp06"></a>P07-CP06 — Giao diện ảnh và Preview | [P07-T12](#p07-t12), [P07-T13](#p07-t13) | [P07-CP01](#p07-cp01), [P07-CP02](#p07-cp02), [P07-CP05](#p07-cp05) | P07-T12: V-09 trong phạm vi AC/TC của task; P07-T13: V-09 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-09 phần ảnh/readiness/Preview của T12/T13; export/history thuộc CP07. V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp07"></a>P07-CP07 — Xuất bản và lịch sử | [P07-T14](#p07-t14) | [P07-CP03](#p07-cp03), [P07-CP04](#p07-cp04), [P07-CP06](#p07-cp06) | P07-T14: V-09 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-09 phần export/history của T14 và hồi quy Preview đã có; reader acceptance toàn M7 ở CP08. V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |
| <a id="p07-cp08"></a>P07-CP08 — Nghiệm thu M7 | [P07-T15](#p07-t15) | M6 Đạt, [P07-CP01](#p07-cp01), [P07-CP02](#p07-cp02), [P07-CP03](#p07-cp03), [P07-CP04](#p07-cp04), [P07-CP05](#p07-cp05), [P07-CP06](#p07-cp06), [P07-CP07](#p07-cp07) | P07-T15: V-10, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M7; V-00 cho code, B review đúng candidate. | feature/epub-export; A chat P07 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Các đường dẫn dưới đây là vị trí mục tiêu từ thiết kế. Không sửa phần ngoài phạm vi chỉ để tiện triển khai. Tất cả task khởi tạo ở trạng thái **Chưa bắt đầu**; lệnh V-* nằm ở mục 13.

<a id="p07-t01"></a>

### P07-T01 — Readiness theo phạm vi và revision set

- **Mục tiêu / đầu ra:** Report đủ/thiếu cho toàn tập trước prepare/export.
- **Căn cứ / phụ thuộc:** Q24/25/33, S-OUTPUT; M6. Dependency thực thi: [P06-T08](PHASE-06.md#p06-t08), [P06-T12](PHASE-06.md#p06-t12), [P02-T08](PHASE-02.md#p02-t08), [P04-T09](PHASE-04.md#p04-t09).
- **Phạm vi:** Application export readiness; domain coverage/exception validity.
- **Việc làm:** Kiểm nguồn→dịch/dịch→nguồn, auxiliary text, chapter conclusion, rules/exception versions và preserved content.
- **Ca lỗi / biên:** Thân truyện đạt nhưng thiếu title/alt/note; exception cũ; story classification chưa chốt.
- **Nghiệm thu:** `P07-AC01` — Chỉ đủ mọi phần bắt buộc mới ready; retained source không được tính là đã dịch.
- **Kiểm chứng:** `P07-TC01`, V-01; dữ liệu kiểm tra: All-text/story, shared note, thiếu phụ trợ và stale Review.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Đọc source map, chế độ nội dung và revision set hiện hành. Xây bảng phần đã dịch/giữ nguyên/thiếu theo spine và phụ trợ.
2. Đối chiếu coverage, Review chương và điều kiện kỹ thuật, approval, rules, ngoại lệ trên cùng căn cứ. Trả blockers với vị trí và việc cần làm.
3. Phát hành report theo bộ căn cứ đã đọc cho prepare/export và giao diện. Nếu căn cứ đổi, tính lại readiness.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC01.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T01 |
|---|---|
| Đầu vào bắt buộc | [P06-T08](PHASE-06.md#p06-t08), [P06-T12](PHASE-06.md#p06-t12), [P02-T08](PHASE-02.md#p02-t08), [P04-T09](PHASE-04.md#p04-t09). Nhận coverage/scope, revision/Review sets và ngoại lệ còn hiệu lực M6. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Readiness trả trạng thái cùng blockers/retained parts theo exact basis, issue list rỗng chưa đủ chứng minh coverage. Owner đầu ra là A-P07-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T01 giữ Application export readiness; domain coverage/exception validity. Phần chung do M6 giữ quality/basis, T01 giữ readiness read model. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC01 / P07-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Report đúng item/reason; không chấp nhận issue list rỗng thay coverage. |
| Điểm tích hợp | [P07-CP02](#p07-cp02) — Readiness, bundle và render; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t02"></a>

### P07-T02 — Khớp và kiểm ảnh thay

- **Mục tiêu / đầu ra:** Lựa chọn source/edit với diagnostic thực.
- **Căn cứ / phụ thuộc:** Q30, S-IMAGE/S-SEC; P02 inventory. Dependency thực thi: [P02-T09](PHASE-02.md#p02-t09).
- **Phạm vi:** nt-export image matching/decoding/validation.
- **Việc làm:** Exact relative path/case/ext; sniff format; limits bytes/dimensions/frame memory; SVG static allowlist.
- **Ca lỗi / biên:** Edit hỏng, MIME sai, animation bomb, SVG script/external ref, case collision.
- **Nghiệm thu:** `P07-AC02` — Edit khớp hỏng chặn; file hợp lệ giữ nguyên byte; file ngoài mapping không tự được dùng.
- **Kiểm chứng:** `P07-TC02`, V-02; dữ liệu kiểm tra: Ảnh tự sinh các định dạng, malicious SVG và limit boundaries.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dò ảnh edit theo đúng tên, chữ hoa/thường, đuôi và đường dẫn tương đối trong inventory. Báo file dư và vị trí dùng chung tài nguyên.
2. Nhận định dạng được S-IMAGE cho phép, đọc header/cấu trúc trước giải mã và áp caps bytes, pixel, frames. Xác minh SVG tĩnh theo allowlist.
3. Nếu ảnh thay đúng tên nhưng hỏng, ghi blocker đúng file. Với ảnh hợp lệ khác tỷ lệ, giữ byte và hiển thị thông tin bố cục.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC02.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T02 |
|---|---|
| Đầu vào bắt buộc | [P02-T09](PHASE-02.md#p02-t09). Nhận inventory ảnh và source hash của M2, limits S-IMAGE/S-SEC. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Match chính xác relative path/case/ext, sniff/decode theo caps, edit đúng tên mà hỏng tạo blocker. Owner đầu ra là A-P07-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T02 giữ nt-export image matching/decoding/validation. Phần chung do P02-T05/T09 giữ inventory, T02 giữ image validation. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T01](PHASE-06.md#p06-t01), [P06-T02](PHASE-06.md#p06-t02), [P06-T03](PHASE-06.md#p06-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC02 / P07-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu Selected hash/diagnostic đúng; không fallback source khi edit hợp lệ tên nhưng lỗi. |
| Điểm tích hợp | [P07-CP01](#p07-cp01) — Kiểm và ghim ảnh; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t03"></a>

### P07-T03 — Stable image capture và change detection

- **Mục tiêu / đầu ra:** Assets bất biến cho ảnh chọn và ảnh gốc so sánh.
- **Căn cứ / phụ thuộc:** Q30/33, S-IMAGE; T02, immutable store P02. Dependency thực thi: [P07-T02](#p07-t02), [P02-T02](PHASE-02.md#p02-t02).
- **Phạm vi:** nt-export capture; storage objects; application invalidation.
- **Việc làm:** Đọc ổn định, verify/hash/pin; quét cả edit mới xuất hiện; ghi chosen/source identities.
- **Ca lỗi / biên:** File thay giữa đọc; edit mới; file bị xóa; symlink thoát root.
- **Nghiệm thu:** `P07-AC03` — Bundle chỉ dùng bytes đã kiểm; image changes làm stale phần phụ thuộc mà không xóa bản dịch chữ.
- **Kiểm chứng:** `P07-TC03`, V-02; dữ liệu kiểm tra: Writer thay bytes giữa read và directory có edit mới.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Đọc source/edit theo phiên file ổn định, kiểm hash và validation trước khi ghim asset. Đối chiếu source với snapshot gốc.
2. Ghi cả ảnh được chọn và ảnh gốc cần so. Theo dõi cả những đường dẫn edit chưa có để phát hiện ảnh thay mới.
3. Nếu file đổi trong lúc đọc, thử lại theo chính sách capture. Giữ diagnostic nếu chưa lấy được phiên ổn định, không công bố bundle thiếu.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC03.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T03 |
|---|---|
| Đầu vào bắt buộc | [P07-T02](#p07-t02), [P02-T02](PHASE-02.md#p02-t02). Nhận image decision và kho file bất biến M2. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Stable capture ghim chosen/source hashes, phát hiện file đổi giữa read và edit mới xuất hiện. Owner đầu ra là A-P07-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T03 giữ nt-export capture; storage objects; application invalidation. Phần chung do P02-T02 giữ store, T02 giữ decoder policy, T03 giữ capture identity. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T01](PHASE-06.md#p06-t01), [P06-T02](PHASE-06.md#p06-t02), [P06-T03](PHASE-06.md#p06-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC03 / P07-TC03, gồm các ca chi tiết §13.3. V-02. Đối chiếu Hash/bytes nhất quán; old bundle vẫn đọc đúng assets đã pin. |
| Điểm tích hợp | [P07-CP01](#p07-cp01) — Kiểm và ghim ảnh; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t04"></a>

### P07-T04 — PreparedBundle manifest bất biến

- **Mục tiêu / đầu ra:** Manifest đầy đủ input versions và reuse dependencies.
- **Căn cứ / phụ thuộc:** Q33, S-OUTPUT; T01/T03. Dependency thực thi: [P07-T01](#p07-t01), [P07-T03](#p07-t03), [P03-T02](PHASE-03.md#p03-t02), [P06-T12](PHASE-06.md#p06-t12).
- **Phạm vi:** Domain bundle manifest; application prepare; storage manifest refs.
- **Việc làm:** Pin snapshot/mapping/scope/retained parts/revisions/Reviews/exceptions/rules/ruby/metadata/layout/assets.
- **Ca lỗi / biên:** Input đổi giữa readiness và commit; thiếu asset/hash; metadata đổi sau prepare.
- **Nghiệm thu:** `P07-AC04` — Một manifest hash xác định đúng toàn bộ inputs; stale prepare không giả current.
- **Kiểm chứng:** `P07-TC04`, V-03; dữ liệu kiểm tra: Cùng data/reordered serialization và từng loại input thay đổi.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Định nghĩa manifest theo inputs và consumer thật, dùng định danh ổn định cho snapshot, mapping, các tập version và asset. Chuẩn hóa biểu diễn trước khi lấy hash.
2. Tạo bundle từ readiness và stable assets trên căn cứ nhất quán. Giữ data cần thiết để render và so ảnh về sau.
3. Khi cần làm mới, chỉ dựng lại phần bị ảnh hưởng bởi metadata, layout, quyết định hoặc ảnh. Tái sử dụng kết quả dịch chữ còn hợp lệ.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC04.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T04 |
|---|---|
| Đầu vào bắt buộc | [P07-T01](#p07-t01), [P07-T03](#p07-t03), [P03-T02](PHASE-03.md#p03-t02), [P06-T12](PHASE-06.md#p06-t12). Nhận readiness basis cùng pinned chosen/original assets. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Manifest bất biến gồm snapshot/mapping/scope/revisions/Reviews/exceptions/rules/ruby/metadata/layout và assets. Owner đầu ra là A-P07-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T04 giữ Domain bundle manifest; application prepare; storage manifest refs. Phần chung do T01/T03 giữ inputs, T04 giữ bundle identity. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC04 / P07-TC04, gồm các ca chi tiết §13.3. V-03. Đối chiếu Canonical hash ổn định theo contract; mỗi thay đổi liên quan làm invalidation đúng. |
| Điểm tích hợp | [P07-CP02](#p07-cp02) — Readiness, bundle và render; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t05"></a>

### P07-T05 — Render XHTML/CSS an toàn

- **Mục tiêu / đầu ra:** Nội dung hiển thị sạch dựa trên bundle.
- **Căn cứ / phụ thuộc:** Q31–32, S-SEC/S-OUTPUT; T04. Dependency thực thi: [P07-T04](#p07-t04).
- **Phạm vi:** nt-export render/sanitize/resource mapping.
- **Việc làm:** Allowlist elements/attributes/CSS; loại active/network/file refs; giữ semantic markup và ID references.
- **Ca lỗi / biên:** Script/events/form/embedded content, CSS URL leaks, malicious nested SVG, encoded traversal.
- **Nghiệm thu:** `P07-AC05` — Không thực thi/tải nguồn ngoài; text/anchor/ruby/nhấn mạnh hợp lệ còn đúng.
- **Kiểm chứng:** `P07-TC05`, V-04; dữ liệu kiểm tra: XHTML/CSS ác ý và nguồn hợp lệ có Unicode/note/ảnh.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dựng render model từ bundle, bảo toàn thứ tự, marker chú thích, liên kết nội bộ và ruby đã duyệt.
2. Parse XHTML/CSS rồi áp policy phần tử, thuộc tính, URL và tài nguyên. Báo phần cần xem lại khi lọc ảnh hưởng ý nghĩa.
3. Kiểm render theo allowlist trước khi dùng cho Preview và package. Giữ source snapshot và ghi provenance phần được chuẩn hóa.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC05.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T05 |
|---|---|
| Đầu vào bắt buộc | [P07-T04](#p07-t04). Nhận immutable manifest và asset IDs. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Render model XHTML/CSS loại active/external refs theo policy, bảo toàn semantic markup và links hợp lệ. Owner đầu ra là A-P07-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T05 giữ nt-export render/sanitize/resource mapping. Phần chung do T04 giữ bundle, T05 giữ sanitizer/render contract. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC05 / P07-TC05, gồm các ca chi tiết §13.3. V-04. Đối chiếu Rendered document/resource requests chỉ dùng allowed IDs; không mất nội dung hợp lệ. |
| Điểm tích hợp | [P07-CP02](#p07-cp02) — Readiness, bundle và render; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t06"></a>

### P07-T06 — Dựng EPUB 3.3 và chuyển nguồn EPUB 2

- **Mục tiêu / đầu ra:** Package/manifest/spine/Nav/metadata và layout đúng.
- **Căn cứ / phụ thuộc:** Q32, S-OUTPUT; T04–T05. Dependency thực thi: [P07-T04](#p07-t04), [P07-T05](#p07-t05).
- **Phạm vi:** nt-export epub/zip/xml/layout.
- **Việc làm:** Mimetype first stored; NCX→Nav giữ cấp/thứ tự/đích; Vietnamese LTR; fixed image pages; approved ruby policy; title/notes reuse.
- **Ca lỗi / biên:** Anchor mất, link note ngược sai, Part/request thành chương mới, ruby nghĩa bị bỏ, metadata thiếu.
- **Nghiệm thu:** `P07-AC06` — EPUB dùng đúng accepted content và preserved sources; không gọi model/đề xuất nghĩa khi export.
- **Kiểm chứng:** `P07-TC06`, V-05; dữ liệu kiểm tra: EPUB2/3, multi-file chapter, ruby phonetic/semantic và shared notes.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Tạo package EPUB 3.3 từ render model. Sinh manifest/spine/Nav và metadata tiếng Việt theo bản đã chốt, giữ thông tin nguồn cần đối chiếu.
2. Với EPUB 2, chuyển NCX theo cấp, thứ tự và đích thật. Kiểm các tài nguyên và fragment sau khi đổi đường dẫn.
3. Viết file tạm với mimetype đúng vị trí/kiểu nén, ảnh và layout theo bundle. Giữ bìa, trang ảnh, ruby, chú thích và hướng đọc theo S-OUTPUT.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC06.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T06 |
|---|---|
| Đầu vào bắt buộc | [P07-T04](#p07-t04), [P07-T05](#p07-t05). Nhận render model, metadata/layout/ruby cùng bundle hash. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Package EPUB 3.3 giữ spine/Nav/NCX conversion, mimetype và tài nguyên đúng bản ghim. Owner đầu ra là A-P07-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T06 giữ nt-export epub/zip/xml/layout. Phần chung do T04/T05 giữ inputs, T06 giữ packaging. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC06 / P07-TC06, gồm các ca chi tiết §13.3. V-05. Đối chiếu Readback XML/resource/content oracle, archive entry properties và navigation đúng. |
| Điểm tích hợp | [P07-CP03](#p07-cp03) — Dựng và kiểm EPUB; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t07"></a>

### P07-T07 — Readback và EPUBCheck

- **Mục tiêu / đầu ra:** Validation proof gắn đúng temp hash/tool version.
- **Căn cứ / phụ thuộc:** Q32/36, S-OUTPUT; T06. Dependency thực thi: [P07-T06](#p07-t06).
- **Phạm vi:** nt-export validator/EPUBCheck process wrapper.
- **Việc làm:** Mở đọc lại, kiểm XML/resource/link/coverage; chạy 5.3.0; classify errors/warnings; ghi report.
- **Ca lỗi / biên:** Tool thiếu/sai version, nonzero/timeout, malformed report, warning ảnh hưởng nội dung.
- **Nghiệm thu:** `P07-AC07` — Chỉ file đúng hash đã qua validation được eligible publish; error/meaningful warning chặn.
- **Kiểm chứng:** `P07-TC07`, V-05, V-06; dữ liệu kiểm tra: Valid/invalid temp EPUB và reports/tool process fixtures.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Mở file tạm và đối chiếu container, XML, references, coverage và nội dung bắt buộc với manifest. Ghi chênh lệch có thể truy về source.
2. Chạy EPUBCheck 5.3.0 đã chuẩn bị. Lưu version, lỗi/cảnh báo và kết quả phân loại, xử lý lỗi ảnh hưởng nội dung trước publish.
3. Gắn report và proof với hash đúng file temp. Kiểm lại hash ở boundary công bố.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC07.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T07 |
|---|---|
| Đầu vào bắt buộc | [P07-T06](#p07-t06). Nhận temp file thật và expected bundle coverage/resources. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Readback/EPUBCheck 5.3.0 tạo proof theo temp hash/tool version, errors và warnings được phân loại. Owner đầu ra là A-P07-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T07 giữ nt-export validator/EPUBCheck process wrapper. Phần chung do T06 giữ package, T07 giữ validator proof. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC07 / P07-TC07, gồm các ca chi tiết §13.3. V-05, V-06. Đối chiếu Có test với EPUBCheck thật cài sẵn; mock wrapper không thay bằng chứng validator thật. |
| Điểm tích hợp | [P07-CP03](#p07-cp03) — Dựng và kiểm EPUB; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t08"></a>

### P07-T08 — Reserve số phiên bản và export operation

- **Mục tiêu / đầu ra:** Version reservation độc lập giữa lệnh mới và resume.
- **Căn cứ / phụ thuộc:** Q33, DB-15; T04, ADR publish. Dependency thực thi: [P07-T04](#p07-t04).
- **Phạm vi:** Application export begin/resume; storage versions/operations.
- **Việc làm:** Reserve theo volume trong transaction; lưu operation ID/state/path intent; không tái cấp số đã dùng.
- **Ca lỗi / biên:** Hai export cùng lúc, r003 fail rồi new command, resume wrong ID.
- **Nghiệm thu:** `P07-AC08` — Lệnh mới lấy r004; resume r003 đối chiếu r003; không có hai operation sở hữu cùng version.
- **Kiểm chứng:** `P07-TC08`, V-07; dữ liệu kiểm tra: Concurrent reservations và failed/resumed r003.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chốt sequence theo volume và identity của export intent trong ADR. Reserve số và operation record bằng một transaction trước render của lần xuất.
2. Tách lệnh xuất mới, retry ở mức transport và resume operation đã lưu. Lưu bundle, đích dự kiến và lý do thất bại.
3. Đọc lại journal để resume đúng số. Giữ lịch sử số đã cấp qua lỗi, cancel hoặc restart.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC08.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T08 |
|---|---|
| Đầu vào bắt buộc | [P07-T04](#p07-t04). Nhận bundle identity, volume và DB-15/ADR-008 đã chốt. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Reserve version/operation trong transaction, resume dùng cùng số, lệnh mới tăng sequence qua lỗi. Owner đầu ra là A-P07-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T08 giữ Application export begin/resume; storage versions/operations. Phần chung do T04 giữ bundle, T08 giữ export ledger/schema. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC08 / P07-TC08, gồm các ca chi tiết §13.3. V-07. Đối chiếu Unique/version sequence và durable history đúng qua restart. |
| Điểm tích hợp | [P07-CP04](#p07-cp04) — Reserve, publish và recovery; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t09"></a>

### P07-T09 — Publish giữ file cũ và recovery

- **Mục tiêu / đầu ra:** Công bố no-clobber với ID/hash/report có thể reconcile.
- **Căn cứ / phụ thuộc:** Q33, DB-15; T07–T08. Dependency thực thi: [P07-T07](#p07-t07), [P07-T08](#p07-t08).
- **Phạm vi:** nt-export publisher; storage export reconcile.
- **Việc làm:** Lưu hash/dest/proof trước publish; primitive không overwrite; finalize history; kiểm data đổi theo contract.
- **Ca lỗi / biên:** Crash sau publish trước done, dest khác hash, disk full, source temp đổi sau validate.
- **Nghiệm thu:** `P07-AC09` — Không đè file hiện có; reopen nhận đúng file đã publish hoặc conflict; không tạo bản mới cho cùng resume.
- **Kiểm chứng:** `P07-TC09`, V-07; dữ liệu kiểm tra: Failpoints temp/proof/publish/finalize và file sentinel ở dest.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Lưu validated hash, dest và operation/proof trước publish. Chốt các bước bảo đảm lưu bền và primitive chống ghi đè trên hệ đích.
2. Đối chiếu căn cứ theo contract rồi công bố file. Kiểm chứng file đích và finalize cùng operation.
3. Recovery phân biệt chưa công bố, đã công bố đúng hash và đích xung đột. Giữ nguyên các file trước và trả hành động tiếp theo.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC09.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T09 |
|---|---|
| Đầu vào bắt buộc | [P07-T07](#p07-t07), [P07-T08](#p07-t08). Nhận validated temp hash/proof và reserved operation/destination. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Publish giữ file đích, lưu proof trước công bố, recovery đối chiếu file/operation/history. Owner đầu ra là A-P07-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T09 giữ nt-export publisher; storage export reconcile. Phần chung do T07 giữ proof, T08 giữ ledger, T09 giữ publication protocol. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC09 / P07-TC09, gồm các ca chi tiết §13.3. V-07. Đối chiếu Hash/path/version/history cùng một operation; prior files byte-identical. |
| Điểm tích hợp | [P07-CP04](#p07-cp04) — Reserve, publish và recovery; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t10"></a>

### P07-T10 — Preview server và quyền phiên

- **Mục tiêu / đầu ra:** Loopback session có token một lần và lifecycle.
- **Căn cứ / phụ thuộc:** Q31, S-SEC; T04–T05. Dependency thực thi: [P07-T04](#p07-t04), [P07-T05](#p07-t05).
- **Phạm vi:** nt-export preview server/auth/session.
- **Việc làm:** Port tạm, 256-bit token fragment→POST same origin, xóa fragment, token access trong memory; Host/origin/idle/close checks.
- **Ca lỗi / biên:** Token dùng lại/sai, Host spoof, cross-origin, path traversal, idle hết hạn, server restart.
- **Nghiệm thu:** `P07-AC10` — Unauthorized không đọc tài nguyên; secret không xuất hiện ở URL query/log/referrer; session hết hạn bị thu hồi.
- **Kiểm chứng:** `P07-TC10`, V-08; dữ liệu kiểm tra: HTTP client giả/browser harness cục bộ, fake clock.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Mở server trên loopback cổng tạm. Tạo token một lần 256 bit, chuyển qua fragment tới trang tin cậy.
2. Đổi token bằng POST cùng origin, kiểm Host/origin, xóa fragment và giữ quyền phiên trong bộ nhớ. Lọc token khỏi log/referrer.
3. Thu hồi quyền khi đóng, process kết thúc hoặc hết thời gian không hoạt động. Kiểm thời hạn cả ở đường lấy asset.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC10.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T10 |
|---|---|
| Đầu vào bắt buộc | [P07-T04](#p07-t04), [P07-T05](#p07-t05). Nhận bundle/render và S-SEC session policy. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Loopback session dùng one-time token handshake, kiểm Host/origin, thu hồi khi close/idle theo policy. Owner đầu ra là A-P07-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T10 giữ nt-export preview server/auth/session. Phần chung do T04/T05 giữ resources, T10 giữ server/auth/lifetime. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC10 / P07-TC10, gồm các ca chi tiết §13.3. V-08. Đối chiếu Auth protocol và lifetime đúng; server không bind ra interface ngoài loopback. |
| Điểm tích hợp | [P07-CP05](#p07-cp05) — Preview server và viewer; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t11"></a>

### P07-T11 — Viewer cô lập và asset allowlist

- **Mục tiêu / đầu ra:** Trang tin cậy xem bundle, so ảnh và nhận biết stale.
- **Căn cứ / phụ thuộc:** Q31/33, S-SEC/S-IMAGE; T05/T10. Dependency thực thi: [P07-T05](#p07-t05), [P07-T10](#p07-t10).
- **Phạm vi:** nt-export preview viewer/resource serving.
- **Việc làm:** Tách quyền phiên khỏi nội dung sách; sandbox/allowlist theo quyết định; asset IDs từ manifest; compare pinned original.
- **Ca lỗi / biên:** Book content lấy token, request file ngoài bundle, external network, stale view vẫn ghi current.
- **Nghiệm thu:** `P07-AC11` — Nội dung sách không có quyền thao tác app; mọi asset đúng bundle và viewer hiện stale khi inputs đổi.
- **Kiểm chứng:** `P07-TC11`, V-08; dữ liệu kiểm tra: Malicious book và legitimate inline images/links, mutated inputs.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Tách UI điều khiển có quyền khỏi nội dung sách đã render. Áp isolation/CSP theo S-SEC và quyết định T11.
2. Ánh xạ asset ID qua bundle allowlist, gồm ảnh chọn và ảnh gốc để so. Kiểm mọi đường lấy nội dung trước khi trả bytes.
3. Đổi chế độ so ảnh trong viewer không đổi manifest. Hiển thị stale và thao tác nạp lại qua đường T13.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC11.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T11 |
|---|---|
| Đầu vào bắt buộc | [P07-T05](#p07-t05), [P07-T10](#p07-t10). Nhận sanitized content, authenticated asset API và pinned source/chosen images. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Viewer tách nội dung sách khỏi quyền phiên, asset ID theo allowlist, compare không đổi ảnh xuất. Owner đầu ra là A-P07-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T11 giữ nt-export preview viewer/resource serving. Phần chung do T05 giữ render, T10 giữ auth, T11 giữ viewer/isolation. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC11 / P07-TC11, gồm các ca chi tiết §13.3. V-08. Đối chiếu Browser/resource traces không có quyền/file/network ngoài contract; compare không đổi ảnh chọn. |
| Điểm tích hợp | [P07-CP05](#p07-cp05) — Preview server và viewer; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t12"></a>

### P07-T12 — UI-11 xem ảnh và lỗi

- **Mục tiêu / đầu ra:** Ảnh source/edit cùng thông tin vị trí và validation.
- **Căn cứ / phụ thuộc:** S-TUI, S-IMAGE; T02–T03 và T11 cho thao tác so ảnh qua Preview. Phần read model/list được chuẩn bị sau T03, nghiệm thu đủ task khi viewer đã có. Dependency thực thi: [P07-T02](#p07-t02), [P07-T03](#p07-t03), [P07-T11](#p07-t11).
- **Phạm vi:** nt-tui images subviews/actions.
- **Việc làm:** List occurrences, selected hash, error file, dimensions/ratio; mở so sánh qua Preview khi có.
- **Ca lỗi / biên:** Edit lỗi, ảnh thiếu, nhiều occurrence một resource, selection sau refresh.
- **Nghiệm thu:** `P07-AC12` — Lỗi chỉ đúng file và chặn đúng use case; không thể bấm bỏ qua để xuất.
- **Kiểm chứng:** `P07-TC12`, V-09; dữ liệu kiểm tra: Valid/invalid edits và source used nhiều nơi.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Dùng read model danh sách ảnh, occurrences, format/dimensions, selected hash và diagnostic.
2. Giữ selection theo asset/occurrence khi đổi kích thước hoặc quay lại. Nối so sánh qua Preview khi use case tương ứng có sẵn.
3. Hiện đường dẫn và thao tác xử lý ảnh lỗi. Khi refresh, chuyển từ kết quả cũ sang snapshot kiểm mới cùng trạng thái stale.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC12.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T12 |
|---|---|
| Đầu vào bắt buộc | [P07-T02](#p07-t02), [P07-T03](#p07-t03), [P07-T11](#p07-t11). Nhận image read model cùng viewer để so source/edit. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | UI-11 giữ occurrence/selection/hash, blocker chỉ đúng file, thao tác so ảnh đi qua viewer đã kiểm. Owner đầu ra là A-P07-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T12 giữ nt-tui images subviews/actions. Phần chung do T02/T03 giữ image decision, T11 giữ viewer, người tích hợp giữ router. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC12 / P07-TC12, gồm các ca chi tiết §13.3. V-09. Đối chiếu UI read model khớp core image decision và selection còn đúng. |
| Điểm tích hợp | [P07-CP06](#p07-cp06) — Giao diện ảnh và Preview; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t13"></a>

### P07-T13 — UI-12 readiness và Preview

- **Mục tiêu / đầu ra:** Check/prepare/view/reload với trạng thái rõ.
- **Căn cứ / phụ thuộc:** S-TUI, Q31/33; T01/T04/T10/T11. Dependency thực thi: [P07-T01](#p07-t01), [P07-T04](#p07-t04), [P07-T10](#p07-t10), [P07-T11](#p07-t11).
- **Phạm vi:** nt-tui prepare/preview; app check/preview commands.
- **Việc làm:** Hiện phần dịch/giữ nguyên/exceptions/blockers; preview optional; show stale/reload/close session.
- **Ca lỗi / biên:** Not ready, bundle vừa stale, lỗi browser mở, session expired.
- **Nghiệm thu:** `P07-AC13` — Không tự export khi preview; reload tạo/nhận bundle hợp lệ mới, không sửa bundle cũ.
- **Kiểm chứng:** `P07-TC13`, V-09; dữ liệu kiểm tra: Ready/blocked/stale/session-expired flows.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Hiện readiness và phân loại phần dịch, giữ nguyên, ngoại lệ, lỗi cần xử lý theo scope. Liên kết blocker với màn sửa tương ứng.
2. Mở phiên từ bundle đã ghim khi người dùng chọn Preview. Tách báo hiệu stale, nạp lại và đóng phiên.
3. Nạp lại kiểm readiness và tạo bundle mới. Khi thiếu điều kiện, giữ lỗi để người dùng xử lý trước khi mở bộ mới.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC13.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T13 |
|---|---|
| Đầu vào bắt buộc | [P07-T01](#p07-t01), [P07-T04](#p07-t04), [P07-T10](#p07-t10), [P07-T11](#p07-t11). Nhận readiness, bundle, session và viewer controls. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | UI-12 check/prepare/view/reload/close giữ exact bundle, stale được hiện và revalidate khi nạp lại. Owner đầu ra là A-P07-T13. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T13 giữ nt-tui prepare/preview; app check/preview commands. Phần chung do T01/T04 giữ basis, T10/T11 giữ phiên, T13 giữ readiness UI. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC13 / P07-TC13, gồm các ca chi tiết §13.3. V-09. Đối chiếu Command/result/session actions đúng; không có publication từ view. |
| Điểm tích hợp | [P07-CP06](#p07-cp06) — Giao diện ảnh và Preview; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t14"></a>

### P07-T14 — UI-13 export và lịch sử

- **Mục tiêu / đầu ra:** Export operation theo dõi được và giữ các phiên bản trước.
- **Căn cứ / phụ thuộc:** S-TUI, Q33; T07–T09/T13. Dependency thực thi: [P07-T07](#p07-t07), [P07-T08](#p07-t08), [P07-T09](#p07-t09), [P07-T13](#p07-t13).
- **Phạm vi:** nt-tui export/history; app export command.
- **Việc làm:** Gọi direct export/after preview cùng core; hiện validating/publishing/failed/done; path/version/next action.
- **Ca lỗi / biên:** Validation fail, same-name conflict, duplicate click, resume after crash.
- **Nghiệm thu:** `P07-AC14` — UI/CLI không báo done trước publication; lịch sử có đúng version/path và failure.
- **Kiểm chứng:** `P07-TC14`, V-09; dữ liệu kiểm tra: r001/r002/r003 fail/r004 và resume r003.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Gọi lệnh export/resume dùng cùng operation service với CLI. Hiện scope, số file và trạng thái từ persisted history.
2. Cập nhật validating, publishing, failed hoặc done theo operation thực. Hiện đường dẫn, proof và thao tác tiếp theo theo trạng thái.
3. Phân biệt lệnh mới với tiếp tục lần lỗi. Khử lặp thao tác trong UI và bảo toàn file cũ.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC14.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T14 |
|---|---|
| Đầu vào bắt buộc | [P07-T07](#p07-t07), [P07-T08](#p07-t08), [P07-T09](#p07-t09), [P07-T13](#p07-t13). Nhận proof/operation/history và readiness UI đã nối. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Export trực tiếp hoặc sau Preview qua cùng core, UI/JSON phản ánh validating/publishing/done/failed và đúng next action. Owner đầu ra là A-P07-T14. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T14 giữ nt-tui export/history; app export command. Phần chung do T08/T09 giữ nghiệp vụ, T13 giữ route, người tích hợp giữ CLI/composition. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T13](PHASE-06.md#p06-t13), [P06-T14](PHASE-06.md#p06-t14), [P06-T15](PHASE-06.md#p06-t15). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC14 / P07-TC14, gồm các ca chi tiết §13.3. V-09. Đối chiếu JSON/exit/UI khớp operation state, prior versions không đổi. |
| Điểm tích hợp | [P07-CP07](#p07-cp07) — Xuất bản và lịch sử; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T02](PHASE-08.md#p08-t02), [P08-T03](PHASE-08.md#p08-t03), [P08-T04](PHASE-08.md#p08-t04), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p07-t15"></a>

### P07-T15 — Nghiệm thu bảo mật, recovery và trình đọc

- **Mục tiêu / đầu ra:** M7 chứng minh bundle/preview/export đồng nhất và file đọc được.
- **Căn cứ / phụ thuộc:** Q30–33/36; T01–T14. Dependency thực thi: [P07-T01](#p07-t01), [P07-T02](#p07-t02), [P07-T03](#p07-t03), [P07-T04](#p07-t04), [P07-T05](#p07-t05), [P07-T06](#p07-t06), [P07-T07](#p07-t07), [P07-T08](#p07-t08), [P07-T09](#p07-t09), [P07-T10](#p07-t10), [P07-T11](#p07-t11), [P07-T12](#p07-t12), [P07-T13](#p07-t13), [P07-T14](#p07-t14), M6 Đạt.
- **Phạm vi:** App export integration/security tests, manual reader evidence.
- **Việc làm:** Cross-check manifest/content/hashes; fault publication; malicious fixture; EPUBCheck và mở reader.
- **Ca lỗi / biên:** Preview khác bản xuất, thiếu tài nguyên, unsupported warning bị bỏ, reader hiển thị lỗi.
- **Nghiệm thu:** `P07-AC15` — EPUBCheck đạt, ảnh/chữ/notes/nav/ruby đúng trong readers đã ghi nhận; mọi ca chặn giữ file cũ.
- **Kiểm chứng:** `P07-TC15`, V-10, V-00; dữ liệu kiểm tra: Corpus EPUB tự sinh và ảnh edited tổng hợp.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

**Trình tự triển khai**

1. Chạy fixture từ readiness qua so ảnh, nạp lại, xuất và mở lại. Đối chiếu bundle, manifest và hash xuất.
2. Chạy malicious fixture và fault injection ở ranh giới record/publish/history. Xác nhận source, file cũ và workspace ngoài allowlist được bảo toàn.
3. Trên artifact đúng hash, chạy EPUBCheck và mở Calibre cùng một trình đọc mục tiêu. Ghi tool/reader version và quan sát thứ tự, ảnh, ruby, liên kết, chú thích.

Ca thử chi tiết tại [mục 13.3](#p07-cases) theo mã P07-TC15.

**Sáu trường giao việc**

| Trường | Nội dung cho P07-T15 |
|---|---|
| Đầu vào bắt buộc | [P07-T01](#p07-t01), [P07-T02](#p07-t02), [P07-T03](#p07-t03), [P07-T04](#p07-t04), [P07-T05](#p07-t05), [P07-T06](#p07-t06), [P07-T07](#p07-t07), [P07-T08](#p07-t08), [P07-T09](#p07-t09), [P07-T10](#p07-t10), [P07-T11](#p07-t11), [P07-T12](#p07-t12), [P07-T13](#p07-t13), [P07-T14](#p07-t14), M6 Đạt. Nhận candidate image/bundle/Preview/export, real validator và reader versions. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Hồ sơ M7 so manifest/content/hashes, security/recovery và mẫu mở thực trên Calibre/reader đích. Owner đầu ra là A-P07-T15. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P07-T15 giữ App export integration/security tests, manual reader evidence. Phần chung do owners T01–T14 nhận lỗi module, A-T15 tích hợp. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P08-T01](PHASE-08.md#p08-t01), [P08-T02](PHASE-08.md#p08-t02), [P08-T03](PHASE-08.md#p08-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P07-AC15 / P07-TC15, gồm các ca chi tiết §13.3. V-10, V-00. Đối chiếu Report ghi tool/reader/OS/version; chưa đọc reader nào thì không đánh dấu đạt trên reader đó. |
| Điểm tích hợp | [P07-CP08](#p07-cp08) — Nghiệm thu M7; A chat P07 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P07-AC01 | Chỉ đủ mọi phần bắt buộc mới ready; retained source không được tính là đã dịch. | P07-T01 | P07-TC01; V-01 |
| P07-AC02 | Edit khớp hỏng chặn; file hợp lệ giữ nguyên byte; file ngoài mapping không tự được dùng. | P07-T02 | P07-TC02; V-02 |
| P07-AC03 | Bundle chỉ dùng bytes đã kiểm; image changes làm stale phần phụ thuộc mà không xóa bản dịch chữ. | P07-T03 | P07-TC03; V-02 |
| P07-AC04 | Một manifest hash xác định đúng toàn bộ inputs; stale prepare không giả current. | P07-T04 | P07-TC04; V-03 |
| P07-AC05 | Không thực thi/tải nguồn ngoài; text/anchor/ruby/nhấn mạnh hợp lệ còn đúng. | P07-T05 | P07-TC05; V-04 |
| P07-AC06 | EPUB dùng đúng accepted content và preserved sources; không gọi model/đề xuất nghĩa khi export. | P07-T06 | P07-TC06; V-05 |
| P07-AC07 | Chỉ file đúng hash đã qua validation được eligible publish; error/meaningful warning chặn. | P07-T07 | P07-TC07; V-05, V-06 |
| P07-AC08 | Lệnh mới lấy r004; resume r003 đối chiếu r003; không có hai operation sở hữu cùng version. | P07-T08 | P07-TC08; V-07 |
| P07-AC09 | Không đè file hiện có; reopen nhận đúng file đã publish hoặc conflict; không tạo bản mới cho cùng resume. | P07-T09 | P07-TC09; V-07 |
| P07-AC10 | Unauthorized không đọc tài nguyên; secret không xuất hiện ở URL query/log/referrer; session hết hạn bị thu hồi. | P07-T10 | P07-TC10; V-08 |
| P07-AC11 | Nội dung sách không có quyền thao tác app; mọi asset đúng bundle và viewer hiện stale khi inputs đổi. | P07-T11 | P07-TC11; V-08 |
| P07-AC12 | Lỗi chỉ đúng file và chặn đúng use case; không thể bấm bỏ qua để xuất. | P07-T12 | P07-TC12; V-09 |
| P07-AC13 | Không tự export khi preview; reload tạo/nhận bundle hợp lệ mới, không sửa bundle cũ. | P07-T13 | P07-TC13; V-09 |
| P07-AC14 | UI/CLI không báo done trước publication; lịch sử có đúng version/path và failure. | P07-T14 | P07-TC14; V-09 |
| P07-AC15 | EPUBCheck đạt, ảnh/chữ/notes/nav/ruby đúng trong readers đã ghi nhận; mọi ca chặn giữ file cũ. | P07-T15 | P07-TC15; V-10, V-00 |

Các AC không thay thế nhau. Build thành công, fixture trả đúng kết quả định sẵn hoặc một màn hiển thị đẹp không chứng minh toàn bộ hành vi nghiệp vụ.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P07-TC01 / P07-AC01 | All-text/story, shared note, thiếu phụ trợ và stale Review. | Report đúng item/reason; không chấp nhận issue list rỗng thay coverage. | V-01 |
| P07-TC02 / P07-AC02 | Ảnh tự sinh các định dạng, malicious SVG và limit boundaries. | Selected hash/diagnostic đúng; không fallback source khi edit hợp lệ tên nhưng lỗi. | V-02 |
| P07-TC03 / P07-AC03 | Writer thay bytes giữa read và directory có edit mới. | Hash/bytes nhất quán; old bundle vẫn đọc đúng assets đã pin. | V-02 |
| P07-TC04 / P07-AC04 | Cùng data/reordered serialization và từng loại input thay đổi. | Canonical hash ổn định theo contract; mỗi thay đổi liên quan làm invalidation đúng. | V-03 |
| P07-TC05 / P07-AC05 | XHTML/CSS ác ý và nguồn hợp lệ có Unicode/note/ảnh. | Rendered document/resource requests chỉ dùng allowed IDs; không mất nội dung hợp lệ. | V-04 |
| P07-TC06 / P07-AC06 | EPUB2/3, multi-file chapter, ruby phonetic/semantic và shared notes. | Readback XML/resource/content oracle, archive entry properties và navigation đúng. | V-05 |
| P07-TC07 / P07-AC07 | Valid/invalid temp EPUB và reports/tool process fixtures. | Có test với EPUBCheck thật cài sẵn; mock wrapper không thay bằng chứng validator thật. | V-05, V-06 |
| P07-TC08 / P07-AC08 | Concurrent reservations và failed/resumed r003. | Unique/version sequence và durable history đúng qua restart. | V-07 |
| P07-TC09 / P07-AC09 | Failpoints temp/proof/publish/finalize và file sentinel ở dest. | Hash/path/version/history cùng một operation; prior files byte-identical. | V-07 |
| P07-TC10 / P07-AC10 | HTTP client giả/browser harness cục bộ, fake clock. | Auth protocol và lifetime đúng; server không bind ra interface ngoài loopback. | V-08 |
| P07-TC11 / P07-AC11 | Malicious book và legitimate inline images/links, mutated inputs. | Browser/resource traces không có quyền/file/network ngoài contract; compare không đổi ảnh chọn. | V-08 |
| P07-TC12 / P07-AC12 | Valid/invalid edits và source used nhiều nơi. | UI read model khớp core image decision và selection còn đúng. | V-09 |
| P07-TC13 / P07-AC13 | Ready/blocked/stale/session-expired flows. | Command/result/session actions đúng; không có publication từ view. | V-09 |
| P07-TC14 / P07-AC14 | r001/r002/r003 fail/r004 và resume r003. | JSON/exit/UI khớp operation state, prior versions không đổi. | V-09 |
| P07-TC15 / P07-AC15 | Corpus EPUB tự sinh và ảnh edited tổng hợp. | Report ghi tool/reader/OS/version; chưa đọc reader nào thì không đánh dấu đạt trên reader đó. | V-10, V-00 |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo trong task tương ứng; nếu đổi phải cập nhật kế hoạch. Lệnh exit 0 với 0 ca liên quan không được ghi là đạt. 

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | cargo test -p nt-application --test export_readiness --locked |
| V-02 | cargo test -p nt-export --test image_contract --locked |
| V-03 | cargo test -p nt-export --test prepared_bundle --locked |
| V-04 | cargo test -p nt-export --test content_safety --locked |
| V-05 | cargo test -p nt-export --test epub_roundtrip --locked |
| V-06 | cargo test -p nt-export --test epubcheck_integration --locked; dùng EPUBCheck 5.3.0 đã cài, lưu version/report |
| V-07 | cargo test -p novel-translate --test export_publication --locked |
| V-08 | cargo test -p nt-export --test preview_security --locked; browser harness cục bộ kiểm token/isolation |
| V-09 | cargo test -p nt-tui --test images_export --locked<br>cargo test -p novel-translate --test export_cli --locked |
| V-10 | cargo test -p novel-translate --test phase07_export --locked; mở corpus trong Calibre và trình đọc mục tiêu, lưu checklist có phiên bản |

**V-00 — Bộ kiểm tra Rust bắt buộc trước PR Rust và trước đóng phase có thay đổi Rust:**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

EPUB/ảnh sinh từ nguồn tự soạn vào thư mục tạm; không commit binary EPUB hoặc output. Chuẩn bị EPUBCheck/runtime/browser trước test. Preview dùng loopback cục bộ được kiểm soát; không truy cập Internet/provider. Test security/browser và manual readers có evidence riêng, không giả bằng mock validators.

### 13.2. Cách dùng bằng chứng

Test phải kiểm tra kết quả quan sát được ở boundary, cả ca thành công và lỗi làm sai dữ liệu/quyền. Không viết test chỉ nhắc lại implementation. Đối với fault injection, ghi chính xác điểm ngắt, trạng thái trước/sau reopen và side effect đã/không xảy ra. Lỗi CI/môi trường và lỗi sản phẩm được phân biệt; chưa chạy thì ghi chưa xác minh.

<a id="p07-cases"></a>

### 13.3. Ca thử chi tiết theo task

Mỗi ca dưới đây thuộc nhóm V của mã TC gốc. Dùng fixture EPUB tự soạn, thư mục tạm, clock kiểm soát được và các điểm ngắt do harness kích hoạt. Với xuất/khôi phục, so file bytes, operation ID, số phiên bản, proof và lịch sử. Với Preview, ghi những request thật sự được phục vụ. Bằng chứng EPUBCheck và mở trình đọc thật được ghi riêng, gắn đúng hash artifact.

| Mã ca | Fixture và thao tác | Kết quả cần đối chiếu |
|---|---|---|
| P07-TC01a | All-text thiếu lời bạt hoặc một liên kết chú thích. | Readiness chặn ở đúng phần nguồn và chỉ ra nguyên nhân. |
| P07-TC01b | Story giữ nguyên trang phụ theo lựa chọn. | Trang có nhãn giữ nguyên và không bị tính như lỗi coverage. |
| P07-TC01c | Có ngoại lệ hợp lệ cùng ruby chờ duyệt. | Gate vẫn chặn vì ruby. Khi duyệt xong, trạng thái có ngoại lệ giữ đúng revision. |
| P07-TC01d | Sửa revision sau khi report báo sẵn sàng. | Report cũ không đủ căn cứ để dùng cho lần prepare mới. |
| P07-TC02a | Hai ảnh trùng basename khác thư mục, edit sai chữ hoa hoặc đuôi. | Chỉ đường dẫn khớp dùng edit, file còn lại có diagnostic. |
| P07-TC02b | Ảnh edit giả đuôi, hỏng cấu trúc hoặc vượt frame/pixel cap. | Chặn ở đúng ảnh và không fallback về source. |
| P07-TC02c | SVG có script, external reference hoặc thuộc tính chạy mã. | Bị từ chối qua cùng policy dùng cho Preview và xuất. |
| P07-TC02d | Ảnh edit hợp lệ khác tỷ lệ với source. | Byte giữ nguyên, UI có thông tin chênh lệch và vẫn cho prepare nếu đủ điều kiện. |
| P07-TC03a | Đổi edit trong lúc đọc cùng tên file. | Chỉ pin một phiên ổn định hoặc báo cần chuẩn bị lại. |
| P07-TC03b | Edit mới xuất hiện sau khi bundle chọn ảnh gốc. | Bundle cũ báo stale và lần nạp lại chọn hash edit. |
| P07-TC03c | Ảnh trong images/source bị thay byte ngoài snapshot. | Báo nguồn lệch và yêu cầu xử lý, giữ snapshot ban đầu. |
| P07-TC04a | Cùng inputs, thứ tự duyệt map khác. | Manifest định danh cùng nội dung theo cách canonical đã chọn. |
| P07-TC04b | Chỉ đổi metadata tiếng Việt hoặc layout. | Bundle mới phản ánh thay đổi, revisions/Review phần chữ hợp lệ giữ nguyên. |
| P07-TC04c | Ảnh hoặc căn cứ đổi giữa readiness và capture. | Chặn ghim bộ trộn phiên hoặc tính lại với cùng căn cứ. |
| P07-TC05a | XHTML có script, handler, entity ngoại hoặc URL file/network. | Render không thể kích hoạt hay truy cập các tài nguyên đó. |
| P07-TC05b | CSS có import hoặc URL vượt asset set qua ký tự thoát. | Parser và policy chặn đường vòng, cấu trúc đã ghim còn nguyên. |
| P07-TC05c | Nội dung có ruby, emphasis, anchor và chú thích quay lại. | Readback giữ các đích và ý nghĩa định dạng dự kiến. |
| P07-TC06a | EPUB 2 với NCX lồng cấp và ghi chú qua nhiều file. | Nav giữ thứ tự/cấp, đích thực sự mở được trong package mới. |
| P07-TC06b | Một trang ảnh riêng và một chương chữ có ruby Việt. | Ảnh, layout, hướng đọc và ruby đúng lựa chọn đã ghim. |
| P07-TC06c | Metadata chứa dấu Việt, ký tự cần escape và tên dài. | Package hợp lệ, giá trị đọc lại đúng tên đã lưu. |
| P07-TC06d | Mở container vừa tạo và đọc entry đầu. | Mimetype đúng, nằm đầu và không nén. |
| P07-TC07a | Temp có link gãy, thiếu asset hoặc hụt một đoạn dù XML hợp lệ. | Readback chặn và chỉ ra khác biệt. |
| P07-TC07b | Validator thiếu, sai version hoặc trả lỗi. | Không có proof đạt cho artifact này. |
| P07-TC07c | Có cảnh báo EPUBCheck chưa phân loại. | Report ghi mục cần xử lý trước công bố. |
| P07-TC07d | Temp bị đổi sau validation. | Proof cũ không mở được publish gate. |
| P07-TC08a | Hai lệnh xuất mới đồng thời cho một tập. | Hai operation nhận số khác nhau, tăng theo sequence của tập. |
| P07-TC08b | r003 lỗi rồi thực hiện một lệnh mới. | r004 được cấp, r003 vẫn hiện trong lịch sử. |
| P07-TC08c | Resume đúng r003 sau restart. | Dùng lại r003 và đối chiếu proof/trạng thái đã lưu. |
| P07-TC09a | Ngắt trước publish sau khi có proof. | Resume tiếp tục đúng operation khi căn cứ/proof còn hợp lệ. |
| P07-TC09b | Ngắt sau publish trước khi cập nhật lịch sử. | Đối chiếu file và hash rồi hoàn tất một bản lịch sử. |
| P07-TC09c | Đích có file sẵn với hash khác. | Giữ nguyên file đó và báo xung đột có vị trí. |
| P07-TC09d | Quyết định hoặc ảnh thay đổi trước publish. | Căn cứ cũ phải được xử lý theo contract, chưa công bố file stale. |
| P07-TC10a | Replay token sau khi đổi thành công hoặc thử Host/origin sai. | Quyền phiên chỉ được cấp đúng một lần cho origin hợp lệ. |
| P07-TC10b | Request asset trước đổi token hoặc khi hết hạn/đóng. | Không phục vụ bytes. Kiểm mốc mặc định 30 phút bằng fake clock. |
| P07-TC10c | Ghi log access/error trong handshake và vòng đời phiên. | Sentinel token không xuất hiện trong log hoặc referrer. |
| P07-TC11a | Asset ID lạ, traversal đã encode hoặc tài nguyên file/network từ nội dung. | Mọi cách đi ngoài allowlist bị chặn, không lộ workspace path. |
| P07-TC11b | Nội dung sách thử đọc quyền phiên hoặc gửi request tùy ý. | Viewer cách ly giữ quyền trong trang điều khiển. |
| P07-TC11c | Bật so ảnh gốc, rồi xuất từ cùng bundle. | Viewer hiện cả hai hash đúng, file xuất vẫn dùng hash edit đã chọn. |
| P07-TC12a | Một ảnh được dùng ở hai chương. | UI hiện đủ vị trí và đúng asset khi điều hướng. |
| P07-TC12b | Chọn edit khác tỷ lệ hoặc sai định dạng. | Thông tin tỷ lệ và blocker phân biệt rõ, có file cần sửa. |
| P07-TC12c | Sửa edit rồi quay về từ màn so ảnh. | Giữ vị trí xem, hiện căn cứ mới hoặc stale chính xác. |
| P07-TC13a | Một blocker và một ngoại lệ còn hiệu lực cùng tập. | Hiện cả hai, nút xuất chịu gate chung và deep link tới đúng blocker. |
| P07-TC13b | Đổi ảnh hoặc metadata trong khi viewer mở. | Bộ đang xem báo stale. Nạp lại lấy bundle hash mới và giữ phiên bản cũ trong lịch sử khi cần. |
| P07-TC13c | Người dùng bỏ qua Preview và chọn export trực tiếp. | Cùng readiness/bundle contract, không buộc mở trình duyệt. |
| P07-TC14a | Xuất trực tiếp rồi xuất qua Preview với inputs giống nhau. | Hai version khác nhau, bundle/căn cứ và nội dung khớp. |
| P07-TC14b | Ngắt ở validating, mở lại từ history. | Đúng operation còn dở và thao tác tiếp tục đúng số. |
| P07-TC14c | Bấm export lặp khi đang publish. | Một intent chỉ tạo một operation, giữ các file trước. |
| P07-TC15a | EPUB 2 chuyển sang 3.3 và EPUB 3 có cover/ảnh/chú thích. | Readback và EPUBCheck đạt trên từng file cụ thể. |
| P07-TC15b | Mở các file đã kiểm trên Calibre và một reader mục tiêu. | Duyệt thực tế mục lục, chương, ảnh, ruby và đi/về chú thích đúng oracle. |
| P07-TC15c | Stale Preview, ảnh độc hại và crash sau công bố. | Report phân biệt ba loại lỗi, giữ source/file cũ, không cấp proof sai. |
| P07-TC15d | Chạy mục 16.4 cùng release build hiện hành của M7. | Người dùng theo dõi được lỗi ảnh, bundle reload và các version xuất. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md): A thực hiện task, kiểm thử và tự review rồi bàn giao **Chờ review**; B đối chiếu yêu cầu, kiểm bản bàn giao và tự chạy kiểm chứng phù hợp trong phiên riêng. B ghi báo cáo độc lập, không sửa source/test chính thức hoặc trạng thái PLAN. A xử lý findings; B kiểm tra lại trên candidate mới. Quy tắc này áp dụng cả task tài liệu/ADR/thử nghiệm của phase.

- Readiness có bao phủ phụ trợ/retained parts và exact current basis?
- File ảnh/user output có bị convert/overwrite ngầm?
- Hash validate và bytes publish có cùng một file/bundle bất biến?
- Preview token/Host/origin có bảo vệ mọi asset endpoint và không rò qua nội dung sách?
- New export vs resume có đúng số bản và recovery giữ file cũ?

Rà thêm diff ngoài phạm vi, API công khai không có consumer, dependency cấm, error bị nuốt và test không được phát hiện/chạy. Không xóa hoặc nới test chỉ để qua kiểm tra. Sửa lỗi ở owner của quy tắc thay vì thêm ngoại lệ trong UI.

## 15. Điều kiện đóng phase

- [ ] Mọi task/nhóm trong phạm vi cổng có review B **Đạt** còn hiệu lực; đủ AC, kiểm tra bắt buộc và không còn finding chặn. A không tự duyệt phần mình triển khai.
- [ ] B đã review bản tích hợp cuối theo [mẫu review phase](../templates/PHASE_REVIEW.md); candidate và căn cứ khớp, bằng chứng các task còn áp dụng hoặc đã được kiểm lại theo ảnh hưởng.

- [ ] UI-11/12/13 và CLI check/preview/export hoàn thành luồng trực tiếp lẫn qua Preview.
- [ ] Image safety/Preview isolation/no-clobber/recovery có ca âm và bằng chứng.
- [ ] EPUBCheck 5.3.0 đạt và corpus đã mở trên Calibre cùng một reader mục tiêu; nội dung được đối chiếu.
- [ ] Tất cả task và AC của phase có bằng chứng, điểm chặn đã được giải quyết; ca chưa kiểm tra được ghi đúng trạng thái.
- [ ] V-00 áp dụng cho thay đổi Rust đã đạt; checks riêng trong mục 13 và các kiểm tra Go bị ảnh hưởng đã có kết quả.
- [ ] README được cập nhật nếu command/layout/nền tảng thay đổi; tài liệu nội bộ, key, EPUB, workspace và đầu ra tiếp tục ngoài Git.
- [ ] Bàn giao mục 16 đủ cho phase sau; không tự xóa đường Go hoặc công bố feature parity.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md); B dùng [mẫu review task](../templates/TASK_REVIEW.md), và [mẫu review phase](../templates/PHASE_REVIEW.md) khi nghiệm thu cổng. Báo cáo nội bộ đặt theo lượt trong `docs/roadmap/evidence/phase-07/`: `P07-TYY.handoff-r01.md` và `P07-TYY.review-r01.md`; giữ lượt trước khi sửa. Hồ sơ cổng dùng ID M0-01/M0-02/M0 hoặc M1…M9 theo phạm vi. Các mẫu chưa phải bằng chứng thực thi.

Sổ task/cổng khi thực hiện ghi trạng thái, candidate, link báo cáo A, review B, finding còn mở và bước tiếp theo. A phase cập nhật sổ này và gửi điều phối cập nhật PLAN trung tâm sau bàn giao/kết luận; B chỉ ghi báo cáo review. Thiếu review B hoặc kiểm tra bắt buộc thì chưa đánh dấu hoàn thành.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 8 checkpoint của lịch v2: [P07-CP01](#p07-cp01), [P07-CP02](#p07-cp02), [P07-CP03](#p07-cp03), [P07-CP04](#p07-cp04), [P07-CP05](#p07-cp05), [P07-CP06](#p07-cp06), [P07-CP07](#p07-cp07), [P07-CP08](#p07-cp08); xem [§10.2](#p07-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task | Trạng thái | Bằng chứng |
|---|---|---|---|
| P07-A | T01–T03 | Chưa bắt đầu | Chưa có |
| P07-B | T04–T07 | Chưa bắt đầu | Chưa có |
| P07-C | T08–T09 | Chưa bắt đầu | Chưa có |
| P07-D | T10–T11 | Chưa bắt đầu | Chưa có |
| P07-E | T12–T14 | Chưa bắt đầu | Chưa có |
| P07-F | T15 | Chưa bắt đầu | Chưa có |

### 16.2. Báo cáo mỗi task

Ghi task/AC, thay đổi hành vi, file/module, quyết định và tác động migration/compatibility; lệnh đã chạy, toolchain/OS, tên/số ca liên quan, kết quả; test thủ công và ca chưa chạy; rủi ro còn lại và task có thể nhận tiếp. Nếu lưu log/báo cáo, dùng `docs/roadmap/evidence/phase-07/`, đã redaction, không chứa dữ liệu cá nhân hoặc nội dung sách thật.

### 16.3. Đầu ra cho bước tiếp theo

[Phase 08](PHASE-08.md) hoàn thiện thao tác xuyên màn và trạng thái lỗi/chờ. [Phase 09](PHASE-09.md) nhận corpus, reader reports, validation tool/version, security/failpoint suite và export operation protocol để chạy nghiệm thu release. Không dùng khả năng mở Preview như bằng chứng EPUB hoàn chỉnh đã xuất.

Trạng thái kế hoạch không phải trạng thái implementation. Chỉ cập nhật hoàn thành khi có bằng chứng của hành vi thực tế.

<a id="p07-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử candidate **M7** với một tập mẫu đã đủ điều kiện xuất. Bài thử kiểm ảnh được chọn, Preview, hai lần xuất và nội dung mở trong trình đọc. Khi bàn giao, A ghi candidate, lệnh mở app, đường dẫn ảnh và thư mục xuất. B thực hiện lại trên bản sao mới và ghi phiên bản trình đọc.

#### 16.4.2. Chuẩn bị

Dành khoảng 15–20 phút. Chuẩn bị workspace mẫu, một ảnh thay hợp lệ, một ảnh thay hỏng có tên khớp, thư mục xuất thử và trình đọc theo V-10. Bảng kỳ vọng của mẫu cần có tên chương, vị trí ảnh, ruby và một cặp liên kết chú thích.

#### 16.4.3. Mở đúng bản

Mở bản Rust M7 và đúng tập mẫu, vào **Ảnh và xuất**. Xem trạng thái sẵn sàng trước khi sửa ảnh. Có thể chạy CLI `check --volume <id>` với workspace mẫu theo lệnh bàn giao để đối chiếu cùng tập.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P07-UT01 — Chọn đúng ảnh thay | Đặt ảnh thay hợp lệ vào đường dẫn đã ghi trong bộ mẫu. Mở lại chi tiết ảnh. | Thấy ảnh nguồn, ảnh được chọn và vị trí dùng đúng như bảng mẫu. |
| P07-UT02 — Nạp lại Preview | Mở **Preview**. Đổi ảnh hoặc metadata như hướng dẫn mẫu, sau đó chọn nạp lại. | Phiên đang xem báo đã cũ khi dữ liệu đổi. Bản xem mới phản ánh đúng thay đổi sau khi nạp lại. |
| P07-UT03 — Xuất hai phiên bản | Xuất tập mẫu hai lần, ghi tên file và hash của bản đầu trước lần xuất thứ hai. Mở lịch sử xuất. | Có hai phiên bản riêng, đường dẫn trong lịch sử khớp file thực tế và hash file đầu không đổi. |
| P07-UT04 — Đọc file đã xuất | Mở EPUB vừa xuất bằng Calibre và trình đọc mục tiêu đã ghi. Dùng mục lục đến chương mẫu, mở ảnh và đi theo liên kết chú thích. | Chương, ảnh, ruby và liên kết đúng bảng mẫu trong cả hai trình đọc. |

#### 16.4.5. Thử ca lỗi và mở lại

**P07-UT05 — Ảnh thay hỏng và giữ bản xuất.** Thay đúng file mẫu bằng ảnh lỗi rồi kiểm tra sẵn sàng xuất. App chỉ ra ảnh gây chặn. Đóng rồi mở lại workspace và xác nhận hai bản xuất trước vẫn có trong lịch sử và mở được. Khôi phục ảnh hợp lệ và kiểm lại trạng thái sẵn sàng.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Nếu sai, gửi candidate M7, tên EPUB, tên/phiên bản reader, mục lục hoặc vị trí nội dung sai và ảnh của mẫu khi cần.

#### 16.4.7. Thử lại từ đầu

Lấy bản sao workspace mới cùng ảnh gốc trong gói mẫu. Dùng thư mục xuất của bản sao mới để số phiên bản và lịch sử bắt đầu cùng điều kiện.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-07. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P07 rà **Ảnh, Preview và xuất EPUB** tại các mốc **P07-CP01/CP03/CP05/CP07; kiểm reader/tích hợp tại M7**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Dọn chức năng đã thay và hết caller sau kiểm tài nguyên, readiness, publish/recovery và reader bắt buộc của scope. Không xóa nguồn EPUB/ảnh, thư mục workspace, output hay lịch sử người dùng.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
