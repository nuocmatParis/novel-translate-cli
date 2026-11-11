use ratatui::style::{Color, Modifier, Style};

/// Centralized color palette and styling for the modern developer TUI.
/// Default theme: Dark Navy / Slate with Cyan accents.
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub sidebar_bg: Color,
    pub panel_bg: Color,
    pub panel_border: Color,
    pub panel_border_focused: Color,

    pub primary: Color,
    pub primary_dim: Color,
    pub secondary: Color,
    pub accent: Color,

    pub text: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub text_inverted: Color,

    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    pub selection_bg: Color,
    pub selection_fg: Color,
    pub cursor_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Modern Deep Slate/Navy
            bg: Color::Rgb(15, 23, 42),                     // slate-900
            sidebar_bg: Color::Rgb(11, 17, 33),             // slate-950/deep navy
            panel_bg: Color::Rgb(20, 30, 54),               // slate-850
            panel_border: Color::Rgb(51, 65, 85),           // slate-700
            panel_border_focused: Color::Rgb(56, 189, 248), // sky-400 / bright cyan

            primary: Color::Rgb(56, 189, 248),     // sky-400
            primary_dim: Color::Rgb(14, 116, 144), // cyan-700
            secondary: Color::Rgb(168, 85, 247),   // purple-500
            accent: Color::Rgb(244, 114, 182),     // pink-400

            text: Color::Rgb(241, 245, 249),       // slate-100
            text_muted: Color::Rgb(148, 163, 184), // slate-400
            text_dim: Color::Rgb(100, 116, 139),   // slate-500
            text_inverted: Color::Rgb(15, 23, 42),

            success: Color::Rgb(52, 211, 153), // emerald-400
            warning: Color::Rgb(251, 191, 36), // amber-400
            error: Color::Rgb(248, 113, 113),  // rose-400
            info: Color::Rgb(96, 165, 250),    // blue-400

            selection_bg: Color::Rgb(30, 58, 102), // dark blue highlight
            selection_fg: Color::Rgb(255, 255, 255),
            cursor_style: Style::default()
                .bg(Color::Rgb(56, 189, 248))
                .fg(Color::Rgb(15, 23, 42))
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn subtitle_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn header_style(&self) -> Style {
        Style::default()
            .bg(self.sidebar_bg)
            .fg(self.text)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_bar_style(&self) -> Style {
        Style::default().bg(self.sidebar_bg).fg(self.text)
    }

    pub fn footer_style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.text_dim)
    }

    pub fn key_badge_style(&self) -> Style {
        Style::default()
            .bg(self.panel_border)
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .fg(self.selection_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn active_tab_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .bg(self.panel_bg)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }

    pub fn inactive_tab_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    pub fn badge_style(&self, bg: Color, fg: Color) -> Style {
        Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD)
    }
}
