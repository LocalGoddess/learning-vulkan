use vulkan::vulkan_instance::VulkanInstance;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    raw_window_handle::HasDisplayHandle,
    window::{Window, WindowAttributes},
};

pub mod util;
pub mod vulkan;

#[derive(Default)]
pub struct AppState {
    pub window: Option<Window>,
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        tracing::info!("Creating window");

        let attributes = WindowAttributes::default()
            .with_title("Learning Vulkan")
            .with_resizable(false);

        self.window = Some(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window"),
        );

        let _vulkan_instance = VulkanInstance::create(
            self.window
                .as_ref()
                .unwrap()
                .display_handle()
                .unwrap()
                .as_raw(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close request recieved on window with {window_id:?}");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Draw frame here
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
