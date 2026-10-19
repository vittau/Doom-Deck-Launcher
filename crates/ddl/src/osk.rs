//! Steam URI keyboard request; actual overlay delivery needs Deck validation.
use std::io;

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
pub fn focused(launcher: &dyn Launcher) -> io::Result<()> {
    launcher.launch("steam", &["steam://open/keyboard"])
}
