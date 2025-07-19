use gpui::{div, prelude::*, px, rgb, Context, Window};

/// Root application view — renders the 3-panel shell:
///   ┌──────────┬──────────────────┬──────────────┐
///   │ Sidebar  │  Request Editor  │   Response   │
///   │(240 px)  │   (flex: 1)      │  (400 px)    │
///   └──────────┴──────────────────┴──────────────┘
pub struct AppView;

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            // ── Sidebar ────────────────────────────────────────────
            .child(
                div()
                    .w(px(240.0))
                    .h_full()
                    .bg(rgb(0x1a1a2e)) // dark navy — collections list
                    .p_4()
                    .text_color(rgb(0x8888aa))
                    .child("Colecciones"),
            )
            // ── Central request editor ─────────────────────────────
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .bg(rgb(0x24243e)) // deep purple-grey — request config
                    .p_4()
                    .text_color(rgb(0xccccee))
                    .child("Editor de Petición"),
            )
            // ── Response panel ─────────────────────────────────────
            .child(
                div()
                    .w(px(420.0))
                    .h_full()
                    .bg(rgb(0x0f3460)) // midnight blue — response viewer
                    .p_4()
                    .text_color(rgb(0x88bbdd))
                    .child("Respuesta"),
            )
    }
}
