use crate::navigation::Screen;
use crate::screens::WorkspaceFormSnapshot;
use crate::screens::empty_state;
use crate::screens::help_summary;
use crate::state::{AppState, FocusRegion, MIN_TERMINAL_COLUMNS, MIN_TERMINAL_ROWS};
use ratatui::buffer::{Buffer, CellWidth};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::StyledGrapheme;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn render(buffer: &mut Buffer, area: Rect, state: &AppState) {
    render_with_optional_workspace(buffer, area, state, None);
}

pub(crate) fn render_with_workspace(
    buffer: &mut Buffer,
    area: Rect,
    state: &AppState,
    workspace: &WorkspaceFormSnapshot,
) {
    render_with_optional_workspace(buffer, area, state, Some(workspace));
}

fn render_with_optional_workspace(
    buffer: &mut Buffer,
    area: Rect,
    state: &AppState,
    workspace: Option<&WorkspaceFormSnapshot>,
) {
    if !terminal_size_supported(area) {
        render_too_small(buffer, area);
        return;
    }

    let (header_area, navigation_area, screen_area, footer_area) = layout_areas(area);

    let header = Paragraph::new("Novel Translate")
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::BOTTOM));
    header.render(header_area, buffer);

    render_navigation(
        buffer,
        state.selected_screen(),
        state.focus_region(),
        navigation_area,
    );
    render_screen(buffer, state, screen_area, workspace);

    let footer_line = if state.current_screen() == Screen::Overview {
        workspace.map_or_else(
            || footer_text(state, area.width),
            |workspace| Line::from(workspace.footer_text()),
        )
    } else {
        footer_text(state, area.width)
    };
    let footer = Paragraph::new(footer_line);
    footer.render(footer_area, buffer);

    if state.help_open() {
        render_help(buffer, area, state, workspace);
        Clear.render(footer_area, buffer);
        Paragraph::new("Esc: Đóng trợ giúp · Ctrl+C: Thoát").render(footer_area, buffer);
    }
}

fn terminal_size_supported(area: Rect) -> bool {
    area.width >= MIN_TERMINAL_COLUMNS && area.height >= MIN_TERMINAL_ROWS
}

fn render_too_small(buffer: &mut Buffer, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let message = format!(
        "Terminal {}×{} quá nhỏ; cần ít nhất {}×{}. Mở rộng cửa sổ để tiếp tục. Ctrl+C: Thoát.",
        area.width, area.height, MIN_TERMINAL_COLUMNS, MIN_TERMINAL_ROWS
    );
    Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .render(area, buffer);
}

fn layout_areas(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let [header_area, content_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    let (navigation_area, screen_area) = if area.width >= 120 {
        let [navigation, screen] =
            Layout::horizontal([Constraint::Length(34), Constraint::Min(0)]).areas(content_area);
        (navigation, screen)
    } else {
        let [screen, navigation] =
            Layout::vertical([Constraint::Min(10), Constraint::Length(10)]).areas(content_area);
        (navigation, screen)
    };
    (header_area, navigation_area, screen_area, footer_area)
}

pub(crate) fn scroll_metrics(area: Rect, state: &AppState) -> (u16, u16) {
    if !terminal_size_supported(area) {
        return (1, 0);
    }
    let body_area = body_area(area);
    let body = empty_state(state.current_screen()).body;
    scroll_metrics_for_body(body_area, body)
}

pub(crate) fn capture_scroll_anchor(state: &mut AppState, area: Rect) {
    if !terminal_size_supported(area) {
        return;
    }
    let body = empty_state(state.current_screen()).body;
    let width = Block::default()
        .borders(Borders::ALL)
        .inner(body_area(area))
        .width
        .max(1);
    let byte = visual_row_start_byte(body, width, state.scroll_offset());
    state.set_scroll_anchor_byte(byte);
}

pub(crate) fn scroll_offset_for_anchor(area: Rect, state: &AppState) -> u16 {
    if !terminal_size_supported(area) {
        return 0;
    }
    let body = empty_state(state.current_screen()).body;
    let content_area = body_area(area);
    let width = Block::default()
        .borders(Borders::ALL)
        .inner(content_area)
        .width
        .max(1);
    let (_, limit) = scroll_metrics_for_body(content_area, body);
    scroll_offset_at_anchor(body, width, state.scroll_anchor_byte(), limit)
}

fn scroll_offset_at_anchor(text: &str, width: u16, byte: usize, limit: u16) -> u16 {
    visual_row_for_source_byte(text, width, byte).min(limit)
}

fn body_area(area: Rect) -> Rect {
    let (_, _, screen_area, _) = layout_areas(area);
    let [_, body_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(screen_area);
    body_area
}

fn visual_row_start_byte(text: &str, width: u16, row: u16) -> usize {
    let mut anchor = text.len();
    visit_wrapped_rows(text, width.max(1), |index, graphemes, fallback| {
        if index == row as usize {
            anchor = graphemes.first().copied().unwrap_or(fallback);
            true
        } else {
            false
        }
    });
    anchor
}

fn visual_row_for_source_byte(text: &str, width: u16, byte: usize) -> u16 {
    let requested_byte = byte.min(text.len());
    let byte = grapheme_start_at_or_before(text, requested_byte);
    let mut exact = None;
    let mut following = None;
    let mut last = 0usize;
    visit_wrapped_rows(text, width.max(1), |index, graphemes, fallback| {
        last = index;
        if fallback == requested_byte || graphemes.contains(&byte) {
            exact = Some(index);
            return true;
        }
        if graphemes.iter().any(|start| *start > byte) {
            following = Some(index);
            return true;
        }
        false
    });
    exact.or(following).unwrap_or(last).min(u16::MAX as usize) as u16
}

fn grapheme_start_at_or_before(text: &str, byte: usize) -> usize {
    let byte = byte.min(text.len());
    if byte == text.len() {
        return byte;
    }
    text.grapheme_indices(true)
        .take_while(|(start, _)| *start <= byte)
        .last()
        .map_or(0, |(start, _)| start)
}

fn visit_wrapped_rows(
    text: &str,
    width: u16,
    mut visit: impl FnMut(usize, &[usize], usize) -> bool,
) {
    let mut row_index = 0usize;
    if text.is_empty() {
        let _ = visit_wrapped_line(text, 0, width, &mut row_index, &mut visit);
        return;
    }

    let mut source_base = 0usize;
    for line in text.lines() {
        if visit_wrapped_line(line, source_base, width, &mut row_index, &mut visit) {
            return;
        }
        source_base += line.len();
        if text[source_base..].starts_with("\r\n") {
            source_base += 2;
        } else if text[source_base..].starts_with('\n') {
            source_base += 1;
        }
    }
}

fn visit_wrapped_line(
    text: &str,
    source_base: usize,
    width: u16,
    row_index: &mut usize,
    visit: &mut impl FnMut(usize, &[usize], usize) -> bool,
) -> bool {
    let first_row_index = *row_index;
    let max_width = width as usize;
    let mut wrapped_line = Vec::new();
    let mut line_width = 0usize;
    let mut word_width = 0usize;
    let mut whitespace_width = 0usize;
    let mut pending_word = Vec::new();
    let mut pending_whitespace = std::collections::VecDeque::new();
    let mut non_whitespace_previous = false;

    for (relative_start, grapheme) in text.grapheme_indices(true) {
        let source_start = source_base + relative_start;
        if grapheme.contains(char::is_control) {
            continue;
        }
        let is_whitespace = StyledGrapheme::new(grapheme, Style::default()).is_whitespace();
        let symbol_width = grapheme.cell_width() as usize;
        if symbol_width > max_width {
            continue;
        }

        let word_found = non_whitespace_previous && is_whitespace;
        let trimmed_overflow = wrapped_line.is_empty() && word_width + symbol_width > max_width;
        let whitespace_overflow =
            wrapped_line.is_empty() && whitespace_width + symbol_width > max_width;

        if word_found || trimmed_overflow || whitespace_overflow {
            if !wrapped_line.is_empty() {
                while let Some((start, _)) = pending_whitespace.pop_front() {
                    wrapped_line.push(start);
                }
                line_width += whitespace_width;
            }
            wrapped_line.append(&mut pending_word);
            line_width += word_width;
            pending_whitespace.clear();
            whitespace_width = 0;
            word_width = 0;
        }

        let line_full = line_width >= max_width;
        let pending_word_overflow =
            symbol_width > 0 && line_width + whitespace_width + word_width >= max_width;
        if line_full || pending_word_overflow {
            let mut remaining_width = max_width.saturating_sub(line_width);
            let fallback = wrapped_line.first().copied().unwrap_or(source_start);
            if visit(*row_index, &wrapped_line, fallback) {
                return true;
            }
            *row_index += 1;
            wrapped_line.clear();
            line_width = 0;

            while let Some((_, space_width)) = pending_whitespace.front().copied() {
                if space_width > remaining_width {
                    break;
                }
                whitespace_width = whitespace_width.saturating_sub(space_width);
                remaining_width -= space_width;
                pending_whitespace.pop_front();
            }

            if is_whitespace && pending_whitespace.is_empty() {
                continue;
            }
        }

        if is_whitespace {
            whitespace_width += symbol_width;
            pending_whitespace.push_back((source_start, symbol_width));
        } else {
            word_width += symbol_width;
            pending_word.push(source_start);
        }
        non_whitespace_previous = !is_whitespace;
    }

    if wrapped_line.is_empty() && pending_word.is_empty() && !pending_whitespace.is_empty() {
        let fallback = pending_whitespace
            .front()
            .map_or(source_base, |(start, _)| *start);
        if visit(*row_index, &[], fallback) {
            return true;
        }
        *row_index += 1;
    }
    if !wrapped_line.is_empty() {
        while let Some((start, _)) = pending_whitespace.pop_front() {
            wrapped_line.push(start);
        }
    }
    wrapped_line.append(&mut pending_word);

    if !wrapped_line.is_empty() {
        let fallback = wrapped_line[0];
        if visit(*row_index, &wrapped_line, fallback) {
            return true;
        }
        *row_index += 1;
    } else if *row_index == first_row_index {
        if visit(*row_index, &[], source_base) {
            return true;
        }
        *row_index += 1;
    }
    false
}

fn scroll_metrics_for_body(body_area: Rect, body: &str) -> (u16, u16) {
    let inner = Block::default().borders(Borders::ALL).inner(body_area);
    let paragraph = Paragraph::new(body).wrap(Wrap { trim: true });
    let page_lines = inner.height.max(1);
    let total_lines = paragraph.line_count(inner.width.max(1));
    let max_offset = total_lines
        .saturating_sub(page_lines as usize)
        .min(u16::MAX as usize) as u16;
    (page_lines, max_offset)
}

fn scroll_hint(state: &AppState, width: u16) -> Option<&'static str> {
    match (state.can_scroll_up(), state.can_scroll_down(), width >= 120) {
        (false, false, _) => None,
        (false, true, false) => Some("PgDn cuộn"),
        (true, false, false) => Some("PgUp cuộn"),
        (true, true, false) => Some("PgUp/PgDn cuộn"),
        (false, true, true) => Some("PageDown: Cuộn"),
        (true, false, true) => Some("PageUp: Cuộn"),
        (true, true, true) => Some("PageUp/PageDown: Cuộn"),
    }
}

fn footer_text(state: &AppState, width: u16) -> Line<'static> {
    let text = match state.focus_region() {
        FocusRegion::Navigation => {
            "Điều hướng · ↑/↓: Chọn · Enter: Mở · Tab: Thoát · F1: Trợ giúp · Ctrl+C: Thoát"
        }
        FocusRegion::Exit => {
            "[ Thoát ] · Enter: Thoát · Tab: Điều hướng · F1: Trợ giúp · Ctrl+C: Thoát"
        }
    };
    match scroll_hint(state, width) {
        Some(hint) if width < 120 => {
            let compact = match state.focus_region() {
                FocusRegion::Navigation => {
                    "↑/↓ Chọn · Enter Mở · Tab Thoát · F1 Trợ giúp · Ctrl+C Thoát"
                }
                FocusRegion::Exit => "Enter Thoát · Tab Điều hướng · F1 Trợ giúp · Ctrl+C Thoát",
            };
            Line::from(format!("{compact} · {hint}"))
        }
        Some(hint) => Line::from(format!("{text} · {hint}")),
        None => Line::from(text),
    }
}

fn render_help(
    buffer: &mut Buffer,
    area: Rect,
    state: &AppState,
    workspace: Option<&WorkspaceFormSnapshot>,
) {
    let width = area.width.saturating_sub(4).min(72);
    let height = area.height.saturating_sub(4).min(12);
    let popup = Rect::new(
        area.x.saturating_add((area.width - width) / 2),
        area.y.saturating_add((area.height - height) / 2),
        width,
        height,
    );
    Clear.render(popup, buffer);

    let interaction = match state.focus_region() {
        FocusRegion::Navigation => "↑/↓: Chọn màn · Enter: Mở màn · Tab: Chuyển tới Thoát",
        FocusRegion::Exit => "Enter: Thoát · Tab: Quay về Điều hướng",
    };
    let summary = if state.current_screen() == Screen::Overview {
        workspace.map_or_else(
            || help_summary(state.current_screen()).to_owned(),
            WorkspaceFormSnapshot::help_summary,
        )
    } else {
        help_summary(state.current_screen()).to_owned()
    };
    let content = format!(
        "{}\n\nSau khi đóng trợ giúp bằng Esc, focus {} sẽ tiếp tục:\n{}",
        summary,
        state.focus_region().label(),
        interaction
    );
    Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(format!("Trợ giúp · {}", state.current_screen().title()))
                .borders(Borders::ALL),
        )
        .render(popup, buffer);
}

fn render_navigation(buffer: &mut Buffer, selected: Screen, focus: FocusRegion, area: Rect) {
    let items = Screen::ALL.into_iter().map(|screen| {
        let marker = if screen == selected { "› " } else { "  " };
        ListItem::new(format!("{marker}{}", screen.title()))
    });
    let title = if focus == FocusRegion::Navigation {
        "Màn hình · Điều hướng"
    } else {
        "Màn hình"
    };
    let navigation = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    navigation.render(area, buffer);
}

fn render_screen(
    buffer: &mut Buffer,
    state: &AppState,
    area: Rect,
    workspace: Option<&WorkspaceFormSnapshot>,
) {
    let screen = state.current_screen();
    let content = empty_state(screen);
    let workspace = (screen == Screen::Overview).then_some(workspace).flatten();
    let [heading, body_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);
    let heading_text = workspace.map_or(content.heading, WorkspaceFormSnapshot::heading);
    Paragraph::new(heading_text)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL))
        .render(heading, buffer);
    let body_text = workspace.map_or_else(|| content.body.to_owned(), WorkspaceFormSnapshot::body);
    render_wrapped_body(
        buffer,
        body_area,
        screen.title(),
        &body_text,
        state.scroll_offset(),
    );
}

fn render_wrapped_body(
    buffer: &mut Buffer,
    area: Rect,
    title: &str,
    text: &str,
    scroll_offset: u16,
) {
    Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset, 0))
        .block(Block::default().title(title).borders(Borders::ALL))
        .render(area, buffer);
}

#[cfg(test)]
mod tests {
    use super::footer_text;
    use super::render;
    use super::{
        grapheme_start_at_or_before, layout_areas, render_wrapped_body, scroll_metrics_for_body,
        scroll_offset_at_anchor, visual_row_for_source_byte, visual_row_start_byte,
    };
    use crate::navigation::Screen;
    use crate::state::{AppState, FocusRegion};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

    fn buffer_text(buffer: &Buffer, area: Rect) -> String {
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn rendered_text(state: &AppState, width: u16, height: u16) -> String {
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        render(&mut buffer, area, state);
        buffer_text(&buffer, area)
    }

    fn body_frame(area: Rect) -> Rect {
        let (_, _, screen_area, _) = layout_areas(area);
        let [_, body_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(screen_area);
        body_area
    }

    fn normalized(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn default_state_opens_overview_with_navigation_focus() {
        let state = AppState::default();

        assert_eq!(state.current_screen(), Screen::Overview);
        assert_eq!(state.focus_region(), FocusRegion::Navigation);
        let text = rendered_text(&state, 80, 24);
        assert!(text.contains("Tổng quan"));
        assert!(text.contains("Chưa mở workspace"));
        assert!(text.contains("F1: Trợ giúp"));
        assert!(text.contains("↑/↓: Chọn"));
        assert!(!text.contains("/ Search"));
    }

    #[test]
    fn route_set_is_exact_and_each_route_renders_its_empty_state() {
        let expected = [
            (Screen::Overview, "Chưa mở workspace"),
            (Screen::VolumesAndChapters, "Chưa có nguồn"),
            (Screen::Tasks, "Chưa có tác vụ"),
            (Screen::JapaneseVietnameseEditor, "Chưa chọn nội dung"),
            (Screen::Reviewer, "Chưa có kết quả Review"),
            (
                Screen::SkillsAndContext,
                "Chưa có dữ liệu để xem hoặc duyệt",
            ),
            (
                Screen::ImagesAndExport,
                "Chưa có nội dung để kiểm tra hoặc xuất",
            ),
            (Screen::SettingsReportsAndHelp, "Thông tin bản build"),
        ];
        assert_eq!(Screen::ALL.len(), 8);
        assert_eq!(
            Screen::ALL.map(Screen::title),
            [
                "Tổng quan",
                "Tập và chương",
                "Tác vụ",
                "Editor Nhật–Việt",
                "Reviewer",
                "Skills và ngữ cảnh",
                "Ảnh và xuất",
                "Cấu hình, báo cáo và trợ giúp",
            ]
        );

        for (screen, empty_heading) in expected {
            let mut state = AppState::default();
            state.navigate_to(screen);
            let text = rendered_text(&state, 80, 24);
            assert!(
                text.contains(screen.title()),
                "missing route title: {screen:?}"
            );
            let selected_label = format!("› {}", screen.title());
            assert!(
                text.lines().any(|line| line.contains(&selected_label)),
                "selected route is not fully visible in navigation: {screen:?}"
            );
            assert!(
                text.contains(empty_heading),
                "missing empty state for {screen:?}"
            );
            assert!(
                text.contains("Điều hướng"),
                "missing focus region: {screen:?}"
            );
            assert!(!text.contains("GPT-"), "fake model rendered on {screen:?}");
            assert!(
                !text.contains("Chapter 01"),
                "fake chapter rendered on {screen:?}"
            );
            if screen == Screen::SettingsReportsAndHelp {
                assert!(text.contains("Cấu hình và báo cáo chưa khả dụng."));
                assert!(!text.contains("Trợ giúp: Chưa khả dụng."));
            }
        }
    }

    #[test]
    fn long_body_text_wraps_inside_the_content_frame() {
        let expected = "Không có nội dung để sửa; chưa có revision nào được lưu.";
        let area = Rect::new(0, 0, 40, 6);
        let mut buffer = Buffer::empty(area);
        render_wrapped_body(&mut buffer, area, "Nội dung dài", expected, 0);
        let rendered = buffer_text(&buffer, area);
        let text = normalized(&rendered);
        let mut remaining = text.as_str();
        for word in expected.split_whitespace() {
            let Some(position) = remaining.find(word) else {
                panic!("rendered buffer lost `{word}` from the long message: {rendered}");
            };
            remaining = &remaining[position + word.len()..];
        }

        let lines = rendered.lines().collect::<Vec<_>>();
        let first_line = lines
            .iter()
            .position(|line| line.contains("Không có nội dung"))
            .expect("first part of long message is rendered");
        let final_line = lines
            .iter()
            .position(|line| line.contains("lưu."))
            .expect("last part of long message is rendered");
        assert!(
            first_line < final_line,
            "long message wraps in order: {rendered}"
        );
    }

    #[test]
    fn rendering_does_not_mutate_presentation_or_business_state() {
        let state = AppState::default();
        let before = state;

        let _ = rendered_text(&state, 80, 24);

        assert_eq!(state, before);
    }

    #[test]
    fn layout_uses_vertical_navigation_below_120_columns_and_side_by_side_at_120() {
        let (_, narrow_navigation, narrow_screen, _) = layout_areas(Rect::new(0, 0, 119, 24));
        assert!(narrow_screen.y < narrow_navigation.y);
        assert_eq!(narrow_screen.x, 0);

        let (_, wide_navigation, wide_screen, _) = layout_areas(Rect::new(0, 0, 120, 24));
        assert_eq!(wide_navigation.y, wide_screen.y);
        assert_eq!(wide_navigation.x, 0);
        assert!(wide_screen.x >= wide_navigation.width);
    }

    #[test]
    fn scroll_footer_advertises_only_page_keys_that_can_move_the_view() {
        let mut state = AppState::default();
        state.set_scroll_metrics(4, 8);

        let top = footer_text(&state, 80).to_string();
        assert!(top.contains("PgDn cuộn"));
        assert!(!top.contains("PgUp"));

        state.scroll_page(true);
        let middle = footer_text(&state, 80).to_string();
        assert!(middle.contains("PgUp/PgDn cuộn"));

        state.set_scroll_metrics(4, state.scroll_offset());
        let bottom = footer_text(&state, 80).to_string();
        assert!(bottom.contains("PgUp cuộn"));
        assert!(!bottom.contains("PgDn"));

        state.set_scroll_metrics(4, 0);
        let not_scrollable = footer_text(&state, 80).to_string();
        assert!(!not_scrollable.contains("PgUp"));
        assert!(!not_scrollable.contains("PgDn"));
    }

    #[test]
    fn minimum_width_footer_keeps_available_scroll_keys_visible() {
        let mut state = AppState::default();
        state.set_scroll_metrics(4, 8);

        let top = rendered_text(&state, 80, 24);
        assert!(top.contains("PgDn cuộn"), "{top}");

        state.scroll_page(true);
        let middle = rendered_text(&state, 80, 24);
        assert!(middle.contains("PgUp/PgDn cuộn"), "{middle}");

        state.set_scroll_metrics(4, state.scroll_offset());
        let bottom = rendered_text(&state, 80, 24);
        assert!(bottom.contains("PgUp cuộn"), "{bottom}");
        assert!(!bottom.contains("PgDn"), "{bottom}");
    }

    #[test]
    fn too_small_layout_warns_without_rendering_hidden_controls_or_panicking() {
        for area in [
            Rect::new(0, 0, 79, 24),
            Rect::new(0, 0, 80, 23),
            Rect::new(0, 0, 1, 1),
            Rect::new(0, 0, 0, 0),
        ] {
            let state = AppState::default();
            let mut buffer = Buffer::empty(area);
            render(&mut buffer, area, &state);
            let text = buffer_text(&buffer, area);

            assert!(!text.contains("Màn hình · Điều hướng"));
            assert!(!text.contains("F1: Trợ giúp"));
            if area.width >= 20 && area.height >= 3 {
                let normalized = normalized(&text);
                assert!(normalized.contains("quá nhỏ"));
                assert!(normalized.contains("80×24"));
                assert!(normalized.contains("Ctrl+C: Thoát"));
            }
        }
    }

    #[test]
    fn unicode_wrap_and_page_scroll_preserve_source_and_render_by_display_cells() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        let source = format!(
            "日本語の長い文を表示します。Tiếng Việt có dấu và dấu kết hợp a\u{0301}.\n{}\n最後の行です。",
            "日本語の長い文を表示します。Tiếng Việt có dấu và dấu kết hợp a\u{0301}.".repeat(8)
        );
        let unchanged_source = source.clone();
        let area = Rect::new(0, 0, 28, 8);
        let mut state = AppState::default();
        let (page_lines, scroll_limit) = scroll_metrics_for_body(area, &source);
        assert!(scroll_limit > page_lines);
        state.set_scroll_metrics(page_lines, scroll_limit);

        let mut first = Buffer::empty(area);
        render_wrapped_body(
            &mut first,
            area,
            "日本語 / Tiếng Việt",
            &source,
            state.scroll_offset(),
        );
        let first_text = normalized(&buffer_text(&first, area));
        let compact_first_text = first_text.replace(' ', "");
        assert!(compact_first_text.contains("日本語"), "{first_text}");
        assert!(first_text.contains("Tiếng Việt"), "{first_text}");
        assert!(first_text.contains("a\u{0301}"), "{first_text}");

        assert_eq!(
            crate::input::dispatch_event(
                &mut state,
                &Event::Key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
                None,
                None,
            ),
            crate::input::UiAction::None
        );
        assert_eq!(state.scroll_offset(), page_lines);
        let mut next = Buffer::empty(area);
        render_wrapped_body(
            &mut next,
            area,
            "日本語 / Tiếng Việt",
            &source,
            state.scroll_offset(),
        );
        assert_ne!(normalized(&buffer_text(&next, area)), first_text);
        assert_eq!(source, unchanged_source);

        assert_eq!(source, unchanged_source);
    }

    #[test]
    fn resize_reflow_preserves_source_grapheme_across_lines_and_temporary_clamps() {
        let source = format!(
            "日本語 a\u{0301} Tiếng Việt\n{}",
            (0..32)
                .map(|line| format!("L{line:02} {}末尾{line:02}", "日本語".repeat(12)))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let narrow = Rect::new(0, 0, 80, 24);
        let wide = Rect::new(0, 0, 120, 24);
        let narrow_layout = body_frame(narrow);
        let narrow_body = Rect::new(0, 0, narrow_layout.width, narrow_layout.height);
        let wide_layout = body_frame(wide);
        let wide_body = Rect::new(0, 0, wide_layout.width, wide_layout.height);
        let narrow_width = Block::default()
            .borders(Borders::ALL)
            .inner(narrow_body)
            .width;
        let wide_width = Block::default()
            .borders(Borders::ALL)
            .inner(wide_body)
            .width;
        let (page_lines, narrow_limit) = scroll_metrics_for_body(narrow_body, &source);
        let (wide_page_lines, wide_limit) = scroll_metrics_for_body(wide_body, &source);
        assert_eq!(page_lines, 5);

        let narrow_offset = page_lines * 2;
        let anchor = visual_row_start_byte(&source, narrow_width, narrow_offset);
        let render_body = |area: Rect, text: &str, offset: u16| {
            let mut buffer = Buffer::empty(area);
            render_wrapped_body(&mut buffer, area, "Fixture", text, offset);
            buffer_text(&buffer, area)
        };
        let marker_at_top = |text: &str| {
            let row = text
                .lines()
                .find(|line| line.contains('日') || line.contains('本'))
                .expect("fixture Japanese row");
            row.replace(' ', "")
                .split_once("末尾")
                .expect("source marker")
                .1
                .chars()
                .take(2)
                .collect::<String>()
        };

        let narrow_marker = marker_at_top(&render_body(narrow_body, &source, narrow_offset));
        assert_eq!(narrow_marker, "04");

        let wide_offset = scroll_offset_at_anchor(&source, wide_width, anchor, wide_limit);
        let wide_marker = marker_at_top(&render_body(wide_body, &source, wide_offset));
        assert_eq!(wide_marker, narrow_marker);

        let narrow_round_trip =
            scroll_offset_at_anchor(&source, narrow_width, anchor, narrow_limit);
        assert_eq!(
            marker_at_top(&render_body(narrow_body, &source, narrow_round_trip)),
            narrow_marker
        );

        let mut state = AppState::default();
        state.set_scroll_metrics(page_lines, narrow_limit);
        state.set_scroll_offset(narrow_offset);
        state.set_scroll_anchor_byte(anchor);
        state.set_terminal_size(79, 24);
        state.set_scroll_metrics(1, 0);
        assert_eq!(state.scroll_offset(), 0);
        assert_eq!(state.scroll_anchor_byte(), anchor);
        state.set_terminal_size(120, 24);
        state.set_scroll_metrics(wide_page_lines, wide_limit);
        state.set_scroll_offset(scroll_offset_at_anchor(
            &source,
            wide_width,
            state.scroll_anchor_byte(),
            wide_limit,
        ));
        assert_eq!(
            marker_at_top(&render_body(wide_body, &source, state.scroll_offset())),
            narrow_marker
        );
        assert_eq!(state.scroll_anchor_byte(), anchor);

        let near_end_source = format!(
            "日本語  a\u{0301} Tiếng Việt\n{}",
            (0..32)
                .map(|line| format!("L{line:02} {}末尾{line:02}", "日本語".repeat(12)))
                .collect::<Vec<_>>()
                .join("\n")
        ) + "\n";
        let near_end_offset = 60;
        let near_end_anchor =
            visual_row_start_byte(&near_end_source, narrow_width, near_end_offset);
        let wide = Rect::new(0, 0, 120, 24);
        let wide_layout = body_frame(wide);
        let wide_body = Rect::new(0, 0, wide_layout.width, wide_layout.height);
        let wide_width = Block::default()
            .borders(Borders::ALL)
            .inner(wide_body)
            .width;
        let (_, near_end_narrow_limit) = scroll_metrics_for_body(narrow_body, &near_end_source);
        let (_, wide_limit) = scroll_metrics_for_body(wide_body, &near_end_source);
        let clamped_wide_offset =
            scroll_offset_at_anchor(&near_end_source, wide_width, near_end_anchor, wide_limit);
        assert_eq!(clamped_wide_offset, 18);
        assert_eq!(wide_limit, 18);
        assert!(clamped_wide_offset < near_end_offset);
        let near_end_narrow_render = render_body(narrow_body, &near_end_source, near_end_offset);
        assert!(
            near_end_narrow_render.replace(' ', "").contains("末尾31"),
            "limit={near_end_narrow_limit}, offset={near_end_offset}, view={near_end_narrow_render}"
        );
        let near_end_wide_render = render_body(wide_body, &near_end_source, clamped_wide_offset);
        assert!(near_end_wide_render.replace(' ', "").contains("末尾31"));
        assert_eq!(
            scroll_offset_at_anchor(
                &near_end_source,
                narrow_width,
                near_end_anchor,
                near_end_narrow_limit,
            ),
            near_end_offset
        );
    }

    #[test]
    fn resize_reflow_keeps_a_single_line_anchor_and_grapheme_boundaries() {
        let source = (0..240)
            .map(|segment| format!("SEG{segment:03}日本語"))
            .collect::<Vec<_>>()
            .join(" ");
        let narrow = Rect::new(0, 0, 80, 24);
        let wide = Rect::new(0, 0, 120, 24);
        let narrow_layout = body_frame(narrow);
        let narrow_body = Rect::new(0, 0, narrow_layout.width, narrow_layout.height);
        let wide_layout = body_frame(wide);
        let wide_body = Rect::new(0, 0, wide_layout.width, wide_layout.height);
        let narrow_width = Block::default()
            .borders(Borders::ALL)
            .inner(narrow_body)
            .width;
        let wide_width = Block::default()
            .borders(Borders::ALL)
            .inner(wide_body)
            .width;
        let (_, narrow_limit) = scroll_metrics_for_body(narrow_body, &source);
        let (_, wide_limit) = scroll_metrics_for_body(wide_body, &source);
        let anchor = visual_row_start_byte(&source, narrow_width, 20);
        assert!(source[anchor..].starts_with("SEG120"), "{anchor}");
        let wide_offset = scroll_offset_at_anchor(&source, wide_width, anchor, wide_limit);
        let narrow_round_trip =
            scroll_offset_at_anchor(&source, narrow_width, anchor, narrow_limit);
        let render_body = |area: Rect, offset: u16| {
            let mut buffer = Buffer::empty(area);
            render_wrapped_body(&mut buffer, area, "Fixture", &source, offset);
            buffer_text(&buffer, area)
        };
        let top_body_row =
            |rendered: String| rendered.lines().nth(1).unwrap_or_default().replace(' ', "");
        let wide_rendered = render_body(wide_body, wide_offset);
        let narrow_rendered = render_body(narrow_body, narrow_round_trip);
        assert!(
            top_body_row(wide_rendered.clone()).contains("SEG120"),
            "offset={wide_offset}, anchor={anchor}, top={:?}",
            top_body_row(wide_rendered)
        );
        assert!(
            top_body_row(narrow_rendered.clone()).contains("SEG120"),
            "offset={narrow_round_trip}, anchor={anchor}, top={:?}",
            top_body_row(narrow_rendered)
        );

        let combining_source = "x\u{0301}日本語 Tiếng Việt";
        assert_eq!(grapheme_start_at_or_before(combining_source, 1), 0);
        assert_eq!(grapheme_start_at_or_before(combining_source, 2), 0);
        assert_eq!(visual_row_for_source_byte(combining_source, 20, 2), 0);
    }

    #[test]
    fn resize_anchor_mapping_matches_ratatui_for_cjk_and_whitespace_reflow() {
        let separators = [
            " ",
            "  ",
            "\u{00a0}",
            " \u{00a0}",
            "\u{00a0} ",
            "\t",
            "\u{001b}",
            "\u{0085}",
            "\t\u{0301}",
        ];
        let widths = [5, 6, 7, 9, 78, 79, 83, 84];
        let mut fixtures = Vec::new();

        for separator in separators {
            let mut source = String::new();
            for index in 0..96 {
                if index > 0 {
                    source.push_str(separator);
                }
                source.push(char::from_u32(0x4e00 + index).expect("valid CJK glyph"));
            }
            fixtures.push((format!("separator {separator:?}"), source));
        }

        let mixed_separators = [" ", "  ", "\u{00a0}", "\u{200b}", "\t", "\u{001b}"];
        let mut mixed_source = String::new();
        let mut seed = 7_u64;
        for index in 0..380 {
            mixed_source.push(char::from_u32(0x4e00 + index).expect("valid CJK glyph"));
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if seed >> 60 == 0 {
                mixed_source.push_str(mixed_separators[(seed as usize) % mixed_separators.len()]);
            }
        }
        fixtures.push(("seeded mixed separators".to_owned(), mixed_source));

        for (fixture, source) in fixtures {
            let glyph_starts = source
                .char_indices()
                .filter(|(_, glyph)| ('\u{4e00}'..='\u{9fff}').contains(glyph))
                .map(|(byte, glyph)| (glyph, byte))
                .collect::<Vec<_>>();

            for width in widths {
                let line_count = Paragraph::new(source.as_str())
                    .wrap(Wrap { trim: true })
                    .line_count(width);
                let area = Rect::new(0, 0, width, line_count as u16);
                let mut buffer = Buffer::empty(area);
                Paragraph::new(source.as_str())
                    .wrap(Wrap { trim: true })
                    .render(area, &mut buffer);

                let rendered_line_count = line_count;
                assert!(rendered_line_count > 1, "width={width}, fixture={fixture}");
                for row in 0..rendered_line_count {
                    let first_visible_glyph = glyph_starts.iter().find(|(glyph, _)| {
                        let glyph = glyph.to_string();
                        (0..width).any(|column| buffer[(column, row as u16)].symbol() == glyph)
                    });
                    let Some((_, visible_byte)) = first_visible_glyph else {
                        panic!(
                            "rendered row has no source marker; width={width}, row={row}, fixture={fixture}"
                        );
                    };

                    let anchor = visual_row_start_byte(&source, width, row as u16);
                    if fixture == "separator \" \"" || fixture == "separator \"  \"" {
                        assert_eq!(
                            anchor, *visible_byte,
                            "trimmed ASCII whitespace must leave the rendered CJK glyph as the row anchor; width={width}, row={row}, fixture={fixture}"
                        );
                    }
                    assert!(
                        anchor <= *visible_byte,
                        "anchor passed rendered CJK marker; width={width}, row={row}, fixture={fixture}, anchor={anchor}, marker={visible_byte}"
                    );
                    assert_eq!(
                        visual_row_for_source_byte(&source, width, *visible_byte),
                        row as u16,
                        "rendered CJK marker mapped to the wrong row; width={width}, row={row}, fixture={fixture}, marker={visible_byte}"
                    );
                    assert_eq!(
                        visual_row_for_source_byte(&source, width, anchor),
                        row as u16,
                        "captured row anchor did not round-trip; width={width}, row={row}, fixture={fixture}, anchor={anchor}"
                    );
                }
            }
        }
    }

    #[test]
    fn resize_anchor_mapping_counts_and_round_trips_empty_logical_lines() {
        let source = "\n\nbody\n";
        let width = 8;
        let line_count = Paragraph::new(source)
            .wrap(Wrap { trim: true })
            .line_count(width);
        let mut rows = Vec::new();
        super::visit_wrapped_rows(source, width, |index, graphemes, fallback| {
            rows.push((index, graphemes.to_vec(), fallback));
            false
        });

        assert_eq!(rows.len(), line_count);
        for (index, _, fallback) in rows {
            let anchor = visual_row_start_byte(source, width, index as u16);
            assert_eq!(anchor, fallback);
            assert_eq!(
                visual_row_for_source_byte(source, width, anchor),
                index as u16
            );
        }
    }

    #[test]
    fn resize_anchor_keeps_the_same_cjk_glyph_visible_at_80_to_81_columns() {
        let mut source = "header\n".repeat(4);
        let mut seed = 1_u64;
        for index in 0..380 {
            source.push(char::from_u32(0x4e00 + index).expect("valid CJK glyph"));
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if (seed >> 60) == 0 {
                source.push('\u{00a0}');
            }
        }

        let render_view = |width: u16, offset: u16| {
            let area = Rect::new(0, 0, width, 7);
            let mut buffer = Buffer::empty(area);
            render_wrapped_body(&mut buffer, area, "Tác vụ", &source, offset);
            buffer_text(&buffer, area)
        };
        let narrow_inner_width = 78;
        let wide_inner_width = 79;
        let narrow_offset = 5;
        let anchor = visual_row_start_byte(&source, narrow_inner_width, narrow_offset);
        let (wide_page_lines, wide_limit) =
            scroll_metrics_for_body(Rect::new(0, 0, 81, 7), &source);
        assert_eq!(wide_page_lines, 5);
        let wide_offset = scroll_offset_at_anchor(&source, wide_inner_width, anchor, wide_limit);

        let narrow = render_view(80, narrow_offset);
        let wide = render_view(81, wide_offset);
        let round_trip_offset = scroll_offset_at_anchor(
            &source,
            narrow_inner_width,
            anchor,
            scroll_metrics_for_body(Rect::new(0, 0, 80, 7), &source).1,
        );
        let round_trip = render_view(80, round_trip_offset);

        let top_body_row = |view: &str| view.lines().nth(1).unwrap_or_default().to_owned();
        let preserved_glyph = "严";
        assert!(top_body_row(&narrow).contains(preserved_glyph), "{narrow}");
        assert!(top_body_row(&wide).contains(preserved_glyph), "{wide}");
        assert!(
            top_body_row(&round_trip).contains(preserved_glyph),
            "{round_trip}"
        );
        assert_eq!(source.as_bytes().get(145..148), Some("严".as_bytes()));
    }

    #[test]
    fn resize_anchor_keeps_tab_separated_cjk_visible_without_measuring_controls() {
        let mut source = "header\n".repeat(4);
        let mut seed = 1_u64;
        for index in 0..380 {
            source.push(char::from_u32(0x4e00 + index).expect("valid CJK glyph"));
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if seed >> 60 == 0 {
                source.push('\t');
            }
        }

        let render_view = |width: u16, offset: u16| {
            let area = Rect::new(0, 0, width, 7);
            let mut buffer = Buffer::empty(area);
            render_wrapped_body(&mut buffer, area, "Tác vụ", &source, offset);
            buffer_text(&buffer, area)
        };
        let narrow_inner_width = 78;
        let wide_inner_width = 84;
        let narrow_offset = 5;
        let anchor = visual_row_start_byte(&source, narrow_inner_width, narrow_offset);
        let anchor_glyph = source[anchor..]
            .chars()
            .find(|glyph| ('\u{4e00}'..='\u{9fff}').contains(glyph))
            .expect("tab filtering leaves the row anchored to a CJK glyph");
        let narrow_limit = scroll_metrics_for_body(Rect::new(0, 0, 80, 7), &source).1;
        let wide_limit = scroll_metrics_for_body(Rect::new(0, 0, 86, 7), &source).1;
        let wide_offset = scroll_offset_at_anchor(&source, wide_inner_width, anchor, wide_limit);
        let return_offset =
            scroll_offset_at_anchor(&source, narrow_inner_width, anchor, narrow_limit);

        let narrow = render_view(80, narrow_offset);
        let wide = render_view(86, wide_offset);
        let round_trip = render_view(80, return_offset);
        let top_body_row = |view: &str| view.lines().nth(1).unwrap_or_default().to_owned();

        assert!(top_body_row(&narrow).contains(anchor_glyph), "{narrow}");
        assert!(top_body_row(&wide).contains(anchor_glyph), "{wide}");
        assert!(
            top_body_row(&round_trip).contains(anchor_glyph),
            "{round_trip}"
        );
        assert!(source.contains('\t'));
    }

    #[test]
    fn exit_focus_is_visible_as_a_keyboard_control() {
        let mut state = AppState::default();
        state.cycle_focus(false);

        let text = rendered_text(&state, 80, 24);

        assert!(text.contains("[ Thoát ]"));
        assert!(text.contains("Enter: Thoát"));
        assert!(text.contains("F1: Trợ giúp"));
        assert!(text.contains("Ctrl+C: Thoát"));
    }

    #[test]
    fn focused_route_selection_does_not_open_until_enter() {
        let mut state = AppState::default();
        state.move_selection(true);

        let text = rendered_text(&state, 80, 24);

        assert_eq!(state.current_screen(), Screen::Overview);
        assert!(text.lines().any(|line| line.contains("› Tập và chương")));
        assert!(text.contains("Chưa mở workspace"));
        assert!(!text.contains("Chưa có nguồn"));
    }

    #[test]
    fn help_overlay_is_contextual_for_all_routes_and_shows_only_real_actions() {
        for screen in Screen::ALL {
            for focus in [FocusRegion::Navigation, FocusRegion::Exit] {
                let mut state = AppState::default();
                state.navigate_to(screen);
                if focus == FocusRegion::Exit {
                    state.cycle_focus(false);
                }
                state.open_help();

                let text = normalized(&rendered_text(&state, 80, 24));

                assert!(
                    text.contains("Trợ giúp"),
                    "missing help title: {screen:?}/{focus:?}"
                );
                assert!(
                    text.contains(screen.title()),
                    "help title missing route: {screen:?}/{focus:?}"
                );
                for word in super::help_summary(screen).split_whitespace() {
                    assert!(
                        text.contains(word),
                        "help missing availability text `{word}` for {screen:?}/{focus:?}: {text}"
                    );
                }
                let focus_label = focus.label();
                assert!(
                    text.contains(&format!(
                        "Sau khi đóng trợ giúp bằng Esc, focus {focus_label} sẽ tiếp tục:"
                    )),
                    "help does not identify the restored focus: {screen:?}/{focus:?}: {text}"
                );
                let restored_actions = match focus {
                    FocusRegion::Navigation => {
                        "↑/↓: Chọn màn · Enter: Mở màn · Tab: Chuyển tới Thoát"
                    }
                    FocusRegion::Exit => "Enter: Thoát · Tab: Quay về Điều hướng",
                };
                assert!(
                    text.contains(restored_actions),
                    "help is missing restored actions: {screen:?}/{focus:?}: {text}"
                );
                assert!(text.contains("Esc: Đóng trợ giúp · Ctrl+C: Thoát"));
                assert!(!text.contains("F1 Trợ giúp"));
                assert!(!text.contains("Vùng đang chọn:"));
                assert!(!text.contains("P01-T09"));
                assert!(!text.contains("AC-09"));
            }
        }
    }

    #[test]
    fn help_remains_open_and_restores_the_same_context_after_resize() {
        let mut state = AppState::default();
        state.navigate_to(Screen::VolumesAndChapters);
        state.cycle_focus(false);
        state.open_help();
        let before = state;

        let narrow = rendered_text(&state, 80, 24);
        let resized = normalized(&rendered_text(&state, 120, 30));

        assert!(narrow.contains("Trợ giúp"));
        assert!(resized.contains("Tập và chương"));
        assert!(resized.contains("Esc: Đóng trợ giúp"));
        assert!(resized.contains("Sau khi đóng trợ giúp bằng Esc, focus Thoát sẽ tiếp tục:"));
        assert!(resized.contains("Enter: Thoát · Tab: Quay về Điều hướng"));
        assert_eq!(state, before);
        assert_eq!(state.current_screen(), Screen::VolumesAndChapters);
        assert_eq!(state.focus_region(), FocusRegion::Exit);
        assert!(state.help_open());
    }
}
