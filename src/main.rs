mod network_module;
mod storage_module;
mod ui_module;

use gpui::{px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use ui_module::AppView;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AppView),
        )
        .unwrap();
    });
}
