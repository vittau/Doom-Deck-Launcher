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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Recorder(RefCell<Vec<(String, Vec<String>)>>);
    impl Launcher for Recorder {
        fn launch(&self, program: &str, args: &[&str]) -> io::Result<()> {
            self.0
                .borrow_mut()
                .push((program.into(), args.iter().map(|a| (*a).into()).collect()));
            Ok(())
        }
    }

    /// Spec 059 A4 / R6: under Flatpak the URI goes through `xdg-open`.
    #[test]
    fn the_flatpak_opens_the_keyboard_through_xdg_open() {
        let recorder = Recorder::default();
        focused(&recorder, &Sandbox::flatpak(None)).unwrap();
        assert_eq!(
            *recorder.0.borrow(),
            [(
                "xdg-open".to_owned(),
                vec!["steam://open/keyboard".to_owned()]
            )]
        );
    }

    #[test]
    fn outside_the_sandbox_the_command_is_unchanged() {
        let recorder = Recorder::default();
        focused(&recorder, &Sandbox::none()).unwrap();
        assert_eq!(
            *recorder.0.borrow(),
            [("steam".to_owned(), vec!["steam://open/keyboard".to_owned()])]
        );
        assert_eq!(command(&Sandbox::none()).0, "steam");
    }
}
