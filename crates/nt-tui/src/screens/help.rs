use crate::navigation::Screen;

pub(crate) const fn help_summary(screen: Screen) -> &'static str {
    match screen {
        Screen::Overview => "Chưa mở workspace. Mở hoặc tạo workspace chưa khả dụng.",
        Screen::VolumesAndChapters => "Chưa có nguồn. Cây sách và danh sách chương chưa khả dụng.",
        Screen::Tasks => "Chưa có tác vụ. Tạm dừng, tiếp tục và xem chi tiết chưa khả dụng.",
        Screen::JapaneseVietnameseEditor => {
            "Chưa chọn nội dung. Chỉnh sửa và lưu revision chưa khả dụng."
        }
        Screen::Reviewer => "Chưa có issue hoặc bằng chứng. Đánh giá và sửa issue chưa khả dụng.",
        Screen::SkillsAndContext => "Chưa có thuật ngữ hoặc nhân vật. Duyệt đề xuất chưa khả dụng.",
        Screen::ImagesAndExport => {
            "Chưa có ảnh hoặc bản xem trước. Kiểm tra và xuất nội dung chưa khả dụng."
        }
        Screen::SettingsReportsAndHelp => {
            "Phiên bản CLI hiển thị ở màn này. Cấu hình, kết nối và báo cáo chưa khả dụng."
        }
    }
}
