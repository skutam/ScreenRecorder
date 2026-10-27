use replay_recorder::config::ConfigHolder;
use replay_recorder::DefaultError;
use replay_recorder::tray::TrayMenu;


fn main() -> Result<(), DefaultError> {
    let config = ConfigHolder::new()?;
    let menu_tray = TrayMenu::new()?;
    menu_tray.menu_loop(config)?;

    Ok(())
}
