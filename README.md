# novel-translate-cli

![Novel Translate — banner dự án](assets/brand/novel-translate-banner.png)

**novel-translate-cli** là công cụ dòng lệnh và giao diện terminal (TUI) hỗ trợ dịch light novel từ tiếng Nhật sang tiếng Việt, với quy trình quản lý bản dịch theo bộ truyện và tập.

## Khả năng chính

- Nhập EPUB và lập chỉ mục chương, khối văn bản, ảnh cùng các thành phần liên quan.
- Dịch theo từng phần, lưu tiến độ để tiếp tục công việc và theo dõi kết quả.
- Review và sửa bản dịch; lưu lịch sử sửa, ghi chú, thuật ngữ và quy ước riêng cho từng bộ truyện.
- Quản lý ngữ cảnh và cách xưng hô theo nhân vật, cảnh và thời điểm trong truyện.
- Đánh giá ảnh có chữ, dùng ảnh đã chỉnh sửa hoặc thêm bản dịch văn bản.
- Kiểm tra và xuất EPUB xem trước hoặc bản hoàn chỉnh.

## Trạng thái phát triển

Dự án đang được viết lại bằng Rust.

## Thử bản Rust trên Windows

Mở `novel-translate-cli.exe` trên Windows x64. Tại **Tổng quan**, chọn **Tạo workspace** hoặc **Mở workspace**, duyệt đến thư mục trong hộp thoại Windows, kiểm tra đường dẫn hiển thị rồi xác nhận **Tạo** hoặc **Mở**.

Khi workspace đã mở, chọn **＋ Nhập tập truyện** ở **Tổng quan** hoặc **Tập và chương**. Workspace lưu cơ sở dữ liệu `series.sqlite3` và tệp nguồn trong thư mục gốc đã chọn. Sao lưu cả thư mục để giữ đủ dữ liệu.

Từ thư mục chứa bản chạy, có thể làm các bước tương ứng bằng PowerShell.

```powershell
.\novel-translate-cli.exe workspace create ".\workspace-thu"
.\novel-translate-cli.exe workspace open ".\workspace-thu"
.\novel-translate-cli.exe import ".\tap-01.epub" --workspace ".\workspace-thu" --volume 11111111111111111111111111111111
```
