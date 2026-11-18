# Mẫu review phase hoặc cổng bàn giao — model B

**Đây là mẫu, chưa phải chứng nhận nghiệm thu.** Dùng theo [quy trình hai model](../REVIEW_WORKFLOW.md), sau các review task và khi đầu ra cổng đã sẵn sàng kiểm tra.

## 1. Phạm vi và candidate cuối

- Phase / cổng: M0-01, M0-02, M0, M1…M9; ghi đúng phạm vi:
- Model B / phiên / ngày; model A / báo cáo bàn giao:
- Candidate, artifact checksum nếu có, baseline/manifest và phiên bản tài liệu áp dụng:
- Candidate đã đối chiếu lúc bắt đầu và trước kết luận:
- Danh sách task/AC thuộc cổng; phần ngoài phạm vi:
- OS/target/toolchain/terminal/reader/tool được kiểm tra:

## 2. Truy vết task và review còn hiệu lực

| Task / AC | Review B gần nhất | Candidate lúc task review | Thay đổi sau review đó / ảnh hưởng | Bằng chứng trên candidate cuối | Đạt / chưa đạt / chưa xác minh |
|---|---|---|---|---|---|
| Điền đủ phạm vi cổng | | | | | |

Không cộng các task đã từng đạt để suy ra bản tích hợp cuối đạt. Ghi rõ phần phải kiểm lại khi contract, dependency hoặc consumer thay đổi.

## 3. Nghiệm thu tích hợp

| Điều kiện mục 15 của phase / cổng | Cách kiểm và kỳ vọng | B tự chạy / đối chiếu bằng chứng nào | Kết quả / môi trường | Phần còn thiếu |
|---|---|---|---|---|
| Liệt kê đầy đủ, gồm demo và thủ công khi bắt buộc | | | | |

- Bộ lệnh Rust bắt buộc trên candidate cuối, hoặc lý do không áp dụng cho task tài liệu:
- Kiểm thử tích hợp/recovery/migration/security phù hợp với phase:
- Kiểm tra Go bị ảnh hưởng và bảo toàn dữ liệu:
- Kiểm tra offline/deterministic, CI matrix và target chưa xác minh:
- Terminal/reader thật, chất lượng Nhật–Việt và live evaluation nếu cổng yêu cầu:
- README/help/artifact/known limitations và tài liệu nội bộ ngoài Git:

Ghi lệnh thực chạy, số ca, kết quả, log và version/OS. Bằng chứng do người khác thực hiện phải được ghi là đã đối chiếu, không ghi thành B tự chạy. Không tự chạy model trả phí để hoàn tất mẫu.

## 4. Phát hiện và xử lý

Dùng cấu trúc finding trong [mẫu review task](TASK_REVIEW.md); gắn owner task, mức ảnh hưởng, căn cứ và phép kiểm bản sửa. Link review cũ và giữ ID cho phát hiện đang theo dõi.

| Finding / bằng chứng thiếu | Owner task | Tác động tới cổng | Trạng thái / cách giải quyết |
|---|---|---|---|
| Điền khi có | | | |

## 5. Kết luận cổng và bàn giao

- **Kết luận:** Đạt / Cần sửa / Chưa đủ bằng chứng; chỉ chọn khi đã review.
- **Áp dụng cho:** candidate cuối và phạm vi cổng.
- **Điều kiện còn thiếu / finding còn mở:**
- **Đầu ra và contract phase tiếp theo có thể nhận:**
- **Giới hạn hỗ trợ đã ghi nhận, task tiếp theo:**

M0-01 đạt chỉ bàn giao đầu vào Phase 01, không đóng toàn Phase 00 hoặc toàn P00-T10. M0-02 cũng chỉ áp dụng phần storage đã kiểm tra. Đóng toàn Phase 00 cần đủ các điều kiện riêng của M0. Bản sửa sau kết luận phải được kiểm tra lại theo ảnh hưởng; kết luận cổng không tự cấp quyền publish hoặc xóa đường Go.

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
