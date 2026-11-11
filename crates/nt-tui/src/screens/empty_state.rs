use crate::navigation::Screen;

pub(crate) struct EmptyState {
    pub(crate) heading: &'static str,
    pub(crate) body: &'static str,
}

pub(crate) const fn empty_state(screen: Screen) -> EmptyState {
    match screen {
        Screen::Overview => EmptyState {
            heading: "Chưa mở workspace",
            body: "Chưa có workspace hoặc tác vụ nào để hiển thị.",
        },
        Screen::VolumesAndChapters => EmptyState {
            heading: "Chưa có nguồn",
            body: "Chưa có cây sách hoặc chương nào để hiển thị.",
        },
        Screen::Tasks => EmptyState {
            heading: "Chưa có tác vụ",
            body: "Không có tiến trình model hoặc chi phí nào đang chạy.",
        },
        Screen::JapaneseVietnameseEditor => EmptyState {
            heading: "Chưa chọn nội dung",
            body: "Không có nội dung để sửa; chưa có revision nào được lưu.",
        },
        Screen::Reviewer => EmptyState {
            heading: "Chưa có kết quả Review",
            body: "Chưa có issue hoặc bằng chứng nào để xem.",
        },
        Screen::SkillsAndContext => EmptyState {
            heading: "Chưa có dữ liệu để xem hoặc duyệt",
            body: "Chưa có thuật ngữ, nhân vật hoặc đề xuất nào để hiển thị.",
        },
        Screen::ImagesAndExport => EmptyState {
            heading: "Chưa có nội dung để kiểm tra hoặc xuất",
            body: "Chưa có ảnh hoặc bản xem trước nào.",
        },
        Screen::SettingsReportsAndHelp => EmptyState {
            heading: "Thông tin bản build",
            body: concat!(
                "Phiên bản: ",
                env!("CARGO_PKG_VERSION"),
                "\nCấu hình và báo cáo chưa khả dụng."
            ),
        },
    }
}
