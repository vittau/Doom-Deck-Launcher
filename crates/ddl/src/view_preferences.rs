//! Spec 095: isolated, versioned UI state cannot be clobbered by catalog writers.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewFormat {
    #[default]
    Cards,
    List,
}
impl ViewFormat {
    pub fn toggled(self) -> Self {
        match self {
            Self::Cards => Self::List,
            Self::List => Self::Cards,
        }
    }
}
#[derive(Serialize, Deserialize)]
struct Preferences {
    schema_version: u32,
    #[serde(default)]
    format: ViewFormat,
}
pub fn load_view_format(env: &Environment) -> Result<ViewFormat, Problem> {
    let path = env.paths.config.join("view.json");
    let bytes = match fs::read(path) {
        Ok(bytes) if bytes.len() <= 4096 => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(ViewFormat::Cards),
        _ => return Err(Problem::Storage),
    };
    let prefs: Preferences = serde_json::from_slice(&bytes).map_err(|_| Problem::State)?;
    if prefs.schema_version != 1 {
        return Err(Problem::State);
    }
    Ok(prefs.format)
}
pub fn save_view_format(env: &Environment, format: ViewFormat) -> Result<(), Problem> {
    // This worker owns only this file; ordinary app/catalog saves never write it.
    save_json(
        &env.paths.config.join("view.json"),
        &Preferences {
            schema_version: 1,
            format,
        },
    )
}

/// One sleeping writer and one replaceable intent, independent of UI completion ticks.
/// Closing marks the mailbox finished; normal teardown joins after the UI loop ends.
pub struct ViewWriter {
    shared: std::sync::Arc<(std::sync::Mutex<Pending>, std::sync::Condvar)>,
    result: std::sync::Arc<std::sync::Mutex<Option<Result<(), Problem>>>>,
    worker: Option<std::thread::JoinHandle<()>>,
}
#[derive(Default)]
struct Pending {
    value: Option<ViewFormat>,
    closed: bool,
}
impl ViewWriter {
    pub fn new(env: Environment, wake: impl Fn() + Send + 'static) -> Self {
        Self::spawn(move |format| save_view_format(&env, format), wake)
    }
    fn spawn(
        save: impl Fn(ViewFormat) -> Result<(), Problem> + Send + 'static,
        wake: impl Fn() + Send + 'static,
    ) -> Self {
        let shared = std::sync::Arc::new((
            std::sync::Mutex::new(Pending::default()),
            std::sync::Condvar::new(),
        ));
        let result = std::sync::Arc::new(std::sync::Mutex::new(None::<Result<(), Problem>>));
        let worker_shared = shared.clone();
        let worker_result = result.clone();
        let worker = std::thread::spawn(move || {
            loop {
                let (lock, ready) = &*worker_shared;
                let mut pending = lock.lock().unwrap();
                while pending.value.is_none() && !pending.closed {
                    pending = ready.wait(pending).unwrap();
                }
                let Some(format) = pending.value.take() else {
                    break;
                };
                drop(pending);
                let saved = save(format);
                let mut result = worker_result.lock().unwrap();
                let notify = result.is_none();
                // Retain an error until the UI can show it, even if the next save succeeds.
                if result.as_ref().is_none_or(|r| r.is_ok()) {
                    *result = Some(saved);
                }
                drop(result);
                if notify {
                    wake();
                }
            }
        });
        Self {
            shared,
            result,
            worker: Some(worker),
        }
    }
    /// Call only after the UI event loop stops: finish the last pending save.
    pub fn finish(mut self) {
        self.close();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
    fn close(&self) {
        let (lock, ready) = &*self.shared;
        lock.lock().unwrap().closed = true;
        ready.notify_one();
    }
    pub fn set(&self, format: ViewFormat) {
        let (lock, ready) = &*self.shared;
        lock.lock().unwrap().value = Some(format);
        ready.notify_one();
    }
    pub fn take_result(&self) -> Option<Result<(), Problem>> {
        self.result.lock().unwrap().take()
    }
}
impl Drop for ViewWriter {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_writer_finishes_queued_last_choice_with_one_in_flight_save() {
        let (started, saving) = std::sync::mpsc::sync_channel(1);
        let (release, gate) = std::sync::mpsc::sync_channel(1);
        let saves = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed = saves.clone();
        let writer = ViewWriter::spawn(
            move |format| {
                let mut saves = observed.lock().unwrap();
                if saves.is_empty() {
                    started.send(()).unwrap();
                    gate.recv().unwrap();
                }
                saves.push(format);
                Ok(())
            },
            || {},
        );
        writer.set(ViewFormat::Cards);
        saving
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        for _ in 0..1000 {
            writer.set(ViewFormat::Cards);
            writer.set(ViewFormat::List);
        }
        writer.set(ViewFormat::List);
        assert_eq!(
            writer.shared.0.lock().unwrap().value,
            Some(ViewFormat::List)
        );
        release.send(()).unwrap();
        writer.finish();
        assert_eq!(
            *saves.lock().unwrap(),
            [ViewFormat::Cards, ViewFormat::List]
        );
    }
}
