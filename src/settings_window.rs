use tao::dpi::{LogicalSize, PhysicalPosition};
use tao::event::WindowEvent;
use tao::window::{Window, WindowBuilder, WindowId};

pub struct SettingsWindow {
    window: Option<Window>,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self { window: None }
    }

    pub fn handle_event(&mut self, window_id: WindowId, event: &WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            if let Some(window) = self.window.as_ref() {
                if window.id() == window_id {
                    self.close();
                }
            }
        }
    }

    pub fn open<T: 'static>(&mut self, target: &tao::event_loop::EventLoopWindowTarget<T>) -> Result<(), tao::error::OsError> {
        // Focus on the window when it is already open
        if let Some(window) = self.window.as_ref() {
            window.set_visible(true);
            window.set_minimized(false);
            window.set_focus();
        // Create new window
        } else {
            let window = WindowBuilder::new()
                .with_title("Replay recorder - Nastavenia")
                .with_inner_size(LogicalSize::new(640.0, 480.0))
                .with_visible(false)
                .build(target)?;

            if let Some(monitor) = window.primary_monitor() {
                let monitor_position = monitor.position();
                let monitor_size = monitor.size();
                let window_size = window.outer_size();

                let x = monitor_position.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
                let y = monitor_position.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

                window.set_outer_position(PhysicalPosition::new(x, y));
            }

            window.set_visible(true);
            window.set_focus();
            self.window = Some(window);
        }
        Ok(())
    }

    pub fn close(&mut self) {
        self.window = None;
    }
}