mod network_module;
mod snippet_module;
mod storage_module;
mod ui_module;

use gpui::{
    TitlebarOptions, px, point, size, App, AppContext, Application, Bounds, WindowBounds,
    WindowOptions,
};
use gpui_component::Root;
use ui_module::AppView;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    // Position the traffic-light buttons 12px from left, 12px from top.
                    traffic_light_position: Some(point(px(12.0), px(12.0))),
                }),
                ..Default::default()
            },
            |window, cx| {
                let app_view = cx.new(|cx| AppView::new(window, cx));
                cx.new(|cx| Root::new(app_view, window, cx))
            },
        )
        .unwrap();
    });
}
