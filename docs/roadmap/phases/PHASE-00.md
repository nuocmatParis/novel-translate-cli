# PHASE 00 — QUYẾT ĐỊNH KIẾN TRÚC VÀ CHUẨN BỊ TRIỂN KHAI

> **Draft 1.4 — 08/11/2026 · Hoàn thành trong phạm vi đã review.** P00-T01-r03, P00-T02-r02 và nhóm P00-T03–T10 hoàn thành sau review B độc lập Đạt; T02 giới hạn ở Windows, Linux native hoãn theo quyết định người dùng. M0-01-r01 và M0-02-r01 đều được review riêng Đạt trên Windows. Candidate toàn phase M0-r03 được B review **Đạt trên Windows**; M0-F01/F02 đóng. Xem [review cuối phase](../evidence/phase-00/M0.review-r03.md). Code hiện tại không là chuẩn thiết kế, đầu vào tái sử dụng hoặc bằng chứng hoàn thành.
>
> Cấu trúc theo [Phase 01](PHASE-01.md). Task, đường dẫn module và tên test là kế hoạch ban đầu; rà lại trước khi giao việc khi contract của phase trước thay đổi. Không tự thay yêu cầu đã chốt để khớp implementation.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P00 / M0 |
| Tên | Quyết định kiến trúc và chuẩn bị triển khai |
| Trạng thái | Phase 00 hoàn thành trong phạm vi Windows trên candidate M0-r03; M0-F01/F02 đóng theo review B Đạt. Linux native hoãn; Phase 02 vẫn phụ thuộc M1. |
| Đầu vào chính | Đặc tả đã chốt và thiết kế Bước 1–9; không cần implementation cũ |
| Gói giao diện | Chốt contract UI-01; chưa dựng giao diện |
| Phân rã | 4 gói việc, 10 task |
| Nhánh khi triển khai | rewrite/rust; giữ Go theo R-REPO |
| Người thực hiện / rà soát | Model A triển khai / model B khác review độc lập; A hiện Codex GPT-6; B nhóm T09–T10 được phân công gpt-6.1-sol, phiên riêng sau khi candidate đóng băng |

Xem [roadmap chung](../ROADMAP.md) và [phạm vi tại Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-00). Tài liệu và bằng chứng nội bộ ở trong `docs/`, ngoài Git.

## 2. Mục tiêu và demo cuối phase

Biến các lựa chọn còn mở thành đầu vào đủ rõ cho phase sắp làm. Phase 00 có hai cổng: **M0-01** đủ khởi động Phase 01; **M0-02** đủ khởi động phần lưu trữ/EPUB của Phase 02. Quyết định model, Coordinator, Research và export được ghi lịch giải quyết trước task sử dụng, không buộc chốt toàn hệ thống từ đầu.

1. Mở bảng truy vết Q01–Q36 → phase/AC và nhận diện nguồn nào là yêu cầu, nguồn nào là đề xuất.
2. Đọc ADR nền tảng để xác định toolchain, package boundaries, CLI/terminal/error contract của Phase 01.
3. Xem kết quả thử nghiệm nhỏ về transaction, hash/offset và commit để quyết định đầu vào Phase 02.
4. Lấy một task Phase 01 hoặc 02, chỉ ra đủ quyết định, fixture và điều kiện kiểm chứng cần dùng.

## 3. Phạm vi

Trong phạm vi: đối chiếu yêu cầu; ADR-001/002/003/004 và phần cần sớm của ADR-008; chiến lược test/fixture; thử nghiệm nhỏ có giới hạn khi bằng chứng kỹ thuật còn thiếu. Mỗi thử nghiệm có câu hỏi, cách đo, tiêu chí chọn và kết luận; không tạo ứng dụng hoàn chỉnh.

ADR-005/006/007 và phần mở rộng ADR-008 được lập lịch cho Phase 04–07. Phase 00 không quyết định lại 36 yêu cầu sản phẩm, không viết toàn bộ schema, provider, workflow hay UI.

## 4. Điều kiện bắt đầu

- Có các tài liệu nguồn S-Q và thiết kế tương ứng, đọc được liên kết đến phần áp dụng.
- Xác định phase nhận đầu ra: nền tảng Phase 01 hay storage/import Phase 02.
- Nếu cần thử nghiệm thư viện, có môi trường tạm, dữ liệu tự soạn và giới hạn công việc; chưa sử dụng dữ liệu người dùng.

Chỉ coi điều kiện là đạt khi có đầu ra kiểm chứng từ task/phase cung cấp. Ghi rõ phần bị chặn và phần độc lập vẫn làm được. Trước khi sửa repo, ghi nhận thay đổi đang có để bảo toàn công việc của người dùng; việc này không biến code cũ thành chuẩn thiết kế.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định đã chốt](../../spec/DESIGN_DECISIONS_36.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| S-SEC | [Bảo mật](../../spec/SECURITY_SPEC.md) |
| D-RUST | [Bước 3 — Kiến trúc Rust](../../desgin/RUST_PROJECT_ARCHITECTURE.md) |
| D-MODULE | [Bước 4 — Module và ownership](../../desgin/MODULE_DESIGN_STEP4.md) |
| D-TECH | [Bước 5 — Nền tảng và contract](../../desgin/TECHNICAL_FOUNDATION_STEP5.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-WORKFLOW | [Bước 7 — State machine và recovery](../../desgin/WORKFLOW_STATE_MACHINE_STEP7.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

Ký hiệu Qxx trong tài liệu là quyết định xx của S-Q. Hành vi đã chốt ưu tiên hơn phương án kỹ thuật còn đề xuất. Thư viện, tên bảng/trait và SQL nháp phải được rà theo consumer thực tế; không tự áp dụng DDL thiết kế vào workspace.

- Q01–Q36 là contract sản phẩm; lựa chọn thư viện không được làm thay đổi hành vi.
- ADR phải chứa quyết định, phương án đã cân nhắc, bằng chứng, hệ quả và task cần nó.
- DB-xx/TD-xx/W7-xx là vấn đề cần giải quyết, không phải lựa chọn đã được thông qua.
- SQL DRAFT chỉ phục vụ rà soát; không sao chép thành migration rồi coi là đã chứng minh atomicity.
- Không dùng tiến độ/code hiện tại làm cơ sở bỏ qua task hoặc chốt kiến trúc.

## 6. ADR và điểm còn mở

| Nhóm | Phải có trước | Hồ sơ |
|---|---|---|
| Toolchain, framework, dependency policy | P01-T02/T04 | ADR-001; TD-01/03/04/06/07/09 |
| Bootstrap, CLI JSON, terminal, diagnostic sink | P01-T04–T06 | ADR-008; P01-D04–D06 |
| Storage driver/worker, ID/offset/hash, generation | P02 write/import path | ADR-002/003/004; DB-01–05/07/08/12/13/18 |
| Coverage và checkpoint tối thiểu | P04 request/Review/checkpoint | ADR-007/008; DB-06, W7-01–03 |
| Attempt, tiền, IPC/fencing | P05 gửi model thật | ADR-005/006/007; DB-09/10, W7-04–09 |
| Skill publish, Research, export | P03/P06/P07 task liên quan | DB-16, DB-11/15 và phần mở rộng ADR-008 |

Lịch owner/hạn theo từng ID TD/DB/W7, cùng các mục mở trong ADR chưa mang ID, nằm tại [ROADMAP §4.1](../ROADMAP.md#decision-schedule). Mục được ghi “mở/hoãn” vẫn cần quyết định trước consumer; có owner không đồng nghĩa đã chốt.

Đã có [ADR-001](../../decisions/ADR-001.md) và [ADR-008](../../decisions/ADR-008.md) ghi hướng công nghệ/bốn crate/shell. Candidate T02-r02 bổ sung compiler/MSRV/edition, thư viện/features và bằng chứng Windows; Linux hoãn theo quyết định người dùng 07/11/2026. T03 chốt API/ownership; T04 hoàn thiện CLI/terminal/runtime. Không hỏi lại các hướng đã chọn. Các quyết định khác vẫn cần đối chiếu trước consumer. Dùng ADR đã có nếu còn phù hợp; chỉ mở rộng phần mới. Chỉ phần thật sự thay đổi yêu cầu sản phẩm mới cần quyết định bổ sung. Task đang nhận và phạm vi thực thi theo [PLAN.md](../../../PLAN.md).

## 7. Thiết kế triển khai

Đầu ra tổ chức tại `docs/decisions/ADR-001.md`…`ADR-008.md`, kèm mục quyết định chưa dùng và phase cần giải quyết. Một ADR có thể được mở rộng theo thời điểm; thay đổi quyết định đã dùng phải ghi tác động/migration, không sửa lịch sử như chưa từng chọn.

Thử nghiệm mới, nếu cần, đặt dưới `docs/roadmap/spikes/p00/` với fixture tự soạn. Chỉ đo câu hỏi quyết định như rollback/lock, offset Unicode hoặc lỗi setup terminal. Kết quả được chuyển thành contract/test của crate nhận trách nhiệm, không kéo toàn bộ code thử nghiệm thành production.

Contract có consumer thực tế mới được công khai. Domain/Application giữ quy tắc và quyền công bố; adapter thực hiện IO qua port; TUI/CLI gọi cùng use case. Không nối hai adapter qua implementation nội bộ để né ownership hoặc tạo dependency vòng.

## 8. Dữ liệu, lỗi và phục hồi

| Rủi ro | Cách xử lý |
|---|---|
| Hai nguồn mô tả khác nhau | Ghi rõ câu yêu cầu, phạm vi tác động, nguồn có thẩm quyền; không tự sửa sản phẩm |
| Thử nghiệm cho kết quả không ổn định | Ghi chưa đủ bằng chứng; kiểm soát clock/IO/fixture rồi đo lại |
| Thư viện không đáp ứng MSRV/nền tảng | So phương án khác theo cùng tiêu chí, ghi lý do lựa chọn |
| ADR đổi sau khi đã có consumer | Ghi phiên bản quyết định và task/migration bị ảnh hưởng |

Phase này không tạo dữ liệu nghiệp vụ của người dùng. Dữ liệu thử nghiệm được tạo trong thư mục tạm; kết quả sai không được ghi thành quyết định đã chứng minh.

Mỗi lỗi cần giữ định danh phạm vi và nguyên nhân đã redaction, cùng bước tiếp theo. Không dùng thông báo thành công thay cho trạng thái đang chờ. Mọi thay đổi schema đã áp dụng phải có migration/version và kiểm thử tương thích; dữ liệu, lịch sử và sửa tay được bảo toàn.

## 9. Phần TUI/CLI

Chốt executable, không tham số mở TUI, tám màn, input/focus, 80×24/120 cột, JSON/exit codes, stdout/stderr và cleanup terminal cho UI-01. Màu/kích thước control chi tiết là lựa chọn triển khai khi có giao diện; không chặn nền tảng vì chưa vẽ đủ mọi màn.

Thông báo sản phẩm dùng ngôn ngữ người dùng hiểu được; mã task/ADR/crate chỉ nằm trong tài liệu kỹ thuật. Điều hướng, cuộn, resize và mở trợ giúp không tự gửi model hoặc thay quyết định nghiệp vụ.

## 10. Gói việc và thứ tự

| Gói | Task | Phụ thuộc | Đầu ra |
|---|---|---|---|
| P00-A — Truy vết và nền tảng | T01–T04 | Nguồn thiết kế | ADR đủ cho P01 |
| P00-B — Chiến lược kiểm chứng | T05 | T01–T04 | Fixture và test boundary |
| P00-C — Chuẩn bị lưu trữ và nguồn | T06–T08 | T01, T05 | ADR đủ cho P02 |
| P00-D — Lịch quyết định và bàn giao | T09–T10 | Các đầu ra liên quan | M0-01/M0-02 và lịch ADR sau |

**M0-01:** T01–T05 và phần bàn giao T10 cho Phase 01. Phase 01 có thể bắt đầu sau cổng này. **M0-02:** T06–T08 và phần bàn giao T10 cho Phase 02. T09 giữ lịch quyết định tương lai; không biến các ADR tương lai thành điều kiện của M0-01.

Mỗi task gồm triển khai, test liên quan và tự rà soát. Task tích hợp cuối phase kiểm chứng giao tiếp xuyên thành phần; không dồn toàn bộ việc kiểm thử tới cuối. Có thể chia task bằng hậu tố a/b khi diff quá lớn, nhưng phải giữ đầy đủ dependency và AC của task gốc.

## 11. Thẻ task cho AI

Các đường dẫn dưới đây là vị trí mục tiêu từ thiết kế. Không sửa phần ngoài phạm vi chỉ để tiện triển khai. Trạng thái khởi tạo là **Chưa bắt đầu**; trạng thái thực hiện hiện hành nằm ở mục 16. Lệnh V-* nằm ở mục 13.

<a id="p00-t01"></a>

### P00-T01 — Lập bảng truy vết yêu cầu

- **Mục tiêu / đầu ra:** Q01–Q36 đều có owner phase, nguồn và phép nghiệm thu.
- **Căn cứ / phụ thuộc:** S-Q, D-ROADMAP; không có task trước.
- **Phạm vi:** Roadmap, ma trận truy vết nội bộ.
- **Việc làm:** Phân biệt yêu cầu đã chốt, đề xuất kỹ thuật và vấn đề còn mở; ghi quan hệ giữa các phase.
- **Ca lỗi / biên:** Bỏ sót yêu cầu phụ trợ; hai phase nhận cùng trách nhiệm nhưng không có điểm bàn giao.
- **Nghiệm thu:** `P00-AC01` — Mọi quyết định có nơi triển khai và nơi kiểm tra; không có yêu cầu bị đổi ngầm.
- **Kiểm chứng:** `P00-TC01`, V-01; dữ liệu kiểm tra: 36 quyết định và bảng milestone.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t02"></a>

### P00-T02 — Chọn toolchain và dependency nền tảng

- **Mục tiêu / đầu ra:** ADR-001 có compiler, MSRV, edition, nền tảng test và thư viện cần dùng trước.
- **Căn cứ / phụ thuộc:** D-TECH, D-RUST; T01.
- **Phạm vi:** ADR-001 và phép thử tương thích nhỏ.
- **Việc làm:** Đối chiếu phiên bản/feature theo tài liệu chính thức khi thực thi; kiểm tra build trên target đã chọn.
- **Ca lỗi / biên:** MSRV không khớp dependency; feature kéo IO vào Domain; cam kết nền tảng chưa kiểm tra.
- **Nghiệm thu:** `P00-AC02` — Bộ lựa chọn có nguồn, phiên bản và bằng chứng khả thi; dependency chưa cần được hoãn.
- **Kiểm chứng:** `P00-TC02`, V-01, V-02; dữ liệu kiểm tra: Manifest tối thiểu cho từng lựa chọn và target.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t03"></a>

### P00-T03 — Chốt ranh giới crate và API tối thiểu

- **Mục tiêu / đầu ra:** Ownership/allowlist phụ thuộc cho bốn crate Phase 01.
- **Căn cứ / phụ thuộc:** D-RUST, D-MODULE; T01–T02.
- **Phạm vi:** ADR-008 phần API/ownership; bảng boundaries.
- **Việc làm:** Gán owner, consumer, lỗi và visibility cho từng contract cần sớm; lịch bổ sung adapter.
- **Ca lỗi / biên:** Application biết DB/HTTP implementation; trait không có consumer; vòng phụ thuộc.
- **Nghiệm thu:** `P00-AC03` — Consumer dùng được contract mà không nhập implementation adapter; không ép tạo toàn cây module.
- **Kiểm chứng:** `P00-TC03`, V-01; dữ liệu kiểm tra: Graph hợp lệ và graph có cạnh cấm.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t04"></a>

### P00-T04 — Chốt CLI, terminal và diagnostic contract

- **Mục tiêu / đầu ra:** P01-D04–D06 đủ cụ thể để viết test.
- **Căn cứ / phụ thuộc:** S-TUI, S-SEC, D-TECH; T02–T03.
- **Phạm vi:** ADR-008 phần CLI/terminal, ADR-001 event loop.
- **Việc làm:** Ghi kết hợp flags, trường/nullability JSON, exit codes, non-TTY, cleanup và sink an toàn.
- **Ca lỗi / biên:** Help bật raw mode; Esc thoát app; panic/Debug lộ secret; JSON lẫn log.
- **Nghiệm thu:** `P00-AC04` — Mỗi đầu vào quan trọng có output/exit/lifecycle xác định; giữ đúng S-TUI.
- **Kiểm chứng:** `P00-TC04`, V-01, V-02 khi cần thử terminal; dữ liệu kiểm tra: CLI case table và terminal setup lỗi từng bước.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t05"></a>

### P00-T05 — Thiết kế fixture và kiểm thử độc lập môi trường

- **Mục tiêu / đầu ra:** Quy ước tạo EPUB/JSON/event/clock fixture và evidence.
- **Căn cứ / phụ thuộc:** Q36, R-REPO; T01–T04.
- **Phạm vi:** Test strategy và nguồn fixture tối thiểu nếu cần thử nghiệm.
- **Việc làm:** Chỉ rõ unit/contract/process/terminal test; ZIP tạo tạm; ID/clock điều khiển; không phụ thuộc mạng.
- **Ca lỗi / biên:** Fixture chứa sách/key thật; test sleep theo tốc độ máy; golden chỉ chép output implementation.
- **Nghiệm thu:** `P00-AC05` — Mỗi invariant sớm có ca thành công và ca lỗi với oracle độc lập.
- **Kiểm chứng:** `P00-TC05`, V-01; dữ liệu kiểm tra: Danh mục EPUB 2/3, Unicode, lỗi IO, graph, secret sentinel.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t06"></a>

### P00-T06 — Kiểm chứng storage driver và worker

- **Mục tiêu / đầu ra:** ADR-002 trả lời SQLite transaction/lock/shutdown cho Phase 02.
- **Căn cứ / phụ thuộc:** D-TECH TD-05, D-DATA DB-13; T05.
- **Phạm vi:** Thử nghiệm storage độc lập và ADR-002.
- **Việc làm:** So lựa chọn bằng cùng ca transaction, busy writer, rollback, worker lỗi và đóng tiến trình.
- **Ca lỗi / biên:** Giữ transaction qua model request; writer bị mất nhưng caller tưởng commit; event loop bị block.
- **Nghiệm thu:** `P00-AC06` — Có mô hình writer/reader, lỗi và shutdown đã được kiểm chứng đủ cho slice đầu.
- **Kiểm chứng:** `P00-TC06`, V-02; dữ liệu kiểm tra: DB tạm, hai writer, lỗi giữa giao dịch và worker dừng.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t07"></a>

### P00-T07 — Chốt identity, hash và source offsets

- **Mục tiêu / đầu ra:** ADR-003 định nghĩa encoding canonical và ánh xạ nguồn.
- **Căn cứ / phụ thuộc:** Q04, S-INPUT, D-DATA DB-01–03/05; T05.
- **Phạm vi:** ADR-003 và fixture Unicode/XHTML.
- **Việc làm:** Ghi bản ghi versioned để hash, unique XHTML ID/fallback node path, grapheme/protected inline và range nửa mở.
- **Ca lỗi / biên:** Hai đoạn cùng chữ trùng ID; offset cắt ruby/marker; path khác dạng nhưng cùng tài nguyên.
- **Nghiệm thu:** `P00-AC07` — Ví dụ nguồn có ID/range kỳ vọng, tái chạy giữ kết quả; đổi snapshot không tái dùng mù.
- **Kiểm chứng:** `P00-TC07`, V-02; dữ liệu kiểm tra: Đoạn trùng chữ, combining marks, ruby, XHTML ID trùng và path chuẩn hóa.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t08"></a>

### P00-T08 — Chốt generation, CAS và migration cơ bản

- **Mục tiêu / đầu ra:** ADR-004 đủ cho write path và schema bootstrap Phase 02.
- **Căn cứ / phụ thuộc:** Q06–07, D-DATA DB-07/08/12/18; T06–T07.
- **Phạm vi:** ADR-004, transaction/migration protocol và thử nghiệm fault.
- **Việc làm:** Xác định bootstrap parent, writer lock/CAS, file-before-reference, replay/idempotency, backup và rollback.
- **Ca lỗi / biên:** Crash giữa file và DB; CAS stale; migration sửa file rồi lỗi; backup thiếu file liên quan.
- **Nghiệm thu:** `P00-AC08` — Mỗi điểm ngắt có trạng thái công bố/khôi phục rõ; DB không tham chiếu file chưa hoàn tất.
- **Kiểm chứng:** `P00-TC08`, V-02; dữ liệu kiểm tra: Hai generation cạnh tranh và ngắt trước/sau commit.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t09"></a>

### P00-T09 — Lập lịch ADR cho phase sau

- **Mục tiêu / đầu ra:** Các quyết định chưa dùng có owner và hạn trước task.
- **Căn cứ / phụ thuộc:** D-DATA, D-WORKFLOW, D-ROADMAP; T01, thông tin T06–T08.
- **Phạm vi:** Bảng quyết định trong roadmap/ADR backlog.
- **Việc làm:** Đối chiếu đủ TD-01–16, DB-01–18 và W7-01–14; ghi riêng phần đã chốt theo phạm vi, còn mở hoặc hoãn có điều kiện; gắn owner/task và hạn trước từng consumer. Rà thêm mục mở trong ADR/ghi chú như GC, durability và updater; không tự biến đề xuất thành quyết định.
- **Ca lỗi / biên:** Chốt IPC trước khi đo recovery; hoãn offset đến khi đã có dữ liệu; không có người nhận vấn đề.
- **Nghiệm thu:** `P00-AC09` — Không còn vấn đề chặn bị giấu trong ghi chú; quyết định tương lai chưa được đánh dấu đã chốt.
- **Kiểm chứng:** `P00-TC09`, V-01; dữ liệu kiểm tra: Toàn bộ TD/DB/W7 còn mở và task nhận.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

<a id="p00-t10"></a>

### P00-T10 — Bàn giao hai cổng M0

- **Mục tiêu / đầu ra:** Phase 01 và 02 có gói quyết định/fixture đủ nhận việc.
- **Căn cứ / phụ thuộc:** D-ROADMAP, R-REPO; M0-01 dùng T01–T05; M0-02 dùng T06–T08; lịch T09.
- **Phạm vi:** Báo cáo M0 và cập nhật trạng thái tài liệu.
- **Việc làm:** Tạo hai hồ sơ A riêng cho M0-01 và M0-02; mỗi hồ sơ có candidate, AC/review còn hiệu lực, lệnh và test đã/chưa chạy, dependency, thiếu sót và phần được phép bắt đầu. Rà liên kết/trạng thái ADR; walkthrough P01-T02 và P02-T01 từ nguồn tới AC; P01-T01 chỉ tiếp nhận/rà đầu vào, không lặp quyết định.
- **Ca lỗi / biên:** Ghi đủ điều kiện khi chỉ có đề xuất; gộp M0-02 thành rào cản không cần cho P01.
- **Nghiệm thu:** `P00-AC10` — Mỗi cổng có bằng chứng riêng và danh sách thiếu cụ thể; task sau biết phần nào được bắt đầu.
- **Kiểm chứng:** `P00-TC10`, V-01; dữ liệu kiểm tra: Hồ sơ ADR, log thử nghiệm, bảng dependency.
- **Báo cáo:** Hành vi/file đã đổi, contract/migration liên quan, lệnh và kết quả thực chạy, ca chưa chạy, việc còn chặn theo mục 16.

## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P00-AC01 | Mọi quyết định có nơi triển khai và nơi kiểm tra; không có yêu cầu bị đổi ngầm. | P00-T01 | P00-TC01; V-01 |
| P00-AC02 | Bộ lựa chọn có nguồn, phiên bản và bằng chứng khả thi; dependency chưa cần được hoãn. | P00-T02 | P00-TC02; V-01, V-02 |
| P00-AC03 | Consumer dùng được contract mà không nhập implementation adapter; không ép tạo toàn cây module. | P00-T03 | P00-TC03; V-01 |
| P00-AC04 | Mỗi đầu vào quan trọng có output/exit/lifecycle xác định; giữ đúng S-TUI. | P00-T04 | P00-TC04; V-01, V-02 khi cần thử terminal |
| P00-AC05 | Mỗi invariant sớm có ca thành công và ca lỗi với oracle độc lập. | P00-T05 | P00-TC05; V-01 |
| P00-AC06 | Có mô hình writer/reader, lỗi và shutdown đã được kiểm chứng đủ cho slice đầu. | P00-T06 | P00-TC06; V-02 |
| P00-AC07 | Ví dụ nguồn có ID/range kỳ vọng, tái chạy giữ kết quả; đổi snapshot không tái dùng mù. | P00-T07 | P00-TC07; V-02 |
| P00-AC08 | Mỗi điểm ngắt có trạng thái công bố/khôi phục rõ; DB không tham chiếu file chưa hoàn tất. | P00-T08 | P00-TC08; V-02 |
| P00-AC09 | Không còn vấn đề chặn bị giấu trong ghi chú; quyết định tương lai chưa được đánh dấu đã chốt. | P00-T09 | P00-TC09; V-01 |
| P00-AC10 | Mỗi cổng có bằng chứng riêng và danh sách thiếu cụ thể; task sau biết phần nào được bắt đầu. | P00-T10 | P00-TC10; V-01 |

Các AC không thay thế nhau. Build thành công, fixture trả đúng kết quả định sẵn hoặc một màn hiển thị đẹp không chứng minh toàn bộ hành vi nghiệp vụ.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P00-TC01 / P00-AC01 | 36 quyết định và bảng milestone. | Đối chiếu từng Q với task/phase và nguồn; yêu cầu chưa đủ căn cứ được ghi rõ. | V-01 |
| P00-TC02 / P00-AC02 | Manifest tối thiểu cho từng lựa chọn và target. | Build/test thử nghiệm đúng toolchain; ghi target chưa chạy, không suy từ build chéo. | V-01, V-02 |
| P00-TC03 / P00-AC03 | Graph hợp lệ và graph có cạnh cấm. | Duyệt được hướng phụ thuộc và cách checker Phase 01 nhận diện cạnh cấm. | V-01 |
| P00-TC04 / P00-AC04 | CLI case table và terminal setup lỗi từng bước. | Một người triển khai khác viết được test không phải đoán trường JSON hoặc hành vi shutdown. | V-01, V-02 khi cần thử terminal |
| P00-TC05 / P00-AC05 | Danh mục EPUB 2/3, Unicode, lỗi IO, graph, secret sentinel. | Rà nguồn/kỳ vọng và kiểm tra khả năng chạy khi không có dịch vụ bên ngoài. | V-01 |
| P00-TC06 / P00-AC06 | DB tạm, hai writer, lỗi giữa giao dịch và worker dừng. | Dữ liệu trước/sau và quyền ghi đúng; kết quả đo không chỉ là tốc độ. | V-02 |
| P00-TC07 / P00-AC07 | Đoạn trùng chữ, combining marks, ruby, XHTML ID trùng và path chuẩn hóa. | Oracle độc lập ánh xạ ngược về đúng node/đơn vị nguồn. | V-02 |
| P00-TC08 / P00-AC08 | Hai generation cạnh tranh và ngắt trước/sau commit. | Reopen thấy trọn mốc cũ hoặc mới; lịch sử không rơi vào trạng thái nửa giao dịch. | V-02 |
| P00-TC09 / P00-AC09 | Toàn bộ TD/DB/W7 còn mở và task nhận. | Mỗi vấn đề có phase/task nhận hoặc lý do hoãn; không tạo vòng phụ thuộc. | V-01 |
| P00-TC10 / P00-AC10 | Hồ sơ ADR, log thử nghiệm, bảng dependency. | Walkthrough một task P01 và P02 từ yêu cầu đến điều kiện nghiệm thu. | V-01 |

### 13.1. Lệnh và môi trường

Tên test/package là mục tiêu cần tạo trong task tương ứng; nếu đổi phải cập nhật kế hoạch. Lệnh exit 0 với 0 ca liên quan không được ghi là đạt. Phase 00 chủ yếu là quyết định và tài liệu. Không chạy test sản phẩm hoặc ghi xanh giả khi chưa có implementation; thử nghiệm nào không cần thì ghi lý do.

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | Rà nguồn/ADR, bảng truy vết và dependency; kiểm tra các link tới tài liệu có thật. |
| V-02 | Chạy các phép thử đã mô tả trong ADR bằng môi trường tạm; ghi manifest/toolchain/target và lệnh chính xác trong báo cáo. Nếu tạo Rust spike, chạy V-00 tại workspace của spike; trước PR Rust vẫn chạy V-00 từ repo root. |

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

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md): A thực hiện task, kiểm thử và tự review rồi bàn giao **Chờ review**; B đối chiếu yêu cầu, kiểm bản bàn giao và tự chạy kiểm chứng phù hợp trong phiên riêng. B ghi báo cáo độc lập, không sửa source/test chính thức hoặc trạng thái PLAN. A xử lý findings; B kiểm tra lại trên candidate mới. Quy tắc này áp dụng cả task tài liệu/ADR/thử nghiệm của phase.

- Quyết định có dựa vào đặc tả và bằng chứng, hay chỉ chọn vì thư viện quen thuộc?
- Có phân biệt hành vi sản phẩm với cấu trúc triển khai có thể thay đổi?
- Phép thử có chứng minh failure/recovery, hay chỉ happy path?
- Việc trì hoãn có deadline trước consumer và người nhận rõ không?

Rà thêm diff ngoài phạm vi, API công khai không có consumer, dependency cấm, error bị nuốt và test không được phát hiện/chạy. Không xóa hoặc nới test chỉ để qua kiểm tra. Sửa lỗi ở owner của quy tắc thay vì thêm ngoại lệ trong UI.

## 15. Điều kiện đóng phase

- [ ] Mọi task/nhóm trong phạm vi cổng có review B **Đạt** còn hiệu lực; đủ AC, kiểm tra bắt buộc và không còn finding chặn. A không tự duyệt phần mình triển khai.
- [ ] B đã review bản tích hợp cuối theo [mẫu review phase](../templates/PHASE_REVIEW.md); candidate và căn cứ khớp, bằng chứng các task còn áp dụng hoặc đã được kiểm lại theo ảnh hưởng.

M0-01 nhận review riêng cho T01–T05 và phần T10 bàn giao nền tảng; M0-02 nhận review riêng cho T06–T08 và phần T10 bàn giao storage. Đạt một cổng sớm không đóng toàn T10 hoặc Phase 00; các checkbox toàn phase bên dưới áp dụng khi đóng toàn M0.

- [ ] M0-01 và M0-02 đều có hồ sơ đầy đủ để đóng toàn Phase 00; có thể bàn giao M0-01 sớm.
- [ ] Lịch ADR cho Phase 03–07 tồn tại và không còn điểm chặn chưa có owner.
- [ ] Mọi kết luận thử nghiệm có dữ liệu đầu vào, kết quả và giới hạn bằng chứng.
- [ ] Tất cả task và AC của phase có bằng chứng, điểm chặn đã được giải quyết; ca chưa kiểm tra được ghi đúng trạng thái.
- [ ] V-00 áp dụng cho thay đổi Rust đã đạt; checks riêng trong mục 13 và các kiểm tra Go bị ảnh hưởng đã có kết quả.
- [ ] README được cập nhật nếu command/layout/nền tảng thay đổi; tài liệu nội bộ, key, EPUB, workspace và đầu ra tiếp tục ngoài Git.
- [ ] Bàn giao mục 16 đủ cho phase sau; không tự xóa đường Go hoặc công bố feature parity.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md); B dùng [mẫu review task](../templates/TASK_REVIEW.md), và [mẫu review phase](../templates/PHASE_REVIEW.md) khi nghiệm thu cổng. Báo cáo nội bộ đặt theo lượt trong `docs/roadmap/evidence/phase-00/`: `P00-TYY.handoff-r01.md` và `P00-TYY.review-r01.md`; giữ lượt trước khi sửa. Hồ sơ cổng dùng ID M0-01/M0-02/M0 hoặc M1…M9 theo phạm vi. Các mẫu chưa phải bằng chứng thực thi.

Sổ task/cổng khi thực hiện ghi trạng thái, candidate, link báo cáo A, review B, finding còn mở và bước tiếp theo. A cập nhật sổ này cùng PLAN sau bàn giao/kết luận; B chỉ ghi báo cáo review. Thiếu review B hoặc kiểm tra bắt buộc thì chưa đánh dấu hoàn thành.

### 16.1. Sổ trạng thái

  Ngày 08/11/2026: P00-T01-r03 được B review **Đạt** cho P00-AC01/P00-TC01; T01 hoàn thành. Người dùng chọn Windows trước, Linux sau. P00-T02-r02 được B khác phiên review độc lập **Đạt** cho P00-AC02/P00-TC02 trong phạm vi Windows. Nhóm P00-T03–T05 được ghi trước khi làm và B Đạt. Nhóm T06–T08 review r01/r02 ghi F01–F03; review kỹ thuật r03 ghi Cần sửa F04/F05 nhưng chưa đủ nhận diện B. A xử lý F04/F05 ở r04; B độc lập `gpt-6.1-sol` review **Đạt**, xác nhận 32/32 test trên MSRV/stable, 120/120 hash và đóng F01–F05. Nhóm T09–T10 đã đăng ký trước khi thực hiện.

| Gói | Task | Trạng thái | Bằng chứng |
|---|---|---|---|
| P00-A | T01 | Hoàn thành | `docs/roadmap/evidence/phase-00/P00-T01.handoff-r03.md`; review B **Đạt**: `docs/roadmap/evidence/phase-00/P00-T01.review-r03.md` (candidate P00-T01-r03; r02: **Cần sửa** — `docs/roadmap/evidence/phase-00/P00-T01.review-r02.md`; r01: `docs/roadmap/evidence/phase-00/P00-T01.review-r01.md`; diff r02 → r03: `docs/roadmap/evidence/phase-00/P00-T01.diff-r03.patch`) |
| P00-A | T02 | Hoàn thành (Windows) | Candidate `P00-T02-r02`; [handoff r02](../evidence/phase-00/P00-T02.handoff-r02.md); [review độc lập r02 — Đạt](../evidence/phase-00/P00-T02.review-r02-independent.md). Linux native hoãn theo người dùng. `P00-T02.review-r02.md` cùng-model được lưu lịch sử, không tính là review B. |
| P00-A/B | T03–T05 | Hoàn thành | Candidate `P00-T03-T05-r01`; [handoff A r01](../evidence/phase-00/P00-T03-T05.handoff-r01.md), [review độc lập B r01 — Đạt](../evidence/phase-00/P00-T03-T05.review-r01.md), [manifest r01](../evidence/phase-00/P00-T03-T05.candidate-r01.sha256), SHA-256 manifest `E665235E6BBD80A9F37DBA5D0BCD558F43C86DFA8B34B6C6586879016DFBEAE4`. |
| P00-C | T06–T08 | Hoàn thành | Candidate `P00-T06-T08-r04`: [handoff A](../evidence/phase-00/P00-T06-T08.handoff-r04.md), [review B — Đạt](../evidence/phase-00/P00-T06-T08.review-r04.md), [manifest](../evidence/phase-00/P00-T06-T08.candidate-r04.sha256). F01–F05 đóng. Review kỹ thuật r03 được giữ làm lịch sử, xem [báo cáo](../evidence/phase-00/P00-T06-T08.review-r03-technical.md). |
| P00-D | T09–T10 | **Hoàn thành** | Candidate `P00-T09-T10-r02`: [handoff A](../evidence/phase-00/P00-T09-T10.handoff-r02.md), [review B r02 — Đạt](../evidence/phase-00/P00-T09-T10.review-r02.md), [manifest](../evidence/phase-00/P00-T09-T10.candidate-r02.sha256); [review r01 — Cần sửa](../evidence/phase-00/P00-T09-T10.review-r01.md) giữ lịch sử; F01–F03 đóng. Dossier: [M0-01 r02](../evidence/phase-00/P00-T10-M0-01.dossier-r02.md), [M0-02 r02](../evidence/phase-00/P00-T10-M0-02.dossier-r02.md). M0-01 và M0-02 đều đã Đạt tích hợp riêng. |
| P00-M0 | M0-01 | **Hoàn thành — B Đạt** | Candidate `M0-01-r01`; [handoff A](../evidence/phase-00/M0-01.handoff-r01.md), [review B r01 — Đạt](../evidence/phase-00/M0-01.review-r01.md), [manifest](../evidence/phase-00/M0-01.candidate-r01.sha256). Phạm vi T01–T05 + dossier M0-01; bàn giao nền tảng Windows cho P01-T01. Phase 01 implementation/Linux chưa nghiệm thu. |
| P00-M0 | M0-02 | **Hoàn thành — B Đạt** | Candidate `M0-02-r01`; [handoff A](../evidence/phase-00/M0-02.handoff-r01.md), [review B r01 — Đạt](../evidence/phase-00/M0-02.review-r01.md), [manifest](../evidence/phase-00/M0-02.candidate-r01.sha256). Phạm vi T06–T08 + dossier M0-02 trên Windows; Phase 02 vẫn cần M1. |
| P00-M0 | Toàn phase M0 | **M0-r02: Cần sửa — B** | [Handoff](../evidence/phase-00/M0.handoff-r02.md), [manifest](../evidence/phase-00/M0.candidate-r02.sha256), [review B r02](../evidence/phase-00/M0.review-r02.md). M0-F01 đóng; M0-F02 mở vì manifest thiếu review r01 và 20 tệp evidence V-00/audit. A tạo lượt mới; phase chưa đóng. |
| P00-M0 | Toàn phase M0 | **Hoàn thành — B Đạt trên Windows** | Candidate `M0-r03`; [handoff](../evidence/phase-00/M0.handoff-r03.md), [manifest](../evidence/phase-00/M0.candidate-r03.sha256), [review B — Đạt](../evidence/phase-00/M0.review-r03.md). M0-F01/F02 đóng; Linux native hoãn theo quyết định người dùng. |

Ghi chú nhóm T09–T10 (08/11/2026): A kiểm kê đủ 16 TD, 18 DB và 14 W7 ID; phân loại trạng thái, ghi owner/hạn trước consumer và đưa mục ADR ngoài ID vào lịch. B review r01 kết luận **Cần sửa** F01–F03; A cập nhật ADR-007/ADR-008 và walkthrough. B review r02 kết luận **Đạt**, đóng F01–F03; kiểm độc lập 48 ID, 13 hàng off-ID, 118 owner refs, 405 liên kết và 57/57 hash, handoff khớp đầu/cuối. M0-01-r01 và M0-02-r01 sau đó đều qua review tích hợp Đạt riêng; đóng toàn Phase 00 vẫn theo exit gate §15.

Ghi chú T01 (07/11/2026): r01–r02 xử lý review B (Cần sửa, F01–F07); r03 nối Q05 từ lõi [P04-T02](PHASE-04.md#p04-t02) tới ranh giới người dùng [P05-T13](PHASE-05.md#p05-t13) và [P05-T15](PHASE-05.md#p05-t15). Review B r03 kết luận **Đạt** cho đúng candidate; F01 đóng, F02–F07 giữ Đóng, Q07 deferred không chặn P00-AC01. Xem [handoff r03](../evidence/phase-00/P00-T01.handoff-r03.md) và [review r03](../evidence/phase-00/P00-T01.review-r03.md).

Ghi chú T02 (07/11/2026): Windows 1.99.1 fmt/clippy/test/release và Windows MSRV 1.88.0 test đã chạy lại đạt trong tiền kiểm r01; model B độc lập tự chạy kiểm chứng tương ứng và xác nhận candidate r02. Người dùng quyết định “linux build sau, chỉ build win trước thôi”; Linux native được chuyển sang phần việc sau. T02 hoàn thành trong phạm vi này. [Handoff r02](../evidence/phase-00/P00-T02.handoff-r02.md) và [review độc lập r02 — Đạt](../evidence/phase-00/P00-T02.review-r02-independent.md) là bằng chứng hiện hành; [tiền kiểm r01](../evidence/phase-00/P00-T02.precheck-r01.md) lưu chi tiết của A.

Ghi chú nhóm T03–T05 (07/11/2026): người dùng yêu cầu gom T03, T04, T05 để review một lần. Candidate `P00-T03-T05-r01` được B độc lập (gpt-6.1-sol, phiên review riêng) kết luận **Đạt** cho AC03/04/05, TC03/04/05 và V-01; 20/20 hash khớp, không finding chặn. Xem [handoff A](../evidence/phase-00/P00-T03-T05.handoff-r01.md), [review B](../evidence/phase-00/P00-T03-T05.review-r01.md). Đây là review tài liệu, không phải chứng minh implementation Phase 01 đã chạy. Sau verdict, PLAN đã đăng ký nhóm T06–T08 trước khi triển khai.

Ghi chú nhóm T06–T08 (08/11/2026): T06 storage worker/transaction, T07 source identity/hash/range và T08 generation/CAS/file-reference/migration liên quan trực tiếp tới write path Phase 02. T05 là dependency ngoài nhóm đã B Đạt; T06–T07–T08 được thực hiện nối tiếp và chưa task nào hoàn tất tới khi cả nhóm đủ AC và review. Review r01/r02 ghi F01–F03; A xử lý no-clobber ở r03. Review kỹ thuật r03 ghi Cần sửa F04/F05 và không xác nhận model B; A xử lý ở r04 bằng transaction bootstrap có rollback/retry cùng đồng bộ ADR/test strategy. Review độc lập B r04 kết luận **Đạt** cho AC06–08/TC06–08; 32/32 ca mỗi toolchain, F01–F05 đóng, 120/120 hash. Xem [handoff](../evidence/phase-00/P00-T06-T08.handoff-r04.md), [review B](../evidence/phase-00/P00-T06-T08.review-r04.md), [manifest](../evidence/phase-00/P00-T06-T08.candidate-r04.sha256). Nhóm T09–T10 đã đăng ký trước khi làm.

### 16.2. Báo cáo mỗi task

Ghi task/AC, thay đổi hành vi, file/module, quyết định và tác động migration/compatibility; lệnh đã chạy, toolchain/OS, tên/số ca liên quan, kết quả; test thủ công và ca chưa chạy; rủi ro còn lại và task có thể nhận tiếp. Nếu lưu log/báo cáo, dùng `docs/roadmap/evidence/phase-00/`, đã redaction, không chứa dữ liệu cá nhân hoặc nội dung sách thật.

### 16.3. Đầu ra cho bước tiếp theo

- [Phase 01](PHASE-01.md): ADR-001, phần bootstrap/API của ADR-008, [test strategy](../TEST_STRATEGY.md) và contract UI-01.
- [Phase 02](PHASE-02.md): ADR-002/003/004, fixture source identity/commit và quy trình schema/migration.
- [Roadmap](../ROADMAP.md): lịch các quyết định cần trước Phase 03–07; không tuyên bố chúng đã được giải quyết.

Trạng thái kế hoạch không phải trạng thái implementation. Chỉ cập nhật hoàn thành khi có bằng chứng của hành vi thực tế.

<a id="p00-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Dùng hồ sơ **M0-r03** đã review trên Windows, gồm hai cổng bàn giao **M0-01-r01** và **M0-02-r01**. Bài thử của phase này là đọc và đối chiếu quyết định với bằng chứng bàn giao.

#### 16.4.2. Chuẩn bị

Dành khoảng 10 phút. Mở tài liệu nội bộ và giữ sẵn [bảng truy vết 36 quyết định](../ROADMAP.md#5-truy-vết-36-quyết-định) để tra theo mã Q.

#### 16.4.3. Mở đúng bản

Bắt đầu từ [review M0-r03](../evidence/phase-00/M0.review-r03.md). Kiểm mã candidate ở đầu báo cáo và kết luận ở mục 5. Sau đó mở [review M0-01](../evidence/phase-00/M0-01.review-r01.md) và [review M0-02](../evidence/phase-00/M0-02.review-r01.md).

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P00-UT01 — Từ yêu cầu tới bài test | Tìm Q02 trong bảng truy vết, mở task Phase 02 được dẫn rồi tìm AC/TC tương ứng. | Thấy quy tắc giữ quyết định phân loại theo vị trí, hash EPUB và version mapping, cùng ca kiểm khi một trong hai căn cứ đổi. |
| P00-UT02 — Đầu vào Phase 01 | Từ review M0-01, mở ADR-001/008 rồi đối chiếu mục 4 và mục 9 của Phase 01. | Phạm vi Windows, khung CLI/TUI và mốc nhận việc Phase 01 có căn cứ rõ. |
| P00-UT03 — Đầu vào Phase 02 | Từ review M0-02, mở ADR-002/003/004 và điều kiện bắt đầu của Phase 02. | Tìm được quy tắc writer, ID/range nguồn, thứ tự lưu file trước khi DB tham chiếu, và điều kiện cần thêm M1. |

#### 16.4.5. Thử ca lỗi và mở lại

**P00-UT04 — Đối chiếu một lỗi từng chặn bàn giao.** Mở [review M0-r02](../evidence/phase-00/M0.review-r02.md) và tìm M0-F02. Mở lại review r03, đối chiếu cùng ID và bằng chứng còn thiếu đã được bổ sung. Ca đạt khi hai lượt cho thấy đúng lý do r02 cần sửa và r03 đóng finding trong phạm vi Windows.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Nếu tài liệu lệch nhau hoặc link hỏng, ghi ID quyết định hoặc finding, candidate, tên hai tài liệu và đoạn cần đối chiếu.

#### 16.4.7. Thử lại từ đầu

Mở lại các báo cáo đúng lượt ở mục 16.4.3. Dùng kết luận của từng lượt làm mốc khi kiểm lại.
