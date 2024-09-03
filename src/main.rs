use vulkan::VulkanManager;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowAttributes},
};

pub mod util;
pub mod vulkan;

#[derive(Default)]
pub struct AppState {
    window: Option<Window>,
    vulkan_manager: Option<VulkanManager>,
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        tracing::info!("Creating window");

        let window = event_loop
            .create_window(WindowAttributes::default())
            .expect("Failed to create window");
        window.set_title("Learning Vulkan");

        self.window = Some(window);
        self.vulkan_manager = Some(VulkanManager::new(
            self.window.as_ref().unwrap().inner_size().width,
            self.window.as_ref().unwrap().inner_size().height,
            self.window
                .as_ref()
                .unwrap()
                .display_handle()
                .expect("Failed to get the raw display handle")
                .as_raw(),
            self.window
                .as_ref()
                .unwrap()
                .window_handle()
                .expect("Failed to get the raw window handle")
                .as_raw(),
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested on {:?}", window_id);

                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Drawing
                self.vulkan_manager.as_ref().unwrap().draw_frame();
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Initializing application");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app_state = AppState::default();

    let _ = event_loop.run_app(&mut app_state);
}
