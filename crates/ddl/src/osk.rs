//! Steam URI keyboard request; actual overlay delivery needs Deck validation.
use std::io;

use ddl_core::platform::sandbox::Sandbox;

/// The Steam URI that opens Steam's on-screen keyboard.
const KEYBOARD_URI: &str = "steam://open/keyboard";

pub trait Launcher {
    fn launch(&self, program: &str, args: &[&str]) -> io::Result<()>;
}
pub struct SystemLauncher;
impl Launcher for SystemLauncher {
    fn launch(&self, program: &str, args: &[&str]) -> io::Result<()> {
        #[cfg(all(target_os = "linux", not(test)))]
        {
            let mut child = std::process::Command::new(program).args(args).spawn()?;
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            Ok(())
        }
        #[cfg(any(not(target_os = "linux"), test))]
        {
            let _ = (program, args);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Steam keyboard requires Linux",
            ))
        }
    }
}
/// The program and arguments that request Steam's keyboard (spec 059, R6).
///
/// Under Flatpak `steam` is not on `PATH`, so the URI goes through `xdg-open`
/// and the OpenURI portal; elsewhere it is `steam` itself, unchanged.
pub fn command(sandbox: &Sandbox) -> (&'static str, [&'static str; 1]) {
    if sandbox.is_flatpak() {
        ("xdg-open", [KEYBOARD_URI])
    } else {
        ("steam", [KEYBOARD_URI])
    }
}
pub fn focused(launcher: &dyn Launcher, sandbox: &Sandbox) -> io::Result<()> {
    let (program, args) = command(sandbox);
    launcher.launch(program, &args)
}
