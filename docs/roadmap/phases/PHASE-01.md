# PHASE 01 — NỀN TẢNG RUST, CLI VÀ KHUNG TUI

> Hướng nền tảng đã ghi ở ADR-001/008; contract P01-D04–D06 có review độc lập Đạt ở P00-T03–T05. **M0-01-r01 đã Đạt trên Windows**; xem [review tích hợp cổng](../evidence/phase-00/M0-01.review-r01.md). Toàn Phase 00 sau đó được B review **Đạt trên Windows** ở candidate [M0-r03](../evidence/phase-00/M0.review-r03.md), đủ điều kiện bắt đầu Phase 01. Điều này chưa nghiệm thu implementation/test Phase 01 hoặc M1.
>
> **Căn cứ lập kế hoạch:** đặc tả và thiết kế của dự án. Theo yêu cầu ngày 06/11/2026, code hiện tại không được dùng làm chuẩn thiết kế, đầu vào tái sử dụng hoặc cơ sở ước lượng phần đã hoàn thành. Phase này được phân rã từ hành vi cần đạt và ranh giới kiến trúc cần giữ.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID | `P01` |
| Tên | Rust Workspace Foundation |
| Phiên bản kế hoạch | Draft 0.6 — 09/11/2026 |
| Trạng thái thực thi | P01-T01–T09 đã được B review Đạt theo candidate ghi tại mục 16. P01-T10-r05 B **Đạt trên Windows**, F01–F03 đóng; xem [review r05](../evidence/phase-01/P01-T10.review-r05.md), SHA-256 705CF794A13D01304EA90E33940951FAD521E9F04DF9C8B15AD155B2B3439D61. P01-T11-r01 B **Đạt trên Windows** cho AC-03–06/12/14 và V-03; xem [review r01](../evidence/phase-01/P01-T11.review-r01.md), SHA-256 2ED665147F7C1661DAD96960EEC683677F8041FDFD71897CC50901D885276A7B. P01-T12-r05 **Hoàn thành, B Đạt trên Windows**; V-05 70/70, workspace 108/108, subprocess 1/1, fmt/clippy/debug build đạt; review report SHA-256 DAC0F2FCAE2BF131FCD0120930C27A13B59B329C5666D72533C3A82A0148BB88. F01 đóng. P01-T13-r01 B Chưa đủ bằng chứng; r02 B Cần sửa/F01. P01-T13-r03 B Cần sửa, F01 đóng/F02 mở; report SHA-256 10854FD066CF1B585151E09285FB1EBF86F95225E6CD8DF0AFCC16FB2842D75D. P01-T13-r04 B gpt-6.1-sol/max **Chưa đủ bằng chứng**, F01/F02 đóng; review SHA-256 62E19D98FA553BA2F9DB90AAC0BF8E75BF626D05C0AE77F9FC668B0CBCDF7F7A. Manifest review-time 383/383, SHA-256 0DBF72CDFDA621648921AC1DA7C5F1349FB6DB54C3DC028FD120A00429560FB5. V-00 108/108, V-02 10/10, discovery 108, cache sạch offline workspace 108/108, Actionlint/PowerShell đạt. R04 hosted Rust CI xanh nhưng Go CI Windows fail hai test chọn workspace; r05 giữ đường dẫn metadata ban đầu và cả Rust CI lẫn Go CI Đạt trên commit `de738e9098677be6d1418e820789d68fcb5e7e12`; V-07 offline 108/108, 31 rule tạm được xóa. P01-T13-r05 **Hoàn thành, B gpt-6.1-sol/max Đạt trên Windows**; F01/F02 giữ đóng; [review r05](../evidence/phase-01/P01-T13.review-r05.md), SHA-256 `1EE8F6B3F2E702C9E7ACD0C8B75B6596500E2C6BE0C0F4C3FF1936B574030F80`. Candidate `de738e9098677be6d1418e820789d68fcb5e7e12`, manifest 395/395; hosted Rust/Go CI và V-07 đạt. P01-T14-r01 B Cần sửa; F01/F02 mở theo [review r01](../evidence/phase-01/P01-T14.review-r01.md), SHA-256 `88FF43864C444A98194040E8ACD4C0EBC9D15DF49EDED8130ADB7E740E4DEEAD`. P01-T14-r02 Hoàn thành, B Đạt; F01/F02 đóng; [review r02](../evidence/phase-01/P01-T14.review-r02.md), SHA-256 `9A31B3169243703BB7632CD08A3A905C81804078B7EA9397EB270F904D1BB3F1`. M1-r01 đã nghiệm thu, B Đạt trên Windows: PR #1 commit 14edc793988ecffed7961bc48ac2b65c12ba9eed; Rust/Go hosted CI success trên synthetic merge `2748b6ead1eb27ee1d9a95f839ff1b638ba18398`; [checks report](../evidence/phase-01/P01-T14-r02-hosted-checks.md). Raw log API trả 403 do cần repo-admin; M1-r01 **Hoàn thành, B Đạt trên Windows** theo [review tích hợp](../evidence/phase-01/M1.review-r01.md), SHA-256 `2DA5D8F1CCB96C5BAB3D297D791827A8F8C0A8C0DAA81FECDA03EFDC32780A27`; [handoff](../evidence/phase-01/M1.handoff-r01.md) và [manifest](../evidence/phase-01/M1.candidate-r01.sha256) được B xác minh 420/420 ở thời điểm review. PR #1 còn Draft, chờ quyết định merge; Linux native hoãn theo người dùng.
| Milestone | `M1` — nền tảng build/test/lint, dependency boundaries và shell dùng được |
| Vị trí trong roadmap | Sau các quyết định Phase 00 cần cho nền tảng; trước Phase 02 Persistence & EPUB Foundation |
| Gói giao diện | `UI-01` — khung TUI, điều hướng, focus, trợ giúp và thoát |
| Phân rã | 6 gói việc, 14 task; tiến độ theo từng task và bằng chứng nghiệm thu |
| Nhánh đích khi triển khai Rust | Base `rewrite/rust`; nhánh implementation local hiện tại `feature/tui` theo AGENTS.md |
| Người triển khai / rà soát | Model A triển khai / B review độc lập trong phiên riêng. B T08-r04 được cấu hình `gpt-6.1-sol` nhưng runtime metadata không có; B T09 dùng phiên riêng theo cùng cấu hình |

Nguồn điều phối là [Phase 01 trong Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-01), [gói UI-01](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#ui-work-packages) và [AGENTS.md](../../../AGENTS.md). Tài liệu nội bộ nằm trong `docs/`, được loại khỏi Git.

Xem [roadmap chung và Phase 00–09](../ROADMAP.md) để theo dõi điều kiện bàn giao, UI và truy vết 36 quyết định.

## 2. Mục tiêu và demo cuối phase

### 2.1. Kết quả cần đạt

Tạo nền tảng để Phase 02 có thể gắn chức năng tạo/mở workspace và nhập EPUB mà không phải tổ chức lại entrypoint, chiều phụ thuộc, vòng đời terminal hoặc cơ chế kiểm thử.

Kết thúc phase, executable `novel-translate-cli` có thể mở khung TUI bằng bàn phím, trình bày đúng trạng thái chưa có dữ liệu, hiển thị trợ giúp, xử lý lỗi khởi động và trả terminal về trạng thái sử dụng bình thường. Workspace Rust có bộ kiểm tra tự động chạy được trên các nền tảng đã chọn.

### 2.2. Kịch bản demo bắt buộc

1. Từ repo root, build executable bằng cấu hình đã ghi trong kế hoạch.
2. Chạy `novel-translate-cli --help` và `novel-translate-cli --version`; nhận kết quả rồi trở về shell.
3. Chạy không tham số trong terminal tương tác; vào Tổng quan với trạng thái **Chưa mở workspace**, **Chưa có tác vụ**.
4. Dùng bàn phím chọn một trong tám màn; thấy focus, tên màn và hướng dẫn thao tác phù hợp.
5. Mở trợ giúp bằng `F1`, đóng bằng `Esc`; trở về đúng màn và control trước đó.
6. Chuyển kích thước 80×24 → 120×24 → dưới 80×24 → 80×24; giữ màn, mục chọn và focus.
7. Kích hoạt control **Thoát**; terminal hiển thị con trỏ và nhận nhập như trước khi mở app.
8. Chạy một lệnh sai trong chế độ không tương tác; trả lỗi/mã thoát đúng, kết thúc hữu hạn, không mở TUI.

Demo chỉ chứng minh nền tảng và shell. Khả năng dịch, lưu workspace, Review hoặc xuất EPUB được nghiệm thu trong các phase tương ứng.

## 3. Phạm vi

### 3.1. Phần triển khai trong phase

| Nhóm | Mức hoàn thiện cần có |
|---|---|
| Cargo Workspace | Manifest, lockfile, toolchain, package executable, crate boundaries và cấu hình build/test |
| Domain/Application | Khung thư viện, nguyên tắc API/ownership và các kiểu tối thiểu có consumer thực tế trong phase |
| Composition root | Chọn chế độ CLI/TUI, nối các thành phần shell, quản lý khởi động và dừng |
| CLI | Không tham số, help, version, lỗi tham số, chế độ không tương tác và đầu ra lỗi JSON cơ bản |
| Diagnostics | Phân loại lỗi, thông báo an toàn, logging có redaction, tách stdout/stderr |
| TUI | Khung tám màn, trạng thái trống, điều hướng, focus, trợ giúp, thoát, resize và Unicode |
| Terminal | Thu nhận/giải phóng tài nguyên; xử lý lỗi giữa chừng và panic có thể unwind |
| Kiểm thử | Unit/contract/integration phù hợp từng task, kiểm thử terminal thực và dependency boundaries |
| CI và hướng dẫn | Các lệnh Rust bắt buộc, test offline, hướng dẫn chạy phần đã hoàn thành |

### 3.2. Phần bàn giao cho phase sau

| Chức năng | Nơi triển khai theo roadmap |
|---|---|
| Tạo/mở workspace, SQLite, migration, SourceSnapshot, EPUB/mapping | Phase 02 |
| Revision, generation mở rộng, Editor lưu bản sửa, Skills/approval | Phase 03 |
| Luồng Dịch–Review bằng fake provider | Phase 04 |
| Provider thật, thông tin xác thực, usage/budget, Coordinator/IPC | Phase 05 |
| Dịch–Review–Research–Repair đầy đủ | Phase 06 |
| Ảnh, Preview, EPUB export | Phase 07 |
| Hoàn thiện nghiệp vụ qua tám màn và toàn bộ CLI | Phase 08 |
| Hardening, nghiệm thu phát hành và feature parity | Phase 09 |

Các màn trong Phase 01 chỉ có shell và trạng thái phù hợp với dữ liệu chưa tồn tại. Không tạo bản dịch, phần trăm tiến độ, chi phí, model đang chạy hoặc thông báo lưu/xuất thành công để lấp chỗ trống. Không phát triển adapter DB/model/IPC trước consumer của phase sau.

## 4. Điều kiện bắt đầu

### 4.1. Điều kiện chung

- Nhận đầu vào từ cổng M0-01 của [Phase 00](PHASE-00.md). P01-T01 đối chiếu và tiếp nhận ADR còn phù hợp, không ra lại quyết định đã có bằng chứng; M0-02 về lưu trữ chưa phải điều kiện của shell.
- Người thực hiện đã đọc các nguồn ở mục 5 và nhận một task cụ thể trong mục 11.
- Phần quyết định kỹ thuật mà task sử dụng đã có lựa chọn và lý do được ghi nhận theo mục 6. Không yêu cầu giải quyết ADR của những phase chưa liên quan.
- Có môi trường build theo toolchain được chọn, terminal để kiểm tra thủ công và nơi tạo dữ liệu kiểm thử tạm.
- Việc tải compiler/dependency phục vụ chuẩn bị môi trường được tách khỏi thực thi test. Test không phụ thuộc mạng, tài khoản hoặc model thật.
- Trước khi sửa repo, ghi nhận thay đổi đang có để không ghi đè công việc khác. Việc này bảo vệ dữ liệu làm việc, không dùng implementation cũ làm căn cứ thiết kế.

### 4.2. Cách xử lý khi chưa đủ điều kiện

Task `P01-T01` có thể bắt đầu bằng việc đối chiếu đặc tả và ghi các lựa chọn còn thiếu. Task code chỉ phụ thuộc những quyết định liên quan trực tiếp đến nó. Ghi rõ điểm bị chặn, tác động và task vẫn làm được; không tự ghi mọi điều kiện là đã đạt.

Nếu lựa chọn triển khai làm thay đổi hành vi đã chốt, tách vấn đề ra để có quyết định rõ trước phần bị ảnh hưởng. Các lựa chọn triển khai thông thường được giải quyết và ghi nhận trong phạm vi task.

## 5. Đặc tả và quy tắc phải giữ

### 5.1. Bảng nguồn tham chiếu

| ID | Nguồn | Nội dung áp dụng trong Phase 01 |
|---|---|---|
| S01 | [36 quyết định](../../spec/DESIGN_DECISIONS_36.md) | Hành vi sản phẩm đã chốt; đặc biệt quyết định 34–36 |
| S02 | [TUI/CLI spec](../../spec/TUI_CLI_SPEC.md) | Executable, CLI, tám màn, bàn phím, resize, stdout/stderr và mã thoát |
| S03 | [Security spec](../../spec/SECURITY_SPEC.md) | Redaction, bảo vệ key và không tự đọc/lưu nguồn thông tin xác thực |
| S04 | [Kiến trúc Rust — Bước 3](../../desgin/RUST_PROJECT_ARCHITECTURE.md) | Vai trò crate, chiều phụ thuộc và composition root |
| S05 | [Module design — Bước 4](../../desgin/MODULE_DESIGN_STEP4.md) | Ownership, public API, ranh giới Presentation/Application/Infrastructure |
| S06 | [Nền tảng kỹ thuật — Bước 5](../../desgin/TECHNICAL_FOUNDATION_STEP5.md) | Toolchain, framework, lỗi có kiểu, logging và các quyết định TD |
| S07 | [UI/UX — Bước 8](../../desgin/UI_UX_DESIGN_STEP8.md) | Shell, route, trạng thái UI, bố cục và trợ giúp |
| S08 | [Roadmap — Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) | M1, UI-01, thứ tự phase và Definition of Done |
| S09 | [Quy tắc repo](../../../AGENTS.md) | Nhánh Rust, bảo toàn Go/dữ liệu, lockfile và kiểm tra trước PR |

Khi có khác biệt, đối chiếu yêu cầu hiện hành của người dùng, quy tắc repo và các quyết định sản phẩm đã chốt. Nội dung đề xuất trong tài liệu thiết kế không tự trở thành hành vi bắt buộc hoặc quyết định kỹ thuật đã được thông qua.

### 5.2. Các bất biến của phase

1. Tên executable là `novel-translate-cli`; chạy không tham số trong terminal tương tác mở TUI.
2. TUI có đúng tám màn chính. Help/dialog/panel không được đếm thành màn chính mới.
3. `F1` mở trợ giúp; `Esc` đóng lớp đang mở hoặc trở lại một cấp; không tự thoát app.
4. Phím thuộc control/modal có focus không kích hoạt hành động phía sau.
5. Điều hướng, render, resize và trợ giúp không phát request model hay tạo thay đổi nghiệp vụ.
6. Domain không phụ thuộc UI, runtime hoặc adapter IO; Application không phụ thuộc implementation Infrastructure/Presentation.
7. UI chỉ giữ trạng thái trình bày và gọi contract Application khi có use case. Không chứa logic ghi dữ liệu hay gọi provider.
8. Bí mật không xuất hiện trong log, lỗi, JSON, panic diagnostic hoặc báo cáo kiểm thử.
9. Test tự động offline, deterministic; không đọc key thật hoặc gọi provider trả phí.
10. Đường chạy Go và dữ liệu người dùng được giữ theo S09. Lập kế hoạch từ đầu không đồng nghĩa được phép xóa code hoặc dữ liệu ngoài phạm vi task.

## 6. ADR và điểm còn mở

Toàn Phase 00 candidate `M0-r03` đã được B review **Đạt trên Windows**; đầu vào M0-01 cho Phase 01 có review riêng tại [review cổng M0-01-r01](../evidence/phase-00/M0-01.review-r01.md), và báo cáo tích hợp nằm tại [review M0-r03](../evidence/phase-00/M0.review-r03.md). Các lựa chọn và contract Phase 01 trong ADR-001/008 đã có bằng chứng phù hợp. Điều đó chưa chứng minh implementation, test hoặc M1 của Phase 01 đã đạt. Bảng dưới tách quyết định đã đủ để task phụ thuộc bắt đầu khỏi kiểm chứng implementation còn phải làm.

| ID | Lựa chọn đã chốt và lý do | Task phụ thuộc / phần kiểm chứng còn lại | Trạng thái và nguồn |
|---|---|---|---|
| P01-D01 | Rust stable **1.99.1** làm compiler ghim; MSRV **1.88.0**, edition **2024**, resolver **3**. Ratatui 0.30.2 đặt MSRV cao nhất trong bộ trực tiếp; MSRV phải được kiểm riêng, không suy từ compiler ghim. Windows `x86_64-pc-windows-msvc` là target hiện hành theo quyết định người dùng ngày 07/11/2026; Linux native hoãn tới lượt triển khai sau. | T02 dùng các giá trị này cho workspace; T13 kiểm CI/build/test trên Windows. T02 phải kiểm manifest thật bằng toolchain ghim, còn kiểm MSRV bằng 1.88.0. Linux chưa thuộc phạm vi hiện hành. | **Đủ quyết định; kiểm chứng workspace/CI còn mở.** [ADR-001](../../decisions/ADR-001.md) §2–3; [M0-r03 Đạt trên Windows](../evidence/phase-00/M0.review-r03.md); quyết định target trong [PLAN](../../../PLAN.md). |
| P01-D02 | Tạo đúng bốn package đã chốt: app `novel-translate` (binary `novel-translate-cli`), `nt-domain`, `nt-application`, `nt-tui`. Chỉ cho phép chiều phụ thuộc domain → không crate sản phẩm; application → domain; TUI → application; app → application/TUI. Dependency chỉ được khai báo khi có consumer; không tạo adapter/contract giả. Cách này giữ composition root ở app và không để Domain phụ thuộc UI/IO. | T02 dựng package và ownership; T03 kiểm graph Cargo thật, kể cả dependency đổi tên/target/build dependency, cùng ca âm. | **Đủ quyết định cho scaffold; graph chưa kiểm.** [ADR-008](../../decisions/ADR-008.md) §2–3; mục 7.1–7.2 của phase; contract P00-T03 được B review Đạt trong [nhóm T03–T05](../evidence/phase-00/P00-T03-T05.review-r01.md). |
| P01-D03 | Giữ đúng phiên bản/feature trong ADR-001 §2: Ratatui **0.30.2** (`crossterm_0_29`), Crossterm **0.29.0**, `crossterm_winapi` **0.9.1** chỉ trên `cfg(windows)` cho record phím native của T08, Clap **4.6.7**, tracing **0.1.44**, tracing-subscriber **0.3.23**, serde/serde_json **1.0.229/1.0.154**, thiserror **2.0.21**, anyhow **1.0.104**. Chọn theo consumer: UI/input ở TUI, parser/sink ở app, serialization tại boundary; không bật feature/dependency không dùng. Bộ version/features có compatibility spike Windows, nhưng spike không chứng minh graph/manifests sản phẩm. | T02 khai báo dependency theo consumer; T03 rà graph/features và ownership; T08 giữ native repeat count/physical key identity trên Windows; T04–T10 dùng đúng thư viện khi có consumer. Kiểm MSRV/build tích hợp ở T02/T13. | **A đã kiểm V-02 trên candidate `P01-T08-r03`; chờ review B.** [ADR-001 v0.7](../../decisions/ADR-001.md) §2–4; [ADR-008 v0.6](../../decisions/ADR-008.md) §2–5; bằng chứng spike và giới hạn được tổng kết trong [M0-r03](../evidence/phase-00/M0.review-r03.md). |
| P01-D04 | Bootstrap phân biệt help/version, lỗi parse, command, cờ mode và TTY trước mọi side effect. Chỉ không tham số với stdin **và** stdout là TTY mới vào TUI; nếu thiếu TTY thì lỗi hữu hạn exit 3. `--json`/`--non-interactive` thiếu command trả exit 2; JSON là một object UTF-8 đủ 8 field, schema v1, field type/nullability và exit mapping theo ADR-008 §5.1; stdout không lẫn log/ANSI, diagnostic đã lọc ở stderr. Command-specific `result` chỉ thêm khi command có consumer. | T04 triển khai parse/composition; T05 kiểm lỗi/diagnostic; T11 kiểm process, exit, stdout/stderr và side effect. Test hợp đồng tài liệu không thay process test. | **Contract đủ rõ và P00-T04 được B review Đạt; process tests còn mở.** [ADR-008 §5.1](../../decisions/ADR-008.md#p01-d04), [TUI/CLI spec](../../spec/TUI_CLI_SPEC.md), [review nhóm T03–T05](../evidence/phase-00/P00-T03-T05.review-r01.md). |
| P01-D05 | Shell dùng event loop đồng bộ vì hiện không có async use case/timer; không thêm runtime cho tương lai chưa có consumer. `nt-tui` là terminal owner duy nhất; acquire và cleanup theo thứ tự đảo ngược, best-effort/idempotent; lỗi gốc được giữ, cleanup error là phụ. Guard xử lý panic unwind; abort/force-kill ngoài bảo đảm. Test event dùng seam điều khiển được, không dựa sleep. | T06 triển khai terminal lifecycle/shutdown; T07–T10 nối shell và kiểm trạng thái/điều hướng; T12 kiểm terminal thật, cleanup và panic trong phạm vi bắt được. CI không TTY không được xem là bằng chứng demo terminal. | **Contract đủ rõ và được review Đạt; implementation và kiểm terminal còn mở.** [ADR-001](../../decisions/ADR-001.md) §8; [ADR-008 §5.2](../../decisions/ADR-008.md#p01-d05); [review nhóm T03–T05](../evidence/phase-00/P00-T03-T05.review-r01.md). |
| P01-D06 | Diagnostic TUI chỉ đi qua ring buffer tối đa **256 record** với sequence, severity/component/event allowlist và occurrence count; overflow bỏ record cũ nhất, tăng `dropped_count`. Không lưu message/args/env/URL/error/panic payload tự do. Redact trước buffer, serialize, stderr hoặc render; giữ lỗi cuối tới sau cleanup. Lý do là không làm bẩn TUI và giảm rò dữ liệu nhạy cảm. | T05 xây typed diagnostic/redaction; T06 nối sink vào lifecycle; T12 kiểm tích hợp, sentinel, lỗi cuối và output sau cleanup. | **Contract đủ rõ và được review Đạt; regression/sentinel tests còn mở.** [ADR-008 §5.3](../../decisions/ADR-008.md#p01-d06), [Security spec](../../spec/SECURITY_SPEC.md), [review nhóm T03–T05](../evidence/phase-00/P00-T03-T05.review-r01.md). |

Không có điểm sản phẩm chưa chốt đang chặn P01-T02: quyết định cần cho scaffold/contract đã có nguồn, lý do và phạm vi. Những điều chưa chạy là kiểm chứng implementation thuộc task phía sau, không được đánh dấu đạt trước. Nếu dependency thực tế không thỏa MSRV/feature đã chọn hoặc thiết kế đề xuất mâu thuẫn spec, dừng phần phụ thuộc và ghi ADR trước khi đổi quyết định. CI không có terminal tương tác thì chỉ chạy được process/headless checks; phần TUI terminal thật vẫn phải kiểm ở T12 và cổng M1. SQLite, SourceRange/hash, generation, IPC, provider và checkpoint chưa phải điều kiện bắt đầu Phase 01. Không đưa Tokio hoặc async trait vào Domain; nhu cầu runtime chỉ được mở lại khi có consumer thực tế.

## 7. Thiết kế triển khai

### 7.1. Cấu trúc mục tiêu của phase

Đã thống nhất tạo bốn crate sản phẩm trước theo ADR-008; đây là phần đầu của kiến trúc mở rộng ở Bước 3–4, không thay đổi trách nhiệm các adapter sẽ bổ sung sau.

```text
Cargo.toml                         # workspace, default member là app
Cargo.lock                         # được commit
rust-toolchain.toml                 # toolchain đã chọn
apps/
  novel-translate/
    Cargo.toml                     # package novel-translate
    src/
      main.rs                      # executable novel-translate-cli
      bootstrap.rs
      cli/
      diagnostics/
    tests/
      architecture_boundaries.rs
      cli_contract.rs
crates/
  nt-domain/
    Cargo.toml
    src/lib.rs
  nt-application/
    Cargo.toml
    src/lib.rs
  nt-tui/
    Cargo.toml
    src/
      lib.rs
      app/
      terminal/
      event/
      navigation/
      state/
      components/
      screens/
.github/workflows/rust-ci.yml
```

Tên module con có thể điều chỉnh theo trách nhiệm khi triển khai. Không tạo sẵn toàn bộ cây module của các phase sau; không thêm entity, port, mock repository hoặc test rỗng chỉ để làm đầy crate. Crate chưa có logic nghiệp vụ có thể chỉ định nghĩa ranh giới và tài liệu trách nhiệm.

### 7.2. Ownership và chiều phụ thuộc

| Thành phần | Sở hữu | Phụ thuộc nội bộ được phép trong phase |
|---|---|---|
| `nt-domain` | Kiểu/quy tắc nghiệp vụ thuần khi thực sự cần | Không phụ thuộc crate sản phẩm khác |
| `nt-application` | Contract/use case và lỗi nghiệp vụ khi có consumer | `nt-domain` |
| `nt-tui` | UI state, event routing, render, terminal lifecycle | `nt-application` khi sử dụng contract |
| `novel-translate` | CLI, bootstrap, composition root, diagnostic sink | `nt-application`, `nt-tui` |

Danh sách trên là quyền phụ thuộc, không bắt buộc khai báo dependency chưa được dùng. `main` quyết định chế độ chạy và nối thành phần; `nt-tui` không tự khởi tạo DB/provider. State màn và focus thuộc TUI; task, revision, approval và workspace nghiệp vụ sẽ thuộc core trong các phase sau.

Kiểm tra kiến trúc phải đọc metadata/graph của Cargo, nhận diện package dù dependency được đổi tên, xét normal/build dependency và cấu hình theo target/features. Dev dependency phục vụ kiểm thử được đánh giá riêng, không được làm adapter lọt vào đường chạy sản phẩm. Cần ca âm chứng minh checker phát hiện một cạnh cấm và chỉ ra đường phụ thuộc gây lỗi.

### 7.3. Hợp đồng khởi động và CLI tối thiểu

| Đầu vào | Hành vi đích của Phase 01 |
|---|---|
| Không tham số, có terminal tương tác | Mở TUI, trạng thái chưa có workspace/tác vụ |
| `--help` hoặc `--version` | Trả văn bản ra stdout, mã 0; không khởi tạo terminal hoặc workspace |
| Cờ/lệnh không hỗ trợ hoặc đối số sai | Mã 2, thông báo và hướng xử lý; không mở TUI |
| `--non-interactive` nhưng thiếu lệnh | Mã 2, kết thúc ngay; không chờ người dùng nhập |
| `--json` nhưng thiếu lệnh | Mã 2, một kết quả lỗi JSON; không mở TUI |
| Lỗi tham số kèm `--json` | JSON có đủ envelope của S02; diagnostic an toàn ở stderr, không có ANSI/trace trong stdout |
| Không tham số nhưng stdin/stdout không đủ điều kiện tương tác | Lỗi môi trường terminal, mã 3; trả hữu hạn, không thử bật raw mode |
| Lỗi khởi tạo hoặc IO khi TUI đang chạy | Cleanup phần đã cấp; trả mã 3 và diagnostic an toàn |

Các quy tắc kết hợp cờ và môi trường ở bảng được chốt tại [ADR-008 §5.1](../../decisions/ADR-008.md#p01-d04). `--help`/`--version` vẫn là output văn bản khi đi kèm `--json`; envelope JSON áp dụng cho kết quả/lỗi lệnh. Các cờ trong bảng được parse trước khi khởi tạo terminal. Schema v1 và nullability đã chốt ở ADR-008; command-specific result contract sẽ được thêm cùng implementation của command.

Envelope giữ các field `schema_version`, `command`, `state`, `task_id`, `scope`, `result`, `issues`, `next_action`. Khi lỗi xảy ra trước khi có task/workspace, không sinh ID hoặc kết quả giả; biểu diễn giá trị vắng mặt theo hợp đồng [P01-D04](../../decisions/ADR-008.md#p01-d04). Schema v1, kiểu/nullability và state đã được chốt trong candidate nhóm T03–T05; fixture process/JSON được triển khai và kiểm ở task Phase 01 tương ứng.

Các lệnh nghiệp vụ như `workspace create`, `import`, `translate`, `review`, `status` và `export` được bổ sung ở phase tương ứng với đúng cú pháp S02. Help của bản Phase 01 chỉ quảng bá phần dùng được. Mã 4–7 và ngữ nghĩa `status` được giữ nguyên theo S02, chưa được xem là đã nghiệm thu trong phase này.

### 7.4. Luồng sự kiện TUI

```text
Sự kiện bàn phím/resize
  → modal hoặc control đang có focus
  → hành động UI được phép
  → cập nhật state trình bày
  → render khung hiện tại
```

Hành động nghiệp vụ chỉ được nối khi có use case ở phase sau. Render không ghi dữ liệu, không tự tạo task và không tự gọi model. Phase 01 dùng event loop đồng bộ; nguồn sự kiện được điều khiển trực tiếp trong test và không dùng timer thực. Xem [ADR-001 §8](../../decisions/ADR-001.md#p01-d05-event-loop) và [ADR-008 §5.2](../../decisions/ADR-008.md#p01-d05).

## 8. Dữ liệu, lỗi và phục hồi

### 8.1. Dữ liệu và side effect

| Dữ liệu/tài nguyên | Vòng đời và quy tắc |
|---|---|
| Route, focus, mục chọn, scroll, trạng thái help | Trong bộ nhớ; giữ qua điều hướng/resize; chưa cam kết giữ sau khi thoát app |
| Thông tin version và khả năng của shell | Từ build/config kỹ thuật đã chọn; không giả lập dữ liệu sách/model |
| Diagnostics | Field cho phép, đã redaction; buffer giới hạn khi TUI mở; không tự tạo log trong workspace |
| Raw mode, alternate screen, cursor | Có owner và trạng thái đã cấp; thu hồi khi thoát hoặc lỗi thuộc phạm vi xử lý |
| Fixture kiểm thử | Chuỗi Nhật/Việt tự soạn, lỗi/sự kiện tổng hợp, file tạm; không dùng sách/key/workspace cá nhân |

Phase 01 chưa có schema hoặc migration dữ liệu nghiệp vụ. Khởi động, help và điều hướng không tự dò/import workspace, đọc `.env`, lấy key hệ điều hành hay ghi lại cấu hình người dùng. Kiểm thử xác nhận các file sentinel trong thư mục làm việc không thay đổi.

### 8.2. Ma trận lỗi và phục hồi

| Trường hợp | Kết quả cần có |
|---|---|
| Parse CLI lỗi | Trả lỗi đúng định dạng/mã; terminal chưa bị thay đổi |
| Không có terminal phù hợp | Báo điều kiện thiếu, gợi ý dùng help/lệnh CLI khả dụng; không treo |
| Lỗi sau khi bật raw mode nhưng trước khi setup hoàn tất | Thử khôi phục tài nguyên đã cấp theo thứ tự ngược, gồm bước hiện tại nếu có side effect một phần; giữ nguyên lỗi khởi tạo làm nguyên nhân chính |
| Lỗi read/draw trong event loop | Dừng loop, cleanup, trả thất bại; không in xong lỗi rồi báo mã 0 |
| Cleanup một bước lỗi | Tiếp tục thử các bước cleanup còn lại; thêm mã chẩn đoán an toàn, không che lỗi gốc |
| Panic có thể unwind | Hook/guard phối hợp cleanup; không in raw panic payload có thể chứa bí mật; tiến trình kết thúc thất bại |
| Resize nhỏ hơn mức hỗ trợ | Giữ state, hiển thị hướng dẫn vừa khung; khi đủ kích thước khôi phục đúng vị trí |
| Control chưa có chức năng nghiệp vụ | Trình bày chưa khả dụng hoặc không hiển thị control; không báo thành công |

Phạm vi phục hồi terminal gồm thoát bình thường, lỗi được trả về và panic có thể unwind. Kết thúc tiến trình cưỡng bức/abort không được ghi là đã bảo đảm cleanup. Phase 01 chưa có transaction/checkpoint để nghiệm thu crash recovery dữ liệu.

### 8.3. Chính sách diagnostic

- Log theo danh sách field cho phép như mã sự kiện, thành phần và loại lỗi; không dump toàn bộ args, env, config hoặc chuỗi lỗi nguồn.
- Redaction trước khi dữ liệu tới sink, JSON hoặc UI. Cả error chain, `Display`/`Debug` và panic path phải được xem xét.
- Dùng sentinel bí mật giả trong test, gồm giá trị không có tiền tố quen thuộc, header xác thực và URL có credential/token.
- Trong CLI, stdout dành cho output đã quy định. Trong TUI, log không chen trực tiếp vào vùng render; lỗi cuối được trình bày sau cleanup.
- Không nuốt lỗi để tiếp tục giả thành công; không dùng `unwrap`/`expect` trên input, IO hoặc terminal mà không có bất biến đã chứng minh.

## 9. Phần TUI/CLI

### 9.1. Tám route của shell

| Màn | Nội dung trong Phase 01 | Chức năng nối về sau |
|---|---|---|
| Tổng quan | Chưa mở workspace, chưa có tác vụ; cách điều hướng và trợ giúp | Dữ liệu workspace và tiến độ |
| Tập và chương | Trạng thái chưa có nguồn; không có cây sách giả | Import, cây sách, mapping |
| Tác vụ | Chưa có tác vụ; không có model/chi phí đang chạy | Queue, budget, pause/resume |
| Editor Nhật–Việt | Chưa chọn nội dung; không cho lưu revision giả | Đọc/sửa/lưu và lịch sử |
| Reviewer | Chưa có kết quả Review | Issue, bằng chứng và Review lại |
| Skills và ngữ cảnh | Chưa có dữ liệu để xem hoặc duyệt | Thuật ngữ, nhân vật và approval |
| Ảnh và xuất | Chưa có nội dung để kiểm tra/xuất | Ảnh, Preview và export |
| Cấu hình, báo cáo và trợ giúp | Thông tin bản build, trợ giúp khả dụng | Kết nối, model, budget và báo cáo |

Thông báo trong sản phẩm dùng từ người dùng hiểu được. Không đưa mã phase/task, tên crate hay ADR vào luồng giao diện.

### 9.2. Tương tác cần hoàn thiện

| Tương tác | Hành vi nghiệm thu |
|---|---|
| Mũi tên + `Enter` tại vùng điều hướng | Đổi mục chọn và mở đúng route |
| `Tab` | Đi qua các vùng/control tương tác được; focus nhìn thấy, không mắc kẹt |
| `F1` | Mở help của màn/ngữ cảnh hiện tại, chỉ nêu phím đang dùng được |
| `Esc` trong help/subview | Đóng/quay lại một cấp, khôi phục focus trước đó |
| `Esc` ở màn gốc | Giữ app mở; không biến thành lệnh thoát |
| Control **Thoát** | Dùng được bằng bàn phím; thực hiện shutdown/cleanup |
| `Ctrl+C` khi shell chưa có công việc | Khi event source nhận được, đây là yêu cầu shutdown và đi cùng đường cleanup với control Thoát. Kill/abort/signal không bắt được nằm ngoài bảo đảm theo [ADR-008 §5.2](../../decisions/ADR-008.md#p01-d05) |
| Gõ phím vào control nhập liệu trong harness | Input nhận phím; không kích hoạt route/hành động phía sau |
| `PageUp` / `PageDown` | Cuộn nội dung view theo trang khi có overflow; không đổi route/focus/selection; không hoạt động trong help/modal hoặc khi view vừa khung |
| Resize | Giữ state; đổi bố cục theo 80/120 cột; không hiện control bị che như vẫn bấm được |

Harness input chỉ kiểm chứng cơ chế dispatch dùng chung. Không triển khai Editor nghiệp vụ để phục vụ bài test shell. Phím `/`, `f`, `e`, `p`, `r`, `Ctrl+S` chỉ xuất hiện khi chức năng tương ứng được nối; không tái gán trái ý nghĩa S02.

### 9.3. Mức kiểm chứng giao diện

Kiểm tra tự động state/action và buffer render ở kích thước xác định. Kiểm tra thủ công trên terminal thật cho con trỏ, màu/focus dễ nhận biết, tiếng Nhật/Việt, resize và khả năng gõ bình thường sau thoát. Snapshot chỉ bổ trợ; tiêu chí chính là hành vi và dữ liệu còn đúng.

## 10. Gói việc và thứ tự

| Gói | Task | Đầu ra để rà soát | Phụ thuộc |
|---|---|---|---|
| P01-A — Chốt đầu vào và workspace | T01–T02 | Quyết định nền tảng cần dùng, workspace build được | Đặc tả/thiết kế |
| P01-B — Ranh giới và entrypoint | T03–T04 | Kiểm tra kiến trúc, CLI/khởi động có contract | P01-A |
| P01-C — Diagnostics và terminal | T05–T06 | Lỗi an toàn, vòng đời terminal có fault injection | T03–T04 và quyết định liên quan |
| P01-D — Khung UI-01 | T07–T10 | Tám route, focus/help, resize/Unicode | T06 |
| P01-E — Tích hợp và kiểm thử luồng | T11–T12 | Process-level CLI, shell demo, lỗi và cleanup xuyên lớp | Các task tạo hành vi tương ứng |
| P01-F — CI và bàn giao | T13–T14 | CI đủ checks, hướng dẫn chạy và hồ sơ M1 | CI khởi tạo sau T02; nghiệm thu sau T11–T12 |

Thứ tự đề xuất là T01 → T02 → T03 → T04 → T05 → T06 → T07 → T08 → T09 → T10 → T11 → T12 → hoàn tất T13 → T14. Phần wiring CI của T13 có thể làm sớm sau T02 rồi bổ sung test theo các task.

Mỗi task gồm code, test phù hợp và tự rà soát. T11–T12 kiểm chứng đường chạy xuyên các thành phần đã hoàn thành, không dời toàn bộ kiểm thử đến cuối phase. Không giao cả gói D như một lần viết toàn bộ giao diện. Nếu task cần thay đổi quá nhiều trách nhiệm, tách tiếp bằng hậu tố như `P01-T08a`, giữ liên kết đến AC và dependency ban đầu.

## 11. Thẻ task cho AI

**Quy tắc dùng chung:** Mỗi task chỉ sửa phần được nêu và test trực tiếp liên quan. Đọc S09 trước khi làm; bảo toàn thay đổi ngoài phạm vi. Trạng thái khởi tạo của toàn bộ task là **Chưa bắt đầu**. Các lệnh `V-*` và ca `TC-*` được định nghĩa ở mục 13; không được ghi test đạt khi chưa có kết quả chạy.

<a id="p01-t01"></a>

### P01-T01 — Chốt đầu vào cần cho Phase 01

- **Mục tiêu:** Có đủ quyết định để scaffold và viết contract mà không tự suy từ code cũ.
- **Tham chiếu/phụ thuộc:** S01–S09; chưa phụ thuộc task code.
- **Phạm vi:** Kế hoạch này và các phần liên quan trong ADR-001/ADR-008; chưa sửa source.
- **Việc làm:** Đối chiếu P01-D01–D06; ghi lựa chọn/version khi đã xác định; ghi task bị chặn đối với mục chưa giải quyết; xác nhận cú pháp/JSON và chính sách terminal.
- **Đầu ra:** Bảng quyết định có nguồn, lý do, trạng thái; không đánh dấu ADR DB/IPC/provider là điều kiện của shell.
- **Ca cần xét:** Thiết kế đề xuất khác spec; thiếu terminal trên CI; thư viện không tương thích MSRV; hợp đồng JSON chưa rõ kiểu.
- **Nghiệm thu:** Task phụ thuộc nhận được quyết định đủ cụ thể; các điểm chưa chốt được chỉ rõ, không ẩn bằng mặc định.
- **Kiểm chứng:** Rà tài liệu và liên kết; chưa có lý do chạy test sản phẩm. Bằng chứng ghi ở mục 16.

<a id="p01-t02"></a>

### P01-T02 — Tạo Cargo Workspace và cấu hình build

- **Mục tiêu:** Bốn crate theo mục 7 có thể build từ repo root.
- **Tham chiếu/phụ thuộc:** S04–S06, S09; T01 và P01-D01/D02.
- **Phạm vi:** Root manifests/lockfile/toolchain; manifests và entry/lib tối thiểu ở `apps/`, `crates/`.
- **Việc làm:** Cấu hình workspace/default member; đặt executable đúng tên; pin toolchain; khai báo dependency/features theo nhu cầu; tạo lockfile.
- **Đầu ra:** Workspace build được, artifact có tên đúng, đường dẫn nhất quán; chưa tạo toàn bộ module nghiệp vụ.
- **Ca cần xét:** Chạy từ root, đường dẫn có khoảng trắng, compiler/MSRV không khớp, lockfile thiếu hoặc lỗi thời.
- **Nghiệm thu:** `AC-01`; manifest không phụ thuộc file nội bộ trong `docs/` để build.
- **Kiểm chứng:** `V-01`, `V-00` cho trạng thái nền tảng; không thêm test rỗng để tăng số lượng.

<a id="p01-t03"></a>

### P01-T03 — Khóa chiều phụ thuộc và public API tối thiểu

- **Mục tiêu:** Thay đổi dependency sai được phát hiện tự động.
- **Tham chiếu/phụ thuộc:** S04–S05; T02.
- **Phạm vi:** Library exports, manifest dependencies và test `architecture_boundaries` của app.
- **Việc làm:** Thiết lập allowlist theo mục 7.2; kiểm tra graph; chỉ công khai contract có consumer; để nội bộ private hoặc `pub(crate)`.
- **Đầu ra:** Checker báo rõ cạnh cấm/đường phụ thuộc; cách bổ sung adapter ở phase sau được ghi trong tài liệu test.
- **Ca cần xét:** Application → TUI/storage; Domain → framework IO; dependency đổi tên; cạnh optional/feature/target; external package cùng tên allowlist nhưng lấy từ path; `[patch.crates-io]` đổi nguồn đã resolve; external package bắc cầu vào crate sản phẩm; dependency ngoài owner allowlist; nhầm dev dependency với runtime.
- **Nghiệm thu:** `AC-02`; graph hợp lệ qua, fixture graph vi phạm thất bại đúng lý do.
- **Kiểm chứng:** `V-02`, ca `TC-02`; fixture âm không sửa manifest thật của người dùng.

<a id="p01-t04"></a>

### P01-T04 — Entry point, CLI tối thiểu và composition root

- **Mục tiêu:** Chọn đúng CLI/TUI trước khi gây side effect.
- **Tham chiếu/phụ thuộc:** S02, S05–S06; T03 và P01-D03/D04.
- **Phạm vi:** App `main`, `bootstrap`, `cli`; contract Application chỉ khi có consumer; test CLI liên quan.
- **Việc làm:** Parse help/version/cờ hỗ trợ; phân loại thiếu/sai lệnh; xác định interactive mode; map lỗi ra exit/output đã thống nhất; để một điểm nối TUI.
- **Đầu ra:** Help/version dùng được; CLI không tương tác không khởi tạo terminal; envelope lỗi được định nghĩa và có fixture.
- **Ca cần xét:** Unknown command; thiếu giá trị; `--json`/`--non-interactive` thiếu lệnh; stdin/stdout redirect; tên/version không đúng build.
- **Nghiệm thu:** `AC-03`, `AC-04`; chỗ nối TUI được hoàn tất và nghiệm thu process-level ở T11.
- **Kiểm chứng:** `V-03`, `TC-03`/`TC-04`; fake TUI launcher chỉ nằm trong test.

<a id="p01-t05"></a>

### P01-T05 — Lỗi có kiểu, logging và redaction

- **Mục tiêu:** Có chẩn đoán hữu ích mà không lộ dữ liệu nhạy cảm hoặc làm bẩn output.
- **Tham chiếu/phụ thuộc:** S02–S03, S06; T04 và P01-D06.
- **Phạm vi:** App diagnostics/output mapping; kiểu lỗi tối thiểu ở boundary liên quan; unit/contract tests.
- **Việc làm:** Phân loại startup/argument/terminal error; giữ nguyên nhân chính; thiết lập safe fields, redaction và sink; thống nhất thông báo/next action.
- **Đầu ra:** CLI stdout sạch; stderr an toàn; TUI có sink phù hợp; lỗi nguồn không được dump nguyên văn.
- **Ca cần xét:** Secret lồng trong error chain, URL/header, `Debug`, panic; log burst vượt giới hạn buffer; diagnostic khi cleanup cũng lỗi.
- **Nghiệm thu:** `AC-05`, `AC-06`; sentinel vắng mặt ở mọi output/sink được kiểm tra.
- **Kiểm chứng:** `V-04`, `TC-05`; test không lấy bất kỳ credential thật nào.

<a id="p01-t06"></a>

### P01-T06 — Vòng đời terminal và đường shutdown

- **Mục tiêu:** Terminal trở lại dùng được sau thoát, lỗi và panic trong phạm vi xử lý.
- **Tham chiếu/phụ thuộc:** S02, S06; T05 và P01-D05.
- **Phạm vi:** `nt-tui/terminal`, app lifecycle/wiring và test fault injection.
- **Việc làm:** Một owner quản lý raw mode/alternate screen/cursor; setup theo bước; cleanup best effort trên phần đã cấp; nối shutdown và safe panic diagnostic.
- **Đầu ra:** Entry point không dùng process exit trước cleanup; lỗi runtime/cleanup được trả đúng lên app.
- **Ca cần xét:** Thất bại tại từng bước setup, read/draw lỗi, cleanup lỗi một bước, cleanup được yêu cầu lặp, panic unwind.
- **Nghiệm thu:** `AC-05`, `AC-13`; đường lỗi không trả mã thành công và vẫn thử các bước khôi phục còn lại.
- **Kiểm chứng:** `V-05`, `TC-06`; fake terminal kiểm tra thứ tự, terminal thật được kiểm tra ở T12.

<a id="p01-t07"></a>

### P01-T07 — Khung bố cục, tám route và trạng thái trống

- **Mục tiêu:** Có shell trình bày đúng cấu trúc sản phẩm.
- **Tham chiếu/phụ thuộc:** S02, S05, S07; T06.
- **Phạm vi:** `nt-tui/app`, `navigation`, `state`, `screens`, components chung.
- **Việc làm:** Tạo header/nội dung/footer theo thiết kế được chọn; tám route mục 9.1; tên màn, vùng focus và trạng thái chưa có dữ liệu.
- **Đầu ra:** Mở không tham số tới Tổng quan; điều hướng có đích; không có sách/task/model đang chạy giả.
- **Ca cần xét:** Danh sách rỗng; route chưa có nghiệp vụ; status chưa có task; content dài trong khung.
- **Nghiệm thu:** `AC-07`, `AC-12`; render thuần không phát side effect.
- **Kiểm chứng:** `V-05`, `TC-07`; kiểm tra route/state/buffer có ý nghĩa, không chỉ so ảnh toàn màn.

<a id="p01-t08"></a>

### P01-T08 — Điều hướng và focus bằng bàn phím

- **Mục tiêu:** Mọi control thuộc shell dùng được bằng bàn phím và xử lý đúng ngữ cảnh.
- **Tham chiếu/phụ thuộc:** S02, S07; T07.
- **Phạm vi:** Event/action dispatch, router, focus và test UI; trên Windows, giữ native key identity/repeat count trước khi dispatch.
- **Việc làm:** Điều hướng bằng mũi tên/Enter; vòng focus Tab; ưu tiên modal/control nhập liệu; giữ vị trí khi quay lại; nối control Thoát; bảo toàn phím vật lý đang giữ và repeat native trên Windows.
- **Đầu ra:** Focus luôn xác định; route không nhận phím đã được control phía trước xử lý.
- **Ca cần xét:** `Esc` ở gốc; Tab với ít control; danh sách rỗng; Space/Enter/ký tự trong harness input; phím lặp không tạo hai lần kích hoạt ngoài ý muốn.
- **Nghiệm thu:** `AC-08`, `AC-12`; không có phím mặc định tự gọi nghiệp vụ.
- **Kiểm chứng:** `V-05`, `TC-08`; chuỗi input xác định, không sleep dựa vào thời gian thực.

<a id="p01-t09"></a>

### P01-T09 — Trợ giúp theo ngữ cảnh và thông báo khả dụng

- **Mục tiêu:** Người dùng biết thao tác nào dùng được và trở lại đúng vị trí sau help.
- **Tham chiếu/phụ thuộc:** S02, S07; T08.
- **Phạm vi:** Help dialog/subview, footer và thông báo trạng thái shell.
- **Việc làm:** `F1` theo màn/focus; `Esc` đóng help; footer phản ánh control; thao tác chưa có use case không báo thành công.
- **Đầu ra:** Help của tám route khớp phím thật; không để event lọt xuống màn dưới.
- **Ca cần xét:** Mở help liên tiếp; help tại màn rỗng; route/focus khác nhau; đóng help sau resize.
- **Nghiệm thu:** `AC-09`, `AC-12`; không dùng thông báo chứa mã kỹ thuật nội bộ trong luồng người dùng.
- **Kiểm chứng:** `V-05`, `TC-09`; đối chiếu nội dung help và hành vi cùng control.

<a id="p01-t10"></a>

### P01-T10 — Resize và hiển thị Nhật–Việt

- **Mục tiêu:** Shell giữ trạng thái và đọc được ở kích thước hỗ trợ.
- **Tham chiếu/phụ thuộc:** S02, S07; T09.
- **Phạm vi:** Layout/rendering, wrap/scroll dùng chung, fixture Unicode và test kích thước.
- **Việc làm:** Bố cục từ 80×24; breakpoint 120 cột; trạng thái quá nhỏ; wrap theo ô hiển thị; PageUp/PageDown cuộn nội dung dài theo trang; không sửa nội dung gốc để vừa khung.
- **Đầu ra:** Màn đang chọn, help/focus và scroll còn đúng sau resize; Unicode không bị cắt theo byte; phím cuộn chỉ hiện khi có nội dung vượt khung.
- **Ca cần xét:** 79×24, 80×23, 80×24, 119×24, 120×24; resize liên tiếp; PageUp/PageDown ở đầu/giữa/cuối và khi nội dung vừa khung; Nhật/Việt, dấu kết hợp và dòng dài; kích thước rất nhỏ không panic.
- **Nghiệm thu:** `AC-10`, `AC-11`, `AC-12`; không để nút bị che tiếp tục nhận kích hoạt như đang thấy được.
- **Kiểm chứng:** `V-05`, `TC-10`/`TC-11`; kiểm tra buffer và state trước/sau, bổ sung quan sát terminal thật ở T12.

<a id="p01-t11"></a>

### P01-T11 — Tích hợp CLI ở mức tiến trình

- **Mục tiêu:** Binary thật tuân theo contract khởi động/output mà các test thành phần không đủ chứng minh.
- **Tham chiếu/phụ thuộc:** S02–S03; T04–T06 và shell T07–T10.
- **Phạm vi:** App `tests/cli_contract.rs`, wiring bootstrap cần sửa khi tích hợp.
- **Việc làm:** Spawn binary Cargo tạo; capture stdout/stderr/exit; kiểm tra help/version, lỗi JSON, non-interactive và non-TTY; chạy trong thư mục tạm.
- **Đầu ra:** Test process có giới hạn thời gian để phát hiện treo; cleanup child ở đường thất bại; không gọi provider.
- **Ca cần xét:** Output chứa ANSI/log lẫn JSON, sai exit, thiếu field, chờ nhập ngoài ý muốn, tự tạo workspace/log/key file.
- **Nghiệm thu:** `AC-03`–`AC-06`, `AC-12`, `AC-14`; parser JSON kiểm tra toàn bộ stdout, không chỉ tìm một dòng hợp lệ.
- **Kiểm chứng:** `V-03`, `TC-03`–`TC-05`, `TC-12`; kiểm tra cả process result và file sentinel.

<a id="p01-t12"></a>

### P01-T12 — Kiểm thử luồng shell và terminal thực

- **Mục tiêu:** Chứng minh demo mục 2 chạy trọn vẹn, bao gồm đường lỗi đã nối.
- **Tham chiếu/phụ thuộc:** S02, S07; T05–T10.
- **Phạm vi:** TUI integration harness, test lifecycle, hướng dẫn/bằng chứng kiểm thử thủ công; sửa lỗi đúng boundary phát hiện được.
- **Việc làm:** Chạy chuỗi mở → điều hướng → help → resize → thoát; fault injection setup/render/read/panic; kiểm tra stdout/stderr sau cleanup.
- **Đầu ra:** Bằng chứng tự động và quan sát terminal thật, ghi OS/terminal/kích thước và phần chưa kiểm tra.
- **Ca cần xét:** Resize khi help mở; thoát từ các route; lỗi sau setup một phần; panic có secret giả; cleanup thất bại một bước.
- **Nghiệm thu:** `AC-07`–`AC-14`; sau thoát terminal có con trỏ và gõ được; không tuyên bố PTY/manual đạt từ headless test.
- **Kiểm chứng:** `V-05`, `V-06`, `TC-06`–`TC-12`; fault injection chỉ là công cụ test, không thêm lệnh gây crash cho người dùng.

<a id="p01-t13"></a>

### P01-T13 — CI cho build, test và kiểm tra kiến trúc

- **Mục tiêu:** Thay đổi phá nền tảng bị chặn bằng kiểm tra tự động có thể tái lập.
- **Tham chiếu/phụ thuộc:** S06, S09; wiring sau T02, hoàn tất khi T03–T12 có kết quả.
- **Phạm vi:** Rust CI, cấu hình cache/toolchain và wiring tests; không vô hiệu hóa kiểm tra Go.
- **Việc làm:** Chạy đủ V-00; thêm architecture check vào test suite; chạy matrix đã chọn; chuẩn bị dependency trước test offline; không cần secrets.
- **Đầu ra:** CI fail khi command/test fail; lockfile được dùng; không bỏ check bằng `continue-on-error` hoặc allow warning toàn cục.
- **Ca cần xét:** Máy CI sạch, cache miss, dependency cần mạng trong test, target/feature khác nhau, test treo, zero tests do filter sai.
- **Nghiệm thu:** `AC-14`, `AC-15`; log phân biệt chuẩn bị môi trường với thực thi kiểm thử; CI Go vẫn giữ nguyên các cổng bắt buộc.
- **Kiểm chứng:** `V-00`, `V-02`, `V-07` và kết quả CI tương ứng. Khi chưa có run CI, ghi chưa xác minh trên runner.

<a id="p01-t14"></a>

### P01-T14 — Hướng dẫn sử dụng, rà soát và bàn giao M1

- **Mục tiêu:** Người khác chạy lại được shell và biết Phase 02 sẽ nối vào đâu.
- **Tham chiếu/phụ thuộc:** S08–S09; T01–T13 đã có đầu ra/bằng chứng.
- **Phạm vi:** README cho hành vi/layout đã thay đổi; kế hoạch, bảng tiến độ và báo cáo nội bộ.
- **Việc làm:** Ghi cách build/chạy/check; phân biệt khả năng Rust đã có và Go; thực hiện demo; đối chiếu toàn bộ AC; ghi hạn chế và contract cho Phase 02.
- **Đầu ra:** README dùng được mà không cần tài liệu `docs/` riêng tư; hồ sơ bàn giao ở mục 16; không tuyên bố feature parity.
- **Ca cần xét:** Hướng dẫn trỏ sai executable/path; link nội bộ xuất hiện như tài liệu công khai; test chưa chạy bị ghi pass; scope chưa làm bị ghi hoàn tất.
- **Nghiệm thu:** `AC-16`, đủ điều kiện mục 15.
- **Kiểm chứng:** Rà tài liệu, demo và bằng chứng hiện có; chỉ chạy lại kiểm tra khi có thay đổi hoặc lỗi mới cần xác minh.

## 12. Tiêu chí nghiệm thu

Mỗi AC chỉ đạt khi có bằng chứng đúng phạm vi. Không dùng một lần build thành công thay cho các tiêu chí hành vi.

| ID | Điều kiện quan sát được | Task chịu trách nhiệm | Bằng chứng |
|---|---|---|---|
| AC-01 | Workspace build từ root, executable đúng tên, toolchain/lockfile nhất quán | T02 | V-01, V-00 |
| AC-02 | Không có cạnh phụ thuộc cấm; checker phát hiện fixture vi phạm | T03 | TC-02 |
| AC-03 | Help/version đúng và exit 0; lỗi tham số exit 2; không khởi tạo TUI ở các trường hợp này | T04, T11 | TC-03 |
| AC-04 | Non-interactive/non-TTY không treo; JSON đúng envelope, stdout không lẫn diagnostic | T04, T11 | TC-04 |
| AC-05 | Lỗi khởi động/runtime/cleanup giữ đúng nguyên nhân và trả thất bại | T05, T06, T11 | TC-05, TC-06 |
| AC-06 | Sentinel bí mật không lọt vào text/JSON/log/error chain/panic diagnostic | T05, T11, T12 | TC-05, TC-06 |
| AC-07 | Không tham số mở Tổng quan; có đúng tám route với trạng thái trống chính xác | T07, T12 | TC-07, V-06 |
| AC-08 | Mũi tên/Tab/Enter/Esc theo focus; input/modal không kích hoạt màn phía sau | T08, T12 | TC-08 |
| AC-09 | F1 mở đúng help; Esc khôi phục route/focus; footer đúng hành động khả dụng | T09, T12 | TC-09 |
| AC-10 | 80×24 và 120×24 dùng được; qua ngưỡng hẹp/rộng/quá nhỏ giữ state | T10, T12 | TC-10, V-06 |
| AC-11 | Nhật/Việt/dấu kết hợp/dòng dài không hỏng dữ liệu và không panic khi render | T10, T12 | TC-11, V-06 |
| AC-12 | Help/điều hướng/resize không gọi mạng/model, tạo task hoặc sửa file nghiệp vụ | T07–T12 | TC-12 |
| AC-13 | Thoát, lỗi được xử lý và panic unwind thực hiện cleanup terminal; kiểm tra thực tế sau thoát | T06, T12 | TC-06, V-06 |
| AC-14 | Test dùng fixture tổng hợp, không key/model/mạng thật, không phụ thuộc clock/sleep ngẫu nhiên | T11–T13 | TC-12, V-07 |
| AC-15 | CI chạy đủ kiểm tra bắt buộc và kiến trúc trên matrix đã quyết định; thất bại không bị bỏ qua | T13 | V-00, V-02, log CI |
| AC-16 | Hướng dẫn phản ánh hành vi đã có; giữ Go và dữ liệu; có bàn giao rõ cho Phase 02 | T14 | Rà diff, demo, hồ sơ mục 16 |

## 13. Ma trận kiểm thử

### 13.1. Ca kiểm thử

| Ca | Lớp kiểm tra | Fixture/đầu vào | Kỳ vọng | AC |
|---|---|---|---|---|
| TC-01 | Build/metadata | Workspace theo toolchain/lockfile | Package và executable đúng; build không đọc tài liệu nội bộ | 01 |
| TC-02 | Architecture | Graph Cargo đầy đủ và fixture có cạnh cấm/rename/target/feature, source path/patch và cạnh bắc cầu | Nhận package thực/nguồn đã resolve, owner và đường vi phạm; dev edge được tách riêng | 02 |
| TC-03 | CLI process | Help, version, unknown command, thiếu tham số | stdout/stderr/exit đúng; tiến trình kết thúc | 03 |
| TC-04 | CLI process | Cờ JSON/non-interactive và stream được redirect | Không raw mode/prompt; JSON parse trọn output và có đủ kiểu/field đã chốt | 04 |
| TC-05 | Unit + contract | Nested error, secret sentinel, URL/header, sink đầy | Redaction đúng; còn error code/next action hữu ích; không mất lỗi gốc | 05, 06 |
| TC-06 | Lifecycle + subprocess | Fail mỗi bước setup/read/draw/cleanup; panic unwind | Cleanup đúng tài nguyên, tiếp tục khi một bước lỗi; exit thất bại; không lộ payload | 05, 06, 13 |
| TC-07 | State/render | Tám route, danh sách trống | Tên/route đúng, không có dữ liệu model/task giả | 07 |
| TC-08 | State/action | Chuỗi phím, modal và input harness | Focus/dispatch đúng; Esc gốc không thoát | 08 |
| TC-09 | State/render | F1 tại mỗi route, Esc, resize khi help mở | Help đúng ngữ cảnh, khôi phục focus; footer khớp | 09 |
| TC-10 | Layout/state | 79×24, 80×23, 80×24, 119×24, 120×24, kích thước rất nhỏ | Không panic; state còn đúng; control không bị kích hoạt khi bị che | 10 |
| TC-11 | Unicode/render | Chuỗi Nhật/Việt tự soạn, dấu kết hợp, dòng dài | Nội dung gốc không đổi; wrap/scroll theo ô hiển thị; PageUp/PageDown giữ vị trí cuộn qua resize và không tác động khi không overflow | 11 |
| TC-12 | Integration/isolation | Chuỗi demo với event/clock giả, CWD tạm và file sentinel | Không side effect nghiệp vụ; không mạng/key thật; dữ liệu/sự kiện quyết định kết quả | 12, 14 |

### 13.2. Lệnh kiểm chứng dự kiến

Chạy từ repo root. Các tên package/test dưới đây là mục tiêu của kế hoạch, được tạo bởi task tương ứng. Đổi tên phải cập nhật lệnh cùng task. Kiểm tra số test và tên ca đã chạy; lệnh exit 0 với filter không chọn ca nào không phải bằng chứng nghiệm thu.

| ID | Lệnh/cách chạy | Thời điểm |
|---|---|---|
| V-00 | Bộ bốn lệnh bắt buộc bên dưới | Trước PR Rust và trước đóng phase trên thay đổi cuối |
| V-01 | `cargo metadata --format-version 1 --no-deps --locked` và `cargo check --workspace --all-targets --all-features --locked` | Sau tạo/thay cấu trúc workspace |
| V-02 | `cargo test -p novel-translate --test architecture_boundaries --locked` | T03 và khi thay dependency/features |
| V-03 | `cargo test -p novel-translate --test cli_contract --locked` | T04/T11 và thay đổi CLI/bootstrap |
| V-04 | `cargo test -p novel-translate --bin novel-translate-cli diagnostics::tests --locked` | T05 và thay đổi diagnostic |
| V-05 | `cargo test -p nt-tui --locked` | Các task terminal và UI-01 |
| V-06 | Demo mục 2 trên terminal thật; ghi OS, terminal, kích thước, kết quả và lỗi | T12/T14 |
| V-07 | Sau khi chuẩn bị dependency, `cargo test --workspace --all-features --locked --offline` trong môi trường thực thi test không có truy cập mạng | T13, xác minh offline |

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

`--offline` chỉ kiểm soát truy cập mạng của Cargo, không tự chặn network do chương trình test mở. V-07 phải đi kèm môi trường chặn egress trong bước test hoặc cách cô lập tương đương được ghi nhận, cùng rà soát fixture/adapter. Tải dependency trước đó không phải test gọi dịch vụ. TUI terminal/OS là điểm cần kiểm tra thật; adapter model, credential và database không được khởi tạo trong phase này.

### 13.3. Quy tắc giữ test có giá trị

- Viết test cho hành vi, dữ liệu còn đúng và lỗi quan trọng; không viết assertion chỉ chứng minh constructor trả lại giá trị đã gán.
- Mỗi task triển khai hành vi có test ngay trong task; T11/T12 bổ sung bằng chứng xuyên thành phần.
- Ưu tiên test state/action và output contract; snapshot giao diện không thay cho kiểm tra focus, side effect và cleanup.
- Panic/global hook được kiểm tra trong tiến trình riêng khi cần, tránh làm test khác phụ thuộc thứ tự chạy.
- Hạn thời gian chỉ dùng phát hiện treo; logic pass/fail không phụ thuộc tốc độ máy hoặc đợi một số mili giây tùy ý.
- Giữ các kiểm tra Go mà repo yêu cầu. Nếu sửa hạ tầng dùng chung, chạy các kiểm tra Go bị ảnh hưởng và ghi rõ kết quả; không quy mọi lỗi baseline thành lỗi của phase.

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md): A thực hiện task, kiểm thử và tự review rồi bàn giao **Chờ review**; B đối chiếu yêu cầu, kiểm bản bàn giao và tự chạy kiểm chứng phù hợp trong phiên riêng. B ghi báo cáo độc lập, không sửa source/test chính thức hoặc trạng thái PLAN. A xử lý findings; B kiểm tra lại trên candidate mới. Quy tắc này áp dụng cả task tài liệu/ADR/thử nghiệm của phase.

| Mặt cần rà soát | Câu hỏi bắt buộc | Bằng chứng nên xem |
|---|---|---|
| Đúng yêu cầu | Hành vi có khớp S02/S03 và AC của task? | Test + demo/contract |
| Trách nhiệm | Module có đúng owner, có chứa logic thuộc phase khác? | Diff và dependency graph |
| API | Public API có consumer, có lộ state nội bộ hoặc kiểu framework vào core? | Export list và call site |
| Lỗi | Có nuốt lỗi, trả thành công sai, mất nguyên nhân hoặc rò secret? | Error path và ca fault injection |
| Terminal | Tài nguyên có một owner, setup một phần có cleanup, panic không bỏ sót? | Lifecycle tests và terminal thật |
| UI | Render có side effect, phím có lọt modal/input, resize có mất state? | State/action tests |
| Dependency | Có thư viện/feature chưa dùng, cạnh cấm hoặc abstraction chưa có nhu cầu? | Manifest và architecture test |
| Kiểm thử | Có ca lỗi, biên, integration đúng chỗ; test thực sự chạy và độc lập? | Tên ca, log và fixture |
| Phạm vi diff | Có đụng Go/dữ liệu hoặc code ngoài task mà không cần thiết? | Diff so với trạng thái đầu task |
| Tài liệu | Help/README có quảng bá chức năng chưa có hoặc phụ thuộc docs riêng tư? | Đối chiếu binary và hướng dẫn |

Không đóng task còn lỗi làm sai contract, lộ dữ liệu, treo tiến trình hoặc mất trạng thái cần giữ. Cải tiến không chặn nghiệm thu được ghi thành việc tiếp theo với ảnh hưởng rõ. Không đặt tỷ lệ coverage tùy ý thay cho bằng chứng các AC.

## 15. Điều kiện đóng phase

- [x] Mọi task/nhóm trong phạm vi cổng có review B **Đạt** còn hiệu lực; đủ AC, kiểm tra bắt buộc và không còn finding chặn. A không tự duyệt phần mình triển khai.
- [x] B đã review bản tích hợp cuối theo [mẫu review phase](../templates/PHASE_REVIEW.md); candidate và căn cứ khớp, bằng chứng các task còn áp dụng hoặc đã được kiểm lại theo ảnh hưởng.

- [x] P01-D01–D06 đã có quyết định đủ cho implementation thực tế; không còn điểm mở chặn M1.
- [x] T01–T14 có trạng thái và báo cáo; task được chia nhỏ vẫn truy vết đủ về task gốc.
- [x] AC-01–AC-16 đều có bằng chứng tương ứng, gồm ca âm quan trọng.
- [x] Demo shell chạy trọn vẹn; kiểm tra terminal thật được ghi rõ môi trường.
- [x] Bốn lệnh V-00, architecture check và CI matrix đã chọn đạt trên thay đổi cuối.
- [x] Test offline/deterministic đã được kiểm chứng; không có key thật hoặc lời gọi provider trong CI.
- [x] Không còn lỗi chặn về contract, bảo mật diagnostic, terminal lifecycle hoặc dependency boundaries.
- [x] README phản ánh cách build/chạy, layout và khả năng thực tế; tài liệu nội bộ tiếp tục ngoài Git.
- [x] Đường chạy Go và dữ liệu người dùng được bảo toàn; chưa thay thế Go hoặc tuyên bố feature parity.
- [x] Phase 02 nhận được các điểm nối và việc còn thiếu ở mục 16.

Nếu chưa chạy được một kiểm tra bắt buộc, ghi **chưa xác minh** và lý do. Phase không được đóng chỉ dựa trên kết quả compile hoặc bằng chứng headless thay cho phần terminal thực.

## 16. Bàn giao và bằng chứng

A dùng [mẫu bàn giao task](../templates/TASK_HANDOFF.md); B dùng [mẫu review task](../templates/TASK_REVIEW.md), và [mẫu review phase](../templates/PHASE_REVIEW.md) khi nghiệm thu cổng. Báo cáo nội bộ đặt theo lượt trong `docs/roadmap/evidence/phase-01/`: `P01-TYY.handoff-r01.md` và `P01-TYY.review-r01.md`; giữ lượt trước khi sửa. Hồ sơ cổng dùng ID M0-01/M0-02/M0 hoặc M1…M9 theo phạm vi. Các mẫu chưa phải bằng chứng thực thi.

Sổ task/cổng khi thực hiện ghi trạng thái, candidate, link báo cáo A, review B, finding còn mở và bước tiếp theo. A cập nhật sổ này cùng PLAN sau bàn giao/kết luận; B chỉ ghi báo cáo review. Thiếu review B hoặc kiểm tra bắt buộc thì chưa đánh dấu hoàn thành.

### 16.1. Trạng thái thực thi hiện tại

| Gói | Task | Trạng thái | Bằng chứng thực thi |
|---|---|---|---|
| P01-A | T01 | Hoàn thành — B Đạt r02 | [Handoff r02](../evidence/phase-01/P01-T01.handoff-r02.md), [review r02 — Đạt](../evidence/phase-01/P01-T01.review-r02.md), [artifact index](../evidence/phase-01/P01-T01.artifact-index-r02.md); r01/r02 giữ riêng |
| P01-A | T02 | Hoàn thành — B Đạt r02 | [Handoff r02](../evidence/phase-01/P01-T02.handoff-r02.md), [manifest r02](../evidence/phase-01/P01-T02.candidate-r02.sha256), [review r02 — Đạt](../evidence/phase-01/P01-T02.review-r02.md), [r01 review — Cần sửa](../evidence/phase-01/P01-T02.review-r01.md), [artifact notes](../evidence/phase-01/P01-T02.handoff-notes-r02.md); F01 đóng |
| P01-B | T03 | Hoàn thành — B Đạt r03/r04 | [Handoff r03](../evidence/phase-01/P01-T03.handoff-r03.md), [manifest r03](../evidence/phase-01/P01-T03.candidate-r03.sha256), [review r03 — Đạt](../evidence/phase-01/P01-T03.review-r03.md), [docs-only handoff r04](../evidence/phase-01/P01-T03.handoff-r04.md), [review r04 — Đạt](../evidence/phase-01/P01-T03.review-r04.md); F01–F04 đóng |
| P01-B | T04 | Hoàn thành — B Đạt r02, F01 đóng | [Handoff r02](../evidence/phase-01/P01-T04.handoff-r02.md), [manifest r02](../evidence/phase-01/P01-T04.candidate-r02.sha256), [review r02 — Đạt](../evidence/phase-01/P01-T04.review-r02.md), [review r01 — Cần sửa](../evidence/phase-01/P01-T04.review-r01.md) |
| P01-C | T05 | Hoàn thành — B Đạt trên Windows, `P01-T05-r01` | [Handoff r01](../evidence/phase-01/P01-T05.handoff-r01.md), [manifest r01](../evidence/phase-01/P01-T05.candidate-r01.sha256), [review r01 — Đạt](../evidence/phase-01/P01-T05.review-r01.md); không có finding mở |
| P01-C | T06 | Hoàn thành — `P01-T06-r03`, B Đạt trên Windows, F01–F03 đóng | [Handoff r03](../evidence/phase-01/P01-T06.handoff-r03.md), [manifest r03](../evidence/phase-01/P01-T06.candidate-r03.sha256), [review r03 — Đạt](../evidence/phase-01/P01-T06.review-r03.md), [review r02](../evidence/phase-01/P01-T06.review-r02.md), [review r01](../evidence/phase-01/P01-T06.review-r01.md). V-06 terminal thật còn thuộc T12 |
| P01-D | T07 | Hoàn thành — `P01-T07-r03`, B Đạt trên Windows, F01/F02 đóng | [Handoff r03](../evidence/phase-01/P01-T07.handoff-r03.md), [manifest r03](../evidence/phase-01/P01-T07.candidate-r03.sha256), [review r03 — Đạt](../evidence/phase-01/P01-T07.review-r03.md), [review r02 — Cần sửa](../evidence/phase-01/P01-T07.review-r02.md), [review r01 — Cần sửa](../evidence/phase-01/P01-T07.review-r01.md). |
| P01-D | T08 | Hoàn thành — `P01-T08-r04`, B Đạt trên Windows, F01/F02 đóng | [Handoff r04](../evidence/phase-01/P01-T08.handoff-r04.md), [manifest r04](../evidence/phase-01/P01-T08.candidate-r04.sha256), [review r04 — Đạt](../evidence/phase-01/P01-T08.review-r04.md), [review r03 — Cần sửa](../evidence/phase-01/P01-T08.review-r03.md), [review r02 — Cần sửa](../evidence/phase-01/P01-T08.review-r02.md), [review r01 — Cần sửa](../evidence/phase-01/P01-T08.review-r01.md). B xác minh manifest 67/67, V-02 10/10, V-05 42/42, workspace/MSRV 75/75 và probe Win32 độc lập. V-06 terminal tương tác thuộc T12. |
| P01-D | T09 | Hoàn thành — `P01-T09-r02`, B Đạt trên Windows, F01 đóng | [Handoff A r02](../evidence/phase-01/P01-T09.handoff-r02.md), [manifest r02](../evidence/phase-01/P01-T09.candidate-r02.sha256), [review B r02 — Đạt](../evidence/phase-01/P01-T09.review-r02.md), SHA-256 `26D7A1E2D37FA56DFD5A678906DD0631F4054FA2E82C189B19F3BACC0C14B740`; [review r01 — Cần sửa](../evidence/phase-01/P01-T09.review-r01.md); F01 đóng sau probe 16/16 route/focus. |
| P01-D | T10 | **Hoàn thành — P01-T10-r05, B Đạt trên Windows; F01–F03 đóng.** B tái hiện độc lập debug panic r04 và mất neo release, sau đó xác nhận bản sửa r05; V-02 10/10, V-05 66/66, workspace/MSRV 99/99, fmt/clippy/release đạt. | [Handoff A r05](../evidence/phase-01/P01-T10.handoff-r05.md), [manifest r05](../evidence/phase-01/P01-T10.candidate-r05.sha256); [review B r05 — Đạt](../evidence/phase-01/P01-T10.review-r05.md), SHA-256 705CF794A13D01304EA90E33940951FAD521E9F04DF9C8B15AD155B2B3439D61; lịch sử r01–r04 và probe được dẫn trong handoff. |
| P01-E | T11 | **Hoàn thành — P01-T11-r01, B Đạt trên Windows.** AC-03–06/12/14, TC-03–05/12, V-03; V-03 11/11, workspace/MSRV 102/102, V-02 10/10, fmt/clippy/release đạt; không có finding chặn. | [Handoff A r01](../evidence/phase-01/P01-T11.handoff-r01.md), [manifest r01](../evidence/phase-01/P01-T11.candidate-r01.sha256), [review B r01 — Đạt](../evidence/phase-01/P01-T11.review-r01.md), SHA-256 `2ED665147F7C1661DAD96960EEC683677F8041FDFD71897CC50901D885276A7B`; B xác minh manifest cuối 165/165. |
| P01-E | T12 | **Hoàn thành — P01-T12-r05, B Đạt trên Windows; F01 đóng.** B kiểm AC-07–14, TC-06–12, V-05/V-06; V-05 70/70, workspace 108/108, subprocess 1/1, fmt/clippy/debug build đạt. B chấp nhận V-06 stepwise trên WT thật và TC-06 kết hợp process bootstrap test với fake-driver lifecycle tests; giới hạn được ghi trong review. | [Handoff A r05](../evidence/phase-01/P01-T12.handoff-r05.md), [manifest r05](../evidence/phase-01/P01-T12.candidate-r05.sha256), review-time 354/354, SHA-256 `946B828BBACD336217C599F457A127B1F319EE65F916518FE439E7590D857E40`; [V-06 A r05](../evidence/phase-01/P01-T12-v06-r05.md), [review B r05 — Đạt](../evidence/phase-01/P01-T12.review-r05.md), SHA-256 `DAC0F2FCAE2BF131FCD0120930C27A13B59B329C5666D72533C3A82A0148BB88`. Status docs synced after review; source/test candidate unchanged. |
| P01-F | T13–T14 | **P01-T13-r05 Hoàn thành — B gpt-6.1-sol/max Đạt trên Windows; F01/F02 giữ đóng.** AC-14/15, V-00/V-02/V-07; hosted Rust/Go CI pass, V-07 offline 108/108 và cleanup 31 rule; Go Windows regression đã sửa. Candidate `de738e9098677be6d1418e820789d68fcb5e7e12`, manifest 395/395 SHA-256 `CADCE510E5EAF4F79C7F6F85BF287671F70ECE0EE214ABE4EFE2ECEE57B36C86`. [Handoff r05](../evidence/phase-01/P01-T13.handoff-r05.md), [review B r05 — Đạt](../evidence/phase-01/P01-T13.review-r05.md), SHA-256 `1EE8F6B3F2E702C9E7ACD0C8B75B6596500E2C6BE0C0F4C3FF1936B574030F80`; [Draft PR #1](https://github.com/nuocmatParis/novel-translate-cli/pull/1), [Rust CI](https://github.com/nuocmatParis/novel-translate-cli/actions/runs/42983119546), [Go CI](https://github.com/nuocmatParis/novel-translate-cli/actions/runs/42983119508). P01-T14-r01 B **Cần sửa**, F01/F02 mở; [handoff r01](../evidence/phase-01/P01-T14.handoff-r01.md), [manifest r01](../evidence/phase-01/P01-T14.candidate-r01.sha256), [review r01](../evidence/phase-01/P01-T14.review-r01.md), SHA-256 `88FF43864C444A98194040E8ACD4C0EBC9D15DF49EDED8130ADB7E740E4DEEAD`. P01-T14-r02 **Hoàn thành, B Đạt**, [handoff r02](../evidence/phase-01/P01-T14.handoff-r02.md), [manifest r02](../evidence/phase-01/P01-T14.candidate-r02.sha256), [review r02](../evidence/phase-01/P01-T14.review-r02.md), SHA-256 `9A31B3169243703BB7632CD08A3A905C81804078B7EA9397EB270F904D1BB3F1`. README đã commit/push trong 14edc793988ecffed7961bc48ac2b65c12ba9eed; Rust/Go hosted CI success trên synthetic merge `2748b6ead1eb27ee1d9a95f839ff1b638ba18398`; [checks report](../evidence/phase-01/P01-T14-r02-hosted-checks.md). Raw log API trả 403 do cần repo-admin; giới hạn log và cách đối chiếu được ghi trong review M1. |
| M1 | T01–T14 | **Phase 01 hoàn thành trên Windows — M1-r01, B Đạt; đủ 12 điều kiện §15, không còn finding chặn.** Candidate PR #1 head `14edc793988ecffed7961bc48ac2b65c12ba9eed`, synthetic merge `2748b6ead1eb27ee1d9a95f839ff1b638ba18398`; manifest review-time 420/420, SHA-256 `5C563382F0E4D222185BB6395FB1AF3ADB7FA2C9A531C55A294F1FA657D7E32B`. B tự chạy V-00 (108/108), V-02 (10/10), release build; Rust/Go hosted CI success; B lặp terminal thật theo §16.4 và xác nhận shell, resize, Unicode, CLI, prompt. Raw latest per-test CI counts không được khẳng định vì GitHub API trả 403, giới hạn cùng bằng chứng thay thế có trong review. [Handoff](../evidence/phase-01/M1.handoff-r01.md), [manifest](../evidence/phase-01/M1.candidate-r01.sha256), [review B — Đạt](../evidence/phase-01/M1.review-r01.md), SHA-256 `2DA5D8F1CCB96C5BAB3D297D791827A8F8C0A8C0DAA81FECDA03EFDC32780A27`, [hosted checks](../evidence/phase-01/P01-T14-r02-hosted-checks.md). PLAN/phase được đồng bộ sau verdict, source/test candidate không đổi. PR #1 vẫn Draft chờ quyết định merge; Linux native hoãn theo người dùng. |

Trạng thái phản ánh bằng chứng thực thi và review hiện có; việc code Rust đã xuất hiện trong workspace trước task không được dùng thay quyết định hay tiêu chí nghiệm thu.

### 16.2. Nội dung báo cáo sau mỗi task

1. Task ID, thay đổi hành vi và AC đã đáp ứng.
2. File/module đã sửa; lý do nếu phạm vi thực tế cần điều chỉnh.
3. Quyết định/contract được áp dụng hoặc cập nhật, cùng ảnh hưởng tới consumer.
4. Lệnh đã chạy, môi trường/toolchain, số ca liên quan, kết quả và đường dẫn log nội bộ nếu có.
5. Ca thủ công đã thực hiện; test chưa chạy hoặc không áp dụng kèm lý do cụ thể.
6. Lỗi/rủi ro còn lại, điều kiện chặn và task tiếp theo có thể bắt đầu.

Log/báo cáo nếu lưu vào file đặt dưới `docs/roadmap/evidence/phase-01/` và loại dữ liệu bí mật trước khi ghi. Không đưa nội dung sách, key hoặc workspace thật vào bằng chứng.

### 16.3. Bàn giao cho Phase 02

| Thành phần nhận | Điểm nối đã chuẩn bị | Phase 02 cần bổ sung |
|---|---|---|
| Application | Chiều phụ thuộc và quy tắc contract/error | Use case tạo/mở workspace, import và truy vấn nguồn |
| Infrastructure | Quy tắc adapter/port và kiểm tra graph | `nt-storage`, `nt-epub` cùng các quyết định data/transaction liên quan |
| CLI | Parser, bootstrap, error/output mapping | `workspace create/open`, `import` và cờ theo S02 khi use case sẵn sàng |
| TUI | Tám route, focus/help/layout và state trình bày | UI-02/UI-03, danh sách workspace/cây sách và chẩn đoán từ core |
| Kiểm thử | Harness process/terminal, test offline, architecture gate | Fixture EPUB tự soạn, persistence round-trip và recovery ở ranh giới commit |

Hồ sơ M1 phải nêu rõ nền tảng nào đã kiểm tra, contract nào đã ổn định và chức năng nào còn chờ phase sau. Không xem shell tám màn là tám màn nghiệp vụ hoàn chỉnh.

<a id="p01-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử bản Rust được bàn giao cho M1 trên Windows x64. Phạm vi là khởi động CLI, điều hướng tám màn, trợ giúp, đổi kích thước và thoát. Khi bàn giao, T14 ghi candidate, tên terminal và kết quả thử theo V-06. A chuẩn bị hướng dẫn theo bản đó. B dùng đúng bản và thực hiện lại các ca chính.

#### 16.4.2. Chuẩn bị

Mở PowerShell trong một cửa sổ terminal tương tác và vào thư mục gốc repo của candidate M1. Dùng toolchain được ghim trong repo. Dành khoảng 10 phút sau khi build. Bài thử dùng trạng thái shell trống và không cần EPUB, workspace hay API key.

#### 16.4.3. Mở đúng bản

Từ thư mục gốc repo, build và giữ đường dẫn đến chính executable Rust vừa tạo.

```powershell
cargo build --locked -p novel-translate --bin novel-translate-cli --target x86_64-pc-windows-msvc --target-dir target
if ($LASTEXITCODE -ne 0) { throw 'Build Rust không thành công.' }
$phase01App = (Resolve-Path -LiteralPath '.\target\x86_64-pc-windows-msvc\debug\novel-translate-cli.exe').Path
& $phase01App --help
& $phase01App --version
& $phase01App
```

Hai lệnh đầu in văn bản rồi trả về prompt. Lệnh cuối mở **Tổng quan** với trạng thái **Chưa mở workspace** và **Chưa có tác vụ**.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P01-UT01 — Đổi màn | Đưa focus về vùng điều hướng bằng `Tab` nếu cần. Dùng mũi tên rồi `Enter` để mở **Tập và chương**, sau đó đi lần lượt qua các mục điều hướng. | Focus nhìn thấy, có đúng tám màn, tiêu đề khớp mục đã chọn và trạng thái trống phù hợp. |
| P01-UT02 — Trợ giúp | Ở **Tập và chương**, ghi nhớ vị trí focus. Nhấn `F1` rồi `Esc`. | Help dành cho màn đang xem. Khi đóng help, vẫn ở **Tập và chương** và focus về đúng vị trí trước đó. |
| P01-UT03 — Đổi kích thước | Giữ một màn và mục đang chọn. Đổi cửa sổ từ 80×24 sang 120×24, thu xuống dưới 80×24 rồi trở lại 80×24. | Bố cục thay đổi theo kích thước. Khi trở lại 80×24, màn, mục chọn và focus được giữ. |
| P01-UT04 — Thoát | Ở màn gốc, nhấn `Esc` một lần. Dùng `Tab` tới **Thoát** rồi nhấn `Enter`. Gõ `Get-Location` tại prompt. | `Esc` ở màn gốc giữ app mở. **Thoát** trả về prompt, con trỏ hiển thị và lệnh PowerShell chạy được. |

#### 16.4.5. Thử ca lỗi và mở lại

**P01-UT05 — Thiếu lệnh khi chạy cho script.** Sau khi thoát TUI, chạy trong cùng cửa sổ PowerShell.

```powershell
& $phase01App --json --non-interactive
$LASTEXITCODE
```

Lệnh kết thúc ngay, stdout có một JSON báo thiếu lệnh với `state` là `failed` và mã thoát là `2`. Chạy `& $phase01App` lần nữa để kiểm app vẫn vào được **Tổng quan**.

#### 16.4.6. Ghi kết quả

Với mỗi mã ca, ghi **Đạt**, **Không đạt** hoặc **Chưa thử**. Khi lỗi, gửi candidate M1, phiên bản Windows, tên terminal, kích thước cửa sổ, bước lỗi và kết quả mong đợi/thực tế. Với ca CLI, gửi output và mã thoát. Kết quả thủ công của T12/T14 được ghi riêng theo V-06.

#### 16.4.7. Thử lại từ đầu

Thoát shell, đưa terminal về ít nhất 80×24 rồi chạy lại `& $phase01App` từ cùng thư mục. Nếu đổi candidate, build lại và lấy đường dẫn executable theo mục 16.4.3.


