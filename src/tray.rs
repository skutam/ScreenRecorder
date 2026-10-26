use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{Icon, TrayIconBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use crate::config::ConfigHolder;
use crate::DefaultError;
use crate::recorder::{Recorder};

pub struct TrayMenu {
    // Menu
    menu: Menu,
    // Menu items
    show: MenuItem,
    reload: MenuItem,
    start_recording: MenuItem,
    stop_recording: MenuItem,
    quit: MenuItem,
}

impl TrayMenu {
    pub fn new() -> Result<Self, DefaultError> {
        let menu = Menu::new();

        let show = MenuItem::new("Zobraziť konfiguráciu v konzole", true, None);
        let reload = MenuItem::new("Obnovit konfiguráciu", true, None);
        let start_recording = MenuItem::new("Začať nahrávať", true, None);
        let stop_recording = MenuItem::new("Zastaviť nahrávku", true, None);
        let quit = MenuItem::new("Ukončiť", true, None);

        menu.append_items(&[&show, &reload, &start_recording, &stop_recording, &quit])?;

        Ok(Self{menu, show, reload, start_recording, stop_recording, quit})
    }

    pub fn menu_loop(self, mut config: ConfigHolder, mut recorder: Recorder) -> Result<(), DefaultError> {
        let icon = Icon::from_rgba([40, 140, 220, 255].repeat(16 * 16), 16, 16)?;
        let event_loop = EventLoopBuilder::<MenuEvent>::with_user_event().build();

        // Click will wake up menu
        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(event);
        }));

        let mut tray = None;
        event_loop.run(move |event, _, control_flow| {
            // Wait on event, otherwise do nothing
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    match TrayIconBuilder::new()
                        .with_menu(Box::new(self.menu.clone()))
                        .with_tooltip("Replay Recorder - demo, nenahráva")
                        .with_icon(icon.clone())
                        .build()
                    {
                        Ok(created) => tray = Some(created),
                        Err(error) => {
                            eprintln!("Vytvorenie tray ikony zlyhalo: {error}");
                            *control_flow = ControlFlow::ExitWithCode(1);
                        }
                    }
                }
                Event::UserEvent(event) => {
                    self.handle_event(event, control_flow, &mut config, &mut recorder);
                }
                Event::LoopDestroyed => {
                    drop(tray.take());
                }
                _ => {}
            }
        });
    }

    fn handle_event(&self, event: MenuEvent, control_flow: &mut ControlFlow, config: &mut ConfigHolder, recorder: &mut Recorder) {
        if event.id == self.show.id() {
            println!("{:#?}", config.get_config());
        } else if event.id == self.reload.id() {
            match config.reload_config() {
                Ok(()) => println!("Reloaded config"),
                Err(error) => {
                    eprintln!("Reloading config: {error}");
                }
            }
        } else if event.id == self.start_recording.id() {
            match recorder.start() {
                Ok(()) => println!("Started recording"),
                Err(error) => eprintln!("Recording error: {error}"),
            }
        } else if event.id == self.stop_recording.id() {
            match recorder.stop() {
                Ok(()) => println!("Stopped recording"),
                Err(error) => eprintln!("Recording error: {error}"),
            }
        } else if event.id == self.quit.id() {
            match recorder.shutdown() {
                Ok(()) => println!("Shutting down recording"),
                Err(error) => eprintln!("Recording error: {error}"),
            }
            *control_flow = ControlFlow::Exit;
        }
    }
}
