use gpui::{Context, FontWeight, Window, div, prelude::*, px, rgb};

use crate::network_module::HttpResponse;

// ── Color palette (mirrors mod.rs) ────────────────────────────────────────────
const C_DEEP: u32 = 0x0c0c1c;
const C_SURFACE: u32 = 0x131326;
const C_ELEVATED: u32 = 0x191932;
const C_BORDER_SUBTLE: u32 = 0x1a1a34;
const C_TEXT_DIM: u32 = 0x6868a0;
const C_TEXT_MUTED: u32 = 0x38385a;
const C_ACCENT: u32 = 0x7c6af5;

const STATUS_OK_FG: u32 = 0x4ade80;
const STATUS_OK_BG: u32 = 0x0c2a1a;
const STATUS_WARN_FG: u32 = 0xfbbf24;
const STATUS_WARN_BG: u32 = 0x2a1c00;
const STATUS_ERR_FG: u32 = 0xf87171;
const STATUS_ERR_BG: u32 = 0x2a0c0c;

// ── ResponsePanel ──────────────────────────────────────────────────────────────

pub struct ResponsePanel {
    pub loading: bool,
    pub response: Option<HttpResponse>,
    pub error: Option<String>,
    pub snippet: Option<(String, String)>, // (lang_label, code)
}

impl ResponsePanel {
    pub fn new() -> Self {
        Self {
            loading: false,
            response: None,
            error: None,
            snippet: None,
        }
    }
}

impl Render for ResponsePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let panel_title = if self.snippet.is_some() { "CODE SNIPPET" } else { "RESPONSE" };

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(C_DEEP))

            // ── Panel header ───────────────────────────────────────────────
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .bg(rgb(C_DEEP))
                    .border_b_1()
                    .border_color(rgb(C_BORDER_SUBTLE))
                    .child(
                        div()
                            .w(px(2.0))
                            .h(px(10.0))
                            .rounded_md()
                            .bg(rgb(C_ACCENT)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(C_TEXT_DIM))
                            .child(panel_title),
                    ),
            )

            // ── Panel body ─────────────────────────────────────────────────
            .child(if let Some((lang, code)) = &self.snippet {
                // ── Snippet view ───────────────────────────────────────────
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .child(
                        // Lang badge
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(0x1c1840))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(C_ACCENT))
                                    .child(lang.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(C_TEXT_MUTED))
                                    .child("click Send to dismiss"),
                            ),
                    )
                    .child(
                        // Code block
                        div()
                            .flex_1()
                            .p_3()
                            .bg(rgb(C_SURFACE))
                            .rounded_lg()
                            .border_1()
                            .border_color(rgb(C_BORDER_SUBTLE))
                            .text_sm()
                            .font_family("monospace")
                            .text_color(rgb(0xb8d4f0))
                            .child(code.clone()),
                    )

            } else if self.loading {
                // ── Loading ────────────────────────────────────────────────
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(C_TEXT_DIM))
                            .child("Sending request…"),
                    )

            } else if let Some(ref err) = self.error {
                // ── Error view ─────────────────────────────────────────────
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(STATUS_ERR_BG))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(STATUS_ERR_FG))
                                    .child("Error"),
                            ),
                    )
                    .child(
                        div()
                            .p_3()
                            .bg(rgb(C_SURFACE))
                            .rounded_lg()
                            .border_1()
                            .border_color(rgb(STATUS_ERR_BG))
                            .text_sm()
                            .font_family("monospace")
                            .text_color(rgb(STATUS_ERR_FG))
                            .child(err.clone()),
                    )

            } else if let Some(ref resp) = self.response {
                // ── Response view ──────────────────────────────────────────
                let status = resp.status;
                let (status_fg, status_bg) = if status < 300 {
                    (STATUS_OK_FG, STATUS_OK_BG)
                } else if status < 500 {
                    (STATUS_WARN_FG, STATUS_WARN_BG)
                } else {
                    (STATUS_ERR_FG, STATUS_ERR_BG)
                };

                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    // Status + duration bar
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(status_bg))
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(status_fg))
                                    .child(status.to_string()),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(C_ELEVATED))
                                    .text_xs()
                                    .text_color(rgb(C_TEXT_DIM))
                                    .child(format!("{} ms", resp.duration_ms)),
                            ),
                    )
                    // Response body
                    .child(
                        div()
                            .flex_1()
                            .p_3()
                            .bg(rgb(C_SURFACE))
                            .rounded_lg()
                            .border_1()
                            .border_color(rgb(C_BORDER_SUBTLE))
                            .text_sm()
                            .font_family("monospace")
                            .text_color(rgb(0xaaccbb))
                            .child(resp.body.clone()),
                    )

            } else {
                // ── Empty state ────────────────────────────────────────────
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(C_TEXT_MUTED))
                            .child("No response yet"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x28283a))
                            .child("Press Send to execute the request"),
                    )
            })
    }
}
