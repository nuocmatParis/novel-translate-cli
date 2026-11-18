# Chiến lược kiểm thử và fixture Phase 00–02

**Ngày:** 09/11/2026 · **Phiên bản:** 0.9 · **Trạng thái:** P01-T03-r03 B review Đạt, F01–F04 đóng; v0.9 giữ hướng dẫn adapter và khôi phục hàng Generation/CAS vào đúng bảng sau một sửa bố cục docs-only. T03–T05 và nhóm T06–T08 Phase 00 đã được B review Đạt trong phạm vi hai spike Windows.

## 1. Mục tiêu và giới hạn

Chiến lược này chuyển các invariant nền tảng thành phép kiểm có oracle độc lập, chạy offline và xác định. Test không lấy implementation làm chuẩn vàng, không gọi provider/model, không đọc key thật, không dùng EPUB/workspace người dùng và không phụ thuộc tốc độ máy.

Phạm vi sớm: P00-AC03–AC08; contract Phase 01 cho graph, CLI/JSON, diagnostics, terminal và TUI shell; đầu vào thử nghiệm của Phase 02 cho EPUB/source identity, SQLite worker và công bố generation. P00-T03–T05 đã qua review B; T06–T08 có test spike thực chạy, còn implementation/parser Phase 01–02 chưa được kiểm. Bảng phân biệt bằng chứng spike với testcase dự kiến của sản phẩm. Fixture giữ yêu cầu [Q36](../spec/DESIGN_DECISIONS_36.md#decision-36) và quy tắc [AGENTS.md](../../AGENTS.md) về offline, deterministic, không dùng key/sách/workspace thật.

## 2. Quy tắc fixture và oracle

- Tạo mọi fixture trong thư mục tạm duy nhất cho test; tên/path tổng hợp, cleanup ở teardown. Có thể giữ fixture tối thiểu trong test source nếu nó nhỏ hơn việc tạo lúc chạy; không commit binary EPUB golden hoặc nội dung sách có bản quyền.
- EPUB được tạo từ ZIP tổng hợp, với `mimetype` là entry đầu và không nén, nội dung chính xác `application/epub+zip`. EPUB 2 dùng `META-INF/container.xml`, package OPF, NCX, spine và XHTML; EPUB 3 dùng container/package, navigation XHTML, spine và content XHTML. Kiểm fixture với contract phiên bản của [EPUB_INPUT_SPEC](../spec/EPUB_INPUT_SPEC.md), không chỉ coi ZIP đọc được là EPUB hợp lệ. Bộ fixture bám [EPUB 2.0.1 OPF](https://idpf.org/epub/20/spec/OPF_2.0_latest.htm) và [W3C EPUB 3.3](https://www.w3.org/TR/epub-33/); EPUB 3 yêu cầu navigation document, còn EPUB 2 dùng NCX cho navigation.
- Nội dung chuỗi tự soạn, có chữ Nhật, Việt có dấu, ký tự combining (ví dụ `a` + U+0301), ruby, entity/XML whitespace, đoạn lặp, ID trùng có chủ ý và tài nguyên ảnh giả. Bao gồm một ca Unicode NFC và NFD để tránh âm thầm gộp khác biệt nguồn.
- Dùng ID generator, clock, event source, filesystem, DB location, lock/barrier và fault injector có điều khiển. Tạo lỗi qua interface/harness; không dựa vào chmod/ACL, disk full, sleep hoặc race xác suất để kiểm lỗi.
- Secret sentinel chỉ là chuỗi giả do test tạo (gồm token-like value, URL userinfo và `Authorization` header). Bắt toàn bộ stdout/stderr, JSON, log, buffer TUI và diagnostic panic; assertion phải chứng minh sentinel vắng mặt, đồng thời mã lỗi/next action vẫn hữu ích.
- Oracle độc lập là expected table từ spec/ADR hoặc fixture expected viết trước khi chạy implementation. Golden không được sinh bằng cách chụp output implementation rồi dùng lại làm kỳ vọng. Hash/range/source mapping phải có giá trị tính tay hoặc script tham chiếu độc lập, kèm phiên bản/thuật toán.
- Harness chạy một thread mặc định; concurrent writer dùng barrier và timeout guard rộng chỉ để phát hiện treo. Không dùng `sleep` để sắp thứ tự. Timeout không phải tiêu chí hiệu năng.

## 3. Ma trận invariant / ca success-failure

| Invariant / AC | Ca thành công | Ca lỗi/biên | Lớp kiểm thử và fixture | Oracle / assertion độc lập |
|---|---|---|---|---|
| Dependency graph / P00-AC03, P01-AC-02 | Graph `nt-application → nt-domain`, `novel-translate → nt-tui` được chấp nhận; external package đi qua đúng owner và nguồn crates.io đã chốt | `nt-domain → nt-application`, `nt-application → nt-tui`, app/UI vòng nhau; package bị rename; cạnh normal/build optional theo target/feature; external path trùng tên allowlist; `[patch.crates-io]` thay package đã duyệt; external crate kéo ngược product crate; sai owner | Architecture contract trên Cargo metadata đầy đủ với `--all-features`, không `--filter-platform`; fixture tổng hợp cho path/patch/resolve graph, không sửa manifest repo | So khai báo của từng workspace member để giữ cạnh optional/target chưa active; so source/package identity đã resolve và duyệt đường normal/build bắc cầu. Allowlist theo owner ở ADR-008 §2.1; lỗi nêu package nguồn/đích và đường vi phạm. `dev-dependencies` được phân loại riêng. |
| Bootstrap/mode / AC04 | `--help`/`--version` exit 0, không gọi terminal; no args + stdin/stdout TTY gọi đúng một lần entrypoint TUI | no args non-TTY exit 3 hữu hạn; thiếu command dưới `--json`/`--non-interactive`/`--workspace`, path thiếu/lặp hoặc flag sai exit 2; unsupported command không acquire terminal | Process/CLI contract với fake launcher và TTY capability điều khiển | Case table ADR-008 §5.1; exact exit/stdout/stderr, filesystem/terminal counters và số lần gọi fake launcher. |
| JSON schema / AC04 | Serialize fixture hợp lệ đủ 8 field schema v1; field nullable có giá trị đúng; status success sau này giữ state trong envelope | thiếu field, sai type, `null` ở field bắt buộc, lỗi parse trước command và sau command; stdout có thêm ANSI/log | Unit/JSON contract; fixture JSON tự soạn và expected object cố định | Bảng type/nullability/schema/exit trong ADR-008 §5.1; parse toàn bộ stdout một lần, so cấu trúc JSON và exit. P01 chưa giả lập business command. |
| Safe diagnostics / AC04–05 | Error code đã lọc và next action hữu ích; buffer nằm trong giới hạn | Sentinel lồng trong error source, Debug/Display, URL/header hoặc panic; tràn buffer; cleanup cùng lỗi | Unit/contract + process capture; sentinel giả, bounded ring buffer | Security spec §4 và ADR-008 §5.3; kiểm không rò ở mọi sink; issue primary còn nguyên, lỗi phụ được giữ mã. |
| Terminal lifecycle / AC04–05 | Acquire đủ raw/alternate/cursor rồi release ngược; shutdown qua Exit hoặc Ctrl+C event | Fault từng lệnh setup, lệnh có side effect một phần, read/draw lỗi, một cleanup lỗi, cleanup lặp, panic unwind | `nt-tui` unit test với fake terminal có log thao tác và lỗi tiêm tại từng bước; terminal thật ở P01-T12 | ADR-008 §5.2; expected sequence viết trước; kiểm chỉ release resource có thể đã cấp, mọi cleanup còn lại vẫn chạy, exit thất bại. Abort/force-kill ghi ngoài phạm vi. |
| Event/state/render / AC05 | Queue event cố định tạo transition và buffer render kỳ vọng; resize/help giữ state | Input/modal không chuyển action ra màn sau; render/help/resize không tạo side effect hoặc đổi dữ liệu nghiệp vụ | Unit/state + render buffer, event queue tổng hợp; không clock thật | State transition expected từ UI contract; assertion trên state trước/sau và fake side-effect counters. |
| EPUB package / AC05, AC07 | EPUB 2 và 3 tối thiểu mở được, spine order và resource path theo OPF | ZIP hỏng, thiếu container/package/navigation hoặc manifest resource, XML sai, entry path nguy hiểm/trùng | Unit/contract trên ZIP tạo lúc chạy và parser XML; không file sách thật | EPUB_INPUT_SPEC + fixture descriptor; bảng expected resource/order độc lập với parser. Ghi rõ phiên bản EPUB được thử. |
| Filesystem I/O / AC05, AC08 | Open/read/write/flush trên thư mục tạm hoàn tất; hard-link publish tạo object bất biến; nội dung/hash sau reopen đúng | Tiêm lỗi open/read/write/flush/link và cleanup; caller nhận lỗi, không publish reference nửa chừng hoặc báo thành công | Unit với filesystem port giả và integration temp directory; mỗi failpoint được gọi xác định | Oracle là trạng thái file/DB đọc lại và error kind kỳ vọng; không phụ thuộc ACL/permission/disk đầy của máy. |
| Unicode/source range / AC05, AC07 | Đoạn lặp có identity khác; range half-open khớp grapheme/protected inline và ánh xạ ngược node nguồn | NFD/NFC, ruby, entity, ảnh/marker, duplicate XHTML ID, path tương đương, snapshot đổi | Unit/property examples + fixture XHTML; deterministic ID/hash/clock | Bảng expected ID, byte/hash và range tự tính/kiểm độc lập; replay cho cùng snapshot giữ ID/range, hash snapshot mới không reuse mù. |
| Storage worker / AC06 | Một writer commit, readers thấy snapshot nhất quán; worker stop/join sau drain có kiểm soát | Hai writer gặp busy theo policy; rollback giữa transaction; lỗi worker được caller thấy; shutdown khi đang chờ | V-02 spike với DB file tạm, connection độc lập, barrier và fault hook | Trạng thái DB trước/sau truy vấn độc lập; kết quả future/channel và worker termination; không lấy benchmark tốc độ làm oracle. |
| Generation/CAS / AC08 | File bất biến được hard-link và kiểm hash trước DB publish; bootstrap DDL/g0/ref/schema version cùng commit; restart đọc nguyên generation cũ hoặc mới | stale CAS, cạnh tranh hai generation, ngắt trước/sau hard link hoặc commit, bootstrap abort trước commit, file write lỗi, migration lỗi, replay cùng request | Integration fault matrix trên DB/temp dir, failpoint đặt tại từng bước và reopen sau mỗi lần | ADR-004 state table; DB không trỏ file thiếu/incomplete; bootstrap abort không để schema nửa chừng và retry tạo g0/version 1; history không mất và replay idempotent. |

### 3.1. Mở boundary checker khi phase sau thêm adapter

Chỉ thêm adapter/package khi phase có consumer thực tế và task/ADR đã chốt owner cùng chiều gọi. Khi đó, cập nhật checker và fixture trong cùng task theo thứ tự sau:

1. Thêm đúng package vào inventory sau khi Cargo workspace được cập nhật; chỉ thêm cạnh product đã được ADR/phase cho phép vào `allowed_internal_edge`. Giữ fixture âm cho chiều ngược và vòng phụ thuộc.
2. Khai báo external dependency tại owner có consumer. Dùng version, feature và source theo ADR-001; chỉ nới allowlist đúng owner, không cấp trước dependency cho phase tương lai.
3. Thêm fixture dương cho consumer hợp lệ và fixture âm cho owner sai, path dependency hoặc `[patch]` thay package đã duyệt, và đường normal/build bắc cầu từ external vào crate sản phẩm.
4. Với từng cạnh mới, giữ ca rename (Cargo có thể chuẩn hóa `-` thành `_`), alias dùng chung nhưng khác target hoặc kind, optional/build, và dev-only. Cạnh dev được báo riêng, không nhập vào graph sản phẩm.
5. Rà metadata khai báo lẫn graph đã resolve với `--all-features`, không lọc target; chạy V-02, các lệnh V-00 và MSRV trên candidate. Cập nhật số cạnh/test, candidate manifest và handoff cùng task.

## 4. Bằng chứng thực chạy P00-T06–T08

Các test này nằm trong tài liệu nội bộ bị ignore, không thuộc workspace sản phẩm và không thêm dependency production. Thư mục `P00-T06-T08-spike` dùng rusqlite cho worker/generation/identity; `sqlx-compare` là project Cargo độc lập vì SQLx 0.8.6 và rusqlite 0.40.2 phụ thuộc hai `libsqlite3-sys` có cùng native `links`. SQLite chỉ mở trên đường dẫn tạm; harness không có network adapter, sleep để sắp thứ tự hay EPUB thật.

| AC | Test thực chạy | Oracle và kết quả kỳ vọng |
|---|---|---|
| P00-AC06 | `tests/storage.rs`; `sqlx-compare/tests/driver.rs` | 6 rusqlite + 6 SQLx đạt. Writer 2 nhận SQLite busy code 5 với timeout 0; reader transaction giữ snapshot cũ tới khi kết thúc; worker mất trước ACK không trả success. |
| P00-AC07 | `tests/identity.rs` | 10 test identity đạt. .NET `SHA256.HashData` tính độc lập vector `12979669…e3520`; Rust assertion khớp. `[2,6)` map node 10 `[2,5)`, node 10 `[5,6)`, ruby 11, image 12; `[6,7)` giữ marker pair; combining mark qua nodes 20–21 là một unit; text node thường `ab`/`cd` map bốn grapheme về đúng node và byte span; NFC/NFD byte spans giữ nguyên. |
| P00-AC08 | `src/generation.rs` unit tests + `tests/generation.rs` | 10 test đạt (2 no-clobber interleave + 8 generation/migration/fault). Hai CAS có một commit/một stale; child ghi marker đúng failpoint, kiểm exit không phải fallback 90, rồi kiểm file/DB sau reopen. Abort bootstrap trước commit để lại 0 bảng ứng dụng và version 0; retry reuse object đã kiểm hash và commit đủ schema 1, g0, ref và current. Bốn điểm ngắt generation vẫn mở g0 hoặc g1 đúng trạng thái. Interleave target sai hash giữ nguyên byte/timestamp và không được tham chiếu; target đúng payload được dùng lại không thay timestamp. Đối chứng `rename` làm cả hai regression thất bại. |

Lệnh từ repository root, toolchain MSRV Windows `1.88.0-x86_64-pc-windows-msvc`; Cargo.lock đã tạo trước rồi test offline/locked:

```text
cargo +1.88.0 test --manifest-path docs/roadmap/evidence/phase-00/P00-T06-T08-spike/Cargo.toml --tests --locked --offline
cargo +1.88.0 test --manifest-path docs/roadmap/evidence/phase-00/P00-T06-T08-spike/sqlx-compare/Cargo.toml --tests --locked --offline
```

Lệnh format/lint và version tool ghi trong handoff A. Đây chỉ là project spike; không chạy test/build workspace sản phẩm. P00-AC07 chưa có parser/EPUB 2/3 integration và P00-AC08 chưa có kiểm mất điện/hardware cache.

## 5. Cấp độ kiểm thử và bằng chứng

| Cấp độ | Dùng khi | Bằng chứng cần ghi |
|---|---|---|
| Unit | State machine, codec/hash/range, error mapping, cleanup coordinator | Tên test, số case; expected input/output; điểm fault và state trước/sau |
| Contract | Crate API, graph allowlist, JSON schema, worker protocol | Consumer thật hoặc fixture graph; kiểm cả graph/JSON hợp lệ và âm |
| Process | CLI mode, exit code, stream, không side effect | Args tổng hợp, TTY flags, timeout guard, exit/stdout/stderr và file sentinel |
| Integration | EPUB → source record; DB transaction; immutable file + DB reference | Fixture descriptor, temp paths đã che, failpoint, trạng thái reopen và hash |
| Terminal thủ công | Con trỏ, terminal thật, resize, gõ sau thoát, Unicode rendering | OS/target, tên terminal, kích thước, thao tác và kết quả; không thay bằng headless render |

## 6. Cô lập môi trường và khi nào được kết luận

Sau bước chuẩn bị compiler/dependency, test tự động chạy `--offline` và trong harness không có adapter network. Không dùng API key, credential store thật, provider/model trả phí, Internet, sleep ngẫu nhiên, workspace thật hoặc file EPUB thật. Nếu cần kiểm chứng thủ công với terminal, tách kết quả khỏi automated test.

Theo quyết định hiện hành tại ADR-001, target chạy native trước là Windows `x86_64-pc-windows-msvc`; kiểm build/test native Linux đang hoãn và phải có lượt bằng chứng riêng trước khi tuyên bố hỗ trợ Linux.

Một task chỉ được báo đạt testcase khi test thực sự được phát hiện/chạy, assertion bao phủ oracle và candidate khớp báo cáo. Kiểm tài liệu/link cho T05 không được trình bày thành test ứng dụng; thiết kế fixture cũng không chứng minh parser/DB/terminal đã hoạt động. Ca chưa có implementation giữ trạng thái “kế hoạch cho task tương ứng/chưa chạy”.
