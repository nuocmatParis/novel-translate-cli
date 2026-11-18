# Phiếu giao một chat phụ trách phase

Đây là mẫu, chỉ điền bằng dữ liệu thực khi giao. Theo [quy tắc song song](../PARALLEL_EXECUTION.md).

| Trường | Giá trị cần điền |
|---|---|
| Phase / owner chat / model A | Phase, ID chat, model khác gpt-6.1-sol |
| Người điều phối trung tâm | Một chat A; đường dẫn PLAN trung tâm |
| Reviewer | gpt-6.1-sol / max; phiên riêng; runtime metadata nếu có |
| Scope | Task đầu tiên hoặc nhóm 2–3 task; AC/TC; checkpoint đích |
| Baseline | Commit rewrite/rust đã xác minh; dependency commits đã nhận |
| Đầu vào bắt buộc | Task/CP, B report, candidate/hash, remote SHA |
| Căn cứ/hợp đồng | Phiên bản spec/ADR/phase/graph; input/output/errors/invariants |
| Quyền sửa | Allowlist file/module; shared owner; migration ID được cấp |
| Môi trường | Worktree, feature branch, build dir, DB/profile/output/ports riêng |
| Nội bộ | Manifest bản docs được cấp; thư mục báo cáo riêng; không đưa Git |
| Scratch / retention | Mỗi root tạm, sandbox B và build output: owner, mục đích, điều kiện giữ/giải phóng; runner/toolchain/config và build dir riêng tái dùng; người phối hợp dọn sau review theo [quy trình](../MERGE_WORKFLOW.md#checkpoint-cleanup) |
| Kiểm chứng | Lệnh/ca riêng và bốn lệnh Rust; manual/platform bắt buộc |
| Công bố | Report task/CP, commit/push/draft PR; điều kiện mở consumer |
| Go liên quan | Mốc đánh giá theo PLAN §8; shared owner/allowlist, capability thay, caller còn giữ, task/CP dọn riêng khi đủ bằng chứng |
| Khi bị chặn | Dependency còn thiếu, task độc lập được phép, điều kiện tiếp tục |

Điều phối cấp phiếu trước code. Chat phase cập nhật báo cáo riêng và gửi kết quả; chỉ điều phối sửa PLAN chung. Không tự lấy tên branch/worktree dự kiến làm môi trường đã tồn tại.


## Merge và phần được bảo vệ

Theo [MERGE_WORKFLOW](../MERGE_WORKFLOW.md), ghi CP, dependency receipt, scope; base/head/merged-tree, report B và checks/manual/CI; PR, merge SHA, hậu kiểm/ancestry, consumer, blocker và bước tiếp. Điều phối xếp hàng merge; worker không tranh merge. Áp dụng [ngoại lệ conflict thuần test](../MERGE_WORKFLOW.md#test-conflicts): ghi hunk, căn cứ test-only, kỳ vọng/coverage giữ lại và checks; B kiểm diff/assertion và candidate tích hợp mới. Nếu cần sửa core do conflict, phần bảo vệ ngoài ngoại lệ hoặc không giải được trong phạm vi, dừng phần phụ thuộc và báo Master trước với SHA, file/hành vi, bằng chứng, diff tối thiểu và tests; chờ phê duyệt cụ thể khi cần sửa. CP docs-only ghi Git không áp dụng.
