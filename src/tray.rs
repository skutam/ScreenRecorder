use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{Icon, TrayIconBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use crate::config::ConfigHolder;
use crate::DefaultError;
use crate::recorder::{Recorder, RecorderState};

enum AppEvent {
    Menu(MenuEvent),
    RecorderStateChanged(RecorderState),
}

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

    pub fn menu_loop(self, mut config: ConfigHolder) -> Result<(), DefaultError> {
        let debug_tray = std::env::var_os("REPLAY_TRAY_DEBUG").is_some();
        let idle_icon = Icon::from_rgba([40, 140, 220, 255].repeat(16 * 16), 16, 16)?;
        let recording_icon = Icon::from_rgba([220, 40, 40, 255].repeat(16 * 16), 16, 16)?;
        let error_icon = Icon::from_rgba([120, 40, 40, 255].repeat(16 * 16), 16, 16)?;
        let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();

        // Click will wake up menu
        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(AppEvent::Menu(event));
        }));

        // Second proxy to handle response of state change
        let recorder_proxy = event_loop.create_proxy();
        let mut recorder = Recorder::new(move |state| {
            let _ = recorder_proxy.send_event(AppEvent::RecorderStateChanged(state));
        });

        let mut tray = None;
        event_loop.run(move |event, _, control_flow| {
            // Wait on event, otherwise do nothing
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    match TrayIconBuilder::new()
                        .with_menu(Box::new(self.menu.clone()))
                        .with_tooltip("Replay Recorder - demo, nenahráva")
                        .with_icon(idle_icon.clone())
                        .build()
                    {
                        Ok(created) => {
                            if debug_tray {
                                eprintln!("[tray-debug] Created, shell rect: {:?}", created.rect());
                            }
                            tray = Some(created);
                        },
                        Err(error) => {
                            eprintln!("Vytvorenie tray ikony zlyhalo: {error}");
                            *control_flow = ControlFlow::ExitWithCode(1);
                        }
                    }
                }
                // Handle user events
                Event::UserEvent(AppEvent::Menu(event)) => {
                    self.handle_event(event, control_flow, &mut config, &mut recorder);
                }
                // Handle response events
                Event::UserEvent(AppEvent::RecorderStateChanged(state)) => {
                    if debug_tray {
                        eprintln!("[tray-debug] State: {state:?}, tray exists: {}", tray.is_some());
                    }
                    self.start_recording.set_enabled(state == RecorderState::Idle);
                    self.stop_recording.set_enabled(state == RecorderState::Recording);

                    if let Some(tray_icon) = tray.as_ref() {
                        let icon = match state {
                            RecorderState::Idle => idle_icon.clone(),
                            RecorderState::Recording => recording_icon.clone(),
                            RecorderState::Error => error_icon.clone(),
                        };

                        if let Err(error) = tray_icon.set_icon(Some(icon)) {
                            eprintln!("Icon change failed {error}")
                        }
                        if debug_tray {
                            eprintln!("[tray-debug] After set_icon, shell rect: {:?}", tray_icon.rect());
                        }
                    }
                }
                Event::LoopDestroyed => {
                    if debug_tray {
                        eprintln!("[tray-debug] LoopDestroyed: removing tray");
                    }
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
                Ok(()) => println!("Start recording, request send"),
                Err(error) => eprintln!("Recording error: {error}"),
            }
        } else if event.id == self.stop_recording.id() {
            match recorder.stop() {
                Ok(()) => println!("Stop recording, request send"),
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
