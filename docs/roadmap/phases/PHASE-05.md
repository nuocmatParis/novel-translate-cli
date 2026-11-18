# PHASE 05 — THỰC THI MODEL, NGÂN SÁCH VÀ ĐIỀU PHỐI

> Lịch thực thi v2 · 12/11/2026.

> Lịch task song song và checkpoint commit/push tại [mục 10](#p05-schedule). Áp dụng [quy tắc giao việc](../PARALLEL_EXECUTION.md#dispatch) và [review/Git checkpoint](../PARALLEL_EXECUTION.md#git-checkpoint). Phiếu giao việc ghim version/hash đầu vào khi triển khai.

> Ủy quyền 14/11/2026: [quy trình merge và khóa phần đã nghiệm thu](../MERGE_WORKFLOW.md) áp dụng từ lượt Git tiếp theo. CP code ngoài phase chỉ được nhận sau merge và hậu kiểm; CP docs-only nhận manifest/review. AC, dependency và trạng thái không đổi.

> **Kế hoạch triển khai v0.3 · 09/11/2026 · Chưa bắt đầu.** M5 nhận luồng Dịch–Review giả lập từ M4 rồi bổ sung kết nối, tiền, lần gửi và quyền chạy. Các lựa chọn kỹ thuật chưa có ADR sẽ được chốt ở task sử dụng.
>
> Giữ nhánh người dùng của Q05 đã bổ sung ở P00-T01-r03, finding F01. T13/AC13/TC13 hiển thị **Chương · Block ID · nguyên nhân**, cho chỉnh điểm chia hoặc giới hạn qua core. T15/AC15/TC15 trả JSON issue có vị trí, `next_action` và **exit 7** khi chưa thể lập lượt an toàn.

## 1. Thông tin phase

| Thuộc tính | Nội dung |
|---|---|
| ID / milestone | P05 / M5 |
| Tên | Thực thi model, ngân sách và điều phối |
| Trạng thái | Kế hoạch đã chi tiết. Chờ M4, chưa triển khai hoặc chạy test M5. |
| Đầu vào | M4 gateway/planner/workflow/checkpoint. M3 dữ liệu có version, Skill và revision. |
| Giao diện | UI-06 Tác vụ và UI-07 Cấu hình kết nối/model/ngân sách |
| Phân rã | 5 gói, 16 task |
| Nhánh | Nhánh theo phần việc, ví dụ `feature/model-gateway` hoặc `feature/task-queue`. PR vào `rewrite/rust`. |
| Vai trò | A triển khai và bàn giao. B khác kiểm độc lập đúng candidate. |

Xem [roadmap chung](../ROADMAP.md), [Bước 9](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md#phase-05) và [quy tắc repo](../../../AGENTS.md). Tài liệu, dữ liệu thử riêng và bằng chứng giữ ngoài Git.

## 2. Mục tiêu và demo cuối phase

Chạy một request Dịch hoặc Review qua cùng core từ TUI/CLI, với model được chọn, ngân sách và quyền chạy hợp lệ. Hệ thống lưu từng lần gửi, kết quả và tiền. Sau gián đoạn, mở lại đúng bước và chỉ gửi tiếp khi đủ điều kiện.

1. Dùng hai workspace và hai client cùng hồ sơ test. Cho A chạy, B chờ. Mở chi tiết B để kiểm nhãn toàn app vẫn chỉ A đang giữ quyền.
2. Giả lập A lỗi tạm thời có lịch retry, rồi một ca chưa rõ provider đã xử lý. So thời gian còn lại, usage chưa xác định, khoản dự phòng và bộ đếm trước/sau mở lại.
3. Tạm dừng A, đối chiếu trạng thái đã dừng gửi ở tool, rồi chọn B độc lập. Phản hồi A đến muộn chỉ cập nhật lịch sử/usage của A.
4. Ngắt tại intent, response và các điểm bàn giao quyền. Mở lại, đối chiếu hai DB và số lần transport thực nhận. Giữ bản đã lưu và khoản chưa rõ.
5. Dùng key giả thiếu, hạn mức thiếu và block quá dài. Kiểm lần lượt cách UI/CLI báo lỗi, vị trí nguồn, mã thoát và việc không gửi. Thử chỉnh hợp lệ rồi lập lại đúng phần chưa gửi.

Demo dùng transport giả và bộ dữ liệu tự soạn. Các bước thao tác ở [mục 16.4](#p05-user-test). Lần gọi provider thật nếu có được ghi riêng theo kết nối, phạm vi và ngân sách do người dùng chọn.

## 3. Phạm vi

| Phase 05 thực hiện | Consumer sau |
|---|---|
| Connection/profile/credential, provider đầu tiên, response normalization và tokenizer | Phase 06 gọi Research/repair qua cùng gateway và dispatcher. |
| Giá, currency/FX, usage, tiền dự phòng và call cap | Phase 06 thêm phí công cụ Research theo cùng ledger. Phase 09 kiểm kết quả trên phạm vi được chọn. |
| Request/attempt intent, deadline/retry, late result và restart | Phase 06 mở rộng workflow chờ/tiếp tục. |
| Coordinator, IPC, OS lock, queue, permit và đối chiếu hai DB | Phase 06 dùng để chạy phần đã chọn hoặc đến hết EPUB. |
| UI-06/07 và CLI kết nối/status/pause/resume/dispatch tối thiểu | Phase 08 kiểm lại luồng CLI/TUI đầy đủ. |

Phạm vi hiện hành là Windows theo các mốc trước. Adapter thật chỉ được nối với đường chạy có đầy đủ guard. Thử nghiệm mặc định chạy offline với transport giả. Quyết định thay Go dùng [bảng đối chiếu chuyển đổi](PHASE-03.md#p03-go-retirement) và phạm vi đã kiểm trên bản thay thế.

## 4. Điều kiện bắt đầu

P05-T01 nhận gateway P04-T01 và revision/outbox P03-T02 đã Đạt; T03 cần planner P04-T02. T06 chưa được mở trước khi P04-T10 bàn giao checkpoint bền vững. Chọn provider/model và chốt ADR tại consumer, không suy từ fake. P05-CP06 bàn giao executor/handoff/recovery cho P06-T01; UI/CLI và M5 tiếp tục trong P05. P05-T16/M5 vẫn cần M4.

Cổng mở task là danh sách **Đầu vào bắt buộc** ở §11 và [bảng phụ thuộc chung](../PARALLEL_TASKS.json). Các nhắc tới M2–M8 trong giải thích kỹ thuật chỉ nguồn gốc chức năng; không tự thêm điều kiện chờ toàn phase ngoài danh sách này. Điều kiện đóng phase tại §15 và task tích hợp cuối vẫn giữ đầy đủ.

Trước code, chat owner nhận phiếu giao việc từ [PLAN](../../../PLAN.md), source/contract/spec/ADR đúng phiên bản, report B Đạt và commit của checkpoint cung cấp. Mỗi chat có worktree, nhánh feature, dữ liệu thử và thư mục build riêng. Các quyền sửa chung phải được người điều phối cấp trước. Ghi rõ quyết định §6 còn mở; chốt và review trước consumer. Hợp đồng, fixture và dữ liệu giả chỉ chứng minh phạm vi đã kiểm; ghép với producer thật và chạy lại ca liên quan trước nghiệm thu.

Windows x64 theo M1 là target bắt buộc hiện hành. Linux/WSL có lượt nền tảng riêng theo PLAN; chưa có bằng chứng thì ghi chưa xác minh, không tự suy Windows Đạt thành Linux Đạt. Không gọi model trả phí trong CI.

## 5. Đặc tả và quy tắc phải giữ

| ID | Nguồn |
|---|---|
| S-Q | [36 quyết định](../../spec/DESIGN_DECISIONS_36.md) |
| S-MODEL | [Model, retry và chi phí](../../spec/MODEL_STRATEGY_SPEC.md) |
| S-SEC | [Bảo mật và API key](../../spec/SECURITY_SPEC.md) |
| S-WORKFLOW | [Điều phối Dịch–Review](../../spec/WORKFLOW_ORCHESTRATION_SPEC.md) |
| S-DATA | [Dữ liệu, revision và trạng thái](../../spec/DATA_MODEL_STATE_SPEC.md) |
| S-INPUT | [EPUB đầu vào](../../spec/EPUB_INPUT_SPEC.md) |
| S-TUI | [Hợp đồng TUI/CLI](../../spec/TUI_CLI_SPEC.md) |
| D-TECH | [Bước 5 — Nền tảng và contract](../../desgin/TECHNICAL_FOUNDATION_STEP5.md) |
| D-DATA | [Bước 6 — Kiến trúc dữ liệu](../../desgin/DATA_ARCHITECTURE_STEP6.md) |
| D-WORKFLOW | [Bước 7 — State machine và recovery](../../desgin/WORKFLOW_STATE_MACHINE_STEP7.md) |
| D-MODULE | [Bước 4 — Module và ownership](../../desgin/MODULE_DESIGN_STEP4.md) |
| D-UI | [Bước 8 — UI/UX](../../desgin/UI_UX_DESIGN_STEP8.md) |
| D-ROADMAP | [Bước 9 — Roadmap](../../desgin/IMPLEMENTATION_ROADMAP_STEP9.md) |
| R-REVIEW | [Quy trình hai model](../REVIEW_WORKFLOW.md) |
| R-REPO | [Quy tắc repo](../../../AGENTS.md) |

- **Q05/Q17.** Profile có nguồn/version, limits và bộ đếm. Tính toàn request và kiểm riêng input/output/context. Giữ phần bắt buộc và protected inline. Roles Dịch/Review/Research giữ dữ liệu đúng vai trò, Research có thể dùng cấu hình Review. Đổi model do người dùng chỉ áp dụng phần chưa gửi.
- **11A và Q17.** Key đọc từ kho hệ điều hành hoặc biến môi trường đã chọn. Chỉ gửi bí mật tới kết nối đã cấu hình. Workspace/IPC/log/JSON lưu hoặc truyền tham chiếu cần thiết, giữ bí mật ở đường xác thực. Lưu key và provider chấp nhận key là hai trạng thái.
- **Q18 và 8A/9A.** Lưu phiên bản giá, đơn vị, currency/FX và nhóm usage. Hạn mức dùng **đã ghi nhận + dự phòng đang chạy/chưa rõ + ước lượng lần tới**. Không giới hạn tiền là lựa chọn rõ và vẫn giữ call cap nếu bật. Unknown có trạng thái riêng. Giữ các bộ đếm qua đổi model/chương/restart.
- **Q19.** `request_id` định danh công việc, `attempt_id` định danh lần gửi có intent. Mặc định 10 phút mỗi lần, tối đa hai retry tạm thời sau 10/30 giây hoặc lâu hơn theo provider. Persist thời điểm sớm nhất. Fatal/uncertain dừng để xử lý. Giữ riêng hai sửa cấu trúc và năm vòng sửa chất lượng theo consumer.
- **Q28–29.** Một Coordinator mỗi hồ sơ theo OS lock, một quyền chạy hợp lệ, writer riêng của workspace. A chờ retry vẫn giữ quyền. Handoff khi đã ngừng lên lịch và xử lý xong lần gọi tại tool. Hai DB đối chiếu qua cùng operation ID. Giữ state khi chưa biết outcome.
- **Q13/34.** Kiểm hash/version Skill và dependency trước gửi. Request đã gửi giữ cấu hình và căn cứ đã ghim. CLI và TUI cùng guard, trả trạng thái rõ cho người dùng và script.

## 6. ADR và điểm còn mở

Nhận [ADR-002](../../decisions/ADR-002.md) và [ADR-004](../../decisions/ADR-004.md) cho writer/generation. [ADR-008](../../decisions/ADR-008.md) là căn cứ ranh giới, DTO và JSON. ADR-005/006 và phần W7-04–09 của ADR-007 được lập hoặc mở rộng ở M5.

| ID | Điểm cần chốt và cách kiểm | Owner / hạn |
|---|---|---|
| P05-D01 | Cấu hình hiệu lực từng role, key refs, quyền local, lỗi và lọc dữ liệu. Thử sentinel và kế thừa theo S-MODEL. | T01 trước config/credential. |
| P05-D02 | TD-02/08. Chọn adapter/provider, transport/runtime khi có consumer. Chốt docs phiên bản, lỗi, usage, stream/cut và profile/tokenizer có provenance. | T02/T03 trước provider/planner. |
| P05-D03 | DB-09. Precision, rounding, units, currency/FX, phân loại token tính tiền và reservation/call cap. So oracle số tiền độc lập. | T04/T05 trước ledger/budget. |
| P05-D04 | W7-04/08/09. Intent/reservation trước send, phân loại outcome, timeout/retry/cancel và checkpoint. Thử fault send gap. | T06/T07 trước dispatch/retry. |
| P05-D05 | W7-05. Fencing theo attempt/permit/version, nhận late usage, quyền publication và replay. | T08 trước commit kết quả. |
| P05-D06 | TD-10/11, DB-10. Chọn process/IPC/ACL, lock theo profile, workspace lease, protocol version và lifecycle. Windows Named Pipes là phương án cần kiểm, chưa là quyết định. | T09 trước triển khai IPC. |
| P05-D07 | W7-06/07, DB-10. Queue/eligibility, số lần cấp quyền, grant/ack/release hai DB và startup reconciliation. Thử hai process cùng khởi động và điểm ngắt sau mỗi ghi. | T10/T11 trước queue/handoff, T12 trước resume. |
| P05-D08 | Read model UI/CLI, kết quả chưa rõ chi phí, nhánh Q05 và exit mapping. Xác định lệnh đã nối và điều kiện live smoke. | T13–T15 trước consumer, T16 kiểm tích hợp. |

A ghi lựa chọn, input, kiểm thử và giới hạn theo [lịch DB/TD/W7](../ROADMAP.md#decision-schedule) và [mục ADR còn mở](../ROADMAP.md#adr-followups). Chốt từng quyết định đúng consumer.

## 7. Thiết kế triển khai

### 7.1. Trách nhiệm và dữ liệu

| Thành phần | Sở hữu |
|---|---|
| `nt-domain` và `nt-application` | Quy tắc tiền, eligibility, preflight, request/attempt, retry và điều kiện công bố. Port theo M4. |
| `nt-model` | Provider transport, chuẩn hóa stream/response, capability/tokenizer và credential adapter. |
| `nt-coordinator` | OS lock, lifecycle IPC, queue và quyền chạy theo contract. |
| `nt-storage` | Series ledger/checkpoint/revision và Coordinator DB qua port riêng. |
| App và `nt-tui` | Nối adapters, lệnh CLI, read model và hành động UI. |

Series DB giữ cấu hình task, revision, checkpoint, intent/attempt, reservations/usage và operation ACK liên quan. Coordinator DB giữ hàng đợi, profile owner, grant sequence và handoff. Chỉ nối adapters tại app. Giữ transaction DB ngắn và tách khỏi mạng.

### 7.2. Từ chuẩn bị đến gửi và công bố

1. Giải cấu hình hiệu lực rồi ghim scope, source/mapping, Skill/revision, model, giới hạn và tiền/cap. Kiểm model, token và context theo M4. Key dùng trong lớp xác thực đã chọn.
2. Đối chiếu permit hiện hành, stop intent, căn cứ và kết quả budget. Ghi intent, reservation và checkpoint bền vững theo D04. Guard ngay tại điểm giao transport giữ đúng số quyền và ngăn stale owner gửi.
3. Gửi qua adapter. Ghi response, finish reason, usage và provider ID nếu có. Phần bị cắt hoặc mất outcome giữ trạng thái riêng. Cập nhật tiền của đúng attempt, kể cả attempt đã timeout.
4. Nếu response hợp lệ về trước lần gửi retry, dùng lại. Sau khi retry đã gửi, response cũ giữ history/usage. Chỉ attempt hiện hành còn đúng version/permit qua validation và CAS để công bố một kết quả logic.
5. Tiếp tục từ checkpoint phù hợp. Khi thay đổi source/Skill, dựng lại phần chưa gửi và xét dependency của kết quả đã có.

### 7.3. Chuyển quyền qua hai DB

Coordinator xác định A giữ quyền qua profile, owner/sequence và queue. Khi A tạm dừng hoặc chờ, ghi stop, ngừng dispatch và hoàn tất xử lý lần gọi ở tool. Giữ outcome chưa rõ và dự phòng của A. Chọn B chỉ sau khi kiểm độc lập và có thao tác phù hợp. Handoff dùng operation ID chung, ACK và điều kiện fence theo D07. Reopen đối chiếu OS lock, cả hai DB, attempt ledger và version trước khi cấp hoặc dùng quyền.

Tại ranh giới stop và transport, protocol phải phân biệt lần gửi chưa được nhận với lần đã được tool nhận và cần đợi xử lý xong. Grant B chỉ dùng được sau mốc an toàn đã ghi. Grant, pause, release và resume đều cần đường replay. Hai DB có thể dừng ở hai mốc khác nhau. T11 phải kiểm thứ tự và cách đối chiếu các cặp mốc, gồm ACK trễ và process cũ quay lại. Lấy được lock là điều kiện sở hữu Coordinator, còn quyền gửi phải qua đối chiếu và guard đã chốt.

## 8. Dữ liệu, lỗi và phục hồi

| Tình huống | Hành vi cần giữ |
|---|---|
| Key thiếu hoặc store lỗi | Giữ task/config references. Báo cần xử lý đúng kết nối, chưa gọi model. |
| Vượt budget/cap hoặc thiếu dữ liệu tính hạn mức | Giữ tiến độ, trả lý do và số đã dùng/dự phòng. Thiếu giá trong chế độ không giới hạn hiện chưa xác định. |
| Block không chia an toàn | Giữ blocked, có chương/Block ID/range/nguyên nhân. Chỉ lập lại khi người dùng cấp đầu vào hợp lệ. |
| Có intent, chưa đủ bằng chứng gửi/kết quả | **Chưa rõ đã xử lý**. Giữ khoản dự phòng và lượt đang chờ, đối chiếu nếu provider hỗ trợ. |
| Có response đầy đủ đã lưu | Validate artifact đúng nền trước khi cân nhắc request khác. |
| Lỗi tạm thời đã xác định | Retry cùng model trong giới hạn, tôn trọng not-before và recheck mọi guard. |
| Auth/fatal/unknown | Lưu trạng thái và hành động tiếp theo. Giữ lịch sử, chưa gửi lại tự động. |
| Response cũ đến sau retry hoặc handoff | Usage/response vào đúng attempt cũ. Fence ngăn đổi current. |
| Hai DB lệch sau crash | Đối chiếu operation/sequence và Series ledger. Chưa cấp quyền khi còn mâu thuẫn. |
| Pause/cancel đang gọi | Ngừng lịch gửi mới. Phân biệt tool đã dừng với provider xác nhận hủy. Giữ usage chưa rõ. |

Reopen bảo toàn cấu hình hiệu lực, phiên bản, ngân sách/cap, thời gian retry còn lại và ý định người dùng. Kiểm integrity/migration như M2–M4. Diagnostics và JSON đã lọc thông tin nhạy cảm.

## 9. Phần TUI/CLI

| Phần | Hành vi cần thấy |
|---|---|
| UI-06 Tác vụ | Nhãn toàn app là tác vụ đang giữ quyền. Chi tiết theo task có role/step, requested/actual model, elapsed/deadline/retry, tiền đã ghi/ước tính/dự phòng/chưa rõ và cap. |
| Pause/resume và chọn B | Hành động gửi qua core. Handoff có trạng thái chờ. Chọn B độc lập chỉ hiện khi phù hợp. Xem B giữ nhãn hoạt động A. |
| UI-06 block quá dài | Hiện Chương · Block ID · nguyên nhân từ core. Chỉnh điểm chia hoặc giới hạn ngay màn, validate rồi lập lại. Protected inline vẫn giữ. |
| UI-07 Kết nối/cấu hình | Key được che. Hiện trạng thái lưu và nguồn hiệu lực. Đổi default chỉ áp cho tác vụ mới. Mở màn chưa gửi request tính phí. |
| CLI | `connection set/list/remove`, `status`, `pause`, `resume` và phần `translate/review` đã nối có cùng preflight/permit. Hành vi theo S-TUI và ADR-008. |

Non-interactive kết thúc hữu hạn, lưu state và trả `task_id`, `issues`, `next_action` theo contract. `status` đọc thành công trả 0 kể cả task đang chờ. Nhánh Q05 chưa lập được lượt trả issue có vị trí và **exit 7**. Những thao tác chưa có luồng nghiệp vụ trả chẩn đoán khả dụng. UI-08 và chọn phạm vi đầy đủ thuộc Phase 06.

<a id="p05-schedule"></a>

## 10. Lịch task song song và checkpoint

### 10.1. Quyền phụ trách của chat

Chat P05 phụ trách trọn phase; mỗi lượt chỉ làm một task hoặc nhóm 2–3 task gắn chặt đã đăng ký. P05-T01 nhận gateway P04-T01 và revision/outbox P03-T02 đã Đạt; T03 cần planner P04-T02. T06 chưa được mở trước khi P04-T10 bàn giao checkpoint bền vững. Chọn provider/model và chốt ADR tại consumer, không suy từ fake. P05-CP06 bàn giao executor/handoff/recovery cho P06-T01; UI/CLI và M5 tiếp tục trong P05. P05-T16/M5 vẫn cần M4. Quy tắc đọc/ghi, giao đầu ra và reviewer nằm trong [PARALLEL_EXECUTION](../PARALLEL_EXECUTION.md). Hàng task trong [PARALLEL_TASKS.json](../PARALLEL_TASKS.json) là danh sách dependency để đối chiếu với thẻ §11.

<a id="p05-checkpoints"></a>

### 10.2. Checkpoint công bố sớm

Mọi checkpoint dưới đây **Chưa mở** ở thời điểm viết kế hoạch. Đây là lịch v2, thay cách nhóm CP của bản v0.1; task/AC/TC giữ ID cũ. Mỗi checkpoint có 1–3 task, có thể xong và push trước checkpoint mang số nhỏ hơn khi dependency đã đủ. Trong cùng checkpoint vẫn review từng task/nhóm đúng quy tắc.

**Ngay khi đủ kiểm chứng và B Đạt trên bản ghép của checkpoint, commit và push checkpoint đó, cập nhật draft PR vào rewrite/rust; không chờ các checkpoint khác hoặc hết phase.** Không dồn checkpoint đã Đạt để đợi UI, M cuối hoặc task độc lập. Nếu task/nhóm Đạt sớm nhưng phần khác trong CP còn chờ, người điều phối tách CP thành CPxx-A/CPxx-B trước review tích hợp, giữ đủ AC, dependency và lịch sử. CP docs-only bị ignore ghi Git không áp dụng; không tạo commit rỗng hay force-add docs.

| Checkpoint | Phạm vi 1–3 task | Đầu vào checkpoint | Kiểm trên bản ghép trước commit/push | Nhánh / người nhận |
|---|---|---|---|---|
| <a id="p05-cp01"></a>P05-CP01 — Connections và provider adapter | [P05-T01](#p05-t01), [P05-T02](#p05-t02) | [P03-CP01](PHASE-03.md#p03-cp01), [P04-CP01](PHASE-04.md#p04-cp01) | P05-T01: V-01 trong phạm vi AC/TC của task; P05-T02: V-02 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp02"></a>P05-CP02 — Token, chi phí và hạn mức | [P05-T03](#p05-t03), [P05-T04](#p05-t04), [P05-T05](#p05-t05) | [P04-CP01](PHASE-04.md#p04-cp01), [P05-CP01](#p05-cp01) | P05-T03: V-03 trong phạm vi AC/TC của task; P05-T04: V-04 trong phạm vi AC/TC của task; P05-T05: V-04 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp03"></a>P05-CP03 — Request intent bền vững | [P05-T06](#p05-t06) | [P04-CP05](PHASE-04.md#p04-cp05), [P05-CP02](#p05-cp02) | P05-T06: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp04"></a>P05-CP04 — Retry và phản hồi tới muộn | [P05-T07](#p05-t07), [P05-T08](#p05-t08) | [P04-CP02](PHASE-04.md#p04-cp02), [P05-CP01](#p05-cp01), [P05-CP03](#p05-cp03) | P05-T07: V-05 trong phạm vi AC/TC của task; P05-T08: V-05 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp05"></a>P05-CP05 — Coordinator và hàng đợi | [P05-T09](#p05-t09), [P05-T10](#p05-t10) | [P03-CP03](PHASE-03.md#p03-cp03), [P03-CP05](PHASE-03.md#p03-cp05), [P05-CP03](#p05-cp03) | P05-T09: V-06 trong phạm vi AC/TC của task; P05-T10: V-06 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp06"></a>P05-CP06 — Handoff và phục hồi | [P05-T11](#p05-t11), [P05-T12](#p05-t12) | [P04-CP05](PHASE-04.md#p04-cp05), [P05-CP03](#p05-cp03), [P05-CP04](#p05-cp04), [P05-CP05](#p05-cp05) | P05-T11: V-06, V-07 trong phạm vi AC/TC của task; P05-T12: V-07 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp07"></a>P05-CP07 — Giao diện kết nối và ngân sách | [P05-T14](#p05-t14) | [P05-CP01](#p05-cp01), [P05-CP02](#p05-cp02) | P05-T14: V-08, V-11 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-11 chỉ Connections/Settings/budget của T14; Tasks thuộc CP08, toàn M5 ở CP09. V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp08"></a>P05-CP08 — Tasks, CLI và dispatch | [P05-T13](#p05-t13), [P05-T15](#p05-t15) | [P05-CP02](#p05-cp02), [P05-CP04](#p05-cp04), [P05-CP05](#p05-cp05), [P05-CP06](#p05-cp06), [P05-CP07](#p05-cp07) | P05-T13: V-08, V-11 trong phạm vi AC/TC của task; P05-T15: V-09 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu. V-11 phần Tasks và hồi quy Settings đã có; CLI V-09 thuộc T15. Toàn M5 ở CP09. V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |
| <a id="p05-cp09"></a>P05-CP09 — Nghiệm thu M5 | [P05-T16](#p05-t16) | M4 Đạt, [P05-CP01](#p05-cp01), [P05-CP02](#p05-cp02), [P05-CP03](#p05-cp03), [P05-CP04](#p05-cp04), [P05-CP05](#p05-cp05), [P05-CP06](#p05-cp06), [P05-CP07](#p05-cp07), [P05-CP08](#p05-cp08) | P05-T16: V-07, V-10, V-11, V-00 trong phạm vi AC/TC của task. Kiểm giao tiếp producer/consumer, stale/reopen khi có lưu dữ liệu.  Toàn §12–16, bài tự test và M5; V-00 cho code, B review đúng candidate. | feature/model-execution; A chat P05 tích hợp, B độc lập. |

Cột kiểm chứng liệt kê V theo từng task: chạy toàn bộ AC/TC bắt buộc của task đó và ca giao tiếp/hồi quy đã có trong candidate. Với V là một suite dùng chung, ghi tên/số ca thực chạy thuộc scope trong handoff; không yêu cầu ca của task tương lai để đóng CP sớm và không ghi các ca chưa có là Đạt. Các manual scope được tách rõ ở hàng liên quan; CP cuối chạy trọn phase. V-00 là bốn kiểm tra Rust trong AGENTS, áp dụng đúng nội dung checkpoint. Ca chưa có consumer phải ghi chưa áp dụng và kiểm lại tại checkpoint nhận consumer; không dùng điều này để bỏ AC bắt buộc của task đang đóng. File dùng chung chỉ được ghép theo lượt của người điều phối. Kết quả B của task không tự thay review bản ghép checkpoint hoặc cổng cuối phase.

## 11. Thẻ task cho AI

Đọc thẻ đã giao cùng hợp đồng ở §5–8, AC và ca kiểm tương ứng. Chia thay đổi theo các bước trong thẻ. Mỗi thao tác gửi, bàn giao hoặc lưu phải có bằng chứng trên đúng scope và candidate.

<a id="p05-t01"></a>

### P05-T01 — Connections, model roles và credentials

- **Đầu ra.** Cấu hình hiệu lực, nguồn key rõ và lỗi thiết lập an toàn.
- **Đầu vào.** Dependency thực thi: [P04-T01](PHASE-04.md#p04-t01), [P03-T02](PHASE-03.md#p03-t02). Căn cứ kỹ thuật: Q17, S-MODEL/S-SEC, ADR-008 và D01.
- **Vùng sửa.** Application config/credential port và `nt-model` phần adapter key theo môi trường.

1. Tách metadata kết nối, model alias/profile và tham chiếu key. Giải mặc định tool/riêng bộ và lựa chọn task theo từng loại cấu hình. Lưu snapshot của tác vụ mới.
2. Chọn Dịch và Review theo config. Research nhận cấu hình riêng hoặc kế thừa Review theo spec. Đổi model đã lưu cần thao tác riêng, giữ lịch sử và counters.
3. Dùng OS store cho TUI hoặc biến môi trường được chọn cho CLI. Chốt process thực sự lấy key để gửi. Thiếu biến ở nơi gửi báo lỗi rõ. Không tự đọc .env hoặc đổi sang nguồn key khác. IPC và DB giữ refs, không chuyển key thô để bù nguồn thiếu.
4. Kiểm quyền đọc/lưu/xóa và mask. Thay key áp từ lần chưa gửi. Xóa key giữ connection/history và báo cần thiết lập lại. Lưu key chỉ báo kết quả lưu.
5. Dùng sentinel với fake store/env và kiểm adapter Windows trên profile thử. So file/DB/log/JSON/IPC và lỗi. Giữ secret chỉ ở đường xác thực của kết nối đã chọn.

**Kiểm chứng.** P05-AC01, P05-TC01, V-01. B ưu tiên biến tồn tại nhưng không được chọn, lỗi store và thay key giữa hai request.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T01 |
|---|---|
| Đầu vào bắt buộc | [P04-T01](PHASE-04.md#p04-t01), [P03-T02](PHASE-03.md#p03-t02). Nhận gateway P04-T01, connection policy và revision/schema P03-T01/T02 đã duyệt. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Cấu hình role lưu key reference và lựa chọn hiệu lực, snapshot theo task, credential lỗi được redaction. Owner đầu ra là A-P05-T01. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T01 giữ Application config/credential port và `nt-model` phần adapter key theo môi trường. Phần chung do T01 giữ config/credential contract, người tích hợp giữ app registration. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T03](PHASE-04.md#p04-t03), [P04-T04](PHASE-04.md#p04-t04), [P04-T05](PHASE-04.md#p04-t05). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC01 / P05-TC01, gồm các ca chi tiết §13.3. V-01. Đối chiếu Đúng ref/key được chọn, không leak trong DB/log/IPC/UI. |
| Điểm tích hợp | [P05-CP01](#p05-cp01) — Connections và provider adapter; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t02"></a>

### P05-T02 — Provider gateway và normalize response

- **Đầu ra.** Adapter đầu tiên chuyển request/result theo gateway M4.
- **Đầu vào.** Dependency thực thi: [P05-T01](#p05-t01). Căn cứ kỹ thuật: T01, Q17, D-TECH, D02/TD-02/08. Dùng tài liệu chính thức của provider lúc triển khai.
- **Vùng sửa.** `nt-model` transport, capability, normalization và app composition thử.

1. Chốt provider, API/version/endpoint, transport/runtime và features cần dùng. Lưu nguồn tài liệu và giả định về giới hạn, usage, retry hint và query outcome.
2. Map role và payload M4 sang request thật. Giữ header xác thực theo kết nối đã chọn, chặn redirect hoặc endpoint ngoài policy làm lộ key. Tách thông tin bí mật khỏi payload/log.
3. Chuẩn hóa response một lần và dạng luồng thành cùng hồ sơ. Giữ requested alias, actual ID nếu có, provider request ID, finish reason, contents, usage, thời gian và loại lỗi.
4. Phân biệt hoàn tất, cut, lỗi tạm thời xác định, lỗi cấu hình/fatal và chưa rõ outcome. Giữ phần dở để chẩn đoán. Adapter trả dữ liệu, Application quyết retry và publish.
5. Test transcript tổng hợp của các chế độ và lỗi. Khi dùng loopback, kiểm mạng ngoài bị cô lập. Chỉ gắn đường gọi thật vào executable sau guard T15.

**Kiểm chứng.** P05-AC02, P05-TC02, V-02. B đối chiếu transcript, unknown fields và stream thiếu event hoàn tất.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T02 |
|---|---|
| Đầu vào bắt buộc | [P05-T01](#p05-t01). Nhận connection được chọn và Gateway DTO M4. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Provider normalize complete/cut/error/usage theo tài liệu chính thức được ghim lúc triển khai, giữ unknown fields trung thực. Owner đầu ra là A-P05-T02. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T02 giữ `nt-model` transport, capability, normalization và app composition thử. Phần chung do P04-T01 giữ gateway, T02 giữ capability/transport D02. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T03](PHASE-04.md#p04-t03), [P04-T04](PHASE-04.md#p04-t04), [P04-T05](PHASE-04.md#p04-t05). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC02 / P05-TC02, gồm các ca chi tiết §13.3. V-02. Đối chiếu Không bịa field hoặc dùng response dở làm hoàn tất. |
| Điểm tích hợp | [P05-CP01](#p05-cp01) — Connections và provider adapter; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t03"></a>

### P05-T03 — Tokenizer và planner theo model

- **Đầu ra.** Kế hoạch request theo model/capability có provenance, chặn quá trần trước send.
- **Đầu vào.** Dependency thực thi: [P05-T02](#p05-t02), [P04-T02](PHASE-04.md#p04-t02). Căn cứ kỹ thuật: T02, Q05/17, S-INPUT và planner M4. Chốt D02 cho hồ sơ tokenizer.
- **Vùng sửa.** Provider capability/tokenizer binding và application preflight.

1. Gắn profile với model/version, nguồn input/output/context, tokenizer và adapter overhead. Dùng bộ đếm đúng phiên bản khi có. Thiếu bộ đếm dùng ước lượng có nhãn và biên theo Q05. Thiếu trần đáng tin báo chờ.
2. Đếm toàn payload sau ghép Skill, context, source, schema và phần bao. Tính output/reasoning theo profile, không đếm reasoning hai lần. Áp mặc định thử nghiệm 3.000 token nguồn và biên 5%/10% theo Q05 khi phù hợp.
3. Kiểm riêng ba trần. Cắt ngữ cảnh tùy chọn rồi chia ở biên an toàn. Giữ quy tắc bắt buộc và protected inline. Giữ diagnostic Q05 của M4 khi block không thể chia.
4. Khi người dùng đổi model, giới hạn hoặc điểm chia, validate lại và lập kế hoạch phần chưa gửi. Giữ units/revisions hợp lệ và usage/counters đã lưu.
5. Bàn giao breakdown, nguồn/version và mã lỗi cho T13/T15. Test bằng oracle độc lập với bộ đếm production.

**Kiểm chứng.** P05-AC03, P05-TC03, V-03. B thử exact/estimate, trần output nhỏ dù context còn rộng và re-plan chỉ phần chưa gửi.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T03 |
|---|---|
| Đầu vào bắt buộc | [P05-T02](#p05-t02), [P04-T02](PHASE-04.md#p04-t02). Nhận provider capability và full-payload planner M4. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Tokenizer/profile có provenance, trả count/estimate với margin và chẩn đoán vượt trần trước send. Owner đầu ra là A-P05-T03. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T03 giữ Provider capability/tokenizer binding và application preflight. Phần chung do T02 giữ capability, P04-T02 giữ planner, T03 giữ tokenizer binding. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T03](PHASE-04.md#p04-t03), [P04-T04](PHASE-04.md#p04-t04), [P04-T05](PHASE-04.md#p04-t05). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC03 / P05-TC03, gồm các ca chi tiết §13.3. V-03. Đối chiếu Oracle token, diagnostics và request không gửi ở ca bị chặn. |
| Điểm tích hợp | [P05-CP02](#p05-cp02) — Token, chi phí và hạn mức; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t04"></a>

### P05-T04 — Money, pricing và usage ledger

- **Đầu ra.** Sổ tiền có thể đối chiếu từng attempt, phiên bản giá và phép quy đổi.
- **Đầu vào.** Dependency thực thi: [P05-T02](#p05-t02). Căn cứ kỹ thuật: T02, Q18, S-MODEL, D03/DB-09.
- **Vùng sửa.** Domain money, application pricing/ledger và persistence cần thiết.

1. Chốt precision, phép làm tròn, overflow và biểu diễn giá/currency. Lưu đơn vị tính, ngày, nguồn và version. Cho ghi đè giá có provenance. Fixture tính expected độc lập.
2. Tách nhóm billable theo provider. Dùng rõ input thường/cache/output/reasoning theo dữ liệu hiện có. Thiếu thành phần hoặc cách tính giữ unknown. Phí công cụ Research có dòng riêng khi consumer dùng.
3. Dùng một currency mỗi task. Khi cần FX, lưu tỷ giá do người dùng nhập, ngày và chiều quy đổi cùng phép tính. Thiếu hoặc sai FX giữ chưa xác định.
4. Ghim giá/dự trù lúc gửi. Khi có actual model/usage, tính số ghi nhận hoặc điều chỉnh có căn cứ. Giữ giá gốc và lịch sử khi bảng giá đổi.
5. Dedup usage theo attempt/provider evidence theo D03. Đối chiếu cả attempt cũ và response đến muộn. Không hiển thị dữ liệu thiếu thành 0.

**Kiểm chứng.** P05-AC04, P05-TC04, V-04. B kiểm cached/reasoning, FX khác chiều và việc sửa giá không tính lại lịch sử âm thầm.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T04 |
|---|---|
| Đầu vào bắt buộc | [P05-T02](#p05-t02). Nhận usage/model identity cùng price/currency inputs được chốt. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Money/ledger lưu decimal, price snapshot và actual usage theo attempt, phân biệt chưa rõ với zero. Owner đầu ra là A-P05-T04. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T04 giữ Domain money, application pricing/ledger và persistence cần thiết. Phần chung do T02 giữ usage normalization, T04 giữ DB-09/accounting schema. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T03](PHASE-04.md#p04-t03), [P04-T04](PHASE-04.md#p04-t04), [P04-T05](PHASE-04.md#p04-t05). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC04 / P05-TC04, gồm các ca chi tiết §13.3. V-04. Đối chiếu Số tiền và adjustment đúng oracle, lịch sử còn nguyên. |
| Điểm tích hợp | [P05-CP02](#p05-cp02) — Token, chi phí và hạn mức; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t05"></a>

### P05-T05 — Reservation, hạn mức tiền và call cap

- **Đầu ra.** Preflight có lý do chặn và quyết định dự phòng có thể ghi cùng intent.
- **Đầu vào.** Dependency thực thi: [P05-T03](#p05-t03), [P05-T04](#p05-t04). Căn cứ kỹ thuật: T03–T04, Q18, 8A/9A, D03.
- **Vùng sửa.** Domain budget/cap và application guard, dữ liệu config đã chốt.

1. Giải cả cấu hình ngân sách theo task → bộ → tool. Ghi mode/currency/limit/cap và nguồn. Giá trị thiếu hoặc sai trả cần sửa. Chế độ không giới hạn cần lựa chọn rõ.
2. Kiểm `đã ghi nhận + dự phòng đang chạy/chưa rõ + ước lượng lần tới` theo cùng currency. Ca đúng ngưỡng theo hợp đồng. Nếu không ước lượng đủ cho hạn mức thì chặn.
3. Kiểm cap độc lập với tiền, tính các lần model gồm retry và sửa cấu trúc đã thực sự gửi. Giữ slot đang chờ/chưa rõ theo contract để không lách cap. Phí công cụ là tiền riêng.
4. Tạo reservation với estimate và căn cứ. T06 lưu authoritative reservation trong cùng write boundary với intent. Khi reconcile, chuyển khoản có căn cứ sang đã ghi nhận đúng một lần. Giữ phần chưa rõ.
5. Đổi mode/hạn mức/cap do người dùng có history. Sang chương, đổi role/model hoặc restart giữ số đã dùng và dự phòng.

**Kiểm chứng.** P05-AC05, P05-TC05, V-04. B thử dưới/bằng/trên hạn mức, N và N+1, unlimited có cap và pending reservation.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T05 |
|---|---|
| Đầu vào bắt buộc | [P05-T03](#p05-t03), [P05-T04](#p05-t04). Nhận token budget, giá và ledger/reservations. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Guard nhận effective cap/limit cùng liabilities, reserve trước send, chặn đủ tiền/call cap theo Q18/19. Owner đầu ra là A-P05-T05. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T05 giữ Domain budget/cap và application guard, dữ liệu config đã chốt. Phần chung do T03 giữ token count, T04 giữ money, T05 giữ reservation policy. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T03](PHASE-04.md#p04-t03), [P04-T04](PHASE-04.md#p04-t04), [P04-T05](PHASE-04.md#p04-t05). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC05 / P05-TC05, gồm các ca chi tiết §13.3. V-04. Đối chiếu So tổng tiền, slot và số transport thực nhận. |
| Điểm tích hợp | [P05-CP02](#p05-cp02) — Token, chi phí và hạn mức; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t06"></a>

### P05-T06 — Durable request/attempt intent

- **Đầu ra.** Mỗi lần gửi có ID, input/version, reservation và checkpoint bền vững trước mạng.
- **Đầu vào.** Dependency thực thi: [P05-T05](#p05-t05), [P04-T10](PHASE-04.md#p04-t10). Căn cứ kỹ thuật: T05, Q19/29, M4 checkpoint, D04/W7-04 và generation M2/M3.
- **Vùng sửa.** Application dispatch boundary, Series ledger và persistence. Kiểm với fake trước khi nối executable.

1. Tạo logical request ID và attempt ID với source/Skill/base/model/price/capability, deadline và permit. Phân biệt prepared, intent, được nhận ở transport và outcome.
2. Kiểm lại stop, quyền, versions, budget/cap. Ghi intent, dự phòng và checkpoint trong write boundary nhất quán. Chỉ trao request cho transport sau ACK đã ghi.
3. Tại điểm gửi, giữ guard cho owner/sequence và điều kiện đang dừng. Nối giao thức grant thật ở T09–T11. Test race revoke bằng seam của dispatch, giữ nhật ký transport thực nhận.
4. Ghi response/usage/ID provider khi có và kết quả xác định được. Nếu chỉ còn intent sau crash và chưa biết đã gửi/kết quả, giữ **Chưa rõ đã xử lý**. Tra provider nếu hỗ trợ. Dự phòng và slot đang chờ giữ nguyên đến khi đối chiếu.
5. Chạy failpoint trước intent, sau intent/trước send, sau send và sau response. So số lần transport, ledger và checkpoint. Replay không tự gửi hoặc reserve thêm.

**Kiểm chứng.** P05-AC06, P05-TC06, V-05. B kiểm marker, ACK và causal order từ ghi bền vững đến transport.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T06 |
|---|---|
| Đầu vào bắt buộc | [P05-T05](#p05-t05), [P04-T10](PHASE-04.md#p04-t10). Nhận budget/reservation cùng checkpoint M4 và generation M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Attempt intent lưu identity/input versions/reservation trước IO, hồi phục phân biệt send gap và known outcome. Owner đầu ra là A-P05-T06. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T06 giữ Application dispatch boundary, Series ledger và persistence. Kiểm với fake trước khi nối executable. Phần chung do T04/T05 giữ ledger/budget, T06 giữ W7-04 và dispatch seam. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC06 / P05-TC06, gồm các ca chi tiết §13.3. V-05. Đối chiếu So markers, số request và ledger khi mở lại. |
| Điểm tích hợp | [P05-CP03](#p05-cp03) — Request intent bền vững; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t07"></a>

### P05-T07 — Timeout và retry có giới hạn

- **Đầu ra.** Retry có nguyên nhân, lịch và số lần dùng được giữ qua mở lại.
- **Đầu vào.** Dependency thực thi: [P05-T02](#p05-t02), [P05-T06](#p05-t06). Căn cứ kỹ thuật: T02/T06, Q19, 9A, D04/W7-08/09.
- **Vùng sửa.** Application retry/cancel policy và adapter timer/cancellation seam.

1. Phân loại error từ adapter theo D04. Chỉ lỗi tạm thời đã xác định đi qua retry. Auth/fatal/thiếu cấu hình và mất outcome giữ điểm chờ.
2. Dùng deadline tổng mặc định 10 phút/lần gọi. Hai retry ngoài lần đầu chờ 10 rồi 30 giây, tôn trọng khoảng dài hơn provider yêu cầu. Ghi not-before và counter.
3. Trước retry, kiểm đã có response hợp lệ hay chưa, pause/stop, cùng model, căn cứ, permit và budget/cap. Hết điều kiện giữ trạng thái và lý do.
4. Tách dừng ở tool, hủy transport và provider xác nhận hủy. Khi chưa biết, giữ usage/dự phòng. Retry wait giữ quyền A. T11 mới bàn giao khi có thao tác phù hợp.
5. Test fake clock sát ngưỡng, mở lại trước/sau not-before và ca đã đủ hai lần. Structural fixes và quality repairs giữ bộ đếm riêng.

**Kiểm chứng.** P05-AC07, P05-TC07, V-05. B thử retry hint dài, clock/restart và timeout local chưa rõ outcome.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T07 |
|---|---|
| Đầu vào bắt buộc | [P05-T02](#p05-t02), [P05-T06](#p05-t06). Nhận transport errors, intent IDs và timer/cancel seam. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Retry phân loại nguyên nhân, giữ model/counter/deadline qua restart, outcome chưa rõ đi reconciliation. Owner đầu ra là A-P05-T07. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T07 giữ Application retry/cancel policy và adapter timer/cancellation seam. Phần chung do T02 giữ error class, T06 giữ intent, T07 giữ W7-08/09. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC07 / P05-TC07, gồm các ca chi tiết §13.3. V-05. Đối chiếu Số lần gửi, model, điều kiện và thời điểm đúng. |
| Điểm tích hợp | [P05-CP04](#p05-cp04) — Retry và phản hồi tới muộn; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t08"></a>

### P05-T08 — Late response fencing và publication

- **Đầu ra.** Một kết quả logic được công bố trên đúng nền, usage mọi attempt còn truy được.
- **Đầu vào.** Dependency thực thi: [P05-T06](#p05-t06), [P05-T07](#p05-t07), [P04-T05](PHASE-04.md#p04-t05). Căn cứ kỹ thuật: T06–T07, Q19, D05/W7-05, CAS M3 và seam guard permit. T08 kiểm bằng fixture, T11/T15 nối sequence thật theo D07.
- **Vùng sửa.** Application response selection/publication, ledger/history và write guard.

1. Trước khi retry thực sự gửi, dùng response cũ đầy đủ và còn hợp lệ nếu đã lưu. Chốt trạng thái khi việc gửi retry chưa được xác định, bảo toàn reservation.
2. Khi retry đã gửi, chỉ attempt hiện hành có quyền công bố. Response cũ đến muộn được ghi vào history/usage đúng attempt và đúng A sau handoff.
3. Kiểm request/attempt, scope, snapshot/Skill/base và owner/permit sequence ở điểm commit. Dùng khóa duy nhất cho một kết quả logic trên nền đó cùng generation CAS.
4. Replayed result không ghi hai lần. Nếu mất ACK, đối chiếu operation ID và kết quả đã công bố. Nếu stale, giữ artifact/lý do và current không đổi.
5. Thử hai thứ tự response, duplicate và stale phiên bản/quyền. Khi usage muộn về, đối chiếu đúng tiền cũ và kết quả hiện hành.

**Kiểm chứng.** P05-AC08, P05-TC08, V-05. B xem cả trace transport và lịch sử cho từng attempt.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T08 |
|---|---|
| Đầu vào bắt buộc | [P05-T06](#p05-t06), [P05-T07](#p05-t07), [P04-T05](PHASE-04.md#p04-t05). Nhận attempt state, current revision và guard permit đã mô tả bằng fixture. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Chỉ attempt được phép publish trên nền hiện hành, usage muộn vẫn hạch toán đúng task/attempt. Owner đầu ra là A-P05-T08. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T08 giữ Application response selection/publication, ledger/history và write guard. Phần chung do T06/T07 giữ attempt, P03-T02 giữ CAS, T08 giữ fencing W7-05. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC08 / P05-TC08, gồm các ca chi tiết §13.3. V-05. Đối chiếu So quyền hiện hành, history, tiền và generation. |
| Điểm tích hợp | [P05-CP04](#p05-cp04) — Retry và phản hồi tới muộn; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t09"></a>

### P05-T09 — Coordinator process, IPC và OS lock

- **Đầu ra.** Một Coordinator hợp lệ mỗi hồ sơ, client TUI/CLI cùng kết nối qua giao thức đã kiểm.
- **Đầu vào.** Dependency thực thi: [P05-T06](#p05-t06). Căn cứ kỹ thuật: T06, Q28, D06/TD-10/11 và ADR-005 cần chốt.
- **Vùng sửa.** `nt-coordinator` lifecycle/IPC/lock, workspace lease và app composition.

1. Chọn kiến trúc process, canonical profile identity và IPC trên Windows. Chốt giới hạn message, protocol version, auth/ACL và ai giữ transport/credential reference.
2. Lấy OS lock trước khi tạo owner/endpoint hợp lệ. PID, file tồn tại hoặc mất heartbeat riêng lẻ chưa đủ quyền tiếp quản. Workspace write lease tách khỏi quyền điều phối.
3. Client dùng cùng port và request ID. Reject message sai phiên bản, quá giới hạn, sai hồ sơ/chủ thể và ACK từ server cũ. Chưa kết nối được thì trả trạng thái hữu hạn.
4. Kiểm startup cạnh tranh, client reconnect, shutdown và tiếp quản sau khi OS lock giải phóng. Trước grant phải qua reconciliation T11/T12.
5. Dùng hai process thật trên profile tạm và bản Windows đã ghi. Test quyền truy cập, workspace khác, endpoint stale và khóa đang giữ.

**Kiểm chứng.** P05-AC09, P05-TC09, V-06. B tự kiểm hai process, OS lock và trường hợp client không có quyền.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T09 |
|---|---|
| Đầu vào bắt buộc | [P05-T06](#p05-t06). Nhận intent/task identity và profile/lease contract cần chốt ở ADR-005. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | IPC version/ACL/OS lock xác định Coordinator hợp lệ mỗi profile, trả lỗi hữu hạn khi thiếu quyền. Owner đầu ra là A-P05-T09. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T09 giữ `nt-coordinator` lifecycle/IPC/lock, workspace lease và app composition. Phần chung do T06 giữ task/attempt IDs, T09 giữ transport/lifecycle D06. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC09 / P05-TC09, gồm các ca chi tiết §13.3. V-06. Đối chiếu Chỉ owner hợp lệ, lỗi hữu hạn và không bypass. |
| Điểm tích hợp | [P05-CP05](#p05-cp05) — Coordinator và hàng đợi; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t10"></a>

### P05-T10 — Queue, grant và task eligibility

- **Đầu ra.** Hàng đợi bền vững và tối đa một quyền gửi hiệu lực trong cùng hồ sơ.
- **Đầu vào.** Dependency thực thi: [P05-T06](#p05-t06), [P05-T09](#p05-t09), [P03-T04](PHASE-03.md#p03-t04), [P03-T10](PHASE-03.md#p03-t10). Căn cứ kỹ thuật: T09/T06, Q28, S-WORKFLOW, D07 và dependency M3.
- **Vùng sửa.** Coordinator queue/permit và application eligibility.

1. Lưu task/workspace/profile, thứ tự enqueue, scope và idempotency key. Yêu cầu lặp nhận đúng task/kết quả theo contract.
2. Kiểm task đã giao, prerequisite, pause, workspace lease, version và sự độc lập khi xin chạy thay. Giữ lý do queued/ineligible rõ.
3. Cấp sequence/owner và ACK theo D07. Chưa đủ căn cứ hai phía thì chưa gửi. Client kiểm current grant qua đường có thể xác minh, không tự suy từ trạng thái màn.
4. A chuyển vai trò hoặc chờ retry vẫn giữ quyền. B đang xem hoặc xếp hàng chưa có quyền. Khi A hoàn tất, kiểm B theo thứ tự. Khi A chờ hoặc pause, dùng T11 cho chọn B độc lập.
5. Test nhiều client cùng enqueue/claim, duplicate, shared dependency và restart. So số grant hợp lệ và số transport admission, không chỉ số row.

**Kiểm chứng.** P05-AC10, P05-TC10, V-06. B ưu tiên ca A retry wait, B phụ thuộc A và hai thao tác claim cạnh tranh.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T10 |
|---|---|
| Đầu vào bắt buộc | [P05-T06](#p05-t06), [P05-T09](#p05-t09), [P03-T04](PHASE-03.md#p03-t04), [P03-T10](PHASE-03.md#p03-t10). Nhận Coordinator IPC, durable queue và eligibility từ M3. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Admission/grant gắn task/profile/sequence, tối đa một permit hợp lệ và replay giữ thứ tự. Owner đầu ra là A-P05-T10. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T10 giữ Coordinator queue/permit và application eligibility. Phần chung do T09 giữ IPC/lock, T10 giữ queue/permit D07. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC10 / P05-TC10, gồm các ca chi tiết §13.3. V-06. Đối chiếu Trace không có hai grant hợp lệ hoặc hai admission trái phép. |
| Điểm tích hợp | [P05-CP05](#p05-cp05) — Coordinator và hàng đợi; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t11"></a>

### P05-T11 — Handoff và đối chiếu hai SQLite

- **Đầu ra.** Pause/wait/release/grant đối chiếu được sau ngắt, A cũ không gửi sau bàn giao.
- **Đầu vào.** Dependency thực thi: [P05-T08](#p05-t08), [P05-T10](#p05-t10). Căn cứ kỹ thuật: T08/T10, Q28–29, D07, DB-10/W7-06/07 và ADR-006 cần chốt.
- **Vùng sửa.** Application handoff, Coordinator và Series ledger qua port.

1. Chốt mốc durable intent/ack/release và cùng operation ID ở hai DB. Liệt kê cặp trạng thái, guard gửi và điều kiện replay. Chỉ commit từng DB trong boundary của nó.
2. Khi nhận pause/wait, lưu stop intent và checkpoint. Ngừng lập lịch mới, đưa lần gọi tại tool tới trạng thái đã xử lý hoặc chưa rõ có ghi nhận. Giữ usage/reservation và bước của A.
3. Khi người dùng chọn B độc lập, kiểm dependencies/lease/scope, đối chiếu A đã dừng gửi và thu hồi số quyền cũ. Chỉ B được dùng grant khi đủ xác nhận theo protocol.
4. Khi process chết hoặc ACK trễ, xác minh OS lock, sequence, operation IDs và ledger thực. Hoàn tất replay hoặc giữ chờ nếu hai DB chưa thể đối chiếu. Chủ cũ/ACK cũ bị fence.
5. Nếu provider A còn trả muộn, ghi riêng usage A, không công bố lên dữ liệu hiện hành. Test fault sau từng ghi và race giữa stop/dispatch bằng barrier.

**Kiểm chứng.** P05-AC11, P05-TC11, V-06/V-07. B kiểm marker hai DB, active owner và request trace trước/sau handoff.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T11 |
|---|---|
| Đầu vào bắt buộc | [P05-T08](#p05-t08), [P05-T10](#p05-t10). Nhận fencing attempt, queue/grant cùng hai ledger. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Grant/ACK/release đối chiếu hai DB theo operation/sequence, task cũ hết quyền gửi sau bàn giao. Owner đầu ra là A-P05-T11. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T11 giữ Application handoff, Coordinator và Series ledger qua port. Phần chung do T08 giữ publication guard, T10 giữ permit, T11 giữ handoff ADR-006. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC11 / P05-TC11, gồm các ca chi tiết §13.3. V-06, V-07. Đối chiếu Đối chiếu operation/sequence, quyền gửi và attempt ledger. |
| Điểm tích hợp | [P05-CP06](#p05-cp06) — Handoff và phục hồi; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t12"></a>

### P05-T12 — Recovery checkpoint và resume

- **Đầu ra.** Mở lại đúng bước và căn cứ. Tiếp tục có kiểm quyền, tiền và ý định người dùng.
- **Đầu vào.** Dependency thực thi: [P05-T06](#p05-t06), [P05-T07](#p05-t07), [P05-T08](#p05-t08), [P05-T09](#p05-t09), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P04-T10](PHASE-04.md#p04-t10). Căn cứ kỹ thuật: T06–T11, Q29, M4 checkpoint và D04/D07.
- **Vùng sửa.** Application recovery, storage migration/readback và Coordinator reconciliation.

1. Khóa/kiểm integrity, schema và generation. Đọc job Markdown/Skill pending từ M3. Kiểm version checkpoint M4 và cách nâng/đọc lại.
2. Đối chiếu Coordinator, grant/sequence và Series operation. Kiểm intent/attempt, reservations, call counts và not-before. Giữ trạng thái chưa rõ nếu thiếu chứng cứ.
3. Có response đầy đủ thì validate đúng bản. Có revision đã commit thì Review đúng revision. Có local result hiệu lực thì đi bước kế phù hợp. Replay chỉ khi guard cho phép.
4. Nếu source/Skill/model/base thay đổi, kiểm dependency và dựng lại phần chưa gửi theo cấu hình tác vụ đã lưu và thay đổi người dùng vừa xác nhận. Giữ lịch sử, tiền và artifact đã biết.
5. Paused vẫn chờ lệnh tiếp tục. Resume đưa task qua eligibility/queue, kiểm lại key, limits, budget và thời gian retry. Đối chiếu đủ rồi mới được gửi.

**Kiểm chứng.** P05-AC12, P05-TC12, V-07. B thử workspace M4 nâng lên, checkpoint/ref lỗi, paused và uncertain.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T12 |
|---|---|
| Đầu vào bắt buộc | [P05-T06](#p05-t06), [P05-T07](#p05-t07), [P05-T08](#p05-t08), [P05-T09](#p05-t09), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P04-T10](PHASE-04.md#p04-t10). Nhận attempt/checkpoint/handoff state và schema M4→M5. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Resume xác minh quyền, money, căn cứ và pause intent trước tiếp tục, giữ counters/history. Owner đầu ra là A-P05-T12. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T12 giữ Application recovery, storage migration/readback và Coordinator reconciliation. Phần chung do T06/T11 giữ state writers, T12 giữ recovery/migration. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P03-T03](PHASE-03.md#p03-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC12 / P05-TC12, gồm các ca chi tiết §13.3. V-07. Đối chiếu Readback giữ state, không resend hoặc reset. |
| Điểm tích hợp | [P05-CP06](#p05-cp06) — Handoff và phục hồi; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P06-T01](PHASE-06.md#p06-t01). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t13"></a>

### P05-T13 — UI-06 Tasks và quyền chạy thực tế

- **Đầu ra.** Người dùng thấy task đang chạy thật, các lý do chờ và cách xử lý block quá dài.
- **Đầu vào.** Dependency thực thi: [P05-T03](#p05-t03), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P05-T12](#p05-t12). Căn cứ kỹ thuật: T10–T12, T03, Q05/17/18/28, S-TUI/D-UI, D08.
- **Vùng sửa.** TUI Tasks, read model và hành động pause/resume/chọn B.

1. Tách global active identity khỏi task đang xem. Chi tiết có role/step, requested/actual model, thời gian/deadline/retry, cost, reservation, unknown và cap.
2. Nối pause/resume và chọn B độc lập qua application. Hiển thị chờ handoff và outcome chưa rõ. UI không tự cấp permit hoặc đặt lại bộ đếm.
3. Với block không có chỗ ngắt an toàn, hiện **Chương · Block ID · nguyên nhân** từ core. Cho mở vùng nguồn và chỉnh điểm chia hoặc giới hạn ngay màn.
4. Gửi lựa chọn tường minh cho planner validate/re-plan phần chưa gửi. Nếu điểm chia cắt protected inline hoặc giới hạn không đáng tin, giữ chặn. Các guard tiền/quyền vẫn áp dụng.
5. Test B đang xem khi A active, retry wait, late response, resize và ca chỉnh điểm chia/giới hạn có kết quả. Dùng terminal thật ở V-11.

**Kiểm chứng.** P05-AC13, P05-TC13, V-08/V-11. B xem vị trí nguồn, output sau chỉnh và số request thực ở ca blocked.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T13 |
|---|---|
| Đầu vào bắt buộc | [P05-T03](#p05-t03), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P05-T12](#p05-t12). Nhận task/role/model thực, queue, usage và recovery diagnostics. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | UI Tasks phân biệt task đang xem với task active, pause/resume qua core, Q05 có vị trí và hành động. Owner đầu ra là A-P05-T13. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T13 giữ TUI Tasks, read model và hành động pause/resume/chọn B. Phần chung do T10/T12 giữ core commands, T13 giữ Tasks read state, người tích hợp giữ router. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P06-T01](PHASE-06.md#p06-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC13 / P05-TC13, gồm các ca chi tiết §13.3. V-08, V-11. Đối chiếu UI state và request trace đúng. Block lỗi chưa gửi. |
| Điểm tích hợp | [P05-CP08](#p05-cp08) — Tasks, CLI và dispatch; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T02](PHASE-08.md#p08-t02), [P08-T04](PHASE-08.md#p08-t04). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t14"></a>

### P05-T14 — UI-07 Connections, model và budget

- **Đầu ra.** Kết nối, model và ngân sách dễ hiểu, có nguồn hiệu lực và bảo vệ key.
- **Đầu vào.** Dependency thực thi: [P05-T01](#p05-t01), [P05-T03](#p05-t03), [P05-T04](#p05-t04), [P05-T05](#p05-t05). Căn cứ kỹ thuật: T01/T03–T05, S-TUI/S-SEC/S-MODEL và D08.
- **Vùng sửa.** TUI Settings/connections và app wiring tới ports.

1. Form kết nối hiển thị tên/provider và tình trạng đã lưu. Key nhập/đọc qua port, mask khi gõ và khi render. Lỗi store giữ thông tin có ích, không lộ bí mật.
2. Chọn roles và nguồn kế thừa. Hiển thị requested/actual và capability thiếu khi có. Mở màn hoặc lưu key không tự gọi model trả phí.
3. Hiện chế độ ngân sách, currency/limit, cap và nguồn tool/bộ/task. Cho sửa rõ ở đúng cấp. Thay default chỉ tác động task mới.
4. Đổi/xóa key hoặc model theo user intent và áp phần chưa gửi. Giữ attempt/history và counters. Hiện yêu cầu cấu hình lại khi thiếu key.
5. Test hai task snapshot khác nhau, input focus, mask/diagnostic và fake transport có số gọi bằng 0 khi chỉ xem/cấu hình.

**Kiểm chứng.** P05-AC14, P05-TC14, V-08/V-11. B đối chiếu effective config, sentinel và các thao tác khi chưa cấp quyền.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T14 |
|---|---|
| Đầu vào bắt buộc | [P05-T01](#p05-t01), [P05-T03](#p05-t03), [P05-T04](#p05-t04), [P05-T05](#p05-t05). Nhận effective config, token/price/budget và credential references. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Settings nhập/lưu qua core, giữ snapshot task đang chạy và che giá trị bí mật ở mọi view. Owner đầu ra là A-P05-T14. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T14 giữ TUI Settings/connections và app wiring tới ports. Phần chung do T01/T05 giữ config/budget, T14 giữ forms, người tích hợp giữ app wiring. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T03](PHASE-04.md#p04-t03), [P04-T04](PHASE-04.md#p04-t04), [P04-T05](PHASE-04.md#p04-t05). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC14 / P05-TC14, gồm các ca chi tiết §13.3. V-08, V-11. Đối chiếu Không leak/không send, giữ snapshot và counters cũ. |
| Điểm tích hợp | [P05-CP07](#p05-cp07) — Giao diện kết nối và ngân sách; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T03](PHASE-08.md#p08-t03). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t15"></a>

### P05-T15 — CLI và nối dispatch an toàn

- **Đầu ra.** CLI đi qua cùng quyền, preflight, tiền và checkpoint như TUI.
- **Đầu vào.** Dependency thực thi: [P05-T08](#p05-t08), [P05-T09](#p05-t09), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P05-T12](#p05-t12), [P05-T14](#p05-t14). Căn cứ kỹ thuật: T08–T12/T14, Q05/34, S-TUI/ADR-008 và D08.
- **Vùng sửa.** App CLI/composition và process harness nối Coordinator thật với transport giả.

1. Nối `connection set/list/remove`, `status`, `pause`, `resume` và phần Dịch/Review đã đủ use case. Parse cờ trước IO. Không tạo đường trực tiếp tới provider ngoài dispatcher.
2. Xác minh nguồn key được chọn sẵn ở process gửi và guard permission/budget/approval. Lưu state khi non-interactive. Không chờ stdin hoặc tạo task ID giả.
3. Giữ envelope JSON v1, stdout/stderr và exit mapping theo S-TUI. Status đọc thành công trả 0 dù task đang chờ. Phân biệt cấu hình, lỗi, conflict, hạn mức/cap và hàng đợi.
4. Nhánh block không chia an toàn trả `issues` có vị trí block và nguyên nhân, `next_action` cho bước chỉnh hợp lệ và **exit 7**. Lưu trạng thái, chưa gửi. Re-plan vẫn qua guard.
5. Chạy process harness để so trace/JSON/exit với TUI và DB. Thử CLI lặp cùng task, thiếu key, pause/resume và blocked. Kiểm sentinel ở cả output/lỗi.

**Kiểm chứng.** P05-AC15, P05-TC15, V-09. B kiểm no-send và hành động tiếp theo ở từng mã thoát, kể cả Q05.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T15 |
|---|---|
| Đầu vào bắt buộc | [P05-T08](#p05-t08), [P05-T09](#p05-t09), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P05-T12](#p05-t12), [P05-T14](#p05-t14). Nhận guard/fencing, Coordinator/recovery và config đã nối. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | CLI dispatch dùng đúng permit/reservation, JSON/exit/next_action phản ánh persisted state kể cả Q05. Owner đầu ra là A-P05-T15. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T15 giữ App CLI/composition và process harness nối Coordinator thật với transport giả. Phần chung do T08/T11/T12 giữ send/recovery, người tích hợp M5 giữ CLI/composition. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P04-T11](PHASE-04.md#p04-t11), [P04-T12](PHASE-04.md#p04-t12), [P06-T01](PHASE-06.md#p06-t01). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC15 / P05-TC15, gồm các ca chi tiết §13.3. V-09. Đối chiếu Output/exit khớp DB và transport. Không đợi stdin. |
| Điểm tích hợp | [P05-CP08](#p05-cp08) — Tasks, CLI và dispatch; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: [P08-T01](PHASE-08.md#p08-t01), [P08-T07](PHASE-08.md#p08-t07). Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


<a id="p05-t16"></a>

### P05-T16 — Multi-process fault campaign và M5

- **Đầu ra.** Bằng chứng trên Windows cho quyền, tiền, retry, migration và phục hồi toàn luồng.
- **Đầu vào.** Dependency thực thi: [P05-T01](#p05-t01), [P05-T02](#p05-t02), [P05-T03](#p05-t03), [P05-T04](#p05-t04), [P05-T05](#p05-t05), [P05-T06](#p05-t06), [P05-T07](#p05-t07), [P05-T08](#p05-t08), [P05-T09](#p05-t09), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P05-T12](#p05-t12), [P05-T13](#p05-t13), [P05-T14](#p05-t14), [P05-T15](#p05-t15), M4 Đạt. Căn cứ kỹ thuật: T01–T15, Q17–19/28–29/36, D06/D07 và W7-14.
- **Vùng sửa.** App fault/integration tests, fixture, tài liệu dùng thật và hồ sơ M5.

1. Dùng hai process, hai workspace và profile tạm. Ghi process/OS lock, IDs/sequence, operation markers và transport receipts để so thứ tự thực.
2. Ngắt quanh intent/reserve/send/response và grant/ack/release. Mở lại cả hai DB, so owner, quyền gửi, current/history, usage/unknown/cap. Test stale owner quay lại và replay.
3. Thử nâng workspace M4, lỗi migration và kiểm integrity/backup. Chạy redaction sentinel qua store/IPC/DB/log/CLI/TUI. Kiểm cả cấu hình khác người dùng nếu phần chia profile ảnh hưởng.
4. Chạy toàn demo mục 2 và bài tự test mục 16.4 trên terminal thật. Ghi lệnh dựng fixture, target Windows, OS store test dùng key giả và các ca chưa xác minh.
5. Chạy V-00, V-12 nếu đổi dependency và checks Go nếu đụng phần chung. Live smoke chỉ theo cấu hình/phạm vi/ngân sách được giao riêng, ghi tách khỏi kết quả offline. B review tích hợp theo mục 15.

**Kiểm chứng.** P05-AC16, P05-TC16, V-07/V-10/V-11/V-00. B so quyền/tiền của cả hai task và chứng minh không double publication.

**Sáu trường giao việc**

| Trường | Nội dung cho P05-T16 |
|---|---|
| Đầu vào bắt buộc | [P05-T01](#p05-t01), [P05-T02](#p05-t02), [P05-T03](#p05-t03), [P05-T04](#p05-t04), [P05-T05](#p05-t05), [P05-T06](#p05-t06), [P05-T07](#p05-t07), [P05-T08](#p05-t08), [P05-T09](#p05-t09), [P05-T10](#p05-t10), [P05-T11](#p05-t11), [P05-T12](#p05-t12), [P05-T13](#p05-t13), [P05-T14](#p05-t14), [P05-T15](#p05-t15), M4 Đạt. Nhận candidate model/Coordinator/UI/CLI, clock/failpoints và OS profile riêng. Ghim candidate/report B Đạt, commit checkpoint, contract và hash tài liệu. Dependency ngoài phase phải đã công bố theo §10 và quy tắc chung; không nhận thay đổi đang review. |
| Hợp đồng dùng chung | Bằng chứng M5 đối chiếu quyền gửi, hai DB, giá/usage và checkpoint qua crash trên Windows. Owner đầu ra là A-P05-T16. Quyết định còn mở theo §6 phải được chốt trước consumer. |
| Phạm vi sở hữu | A-P05-T16 giữ App fault/integration tests, fixture, tài liệu dùng thật và hồ sơ M5. Phần chung do owners T01–T15 nhận findings của mình, A-T16 tích hợp. |
| Task có thể chạy cùng | Các task đủ dependency trong sổ điều phối và không có quan hệ trước/sau với task này. Ví dụ: [P06-T01](PHASE-06.md#p06-t01), [P06-T02](PHASE-06.md#p06-t02), [P06-T03](PHASE-06.md#p06-t03). Ví dụ không cấp quyền bắt đầu: phải đối chiếu inputs, quyền ghi, dữ liệu thử và giới hạn 2–3 chat. Nếu cùng file/hợp đồng thì người điều phối xếp nối tiếp. |
| Kiểm thử riêng | P05-AC16 / P05-TC16, gồm các ca chi tiết §13.3. V-07, V-10, V-11, V-00. Đối chiếu So trạng thái trước/sau và trace thật trên Windows. |
| Điểm tích hợp | [P05-CP09](#p05-cp09) — Nghiệm thu M5; A chat P05 nhận và ghép, B kiểm độc lập. Chạy toàn bộ AC/TC/V của task và ca giao tiếp trong checkpoint; kết quả ghi tại §16. Consumer ngoài phase: theo dependency của checkpoint/cổng trong bảng chung. Khi checkpoint Đạt: commit, push và cập nhật draft PR ngay. |


## 12. Tiêu chí nghiệm thu

| AC | Hành vi quan sát được | Owner | Bằng chứng |
|---|---|---|---|
| P05-AC01 | Đúng nguồn key và config hiệu lực. Bí mật chỉ dùng xác thực ở kết nối đã chọn. Đổi/xóa key giữ lịch sử và counters. | P05-T01 | P05-TC01 · V-01 |
| P05-AC02 | Stream và non-stream cùng hồ sơ chuẩn. Giữ lỗi, partial, usage/model unknown đúng trạng thái. | P05-T02 | P05-TC02 · V-02 |
| P05-AC03 | Request vừa input/output/context đáng tin. Giữ phần bắt buộc và protected inline. Model/điểm chia mới chỉ lập lại phần chưa gửi. | P05-T03 | P05-TC03 · V-03 |
| P05-AC04 | Giá/currency/FX/usage tính đúng và truy theo từng attempt. Không đếm trùng hoặc ghi unknown thành 0. | P05-T04 | P05-TC04 · V-04 |
| P05-AC05 | Guard chặn trước send theo tiền và cap riêng. Lần N được ghi, N+1 bị chặn. Unlimited giữ cap nếu bật. | P05-T05 | P05-TC05 · V-04 |
| P05-AC06 | Intent, reservation và checkpoint bền vững trước send. Send gap giữ chưa rõ và đúng khoản dự phòng. | P05-T06 | P05-TC06 · V-05 |
| P05-AC07 | Retry tạm thời có deadline/not-before/số lần đúng. Fatal/uncertain không tự retry. Pause/restart giữ thời gian. | P05-T07 | P05-TC07 · V-05 |
| P05-AC08 | Response cũ không overwrite current sau retry/handoff. Usage giữ đúng attempt và kết quả logic chỉ công bố một lần. | P05-T08 | P05-TC08 · V-05 |
| P05-AC09 | Một Coordinator owner mỗi profile. Client thiếu quyền, sai version hoặc stale không qua được IPC/lock. | P05-T09 | P05-TC09 · V-06 |
| P05-AC10 | Chỉ một tác vụ có quyền gửi. Queue và dedup đúng, B chưa đủ độc lập không được cấp. A chờ retry giữ quyền. | P05-T10 | P05-TC10 · V-06 |
| P05-AC11 | Handoff A→B chỉ sau stop/settle ở tool và xác nhận protocol. Stale A bị fence. Late usage thuộc A. | P05-T11 | P05-TC11 · V-06, V-07 |
| P05-AC12 | Resume đúng bước, dữ liệu, retry, tiền và quyền. Có artifact hợp lệ thì dùng lại. Paused/uncertain giữ trạng thái. | P05-T12 | P05-TC12 · V-07 |
| P05-AC13 | Tác vụ đang giữ quyền hiển thị đúng dù xem B. Hành động qua core. Chương · Block ID · nguyên nhân của Q05 đúng. Chỉnh hợp lệ mới re-plan, giữ protected inline. | P05-T13 | P05-TC13 · V-08, V-11 |
| P05-AC14 | Key được che. Config nguồn hiệu lực rõ. Đổi mặc định áp task mới. Mở màn/lưu key chưa gọi trả phí. | P05-T14 | P05-TC14 · V-08, V-11 |
| P05-AC15 | CLI dùng cùng guard. JSON/exit đúng và đã lọc. Q05 bị chặn có vị trí + next_action và exit 7. Status đọc thành công trả 0. | P05-T15 | P05-TC15 · V-09 |
| P05-AC16 | Tích hợp nhiều process không hai permit hợp lệ, không double publication, giữ usage/unknown/cap qua recovery. | P05-T16 | P05-TC16 · V-07, V-10, V-11, V-00 |

A nối từng AC với ca test và bằng chứng quan sát. B kiểm request thực tới transport, thứ tự ghi và dữ liệu sau reopen. Kết quả UI là một phần của bằng chứng.

## 13. Ma trận kiểm thử

| Ca / AC | Fixture / tình huống | Kỳ vọng cần kiểm chứng | Cách chạy |
|---|---|---|---|
| P05-TC01 / P05-AC01 | OS store/env giả, sentinel và hai task snapshot | Đúng ref/key được chọn, không leak trong DB/log/IPC/UI. | V-01 |
| P05-TC02 / P05-AC02 | Transcript tổng hợp, cut và lỗi transport | Không bịa field hoặc dùng response dở làm hoàn tất. | V-02 |
| P05-TC03 / P05-AC03 | Exact/estimate, margins, reasoning và boundary | Oracle token, diagnostics và request không gửi ở ca bị chặn. | V-03 |
| P05-TC04 / P05-AC04 | Decimal, cached/reasoning, actual model, price snapshots | Số tiền và adjustment đúng oracle, lịch sử còn nguyên. | V-04 |
| P05-TC05 / P05-AC05 | Ngưỡng budget, cap N, pending/uncertain | So tổng tiền, slot và số transport thực nhận. | V-04 |
| P05-TC06 / P05-AC06 | Failpoints intent/reserve/send/response, replay | So markers, số request và ledger khi mở lại. | V-05 |
| P05-TC07 / P05-AC07 | Fake clock, delay hint, timeout và auth | Số lần gửi, model, điều kiện và thời điểm đúng. | V-05 |
| P05-TC08 / P05-AC08 | Hai attempts, đảo thứ tự, stale và duplicate | So quyền hiện hành, history, tiền và generation. | V-05 |
| P05-TC09 / P05-AC09 | Nhiều process/profile, ACL, writer lease | Chỉ owner hợp lệ, lỗi hữu hạn và không bypass. | V-06 |
| P05-TC10 / P05-AC10 | Hai workspace/clients, same task và dependency | Trace không có hai grant hợp lệ hoặc hai admission trái phép. | V-06 |
| P05-TC11 / P05-AC11 | Grant/ack/release, hai DB và process crash | Đối chiếu operation/sequence, quyền gửi và attempt ledger. | V-06, V-07 |
| P05-TC12 / P05-AC12 | Workspace M4, checkpoints, pending Skill và version lỗi | Readback giữ state, không resend hoặc reset. | V-07 |
| P05-TC13 / P05-AC13 | A active/B queued, retry/handoff, block không chia an toàn | UI state và request trace đúng. Block lỗi chưa gửi. | V-08, V-11 |
| P05-TC14 / P05-AC14 | Hai task, store sentinel và input focus | Không leak/không send, giữ snapshot và counters cũ. | V-08, V-11 |
| P05-TC15 / P05-AC15 | Process harness, key thiếu, budget/cap, queue/pause và Q05 | Output/exit khớp DB và transport. Không đợi stdin. | V-09 |
| P05-TC16 / P05-AC16 | Hai DB, OS lock thật, migration, failpoints và terminal | So trạng thái trước/sau và trace thật trên Windows. | V-07, V-10, V-11, V-00 |

### 13.1. Lệnh và môi trường

Tên package/test là mục tiêu của task. Cập nhật nếu đổi theo code. Exit 0 với 0 ca liên quan chưa chứng minh AC.

| ID | Lệnh hoặc cách kiểm tra |
|---|---|
| V-01 | `cargo test -p nt-model --test credential_contract --locked` |
| V-02 | `cargo test -p nt-model --test provider_contract --locked` |
| V-03 | `cargo test -p nt-application --test token_planning --locked` |
| V-04 | `cargo test -p nt-application --test budget_accounting --locked` |
| V-05 | `cargo test -p nt-application --test attempt_lifecycle --locked` |
| V-06 | `cargo test -p nt-coordinator --test multiprocess_permit --locked` |
| V-07 | `cargo test -p novel-translate --test phase05_recovery --locked` |
| V-08 | `cargo test -p nt-tui --test tasks_settings --locked` |
| V-09 | `cargo test -p novel-translate --test execution_cli --locked` |
| V-10 | `cargo test -p novel-translate --test phase05_coordination --locked` |
| V-11 | Kiểm Tasks/Settings và demo M5 trên terminal Windows thật, làm theo [mục 16.4](#p05-user-test). Kiểm OS store với giá trị giả và entry riêng của bài thử. |
| V-12 | Khi thêm/đổi dependency, kiểm MSRV theo ADR-001. Với mốc 1.88.0 dùng `cargo +1.88.0 test --workspace --all-features --locked`. |

**V-00 — Bộ kiểm tra Rust trước PR và trước đóng M5.**

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

Chuẩn bị dependency/toolchain trước rồi chạy offline. Test dùng fake transport, fake clock, key tổng hợp và DB/hồ sơ tạm. Cô lập mạng ngoài của app, giữ IPC local cần cho test. V-06/V-10 kiểm nhiều process và OS lock thật. Live smoke nếu có chỉ chạy thủ công theo phạm vi/kết nối/ngân sách đã giao, ghi tách.

### 13.2. Cách dùng bằng chứng

- **Tiền và giới hạn.** Dùng oracle độc lập cho nhóm token, decimal, FX, unknown và ngưỡng. Ghi giá/usage fixture, không lấy bảng giá hiện hành làm oracle.
- **Lần gửi và quyền.** Lưu trace gồm logical request, attempt, workspace, permit sequence, mốc ACK và transport receipt. Kiểm cả request được gửi và request phải vắng mặt.
- **Fault.** Child ghi marker tại điểm ngắt. Parent so cả Series DB, Coordinator DB, checkpoint và history trước/sau. Dùng barrier và clock kiểm soát được. Giữ log đã lọc.
- **Thủ công.** Ghi terminal/Windows, cấu hình test và kết quả. Chỉ xóa entry credential tạo riêng cho bài thử khi dọn. Thiếu ca bắt buộc giữ chưa xác minh.

### 13.3. Chi tiết ca thử theo task

Hậu tố phân rã từng nhóm TC. Một ca có thể chạy nhiều biến thể. Nối từng ca với test hoặc kịch bản thực ở handoff.

| Ca | Thao tác trên fixture | Kết quả phải đối chiếu |
|---|---|---|
| P05-TC01a | Chọn kho OS dù env khác có sẵn. Lặp với env được chỉ rõ. | Chỉ nguồn được chọn. Secret đến đúng lớp xác thực. |
| P05-TC01b | Biến thiếu tại process gửi, store lỗi hoặc key rỗng. | Báo đúng kết nối/cần xử lý, không send hoặc fallback. |
| P05-TC01c | Thay/xóa key khi một attempt đã gửi và còn request chưa gửi. | Attempt cũ giữ refs/history. Lần sau dùng nguồn mới hoặc báo thiếu. |
| P05-TC01d | Dùng sentinel trong input, lỗi và fake store. So DB, IPC, prompt, logs, JSON. | Không lộ giá trị. Trạng thái đã lưu tách với kết nối đã kiểm. |
| P05-TC02a | Cùng fixture ở non-stream và stream có nhiều event. | Cùng normalized result, ID, nội dung, usage và lý do kết thúc. |
| P05-TC02b | Provider không trả model thực tế/usage hoặc nhóm token bị thiếu. | Fields giữ unknown đúng, không suy từ alias hoặc giá trị 0. |
| P05-TC02c | Cắt stream giữa chừng, auth, transient có hint và lỗi chưa rõ outcome. | Partial tách riêng, phân loại lỗi/hint đúng, không tự publish/retry. |
| P05-TC02d | Thử đích ngoài policy/redirect và payload có lời yêu cầu gửi key. | Guard giữ bí mật ở endpoint đã chọn. Fake không nhận request trái phép. |
| P05-TC03a | Exact tokenizer và estimate với payload có Skill/context/schema/bao. | Breakdown khớp oracle. Hiện nhãn ước lượng và biên 5%/10% phù hợp. |
| P05-TC03b | Vượt riêng trần input, output hoặc tổng context, reasoning đã nằm trong output. | Đúng lý do chặn và reasoning tính một lần. |
| P05-TC03c | Thiếu capability tin cậy, block không có điểm ngắt an toàn. | Chặn trước send, giữ vị trí/lý do và protected inline. |
| P05-TC03d | Đổi model do người dùng sau một lượt đạt, test 3.000 token thử nghiệm. | Chỉ lập lại phần chưa gửi. Giữ units, revisions và số đã dùng. |
| P05-TC04a | Tính giá fixture có số lẻ, scale khác và FX nhập rõ nguồn/ngày. | Oracle decimal, hướng quy đổi và rounding đúng. |
| P05-TC04b | Usage có cached/reasoning với provider tính trong nhóm tổng. | Mỗi nhóm tính đúng một lần, phí công cụ tách dòng. |
| P05-TC04c | Bảng giá đổi hoặc actual model khác alias. | Ghim price snapshot và lưu điều chỉnh có căn cứ, lịch sử không bị sửa âm thầm. |
| P05-TC04d | Thiếu usage/giá/FX, replay usage của attempt cũ. | Unknown giữ riêng, không 0. Replay không cộng trùng. |
| P05-TC05a | So tổng đã ghi + đang dự phòng + lần tới ngay dưới/bằng/trên limit. | Cho khi đủ, chặn khi vượt, đúng currency và lý do. |
| P05-TC05b | Gửi lần N rồi thử N+1 khi cap N, cả limited và unlimited. | Lần N giữ record. N+1 không tới transport nếu cap bật. |
| P05-TC05c | Giữ attempt chưa rõ cùng reservation/slot, cập nhật khi có usage. | Giữ khoản đó trong tổng dự phòng tới khi đối chiếu. Usage đã ghi và khoản dự phòng tương ứng không bị tính hai lần. |
| P05-TC05d | Config sai hoặc đổi limit/mode, sang chương, đổi model rồi mở lại. | Không tự unlimited. Giữ counters, dự phòng và history. |
| P05-TC06a | Ngắt trước commit intent/reserve. | Chưa có request tới transport. Không có record ghi nửa. |
| P05-TC06b | Ngắt sau intent đã lưu và chưa xác định send. | Reopen giữ Chưa rõ đã xử lý, reservation và slot. |
| P05-TC06c | Ngắt sau transport nhận, trước response/usage bền vững. | Có đúng receipt của attempt, không tự resend khi chưa rõ. |
| P05-TC06d | Ngắt sau lưu đủ response nhưng trước validate/ACK. | Dùng lại artifact cùng ID/hash, không gửi thêm. |
| P05-TC06e | Replay cùng operation ID và thử payload khác. | Không tạo thêm intent/reservation/send. Payload sai bị từ chối. |
| P05-TC06f | Thu quyền hoặc stop giữa preflight và transport admission. | Guard xác định mốc thực. Stale owner không có receipt gửi. |
| P05-TC07a | Lỗi tạm thời hai lần với fake clock rồi lần nữa lỗi. | Retry cùng model theo 10/30 giây, tối đa hai lần. |
| P05-TC07b | Provider hint lâu hơn. Restart trước thời điểm cho phép. | Giữ not-before còn lại, chưa gửi sớm. |
| P05-TC07c | Auth/fatal/unknown và hết deadline 10 phút chưa rõ outcome. | Dừng đúng lý do, không tự retry hoặc đổi model. |
| P05-TC07d | Pause đang chờ, hoặc nhận đủ response trước retry. | Chưa gửi mới khi pause. Response hợp lệ được dùng trước retry. |
| P05-TC08a | Response đầu đầy đủ về trước retry thực sự gửi. | Dùng response cũ còn đúng căn cứ. Không thêm attempt gửi. |
| P05-TC08b | Retry đã gửi, nhận response cũ rồi mới nhận response hiện hành. | Response cũ vào history/usage. Chỉ attempt hiện hành có quyền publish. |
| P05-TC08c | Đảo thứ tự hai responses và replay một response. | Một current result. Usage vẫn đủ và không trùng. |
| P05-TC08d | Đổi snapshot/base/Skill giữa gửi và commit. | Stale giữ history, không ghi đè revision mới. |
| P05-TC08e | A bàn giao rồi late response về khi B đang chạy. | Usage còn ở A, owner B và current của cả hai giữ đúng. |
| P05-TC08f | Commit xong nhưng mất ACK. Replay operation có cùng và khác payload. | Tra kết quả cũ, không double publication. Payload khác báo xung đột. |
| P05-TC09a | Hai process khởi động cùng profile bằng barrier. | Chỉ một OS lock/owner hợp lệ, process còn lại nối qua IPC. |
| P05-TC09b | Sai profile/chủ thể, version lạ, frame quá dài hoặc endpoint stale. | Từ chối đúng lý do và không cấp quyền. |
| P05-TC09c | Giữ workspace writer lease rồi thử ghi từ process khác. | Đường đọc/ghi theo policy. Writer trái quyền bị chặn. |
| P05-TC09d | Owner chết rồi process mới lấy lock. | Reconcile trước grant. Server/ACK cũ không dùng được. |
| P05-TC10a | Hai client enqueue cùng task và hai tasks hợp lệ. | Dedup đúng, giữ thứ tự và một grant hiệu lực. |
| P05-TC10b | A chuyển role hoặc chờ retry, B xếp hàng. | A giữ permit. B không có receipt gửi. |
| P05-TC10c | B phụ thuộc A hoặc thiếu lease/config, thử chọn chạy thay. | B chưa được cấp và hiện lý do. |
| P05-TC10d | A hoàn tất, B còn hợp lệ rồi client replay claim. | Một grant cho B đúng sequence. Replay không nhân bản. |
| P05-TC11a | Pause A khi transport đang xử lý, người dùng chọn B độc lập. | B chưa gửi trước stop/settle và xác nhận handoff hợp lệ. |
| P05-TC11b | Chọn B có dependency chưa đủ hoặc cùng tài nguyên bị khóa. | Giữ chờ và lý do. Không lách điều kiện. |
| P05-TC11c | Ngắt sau từng ghi grant/ack/release rồi mở lại hai DB. | Đối chiếu đúng cặp mốc, không có hai grant hợp lệ. |
| P05-TC11d | ACK cũ đến sau khi owner/sequence đã đổi rồi trở lại cùng task. | Fence từ chối theo sequence/operation, không dựa tên task. |
| P05-TC11e | Late response A sau khi B nhận quyền. | Ghi usage A, không đổi current hoặc quyền B. |
| P05-TC11f | Hai DB thiếu một phía/record và replay cùng operation. | Giữ chờ khi chưa đối chiếu. Hoàn tất đúng một lần khi có bằng chứng. |
| P05-TC12a | Nâng workspace M4 và kiểm version/checkpoint/integrity. | Đúng source/revision, ledger mới có căn cứ, bản cũ giữ khi lỗi. |
| P05-TC12b | Reopen ở response đã lưu, nháp đã commit và local Review đã lưu. | Tiếp đúng bước, không gửi lại phần đã có. |
| P05-TC12c | Reopen khi paused, uncertain hoặc retry chưa tới giờ. | Giữ intent, khoản dự phòng và not-before. Không tự gửi. |
| P05-TC12d | Source/Skill/base đổi, pending file hoặc ref checkpoint hỏng. | Chẩn đoán và lập lại phần thích hợp. Không mất history/counters. |
| P05-TC13a | A active, mở B và đổi màn/resize. | Nhãn toàn app vẫn A, chi tiết đúng task và focus. |
| P05-TC13b | A retry wait rồi pause/handoff, late usage về sau. | Hiện đúng retry/quyền/unknown. Hành động qua core. |
| P05-TC13c | Block quá dài không chia an toàn. | Hiện đúng Chương · Block ID · nguyên nhân. Không gọi fake. |
| P05-TC13d | Thử điểm chia/giới hạn sai rồi hợp lệ ngay màn. | Sai giữ blocked. Hợp lệ re-plan đúng qua core và giữ guard. |
| P05-TC14a | Nhập key thử, mở/cập nhật/xóa connection và gặp store lỗi. | Mask đúng, state/lỗi rõ, sentinel không lộ. |
| P05-TC14b | Chọn roles, Research dùng Review hoặc cấu hình riêng. | Nguồn hiệu lực rõ. Mở/chọn/lưu chưa tạo request trả phí. |
| P05-TC14c | Sửa default tool/bộ khi task A đã có, tạo task B. | A giữ config/counters, B nhận đúng nguồn mới. |
| P05-TC14d | Thiếu capability hoặc hạn mức sai và thao tác trong form. | Báo chỗ sửa, không đổi ngầm sang unlimited hoặc kích hoạt phía sau. |
| P05-TC15a | Chạy CLI status khi task queued, waiting hoặc paused. | Exit 0 khi đọc thành công, JSON có trạng thái thực và không gửi. |
| P05-TC15b | Thử lệnh hoàn tất, lỗi flags/config, vận hành, chờ duyệt, hạn mức/cap và xếp hàng. | Exit 0/2/3/4/5/6 tương ứng, một envelope và lý do đúng. |
| P05-TC15c | Non-interactive với block không chia an toàn. | Exit 7, issue có vị trí và next_action. Không prompt hoặc gửi. |
| P05-TC15d | Lặp start/resume từ CLI trong khi TUI cùng hồ sơ giữ quyền. Test key thiếu. | Cùng guards/dedup với core. Output không lộ secret. |
| P05-TC16a | Hai process và hai workspace thực trên Windows, cùng profile rồi tách profile. | OS lock/grant/trace đúng từng profile. Không hai quyền hợp lệ trong cùng profile. |
| P05-TC16b | Ngắt quanh intent/reserve/send/response và mốc commit. | Readback đúng attempt, unknown và current. Không double send do đoán hoặc double publication. |
| P05-TC16c | Ngắt quanh grant/ack/release và để stale owner quay lại. | Reconcile đúng hai DB, chỉ quyền có hiệu lực gửi được. |
| P05-TC16d | Chạy migration M4 và tiêm lỗi trong bản sao. | Integrity, version, refs và backup đúng. Giữ dữ liệu và tiền trước đó. |
| P05-TC16e | Sentinel ở OS store thử, transport/error và IPC/DB/CLI/TUI. | Key chỉ xuất hiện ở chỗ xác thực được phép, không lưu/hiện ngoài scope. |
| P05-TC16f | Chạy V-00/V-11 và bộ test process trên candidate cuối. | Có kết quả môi trường thật và demo offline. Live smoke nếu được giao có báo cáo riêng. |

## 14. Rà soát chất lượng code

Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md). A tự kiểm, tạo candidate/baseline và bàn giao. B đọc AC trước, tự kiểm hành vi và lỗi trên đúng candidate. Khi code, config, hợp đồng hoặc fixture ảnh hưởng kết quả đổi, tạo lượt mới.

| Phần | B cần đối chiếu |
|---|---|
| T01–T03 | Nguồn key, endpoint, payload và log đã lọc. Stream/partial/unknown đúng. Tokenizer/capability có provenance và chặn đúng giới hạn. |
| T04–T05 | Số tiền độc lập theo từng nhóm usage, price/FX snapshot, unknown và cap. Đối chiếu request thật khi dưới/bằng/trên ngưỡng. |
| T06–T08 | Intent và reservation có trước transport. Retry đúng thời điểm và model. Kết quả cũ giữ usage, fence giữ một lần công bố. |
| T09–T12 | Hai process/OS lock thật, IPC auth, queue, một permit. Kiểm mốc stop/settle/grant, cả hai DB, stale owner và reopen. |
| T13–T16 | UI/CLI cùng guard. Q05 đúng vị trí/exit và không gửi. Thử migration, terminal và redaction trên candidate cuối. |

B ghi ca đã tự chạy, bằng chứng đã đối chiếu và phần chưa xác minh. Với fault hoặc race, đọc marker/transport receipts và DB sau reopen. So ca âm bằng fixture có thể phát hiện sai phiên bản, quyền, tiền hoặc secret.

A commit phần đã được B duyệt theo [Git workflow](../../../AGENTS.md#git-workflow). Khi phạm vi PR đủ kiểm tra và review tích hợp, push nhánh theo phần việc rồi mở/cập nhật Draft PR vào `rewrite/rust`. Mô tả từng thay đổi và kiểm tra bằng tiếng Anh đơn giản. Bằng chứng, giá thử riêng và bí mật giữ ở máy người dùng.

## 15. Điều kiện đóng phase

- [ ] T01–T16 đủ AC, test liên quan và D01–D08 đã chốt theo consumer. Review B **Đạt** còn hiệu lực.
- [ ] B review **Đạt** bản tích hợp theo [mẫu review phase](../templates/PHASE_REVIEW.md). Candidate định danh code, tests, config/dependencies, spec và fixture. Ca bị ảnh hưởng đã được kiểm lại.
- [ ] Gateway/credential đã kiểm ở phạm vi Windows. Giá/usage/FX, hạn mức, call cap, retry và late response có oracle và ca âm. Unknown giữ đúng cả tiền lẫn số lượt.
- [ ] Nhiều process dùng IPC/OS lock thật. Fault campaign qua intent/send/response và grant/ack/release có marker, hai DB và transport receipts. Không có hai quyền hợp lệ hoặc double publication.
- [ ] UI-06/07, CLI và bài tự test chạy trên candidate tích hợp. Nhánh Q05 đúng Chương · Block ID · nguyên nhân, re-plan qua core và CLI có next_action/exit 7.
- [ ] Migration từ M4, replay và recovery giữ history/current/ledger. V-00 và checks mục 13 đạt. V-12 chạy khi có đổi dependency. Phần Go bị ảnh hưởng đã được kiểm nếu đụng cấu hình chung.
- [ ] README theo lệnh/nền tảng thực. Báo cáo ghi rõ offline, OS store thử, live smoke nếu đã thực hiện và giới hạn còn lại. Bảo vệ key, dữ liệu mẫu/riêng và output.
- [ ] Bàn giao đủ cho Phase 06, gồm cửa vào dispatcher, checkpoints/permit và trạng thái chờ. Thay Go tiếp theo [lộ trình chuyển đổi](PHASE-03.md#p03-go-retirement) và phạm vi đã có bằng chứng.

## 16. Bàn giao và bằng chứng

A dùng [mẫu handoff task](../templates/TASK_HANDOFF.md). B dùng [mẫu review task](../templates/TASK_REVIEW.md) và [mẫu review phase](../templates/PHASE_REVIEW.md) cho M5. Lưu report/version ở `docs/roadmap/evidence/phase-05/`, ví dụ `P05-T01.handoff-r01.md`, `P05-T01.candidate-r01.sha256` và `P05-T01.review-r01.md`. Giữ các lượt và finding IDs.

A phase cập nhật sổ sau handoff/verdict; điều phối cập nhật PLAN trung tâm từ báo cáo đó. B chỉ ghi báo cáo độc lập. Phần thiếu kiểm tra giữ đúng trạng thái chưa xác minh.

### 16.1. Sổ trạng thái ban đầu

Theo dõi đủ 9 checkpoint của lịch v2: [P05-CP01](#p05-cp01), [P05-CP02](#p05-cp02), [P05-CP03](#p05-cp03), [P05-CP04](#p05-cp04), [P05-CP05](#p05-cp05), [P05-CP06](#p05-cp06), [P05-CP07](#p05-cp07), [P05-CP08](#p05-cp08), [P05-CP09](#p05-cp09); xem [§10.2](#p05-checkpoints). Tất cả đang Chưa mở. Khi thực hiện ghi task/candidate, B review, commit, remote SHA, PR và trạng thái Git theo [sổ checkpoint](../PARALLEL_EXECUTION.md#register).

| Gói | Task / cổng | Trạng thái | Candidate / báo cáo A–B |
|---|---|---|---|
| P05-A | [T01](#p05-t01) | Chưa bắt đầu | — |
| P05-A | [T02](#p05-t02) | Chưa bắt đầu | — |
| P05-A | [T03](#p05-t03) | Chưa bắt đầu | — |
| P05-A | [T04](#p05-t04) | Chưa bắt đầu | — |
| P05-A | [T05](#p05-t05) | Chưa bắt đầu | — |
| P05-B | [T06](#p05-t06) | Chưa bắt đầu | — |
| P05-B | [T07](#p05-t07) | Chưa bắt đầu | — |
| P05-B | [T08](#p05-t08) | Chưa bắt đầu | — |
| P05-C | [T09](#p05-t09) | Chưa bắt đầu | — |
| P05-C | [T10](#p05-t10) | Chưa bắt đầu | — |
| P05-C | [T11](#p05-t11) | Chưa bắt đầu | — |
| P05-D | [T12](#p05-t12) | Chưa bắt đầu | — |
| P05-D | [T13](#p05-t13) | Chưa bắt đầu | — |
| P05-D | [T14](#p05-t14) | Chưa bắt đầu | — |
| P05-D | [T15](#p05-t15) | Chưa bắt đầu | — |
| P05-E | [T16](#p05-t16) | Chưa bắt đầu | — |
| P05-E | M5 | Chưa bắt đầu | — |

### 16.2. Báo cáo mỗi task

1. Ghi task/AC, phạm vi, hành vi và ADR/version áp dụng. Nêu baseline và phần ngoài phạm vi đã có.
2. Định danh candidate bằng commit nếu có, diff và hashes của files, kể cả untracked/config/dependency/fixture/spec.
3. Nối từng AC với test/ca thủ công, command, OS/toolchain, số ca và kết quả. Ghi thao tác đã gửi và bị chặn, marker, readback hai DB và tiền/quyền đã lọc thông tin.
4. Ghi finding, giới hạn, ca chưa chạy và việc tiếp theo. Live run nếu có ghi model, phạm vi, giới hạn được giao và kết quả riêng. Tới đúng mốc Git thì ghi commit/PR.

### 16.3. Đầu ra cho bước tiếp theo

| Nơi nhận | Nội dung bàn giao |
|---|---|
| [Phase 06](PHASE-06.md) | Dispatcher, gateway/profiles, budget/usage, quyền chạy, checkpoint/retry/late result và waits. Research và repair dùng cùng guard. Phí công cụ có dòng riêng. |
| [Phase 08](PHASE-08.md) | UI-06/07 read models, nguồn config, lệnh CLI/exit/JSON và fixture multi-process để kiểm luồng qua màn. |
| [Phase 09](PHASE-09.md) | Fault matrix, supported targets, migration, provider contract evidence, kết quả live nếu có và phần còn chờ. Giữ scope thay Go theo evidence. |

Khi P04-CP01 và P03-CP01 đã Đạt/công bố và PLAN giao Phase 05, bắt đầu ở [P05-T01](#p05-t01). Đọc gateway/revision đã nhận, chốt D01 và kiểm V-01 với nguồn key giả trước khi giao B. Chuyển task phụ thuộc sau verdict còn hiệu lực.

<a id="p05-user-test"></a>

### 16.4. Người dùng tự test

#### 16.4.1. Bản cần thử

Thử candidate **M5** với hồ sơ thử riêng và model giả lập. Phạm vi là cấu hình, hàng đợi, tạm dừng/chuyển quyền chạy, giới hạn và mở lại tác vụ. Khi bàn giao, A ghi candidate, lệnh khởi động hồ sơ thử và cách chọn từng kịch bản. B chạy lại bằng cùng bộ mẫu.

#### 16.4.2. Chuẩn bị

Dành khoảng 15–20 phút. Chuẩn bị hai workspace mẫu với hai tác vụ A và B thuộc cùng hồ sơ thử. Bộ mẫu cần có kịch bản giữ A đang chạy, kịch bản chạm giới hạn và kịch bản timeout chưa rõ usage. Dùng cấu hình giả và lệnh chạy được ghi trong gói M5.

#### 16.4.3. Mở đúng bản

Chạy bản Rust và bộ giả lập theo lệnh của M5. Mở **Cấu hình, báo cáo và trợ giúp** để đối chiếu model/vai trò giả lập, rồi mở **Tác vụ**. Ghi lại ID của A và B từ bộ mẫu hoặc kết quả `status`.

#### 16.4.4. Thử luồng chính

| Ca | Thao tác | Thấy gì thì đạt |
|---|---|---|
| P05-UT01 — Xem cấu hình và tác vụ | Mở cấu hình mẫu, xem model đã chọn rồi mở chi tiết tác vụ A. | Giá trị kết nối nhạy cảm được che. Chi tiết tác vụ phân biệt model được chọn và model thực tế khi đã có dữ liệu. Chỉ mở màn hình thì bộ giả lập chưa nhận lượt gửi mới. |
| P05-UT02 — Một tác vụ chạy, một tác vụ chờ | Chạy kịch bản A và B cùng đủ điều kiện. Xem A rồi chuyển sang xem B. | A giữ quyền chạy, B ở hàng đợi. Việc xem B giữ nguyên trạng thái của cả hai. |
| P05-UT03 — Tạm dừng và bàn giao | Yêu cầu tạm dừng A. Đợi app báo A đã bàn giao, sau đó chọn B để chạy độc lập. | A ở trạng thái tạm dừng và B được chạy sau thao tác xác nhận. Nếu phản hồi cũ của A đến trong kịch bản, kết quả của B vẫn gắn đúng B. |

#### 16.4.5. Thử ca lỗi và mở lại

**P05-UT04 — Bị chặn bởi giới hạn hoặc chưa rõ kết quả.** Chạy hai tình huống đã chuẩn bị trong bộ mẫu. Với giới hạn không đủ, app báo đúng lý do và bộ giả lập ghi nhận không có lượt gửi mới. Với timeout chưa rõ usage, chi phí hiển thị chưa xác định và khoản dự phòng còn giữ. Đóng rồi mở lại hồ sơ thử theo hướng dẫn, xem cùng ID và đối chiếu trạng thái. Số lượt gửi giữ nguyên trong lúc chỉ mở lại và xem.

#### 16.4.6. Ghi kết quả

Ghi **Đạt**, **Không đạt** hoặc **Chưa thử** theo từng mã ca. Khi lỗi, kèm candidate M5, ID tác vụ, kịch bản, bước và trạng thái mong đợi/thực tế. Dùng output từ hồ sơ thử đã lọc thông tin kết nối.

#### 16.4.7. Thử lại từ đầu

Thoát các phiên của hồ sơ thử rồi dùng lệnh tạo lại hồ sơ mẫu trong gói M5. Lấy ID tác vụ mới từ kết quả tạo mẫu trước khi lặp lại.

### 16.5. Giao kết quả giữa các chat

A chat phase ghi báo cáo task và checkpoint riêng ở evidence/phase-05. Người điều phối là đầu mối duy nhất cập nhật PLAN chung; A gửi đường dẫn report, candidate, trạng thái, commit/remote SHA và PR. B chỉ viết report của mình. Khi một task chờ đầu vào hoặc review, chọn task độc lập đủ điều kiện trong phase; hết task sẵn sàng thì báo đúng dependency còn thiếu, không tự mở việc ngoài phạm vi. Công bố checkpoint đủ điều kiện ngay theo §10.2.

### 16.6. Rà và bỏ Go dần

Owner P05 rà **Kết nối, adapter model, chi phí, hàng đợi và recovery** tại các mốc **P05-CP06/CP07/CP08; kiểm M5 và consumer P06**, theo [lộ trình dọn Go trong PLAN](../../../PLAN.md#go-retirement-plan). Dọn phần hết caller sau khi đường Rust tương ứng, guard tiền/quyền chạy, retry và khôi phục đã được review; giữ phần executor còn phục vụ workflow Go. Thử model thật bắt buộc của scope phải có bằng chứng theo ngân sách đã giao, không suy từ fake test.

Mỗi checkpoint ghi phần có thể dọn, phần còn caller/thiếu bằng chứng, task xử lý và mốc rà lại. Khi đủ điều kiện, điều phối cấp allowlist/owner Go, đăng ký task và CP dọn riêng; kiểm và B review candidate sau dọn rồi commit/push ngay. Không chờ lượt dọn để push CP chức năng đã Đạt. Bằng chứng Go và artifact còn lại chuyển vào sổ điều phối; không tự tuyên bố toàn dự án chỉ Rust khi GO-FINAL chưa Đạt.
