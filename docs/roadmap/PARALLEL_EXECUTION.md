# Thực hiện Phase 02–09 bằng các chat song song

**Phiên bản:** 2.0 · **Ngày:** 12/11/2026. M1-r01 đã Đạt trên Windows; PR #1 đã merge. Kế hoạch v2 thay lịch v0.1 ngày 09/11 chưa triển khai. Trạng thái review kế hoạch nằm trong [PLAN](../../PLAN.md); tài liệu này không tự ghi task code đã hoàn thành.

## 1. Mỗi chat phụ trách một phase

Người dùng giao 2–3 chat triển khai, mỗi chat chịu trách nhiệm một phase và tự gọi reviewer B. Một trong các chat vai A kiêm điều phối; không bắt người dùng mở riêng một chat chỉ để chuyển báo cáo review. Mỗi chat chỉ thực hiện một task hoặc nhóm 2–3 task gắn chặt tại một thời điểm. Checkpoint công bố cũng giới hạn 1–3 task; một phase có nhiều checkpoint.

Đơn vị mở việc là task; đơn vị chuyển code giữa các chat là checkpoint đã Đạt, merge và hậu kiểm; đơn vị hoàn thành sản phẩm là milestone M2–M9. Phạm vi phase và AC không bị giảm khi cho phép một task nhận đầu ra sớm. Task cuối của từng phase vẫn cần milestone phase trước và kiểm tích hợp toàn phase. Xem [graph đầy đủ](PARALLEL_TASKS.json), §4/§10/§11 của mỗi phase, [ROADMAP](ROADMAP.md) và [review](REVIEW_WORKFLOW.md).

<a id="activation"></a>

## 2. Điều kiện mở và lịch bố trí chat

1. Kế hoạch có review B Đạt; người điều phối kiểm M1 và PR merge đã nhận đúng trên source nền. Checkout hiện tại có thể vẫn ở feature/tui; không reset/checkout đè thay đổi người dùng. Xác minh commit 9fb133078a126171186a0a12a1359443746363d8 trên rewrite/rust, so tree với head M1 đã review 14edc793988ecffed7961bc48ac2b65c12ba9eed. Nếu merge có sửa code thì kiểm phần khác trước khi coi nền đủ điều kiện.
2. Task ngoài nhóm có review B Đạt còn hiệu lực. Đầu ra từ phase khác còn phải thuộc checkpoint đã merge và hậu kiểm, có manifest/contract đã công bố. Nhận một commit đã duyệt cụ thể; không chạy theo HEAD đang thay đổi.
3. Dependency giữa task cùng phase có thể nhận candidate B Đạt trước khi CP công bố, nếu trong cùng worktree ổn định; điều này không cho consumer ở chat khác nhận snapshot chưa công bố.
4. Có quyền ghi, môi trường thử riêng và phiên bản căn cứ. Thiếu dependency thì chỉ chờ phần phụ thuộc; làm task độc lập đủ điều kiện trong phase. Không có việc sẵn sàng thì báo ID/đầu ra còn thiếu; không viết stub để tuyên bố hoàn thành hay mở hàng loạt chat chờ.

| Thời điểm đã đủ đầu vào | Chat nên chạy | Phạm vi mở / phần còn chờ |
|---|---|---|
| B kế hoạch Đạt + nhận đúng M1 | P02 và P09 | P02-T01/T02; P09-T01 rồi T02. P09 dừng ở corpus nếu cần người rà nghĩa. Chưa cần chat thứ ba. |
| P02-CP05 Đạt và công bố | P02, P03, P07 | P02 hoàn thiện mapping/UI/M2; P03 revision/context; P07-T02/T03 kiểm ảnh. P09 chuẩn bị xong thì để chờ. |
| P03-CP04 Đạt và công bố | P03, P04; có thể giữ P07 nếu còn việc ảnh | P04 gateway/planner; phần approval chờ CP05, Editor chờ CP02. |
| P04-CP01 và P03-CP01 công bố | P03 hoặc P04, cùng P05 | P05 config/provider/token/money; intent T06 vẫn chờ P04-T10/CP05. Giữ tối đa ba chat đang code. |
| P05-CP06 cùng đầu ra Review/approval đủ | P05, P06 | P06 scope/context/Repair/Research; P05 hoàn thiện UI/CLI và M5. |
| P06-CP03/CP05 cùng nguồn/ảnh đủ | P06, P07 | P07 readiness/bundle/export; P06 UI và M6. |
| Các checkpoint UI P02–P07, gồm P07-CP07, đủ | P07, P08 | P08 kiểm và hoàn thiện UI/CLI; M8 vẫn chờ M7. |
| M8 Đạt | P09 | T10 đóng gói; T03–T09 kiểm artifact theo dependency. Có thể giao các task độc lập cho worker riêng khi người dùng mở rộng phạm vi. |

Bảng này là lịch ưu tiên, không phải cam kết mọi chat luôn có việc. Hai task không phụ thuộc vẫn phải xét quyền sửa. Số checkpoint không phải thứ tự bắt buộc; CP đủ điều kiện được công bố trước.

<a id="dispatch"></a>

## 3. Phiếu nhận việc và hợp đồng đã duyệt

Mỗi chat có [phiếu nhận việc](templates/PHASE_ASSIGNMENT.md) trước code: phase/task, người triển khai, reviewer và người điều phối; commit nền; dependency candidate/report/hash; version spec/ADR; danh sách file/module được sửa; checkpoint đích; worktree/branch/profile/DB/output; AC/TC/lệnh và điểm bàn giao. Mỗi task giữ sáu trường chi tiết ở §11.

Hợp đồng phải nêu input/output, identity/version, lỗi và ý nghĩa, side effect, transaction/atomicity, cancel/retry/idempotency nếu task dùng chúng, cùng fixture có kỳ vọng độc lập. Task tạo contract là owner; consumer dùng bản đã được B duyệt. Không thêm trait/thư viện cho một consumer chỉ có trong tương lai. Ví dụ source, SQL DRAFT và tên đề xuất không thay đặc tả đã chốt.

Đổi contract: owner lập phiên bản mới và danh sách consumers; điều phối dừng phần bị ảnh hưởng, cấp lại inputs, chạy contract/integration/regression tương ứng và B kiểm lại candidate mới. Các chat khác tiếp tục dùng bản cũ đã ghim cho phần độc lập. Không tự cập nhật dependency giữa lúc reviewer đang kiểm.

Các điểm nhận đầu ra xuyên phase dưới đây là thay đổi có chủ đích so với lịch chờ toàn milestone:

| Task nhận | Đầu ra ngoài phase phải Đạt | Checkpoint phải công bố trước |
|---|---|---|
| [P03-T01](phases/PHASE-03.md#p03-t01) | [P02-T09](phases/PHASE-02.md#p02-t09) | P02-CP05 |
| [P03-T14](phases/PHASE-03.md#p03-t14) | [P02-T10](phases/PHASE-02.md#p02-t10) | P02-CP06 |
| [P04-T01](phases/PHASE-04.md#p04-t01) | [P03-T07](phases/PHASE-03.md#p03-t07) | P03-CP04 |
| [P04-T04](phases/PHASE-04.md#p04-t04) | [P03-T08](phases/PHASE-03.md#p03-t08), [P03-T09](phases/PHASE-03.md#p03-t09), [P03-T10](phases/PHASE-03.md#p03-t10) | P03-CP05 |
| [P04-T05](phases/PHASE-04.md#p04-t05) | [P03-T02](phases/PHASE-03.md#p03-t02), [P03-T04](phases/PHASE-03.md#p03-t04) | P03-CP01, P03-CP03 |
| [P04-T12](phases/PHASE-04.md#p04-t12) | [P03-T12](phases/PHASE-03.md#p03-t12) | P03-CP02 |
| [P05-T01](phases/PHASE-05.md#p05-t01) | [P04-T01](phases/PHASE-04.md#p04-t01), [P03-T02](phases/PHASE-03.md#p03-t02) | P04-CP01, P03-CP01 |
| [P05-T03](phases/PHASE-05.md#p05-t03) | [P04-T02](phases/PHASE-04.md#p04-t02) | P04-CP01 |
| [P05-T06](phases/PHASE-05.md#p05-t06) | [P04-T10](phases/PHASE-04.md#p04-t10) | P04-CP05 |
| [P05-T08](phases/PHASE-05.md#p05-t08) | [P04-T05](phases/PHASE-04.md#p04-t05) | P04-CP02 |
| [P05-T10](phases/PHASE-05.md#p05-t10) | [P03-T04](phases/PHASE-03.md#p03-t04), [P03-T10](phases/PHASE-03.md#p03-t10) | P03-CP03, P03-CP05 |
| [P05-T12](phases/PHASE-05.md#p05-t12) | [P04-T10](phases/PHASE-04.md#p04-t10) | P04-CP05 |
| [P06-T01](phases/PHASE-06.md#p06-t01) | [P05-T12](phases/PHASE-05.md#p05-t12), [P04-T09](phases/PHASE-04.md#p04-t09), [P03-T10](phases/PHASE-03.md#p03-t10) | P05-CP06, P04-CP04, P03-CP05 |
| [P06-T03](phases/PHASE-06.md#p06-t03) | [P04-T07](phases/PHASE-04.md#p04-t07), [P04-T09](phases/PHASE-04.md#p04-t09) | P04-CP04 |
| [P06-T12](phases/PHASE-06.md#p06-t12) | [P03-T10](phases/PHASE-03.md#p03-t10) | P03-CP05 |
| [P06-T13](phases/PHASE-06.md#p06-t13) | [P02-T12](phases/PHASE-02.md#p02-t12) | P02-CP06 |
| [P06-T14](phases/PHASE-06.md#p06-t14) | [P04-T12](phases/PHASE-04.md#p04-t12) | P04-CP06 |
| [P06-T15](phases/PHASE-06.md#p06-t15) | [P03-T05](phases/PHASE-03.md#p03-t05), [P03-T09](phases/PHASE-03.md#p03-t09), [P03-T10](phases/PHASE-03.md#p03-t10) | P03-CP03, P03-CP05 |
| [P07-T01](phases/PHASE-07.md#p07-t01) | [P06-T08](phases/PHASE-06.md#p06-t08), [P06-T12](phases/PHASE-06.md#p06-t12), [P02-T08](phases/PHASE-02.md#p02-t08), [P04-T09](phases/PHASE-04.md#p04-t09) | P06-CP03, P06-CP05, P02-CP05, P04-CP04 |
| [P07-T02](phases/PHASE-07.md#p07-t02) | [P02-T09](phases/PHASE-02.md#p02-t09) | P02-CP05 |
| [P07-T03](phases/PHASE-07.md#p07-t03) | [P02-T02](phases/PHASE-02.md#p02-t02) | P02-CP01 |
| [P07-T04](phases/PHASE-07.md#p07-t04) | [P03-T02](phases/PHASE-03.md#p03-t02), [P06-T12](phases/PHASE-06.md#p06-t12) | P03-CP01, P06-CP05 |
| [P08-T01](phases/PHASE-08.md#p08-t01) | [P02-T12](phases/PHASE-02.md#p02-t12), [P03-T13](phases/PHASE-03.md#p03-t13), [P04-T12](phases/PHASE-04.md#p04-t12), [P05-T13](phases/PHASE-05.md#p05-t13), [P05-T15](phases/PHASE-05.md#p05-t15), [P06-T13](phases/PHASE-06.md#p06-t13), [P06-T14](phases/PHASE-06.md#p06-t14), [P06-T15](phases/PHASE-06.md#p06-t15), [P07-T12](phases/PHASE-07.md#p07-t12), [P07-T14](phases/PHASE-07.md#p07-t14) | P02-CP06, P03-CP06, P04-CP06, P05-CP08, P06-CP06, P06-CP07, P07-CP06, P07-CP07 |
| [P08-T02](phases/PHASE-08.md#p08-t02) | [P06-T12](phases/PHASE-06.md#p06-t12), [P07-T14](phases/PHASE-07.md#p07-t14), [P05-T13](phases/PHASE-05.md#p05-t13) | P06-CP05, P07-CP07, P05-CP08 |
| [P08-T03](phases/PHASE-08.md#p08-t03) | [P05-T14](phases/PHASE-05.md#p05-t14), [P07-T14](phases/PHASE-07.md#p07-t14) | P05-CP07, P07-CP07 |
| [P08-T04](phases/PHASE-08.md#p08-t04) | [P03-T12](phases/PHASE-03.md#p03-t12), [P04-T12](phases/PHASE-04.md#p04-t12), [P05-T13](phases/PHASE-05.md#p05-t13), [P06-T15](phases/PHASE-06.md#p06-t15), [P07-T14](phases/PHASE-07.md#p07-t14) | P03-CP02, P04-CP06, P05-CP08, P06-CP07, P07-CP07 |
| [P08-T07](phases/PHASE-08.md#p08-t07) | [P02-T12](phases/PHASE-02.md#p02-t12), [P03-T13](phases/PHASE-03.md#p03-t13), [P05-T15](phases/PHASE-05.md#p05-t15), [P06-T13](phases/PHASE-06.md#p06-t13), [P06-T14](phases/PHASE-06.md#p06-t14), [P06-T15](phases/PHASE-06.md#p06-t15), [P07-T14](phases/PHASE-07.md#p07-t14) | P02-CP06, P03-CP06, P05-CP08, P06-CP06, P06-CP07, P07-CP07 |

<a id="ownership"></a>

## 4. Worktree, quyền ghi và hồ sơ dùng chung

- Mỗi chat có worktree/checkout, nhánh feature và dữ liệu chạy riêng; không dùng cùng working tree dù branch khác. Hai reviewer/runner cũng không dùng cùng DB, profile, lock, port hay output có thể bị ghi. Khi dùng công cụ tạo worktree, cấp đường dẫn thực trước; không tự suy theo tên chat.
- Chọn một chat A làm điều phối. Chỉ chat này ghi PLAN trung tâm, sổ quyền sửa, migration register và thứ tự ghép. Chat phase giữ báo cáo và mục evidence phase của mình; B chỉ giữ report riêng. Một chat khác chỉ nhận điều phối sau bàn giao rõ.
- Module ownership theo sáu trường từng task. Cấp thêm allowlist đường dẫn cụ thể trước sửa. Owner module không có quyền sửa mọi file trong crate. Không để hai chat sửa cùng file; tách file tại checkpoint của owner hoặc xếp nối tiếp.
- Các file dùng chung Cargo.toml/Cargo.lock, lib.rs/mod.rs exports, app composition/CLI registry/router, CI, README và ADR có một người ghi trong mỗi lượt. Chat consumer gửi yêu cầu/patch cho owner; không giành quyền bằng merge tự động.
- Owner storage cấp ID migration nối tiếp trong sổ trước khi viết; không sửa migration đã áp dụng, không để hai nhánh tự chọn cùng số. Khi ghép, kiểm nâng cấp/reopen từ baseline đã phát hành, rollback và references/file hashes. Domain/Application giữ hướng phụ thuộc theo ADR-008.
- Docs, PLAN, AGENTS và evidence nội bộ không tự có trong worktree mới. Điều phối cấp bản sao cục bộ đúng manifest, chỉ đọc các căn cứ đã ghim; thư mục trả báo cáo riêng cho từng phase. Bản PLAN trong worktree là snapshot, không được ghi ngược đè PLAN trung tâm. Cập nhật căn cứ phải cấp lại một lượt rõ ràng.
- Không chia sẻ workspace thật/key/EPUB/output để chạy test song song; dùng fixture được phép và bản sao tạm. Giữ Go hoạt động đến khi phần Rust thay thế đã được nghiệm thu; dọn Go theo phạm vi parity đã duyệt, không xóa chỉ vì CP Rust build được.

<a id="review"></a>

## 5. Mỗi chat tự gọi B và xử lý findings

Người dùng đã chọn reviewer gpt-6.1-sol, reasoning max. A gọi sub-agent/phiên mới với đúng model/effort, context riêng (fork_turns=none khi công cụ hỗ trợ); chỉ cung cấp nhiệm vụ, đường dẫn requirements, candidate/baseline/manifest và nơi ghi report. Không truyền kết luận tự review như chỉ thị phải đồng ý. A phải là model khác B theo AGENTS; nếu chat A dùng gpt-6.1-sol thì đổi người triển khai hoặc xin chọn reviewer khác trước lượt code. Ghi cấu hình yêu cầu, model/phiên thực tế nếu runtime xác nhận được, và giới hạn metadata; không tự nhận đã gọi đúng model khi tool không cho xác minh.

Vòng tự động: nhận task đủ điều kiện → code/test/tự rà → đóng băng candidate → gọi B → đọc report → sửa từng finding → tạo lượt mới → B kiểm lại → cập nhật trạng thái từ evidence. Không cần hỏi người dùng sau mỗi task. Không có công cụ/reviewer đúng cấu hình thì giữ Chờ review và báo rõ; không thay bằng A tự đóng vai B. Hết slot reviewer thì xếp hàng, không bỏ gate.

B đọc AC/spec trước code và báo cáo A; tự chạy checks phù hợp, kiểm assertion và lỗi/biên, không sửa code/tests chính thức. Verdict Đạt/Cần sửa/Chưa đủ bằng chứng. Findings phải có căn cứ; không bắt sửa sở thích hoặc bịa lỗi. Candidate và yêu cầu đổi thì B đánh giá lại phần bị ảnh hưởng. Task/CP có manual bắt buộc chưa chạy vẫn chờ kiểm chứng.

Một review có thể bao phủ task đơn cuối cùng đồng thời checkpoint của nó nếu report ghi rõ cả phạm vi task và kết quả ghép/baseline/AC; không bắt lặp toàn bộ lệnh vô ích. Với CP nhiều task đã review riêng, B vẫn phải kiểm bản ghép và giao tiếp giữa chúng. Cuối phase B kiểm thêm toàn §12–16 và bài tự test trên candidate cuối.

<a id="git-checkpoint"></a>

## 6. Commit và push ngay sau checkpoint

1. Từng task/nhóm và bản ghép checkpoint đã đủ AC/test, B Đạt còn hiệu lực. Ghim baseline gồm các CP nhận từ phase khác; kiểm không lẫn thay đổi khác và phiên bản schema/contract phù hợp.
2. Với CP có code Rust, chạy trên đúng nội dung dự định commit:

~~~text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
~~~

Chạy thêm AC/TC của CP, ca consumer bị ảnh hưởng và manual được CP yêu cầu. Nếu ảnh hưởng Go thì kiểm Go tương ứng. Lệnh thành công với zero ca liên quan không đủ bằng chứng. Windows là bắt buộc; Linux chỉ bắt buộc khi đã tiếp nhận nền tảng theo PLAN. CI có ở PR thì kiểm đúng candidate; job bắt buộc hỏng/chưa chạy phải ghi chờ, không coi push thành nghiệm thu.

3. Commit/push thường ngay sau B Đạt và đủ checks, tạo draft PR đúng scope. Điều phối duy nhất xếp hàng, merge từng PR đủ mọi cổng vào rewrite/rust, rồi hậu kiểm; không đợi hết phase. Mỗi PR chỉ gồm CP được duyệt; PR đã merge thì mở PR mới cho CP sau.
4. Áp dụng đầy đủ [quy trình merge và khóa phần đã nghiệm thu](MERGE_WORKFLOW.md): exact base/head/tree, ghép worktree sạch, checks và B trên bản ghép, bảo vệ race, merge commit và hậu kiểm. Thiếu gate/quyền hoặc cần sửa phần bảo vệ ngoài ngoại lệ conflict thuần test thì dừng và báo Master; không force-push hay bypass protection.
5. Cross-phase chỉ nhận CP code đã merge và hậu kiểm, ghim head, merge SHA và base receipt; CP chỉ có docs nội bộ nhận manifest/review, không có Git. Đối chiếu nhánh dependency cũ chưa merge theo quy trình, không cherry-pick trùng.

Checkpoint là điểm công bố code, khác checkpoint trạng thái tác vụ bên trong sản phẩm (P04-T10/P05-T12). Không gộp một phase thành một lần push. Tách CP phát sinh cần cập nhật graph, assignment và review phạm vi trước khi code phần mới; không bỏ AC để tạo mốc giả.

<a id="register"></a>

## 7. Sổ điều phối và bằng chứng

PLAN giữ bảng chat, task, quyền ghi, inputs và bước tiếp. Người điều phối cập nhật bằng report A/B và commit thực tế. Mẫu giao việc nằm ở [PHASE_ASSIGNMENT](templates/PHASE_ASSIGNMENT.md); prompt dùng ngay ở [CHAT_STARTERS](CHAT_STARTERS.md). Danh sách 112 task/64 checkpoint ở [PARALLEL_TASKS.json](PARALLEL_TASKS.json) dùng kiểm dependency/cycle/coverage, chưa thay evidence nghiệm thu.

| Task / CP | A / B / điều phối | Nhánh / worktree / dữ liệu thử | Inputs + contract/hash | Quyền ghi | Candidate / report | Commit / remote SHA / PR | Trạng thái / điều kiện tiếp |
|---|---|---|---|---|---|---|---|
| Ghi khi cấp việc | Model và phiên thật | Đường dẫn thực tế | Phiên bản đã duyệt | Files và shared owner | Báo cáo theo lượt | Ghi sau thao tác thật | Sẵn sàng/Đang làm/Chờ dependency/Chờ review/Cần sửa/Chờ kiểm chứng/Đạt chờ Git/Đã push/chờ merge/Đã merge chờ hậu kiểm/Đã công bố/Bị chặn |

Hồ sơ mỗi phase tại evidence/phase-XX: PXX-TYY.handoff-rNN.md, PXX-TYY.review-rNN.md, PXX-CPYY.handoff-rNN.md, PXX-CPYY.review-rNN.md cùng manifests. CP cuối liên kết report Mx, giữ lịch sử findings. Sau khi B Đạt, A phase gửi kết quả về điều phối; không sửa report B. Task chưa giao hoặc chưa chạy không được ghi Đạt.

## 8. Chất lượng và nền tảng

Mỗi lớp đều có bằng chứng: quy tắc thuần; contract producer/consumer; storage/adapter cùng lỗi và recovery; UI/CLI đi qua use case thật; bản ghép checkpoint; cổng cuối phase và release. Không dồn test vào P09. Fake provider giữ tests offline, không thay đánh giá model thật có ngân sách/người rà nghĩa ở P09. Giữ bài tự test §16.4 của mọi phase để người dùng kiểm kết quả đã công bố.

Lượt Linux/WSL là công việc nền tảng riêng khi được giao: xác minh WSL/distro/toolchain, thử đúng M1, build/test/TUI thật; sửa tương thích thì chạy lại Windows và B review trước khi đổi nền dùng chung. Không đồng thời thay CI/platform chung với các chat nghiệp vụ chưa được điều phối cấp quyền. Kết quả Ubuntu/WSL không tự là chứng nhận CachyOS. Hiện chưa có bằng chứng Linux trong kế hoạch này.

## 9. Dọn Go trong khi các phase chạy song song

Thực hiện [PLAN §8](../../PLAN.md#go-retirement-plan) tại từng checkpoint: rà phần Go liên quan, đăng ký lượt dọn nhỏ khi replacement và caller đủ bằng chứng, kiểm và B review candidate sau dọn, commit/push ngay. CP chức năng đã Đạt được công bố trước, không chờ lượt dọn. Điều phối giữ sổ, cấp một owner Go cùng allowlist và kiểm các consumer/nhánh đang chạy trước khi cấp quyền xóa; chat phase không tự gỡ shared code đang được chat khác dùng.

Các việc GO-PXX-nn/PXX-GO-CPnn được cấp thêm vào PLAN, graph và assignment rồi B kiểm lịch bổ sung; chưa nằm trong 112 task/64 CP chức năng ban đầu. Nhận commit cleanup mới và kiểm ảnh hưởng theo cùng quy tắc đổi baseline. Giữ đường Go còn caller; ghi task/owner/mốc giải quyết. GO-FINAL sau M9 là phần bắt buộc của mục tiêu chỉ Rust, có review bản dọn và artifact mới trên môi trường không Go; merge vào rewrite/rust theo quy trình khi đủ cổng; quyết định phát hành vẫn thuộc người dùng.


## 10. Scratch sau checkpoint

Giao owner, mục đích và điều kiện giữ/giải phóng cho từng scratch, sandbox review và build output; tái dùng build dir riêng theo runner/toolchain/config khi phù hợp. Sau xác nhận remote SHA và tạo/cập nhật PR, A dọn ngay trước CP tiếp theo; CP docs-only dọn sau B Đạt và bàn giao ghi nhận. Ghi receipt và giữ phần cần cho review/CI/consumer, rà lại sau merge/hậu kiểm theo [quy trình dọn checkpoint](MERGE_WORKFLOW.md#checkpoint-cleanup). Không đổi candidate đang review hoặc evidence cũ.
