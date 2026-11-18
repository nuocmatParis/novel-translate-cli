# Mẫu bàn giao task — model A

**Đây là mẫu, chưa phải báo cáo thực thi hoặc bằng chứng đạt.** Tạo bản theo từng lượt trong thư mục evidence của phase. Áp dụng [quy trình hai model](../REVIEW_WORKFLOW.md).

## 1. Nhận diện và phạm vi

| Trường | Giá trị cần ghi |
|---|---|
| Task / nhóm / cổng | ID và danh sách task; với P00-T10 ghi rõ M0-01 hay M0-02 |
| Model A / phiên / ngày | Theo thông tin thực tế; không đoán phiên bản |
| Lượt bàn giao / candidate ID | r01 hoặc lượt tiếp theo |
| Trạng thái gửi B | Chờ review; liệt kê ngay kiểm tra chưa hoàn tất nếu có |
| Phạm vi được giao / phần chưa làm | Hành vi và giới hạn cụ thể |
| Dependency | Task ngoài nhóm đã đạt và link review; thứ tự nội bộ nhóm nếu có |
| Căn cứ | Link task/spec/ADR, revision/hash áp dụng, AC nhận từ PLAN |

## 2. Thay đổi và tác động

| File / thành phần | Hành vi trước → sau | Lý do / AC | Contract, migration, compatibility |
|---|---|---|---|
| Điền khi thực hiện | | | |

Ghi riêng thay đổi đã có trước task, file mới/untracked, thay đổi ngoài dự kiến và lý do. Liệt kê lỗi đã biết, giới hạn và yêu cầu còn mơ hồ.

## 3. Bản gốc và bản bàn giao

- Baseline commit hoặc nội dung nền đã ghi nhận:
- Candidate commit nếu có; nếu chưa commit, danh sách file/trạng thái/SHA-256 và nơi đọc diff/file mới:
- Source/test/manifest/lock/config/consumer liên quan được đóng băng:
- Phiên bản/hash tài liệu nguồn và task áp dụng:
- OS, target, toolchain, terminal/tool và fixture/config dùng kiểm tra:
- Cách B tái lập đúng bản này; giới hạn nếu chưa có baseline đầy đủ:
- Thành phần sinh tạm được loại khỏi manifest (log, báo cáo, build output):

Không chỉ ghi tên branch hoặc một commit khi vẫn còn thay đổi chưa commit. Không chép key, sách/workspace thật hoặc tài liệu nội bộ vào Git để phục vụ review.

## 4. Đối chiếu nghiệm thu của A

| Task / AC | Kỳ vọng từ yêu cầu | Test / cách kiểm | Kết quả A quan sát | Link bằng chứng / phần chưa xác minh |
|---|---|---|---|---|
| Điền đủ từng AC trong phạm vi | | | | |

Đây là tự đánh giá của A, chưa thay kết luận độc lập của B. Task tài liệu ghi phép kiểm nguồn/link/trace/contract phù hợp.

## 5. Kiểm tra thực chạy

| Lệnh / thao tác | Candidate / môi trường | Tên hoặc số ca liên quan | Kết quả / exit | Log / artifact |
|---|---|---|---|---|
| Chỉ điền phép kiểm thực sự chạy | | | | |

- Test thủ công đã làm, OS/terminal/reader và kết quả:
- Test chưa chạy hoặc thất bại, nguyên nhân và tác động tới AC:
- Kiểm tra Go bị ảnh hưởng, hoặc lý do không liên quan:
- Kết quả tự review: diff, ranh giới, lỗi/cleanup, side effect, bảo mật, chất lượng test:

## 6. Trả lời findings của lượt trước

Lượt đầu ghi chưa có review trước. Từ lượt sửa, giữ ID phát hiện và trả lời từng mục:

| Finding ID | Phản hồi A: đã sửa / đề nghị bác bỏ / cần làm rõ | Thay đổi / căn cứ | Regression test và kết quả | Phần B cần kiểm lại |
|---|---|---|---|---|
| Điền khi có finding | | | | |

A không tự ghi finding đã được B đóng. Link báo cáo/candidate lượt trước; nêu phần thay đổi kể từ lần B kiểm tra gần nhất.

## 7. Điểm bàn giao

- Link báo cáo này và candidate đã được ghi trong PLAN:
- Đường dẫn báo cáo B dự kiến cho lượt này; ghi chưa có kết quả đến khi B thực sự review:
- Vai tiếp theo: B review; task/nhóm và phạm vi giữ nguyên:
- Phần độc lập A có thể làm tiếp mà không đổi candidate đang review, nếu đã được giao:
- Điều kiện còn thiếu trước khi task có thể hoàn thành:

## Phối hợp nhiều chat và công bố checkpoint

- Chat phase / model A / reviewer model, effort, session thực tế / điều phối:
- Worktree, nhánh và vùng dữ liệu/build tách biệt:
- Dependency CP + commit/remote SHA + report B; contract/spec/ADR version/hash:
- File/module được sửa, shared owner và migration IDs được cấp:
- Checkpoint, consumer ngoài phase và ca giao tiếp đã/chưa kiểm:
- Candidate ổn định đầu/cuối review; thay baseline/contract cần review lại:
- A gửi report về điều phối để cập nhật PLAN trung tâm; B không sửa trạng thái:
- Sau B Đạt và đủ checks: A commit/push CP ngay, ghi remote SHA/draft PR; nội bộ docs-only ghi không áp dụng.

## Go được giữ hoặc bỏ trong scope

Theo [PLAN §8](../../../PLAN.md#go-retirement-plan), ghi tại mỗi checkpoint: capability Rust thay + commit/report B; file Go và caller/script/config đã đối chiếu; phần được gỡ hoặc lý do giữ với task/owner/mốc rà lại. Nếu có dọn, ghi task/CP dọn riêng, owner/allowlist, mốc Go cuối, checks trên candidate sau dọn, report B và commit/remote SHA/PR. Không có thay đổi Go thì ghi không áp dụng kèm phạm vi đã rà. GO-FINAL ghi môi trường không Go, artifact/checksum sau dọn và nghiệm thu bị ảnh hưởng đã kiểm lại.


## Merge và phần được bảo vệ

Theo [MERGE_WORKFLOW](../MERGE_WORKFLOW.md), ghi CP, dependency receipt, scope; base/head/merged-tree, report B và checks/manual/CI; PR, merge SHA, hậu kiểm/ancestry, consumer, blocker và bước tiếp. Điều phối xếp hàng merge; worker không tranh merge. Áp dụng [ngoại lệ conflict thuần test](../MERGE_WORKFLOW.md#test-conflicts): ghi hunk, căn cứ test-only, kỳ vọng/coverage giữ lại và checks; B kiểm diff/assertion và candidate tích hợp mới. Nếu cần sửa core do conflict, phần bảo vệ ngoài ngoại lệ hoặc không giải được trong phạm vi, dừng phần phụ thuộc và báo Master trước với SHA, file/hành vi, bằng chứng, diff tối thiểu và tests; chờ phê duyệt cụ thể khi cần sửa. CP docs-only ghi Git không áp dụng.


## Receipt dọn checkpoint

Theo [quy trình dọn checkpoint](../MERGE_WORKFLOW.md#checkpoint-cleanup): ghi mốc remote SHA + PR hoặc B Đạt + bàn giao docs-only; đường dẫn/loại đã dọn và dung lượng xấp xỉ thực đo; đường dẫn giữ, lý do, owner và sự kiện rà lại; xác minh status/index/source và đường dẫn evidence trước/sau. Chưa dọn thì ghi chờ và lý do, không khai dung lượng đã thu hồi. Sau merge/hậu kiểm bổ sung receipt khi điều kiện giữ hết.
