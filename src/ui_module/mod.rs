mod headers_editor;
mod response_panel;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui::{ClickEvent, Context, Entity, FontWeight, SharedString, Window, div, prelude::*, px, rgb};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};

use crate::network_module::{self, HttpRequest};
use crate::snippet_module::{self, SnippetLang};
use crate::storage_module::{self, CollectionNode, SavedRequest};
use headers_editor::HeadersEditor;
use response_panel::ResponsePanel;

// ── Color palette ─────────────────────────────────────────────────────────────
// Backgrounds — layered from deep (sidebar/panel) to surface (editor) to elevated (bars)
const C_DEEP: u32 = 0x0c0c1c;        // sidebar, response panel
const C_SURFACE: u32 = 0x131326;     // editor main area
const C_ELEVATED: u32 = 0x191932;    // tab bar, request bar
const C_HOVER: u32 = 0x1c1c38;       // hover state

// Borders
const C_BORDER_SUBTLE: u32 = 0x1a1a34;
const C_BORDER: u32 = 0x252548;

// Text
const C_TEXT_BRIGHT: u32 = 0xe2e0ff;
const C_TEXT_DIM: u32 = 0x6868a0;
const C_TEXT_MUTED: u32 = 0x38385a;

// Accent (indigo-purple)
const C_ACCENT: u32 = 0x7c6af5;

// HTTP method colors  (fg, bg)
const GET_FG: u32 = 0x58a6ff;
const GET_BG: u32 = 0x0c1c38;
const POST_FG: u32 = 0x4ecb8a;
const POST_BG: u32 = 0x0c2a1a;
const PUT_FG: u32 = 0xf5a020;
const PUT_BG: u32 = 0x281800;
const DEL_FG: u32 = 0xff6565;
const DEL_BG: u32 = 0x280c0c;

// ── HTTP method ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    fn label(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => HttpMethod::Get,
        }
    }
}

fn method_colors_for(method_str: &str) -> u32 {
    match method_str {
        "POST" => POST_FG,
        "PUT" => PUT_FG,
        "DELETE" => DEL_FG,
        _ => GET_FG,
    }
}

// ── TabState ──────────────────────────────────────────────────────────────────

pub struct TabState {
    pub label: String,
    pub method: HttpMethod,
    pub url_input: Entity<InputState>,
    pub headers_editor: Entity<HeadersEditor>,
    pub body_input: Entity<InputState>,
    pub response_panel: Entity<ResponsePanel>,
}

impl TabState {
    fn new(window: &mut Window, cx: &mut Context<AppView>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("https://api.example.com/resource")
        });
        let headers_editor = cx.new(|cx| HeadersEditor::new(window, cx));
        let body_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .placeholder("// JSON request body")
        });
        let response_panel = cx.new(|_cx| ResponsePanel::new());
        Self {
            label: "New Tab".to_string(),
            method: HttpMethod::Get,
            url_input,
            headers_editor,
            body_input,
            response_panel,
        }
    }
}

// ── Sidebar tree helpers ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum SidebarKind {
    Folder { expanded: bool },
    Request { method: String },
}

#[derive(Clone)]
struct SidebarItem {
    name: String,
    path: PathBuf,
    kind: SidebarKind,
    depth: usize,
}

fn flatten_visible(
    nodes: &[CollectionNode],
    depth: usize,
    expanded: &HashSet<PathBuf>,
    out: &mut Vec<SidebarItem>,
) {
    for node in nodes {
        match node {
            CollectionNode::Folder { name, path, children } => {
                let is_expanded = expanded.contains(path);
                out.push(SidebarItem {
                    name: name.clone(),
                    path: path.clone(),
                    kind: SidebarKind::Folder { expanded: is_expanded },
                    depth,
                });
                if is_expanded {
                    flatten_visible(children, depth + 1, expanded, out);
                }
            }
            CollectionNode::Request { name, path, method } => {
                out.push(SidebarItem {
                    name: name.clone(),
                    path: path.clone(),
                    kind: SidebarKind::Request { method: method.clone() },
                    depth,
                });
            }
        }
    }
}

// ── AppView ───────────────────────────────────────────────────────────────────

pub struct AppView {
    tabs: Vec<TabState>,
    active_tab: usize,
    collection_dir: PathBuf,
    tree: Vec<CollectionNode>,
    expanded: HashSet<PathBuf>,
    active_env: HashMap<String, String>,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let collection_dir = storage_module::default_collection_dir();
        let tree = storage_module::load_collection_tree(&storage_module::makako_root_dir());
        let active_env = storage_module::load_env(&collection_dir);
        Self {
            tabs: vec![TabState::new(window, cx)],
            active_tab: 0,
            collection_dir,
            tree,
            expanded: HashSet::new(),
            active_env,
        }
    }

    fn refresh_tree(&mut self) {
        self.tree = storage_module::load_collection_tree(&storage_module::makako_root_dir());
    }

    fn visible_sidebar_items(&self) -> Vec<SidebarItem> {
        let mut items = vec![];
        flatten_visible(&self.tree, 0, &self.expanded, &mut items);
        items
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active_tab;
        let method = self.tabs[active].method;

        // ── Tab bar listeners ─────────────────────────────────────────────────
        let tab_count = self.tabs.len();
        let tab_listeners: Vec<_> = (0..tab_count)
            .map(|i| {
                cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.active_tab = i;
                    cx.notify();
                })
            })
            .collect();

        let on_new_tab = cx.listener(|this, _: &ClickEvent, window, cx| {
            this.tabs.push(TabState::new(window, cx));
            this.active_tab = this.tabs.len() - 1;
            cx.notify();
        });

        // ── Method listeners ─────────────────────────────────────────────────
        let on_get = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.tabs[this.active_tab].method = HttpMethod::Get;
            cx.notify();
        });
        let on_post = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.tabs[this.active_tab].method = HttpMethod::Post;
            cx.notify();
        });
        let on_put = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.tabs[this.active_tab].method = HttpMethod::Put;
            cx.notify();
        });
        let on_delete = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.tabs[this.active_tab].method = HttpMethod::Delete;
            cx.notify();
        });

        // ── Send ─────────────────────────────────────────────────────────────
        let on_send = cx.listener(|this, _: &ClickEvent, _, cx| {
            let active = this.active_tab;
            let env = &this.active_env;

            let url = storage_module::interpolate(
                &this.tabs[active].url_input.read(cx).value(),
                env,
            );
            let method = this.tabs[active].method.label().to_string();
            let headers = this.tabs[active]
                .headers_editor
                .read(cx)
                .headers(cx)
                .into_iter()
                .map(|(k, v)| (k, storage_module::interpolate(&v, env)))
                .collect();
            let body = {
                let raw = this.tabs[active].body_input.read(cx).value().to_string();
                let b = storage_module::interpolate(&raw, env);
                if b.trim().is_empty() { None } else { Some(b) }
            };

            let req = HttpRequest { method, url, headers, body };

            this.tabs[active].response_panel.update(cx, |panel, cx| {
                panel.loading = true;
                panel.response = None;
                panel.error = None;
                panel.snippet = None;
                cx.notify();
            });

            cx.spawn(async move |view, async_cx| {
                let (tx, rx) = futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(network_module::execute(req));
                });

                let result = rx.await.unwrap_or_else(|_| Err("thread panicked".to_string()));

                view.update(async_cx, |this, cx| {
                    this.tabs[active].response_panel.update(cx, |panel, cx| {
                        panel.loading = false;
                        match result {
                            Ok(resp) => {
                                panel.response = Some(resp);
                                panel.error = None;
                            }
                            Err(e) => {
                                panel.error = Some(e);
                                panel.response = None;
                            }
                        }
                        cx.notify();
                    });
                })
                .ok();
            })
            .detach();
        });

        // ── Save ─────────────────────────────────────────────────────────────
        let on_save = cx.listener(|this, _: &ClickEvent, _, cx| {
            let active = this.active_tab;
            let url = this.tabs[active].url_input.read(cx).value().to_string();
            let name = url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("Untitled")
                .to_string();

            let req = SavedRequest {
                name: name.clone(),
                method: this.tabs[active].method.label().to_string(),
                url,
                headers: this.tabs[active].headers_editor.read(cx).headers(cx),
                body: this.tabs[active].body_input.read(cx).value().to_string(),
            };

            let dir = this.collection_dir.clone();
            match storage_module::save_request(&dir, &req) {
                Ok(_) => {
                    this.tabs[active].label = name;
                    this.refresh_tree();
                }
                Err(e) => eprintln!("[Makako] save error: {e}"),
            }
            cx.notify();
        });

        // ── Snippet listeners ─────────────────────────────────────────────────
        let make_snippet_listener = |lang: SnippetLang| {
            cx.listener(move |this, _: &ClickEvent, _, cx| {
                let active = this.active_tab;
                let env = &this.active_env;
                let url = storage_module::interpolate(
                    &this.tabs[active].url_input.read(cx).value(),
                    env,
                );
                let method = this.tabs[active].method.label().to_string();
                let headers: Vec<(String, String)> = this.tabs[active]
                    .headers_editor
                    .read(cx)
                    .headers(cx)
                    .into_iter()
                    .map(|(k, v)| (k, storage_module::interpolate(&v, env)))
                    .collect();
                let body_raw = this.tabs[active].body_input.read(cx).value().to_string();
                let body_interp = storage_module::interpolate(&body_raw, env);
                let body = if body_interp.trim().is_empty() { None } else { Some(body_interp) };

                let (label, code) = snippet_module::generate(
                    lang,
                    &method,
                    &url,
                    &headers,
                    body.as_deref(),
                );
                this.tabs[active].response_panel.update(cx, |panel, cx| {
                    panel.snippet = Some((label, code));
                    cx.notify();
                });
            })
        };
        let on_curl = make_snippet_listener(SnippetLang::Curl);
        let on_js = make_snippet_listener(SnippetLang::Fetch);
        let on_rs = make_snippet_listener(SnippetLang::Reqwest);

        // ── Sidebar listeners ─────────────────────────────────────────────────
        let items = self.visible_sidebar_items();

        let sidebar_listeners: Vec<_> = items
            .iter()
            .map(|item| {
                let path = item.path.clone();
                let is_folder = matches!(item.kind, SidebarKind::Folder { .. });
                cx.listener(move |this, _: &ClickEvent, window, cx| {
                    if is_folder {
                        if this.expanded.contains(&path) {
                            this.expanded.remove(&path);
                        } else {
                            this.expanded.insert(path.clone());
                        }
                        cx.notify();
                    } else {
                        let Ok(req) = storage_module::load_request(&path) else {
                            return;
                        };
                        if let Some(parent) = path.parent() {
                            this.active_env = storage_module::load_env(parent);
                        }
                        let active = this.active_tab;
                        this.tabs[active].label = req.name.clone();
                        this.tabs[active].method = HttpMethod::from_str(&req.method);
                        this.tabs[active]
                            .url_input
                            .update(cx, |s, cx| s.set_value(req.url, window, cx));
                        this.tabs[active]
                            .body_input
                            .update(cx, |s, cx| s.set_value(req.body, window, cx));
                        this.tabs[active]
                            .headers_editor
                            .update(cx, |he, cx| he.load_headers(req.headers, window, cx));
                        cx.notify();
                    }
                })
            })
            .collect();

        // Snapshot entity handles for the active tab before building the tree.
        let url_input = self.tabs[active].url_input.clone();
        let headers_editor = self.tabs[active].headers_editor.clone();
        let body_input = self.tabs[active].body_input.clone();
        let response_panel = self.tabs[active].response_panel.clone();

        let tab_labels: Vec<String> = self.tabs.iter().map(|t| t.label.clone()).collect();

        // ── Layout ───────────────────────────────────────────────────────────
        div()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .bg(rgb(C_DEEP))

            // ── Sidebar ────────────────────────────────────────────────────
            .child(
                div()
                    .w(px(220.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .bg(rgb(C_DEEP))
                    .border_r_1()
                    .border_color(rgb(C_BORDER_SUBTLE))

                    // Sidebar header
                    .child(
                        div()
                            .px_4()
                            .pt_4()
                            .pb_3()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .border_b_1()
                            .border_color(rgb(C_BORDER_SUBTLE))
                            .child(
                                div()
                                    .w(px(6.0))
                                    .h(px(6.0))
                                    .rounded_full()
                                    .bg(rgb(C_ACCENT)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(C_TEXT_DIM))
                                    .child("COLLECTIONS"),
                            ),
                    )

                    // Sidebar items
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .pt_2()
                            .children(
                                items.iter().zip(sidebar_listeners).enumerate().map(
                                    |(i, (item, on_click))| {
                                        let indent = px(10.0 + item.depth as f32 * 14.0);
                                        match &item.kind {
                                            SidebarKind::Folder { expanded } => {
                                                let icon = if *expanded { "▾" } else { "▸" };
                                                div()
                                                    .id(("tree-item", i))
                                                    .flex()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap_2()
                                                    .pl(indent)
                                                    .pr_3()
                                                    .py_1()
                                                    .mx_2()
                                                    .rounded_md()
                                                    .cursor_pointer()
                                                    .hover(|s| s.bg(rgb(C_HOVER)))
                                                    .on_click(on_click)
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(C_TEXT_DIM))
                                                            .child(icon),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_sm()
                                                            .text_color(rgb(C_TEXT_BRIGHT))
                                                            .font_weight(FontWeight::BOLD)
                                                            .child(SharedString::from(
                                                                item.name.clone(),
                                                            )),
                                                    )
                                            }
                                            SidebarKind::Request { method: m } => {
                                                let method_color = method_colors_for(m);
                                                div()
                                                    .id(("tree-item", i))
                                                    .flex()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap_2()
                                                    .pl(indent)
                                                    .pr_3()
                                                    .py_1()
                                                    .mx_2()
                                                    .rounded_md()
                                                    .cursor_pointer()
                                                    .hover(|s| s.bg(rgb(C_HOVER)))
                                                    .on_click(on_click)
                                                    .child(
                                                        // Method badge
                                                        div()
                                                            .px_1()
                                                            .rounded_md()
                                                            .text_color(rgb(method_color))
                                                            .text_xs()
                                                            .font_weight(FontWeight::BOLD)
                                                            .child(SharedString::from(m.clone())),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_sm()
                                                            .text_color(rgb(0xaaaacc))
                                                            .child(SharedString::from(
                                                                item.name.clone(),
                                                            )),
                                                    )
                                            }
                                        }
                                    },
                                ),
                            ),
                    ),
            )

            // ── Central editor ─────────────────────────────────────────────
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .bg(rgb(C_SURFACE))

                    // ── Tab bar ───────────────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .bg(rgb(C_ELEVATED))
                            .border_b_1()
                            .border_color(rgb(C_BORDER_SUBTLE))
                            .children(
                                tab_labels
                                    .into_iter()
                                    .zip(tab_listeners)
                                    .enumerate()
                                    .map(|(i, (label, on_click))| {
                                        let is_active = i == active;
                                        div()
                                            .id(("tab-btn", i))
                                            .px_4()
                                            .py_2()
                                            .cursor_pointer()
                                            .text_sm()
                                            .border_b_2()
                                            .when(is_active, |s| {
                                                s.border_color(rgb(C_ACCENT))
                                                    .text_color(rgb(C_TEXT_BRIGHT))
                                                    .bg(rgb(C_SURFACE))
                                            })
                                            .when(!is_active, |s| {
                                                s.border_color(rgb(0x00000000))
                                                    .text_color(rgb(C_TEXT_MUTED))
                                                    .hover(|s| {
                                                        s.bg(rgb(C_HOVER))
                                                            .text_color(rgb(C_TEXT_DIM))
                                                    })
                                            })
                                            .on_click(on_click)
                                            .child(SharedString::from(label))
                                    }),
                            )
                            .child(
                                div()
                                    .id("btn-new-tab")
                                    .w(px(36.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_color(rgb(C_TEXT_MUTED))
                                    .hover(|s| {
                                        s.text_color(rgb(C_TEXT_DIM)).bg(rgb(C_HOVER))
                                    })
                                    .on_click(on_new_tab)
                                    .child("+"),
                            ),
                    )

                    // ── Request bar ───────────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .bg(rgb(C_ELEVATED))
                            .border_b_1()
                            .border_color(rgb(C_BORDER_SUBTLE))

                            // Method selector — colored pill buttons
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .p_1()
                                    .bg(rgb(C_DEEP))
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(rgb(C_BORDER_SUBTLE))
                                    .child(
                                        div()
                                            .id("m-get")
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .when(method == HttpMethod::Get, |s| {
                                                s.bg(rgb(GET_BG)).text_color(rgb(GET_FG))
                                            })
                                            .when(method != HttpMethod::Get, |s| {
                                                s.text_color(rgb(C_TEXT_MUTED)).hover(|s| {
                                                    s.text_color(rgb(GET_FG))
                                                        .bg(rgb(GET_BG))
                                                })
                                            })
                                            .on_click(on_get)
                                            .child("GET"),
                                    )
                                    .child(
                                        div()
                                            .id("m-post")
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .when(method == HttpMethod::Post, |s| {
                                                s.bg(rgb(POST_BG)).text_color(rgb(POST_FG))
                                            })
                                            .when(method != HttpMethod::Post, |s| {
                                                s.text_color(rgb(C_TEXT_MUTED)).hover(|s| {
                                                    s.text_color(rgb(POST_FG))
                                                        .bg(rgb(POST_BG))
                                                })
                                            })
                                            .on_click(on_post)
                                            .child("POST"),
                                    )
                                    .child(
                                        div()
                                            .id("m-put")
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .when(method == HttpMethod::Put, |s| {
                                                s.bg(rgb(PUT_BG)).text_color(rgb(PUT_FG))
                                            })
                                            .when(method != HttpMethod::Put, |s| {
                                                s.text_color(rgb(C_TEXT_MUTED)).hover(|s| {
                                                    s.text_color(rgb(PUT_FG))
                                                        .bg(rgb(PUT_BG))
                                                })
                                            })
                                            .on_click(on_put)
                                            .child("PUT"),
                                    )
                                    .child(
                                        div()
                                            .id("m-del")
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .when(method == HttpMethod::Delete, |s| {
                                                s.bg(rgb(DEL_BG)).text_color(rgb(DEL_FG))
                                            })
                                            .when(method != HttpMethod::Delete, |s| {
                                                s.text_color(rgb(C_TEXT_MUTED)).hover(|s| {
                                                    s.text_color(rgb(DEL_FG))
                                                        .bg(rgb(DEL_BG))
                                                })
                                            })
                                            .on_click(on_delete)
                                            .child("DEL"),
                                    ),
                            )

                            // URL input
                            .child(div().flex_1().child(Input::new(&url_input)))

                            // Save + Send
                            .child(
                                Button::new("btn-save")
                                    .label("Save")
                                    .ghost()
                                    .on_click(on_save),
                            )
                            .child(
                                Button::new("btn-send")
                                    .label("Send")
                                    .primary()
                                    .on_click(on_send),
                            )

                            // Snippet buttons
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .pl_2()
                                    .border_l_1()
                                    .border_color(rgb(C_BORDER))
                                    .child(
                                        Button::new("btn-curl")
                                            .label("cURL")
                                            .ghost()
                                            .on_click(on_curl),
                                    )
                                    .child(
                                        Button::new("btn-js")
                                            .label("JS")
                                            .ghost()
                                            .on_click(on_js),
                                    )
                                    .child(
                                        Button::new("btn-rs")
                                            .label("RS")
                                            .ghost()
                                            .on_click(on_rs),
                                    ),
                            ),
                    )

                    // ── Headers section ───────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .border_b_1()
                            .border_color(rgb(C_BORDER_SUBTLE))
                            // Section header
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(C_DEEP))
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
                                            .child("HEADERS"),
                                    ),
                            )
                            .child(headers_editor),
                    )

                    // ── Body section ──────────────────────────────────────
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            // Section header
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
                                            .child("BODY"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px_3()
                                    .py_3()
                                    .child(Input::new(&body_input).h_full()),
                            ),
                    ),
            )

            // ── Response panel ─────────────────────────────────────────────
            .child(
                div()
                    .w(px(380.0))
                    .h_full()
                    .border_l_1()
                    .border_color(rgb(C_BORDER_SUBTLE))
                    .child(response_panel),
            )
    }
}
