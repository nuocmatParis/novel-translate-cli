# Mẫu review task — model B

**Đây là mẫu, chưa có review hoặc kết luận đạt.** B tạo báo cáo riêng cho mỗi lượt theo [quy trình hai model](../REVIEW_WORKFLOW.md); không ghi đè báo cáo của A.

## 1. Nhận diện bản kiểm tra

| Trường | Giá trị cần ghi |
|---|---|
| Task / nhóm / phạm vi | ID và tất cả AC được review |
| Model B / phiên / ngày | Model khác A theo phân công; phiên riêng |
| Model A / báo cáo bàn giao | Thông tin và link lượt được nhận |
| Candidate / baseline | ID, commit hoặc manifest/hash, diff và file mới đã đối chiếu |
| Lượt review | r01 hoặc lượt mới; link review trước |
| Căn cứ đã đọc | Spec/ADR/task áp dụng, revision/hash |
| Môi trường kiểm tra | OS/target/toolchain/tool/fixture |

- Candidate lúc bắt đầu có khớp bản bàn giao không?
- Candidate trước khi kết luận có còn khớp không?
- Thiếu file mới, tài liệu bị ignore hoặc baseline nào ảnh hưởng kết luận không?

## 2. Kiểm chứng độc lập

| Task / AC | Kết quả mong đợi do B đối chiếu từ yêu cầu | Phép kiểm B tự chạy / bằng chứng B đối chiếu | Kết quả | Phần chưa xác minh |
|---|---|---|---|---|
| Điền đủ từng AC trong phạm vi | | | | |

| Lệnh / thao tác | Candidate / môi trường | Tên hoặc số ca liên quan | Kết quả / exit | Log |
|---|---|---|---|---|
| Chỉ ghi phép kiểm B thực chạy | | | | |

- Test của A có assertion đủ phân biệt đúng/sai, ca lỗi/biên và được phát hiện/chạy không?
- Test thăm dò/đối chứng B bổ sung, cách tái hiện, vị trí tạm và kết quả:
- Code/consumer/boundary đã xem; side effect và cleanup đã kiểm:
- Bằng chứng CI/thủ công được đối chiếu riêng; không ghi thành test B tự chạy:
- Kiểm tra bắt buộc chưa chạy được, lý do và ảnh hưởng:

## 3. Phát hiện cần xử lý

Lặp cấu trúc sau cho từng finding có căn cứ; nếu không có, ghi rõ không tìm thấy lỗi chặn trong phạm vi đã kiểm tra.

### Finding ID: PXX-TYY-F01

- Mức ảnh hưởng P0/P1/P2/P3 và có chặn nghiệm thu hay không:
- Task / AC / yêu cầu bị vi phạm:
- File:dòng hoặc đoạn tài liệu:
- Điều kiện kích hoạt và cách tái hiện tối thiểu:
- Kết quả mong đợi → thực tế:
- Bằng chứng: test/lệnh/log hoặc lập luận đường lỗi cụ thể:
- Ảnh hưởng đến người dùng/dữ liệu/consumer:
- Điều kiện hoặc phép kiểm để xác nhận bản sửa:
- Trạng thái finding: mở / đóng / cần làm rõ; căn cứ:

## 4. Gợi ý và vấn đề ngoài phạm vi

Tách góp ý tùy chọn, lỗi baseline, yêu cầu mở rộng và nghi ngờ chưa chứng minh. Nêu rõ có ảnh hưởng AC hiện tại hay không. Không nâng sở thích trình bày thành lỗi chặn.

## 5. Kiểm tra lại findings trước

| Finding ID | Phản hồi / sửa của A | B đã kiểm lại bằng gì | Còn mở / đóng / cần làm rõ | Lý do và candidate |
|---|---|---|---|---|
| Điền từ lượt r02; r01 ghi không áp dụng | | | | |

## 6. Kết luận

- **Kết luận:** chỉ chọn khi đủ căn cứ: Đạt / Cần sửa / Chưa đủ bằng chứng.
- **Áp dụng cho:** candidate, task/AC và phạm vi đã kiểm tra.
- **Lỗi chặn còn mở:** finding ID và lý do, hoặc không có.
- **Bằng chứng bắt buộc còn thiếu:** liệt kê, hoặc không có.
- **Phần chưa kiểm tra / giới hạn:** ghi rõ, không suy rộng sang phase khác.
- **Hành động tiếp theo của A:** sửa, bổ sung bằng chứng hoặc cập nhật hoàn thành theo đúng kết luận.

Đạt chỉ hợp lệ khi candidate còn khớp, AC và kiểm tra bắt buộc đã đủ, không còn lỗi chặn. B không cập nhật PLAN hoặc sửa code để làm kết luận đúng.

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
