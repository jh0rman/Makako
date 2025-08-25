use gpui::{App, ClickEvent, Context, Entity, FontWeight, Window, div, prelude::*, px, rgb};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

const C_DEEP: u32 = 0x0c0c1c;
const C_TEXT_MUTED: u32 = 0x38385a;
const C_BORDER_SUBTLE: u32 = 0x1a1a34;

// ── HeaderRow ─────────────────────────────────────────────────────────────────

struct HeaderRow {
    key: Entity<InputState>,
    value: Entity<InputState>,
}

impl HeaderRow {
    fn new(window: &mut Window, cx: &mut Context<HeadersEditor>) -> Self {
        let key = cx.new(|cx| InputState::new(window, cx).placeholder("Header name"));
        let value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));
        Self { key, value }
    }
}

// ── HeadersEditor ─────────────────────────────────────────────────────────────

pub struct HeadersEditor {
    rows: Vec<HeaderRow>,
}

impl HeadersEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            rows: vec![HeaderRow::new(window, cx)],
        }
    }

    pub fn load_headers(
        &mut self,
        pairs: Vec<(String, String)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows = pairs
            .into_iter()
            .map(|(k, v)| {
                let row = HeaderRow::new(window, cx);
                row.key.update(cx, |s, cx| s.set_value(k, window, cx));
                row.value.update(cx, |s, cx| s.set_value(v, window, cx));
                row
            })
            .collect();

        if self.rows.is_empty() {
            self.rows.push(HeaderRow::new(window, cx));
        }
        cx.notify();
    }

    pub fn headers(&self, cx: &App) -> Vec<(String, String)> {
        self.rows
            .iter()
            .map(|r| {
                (
                    r.key.read(cx).value().to_string(),
                    r.value.read(cx).value().to_string(),
                )
            })
            .filter(|(k, _)| !k.trim().is_empty())
            .collect()
    }
}

impl Render for HeadersEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on_add = cx.listener(|this, _: &ClickEvent, window, cx| {
            this.rows.push(HeaderRow::new(window, cx));
            cx.notify();
        });

        let remove_listeners: Vec<_> = (0..self.rows.len())
            .map(|i| {
                cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.rows.remove(i);
                    cx.notify();
                })
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .bg(rgb(C_DEEP))

            // Column labels
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_1()
                    .border_b_1()
                    .border_color(rgb(C_BORDER_SUBTLE))
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(C_TEXT_MUTED))
                            .child("KEY"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(C_TEXT_MUTED))
                            .child("VALUE"),
                    )
                    .child(div().w(px(28.0))),
            )

            // Rows
            .children(
                self.rows
                    .iter()
                    .zip(remove_listeners)
                    .enumerate()
                    .map(|(i, (row, on_remove))| {
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1()
                            .border_b_1()
                            .border_color(rgb(C_BORDER_SUBTLE))
                            .child(div().flex_1().child(Input::new(&row.key)))
                            .child(div().flex_1().child(Input::new(&row.value)))
                            .child(
                                Button::new(("header-del", i))
                                    .label("×")
                                    .ghost()
                                    .on_click(on_remove),
                            )
                    }),
            )

            // Add row button
            .child(
                div()
                    .px_3()
                    .py_1()
                    .child(
                        Button::new("btn-add-header")
                            .label("+ Add Header")
                            .ghost()
                            .on_click(on_add),
                    ),
            )
    }
}
