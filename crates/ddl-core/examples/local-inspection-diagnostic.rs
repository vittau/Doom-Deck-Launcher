//! Offline host diagnostic, synthetic sparse WADs only; never launches an engine.
use ddl_core::local::Inspection;
use std::{fs, io::Write, time::Instant};
fn main() {
    let root = std::env::temp_dir().join("ddl-106-synthetic");
    fs::create_dir_all(&root).unwrap();
    for i in 0..8 {
        let mut file = fs::File::create(root.join(format!("large-{i}.wad"))).unwrap();
        file.write_all(b"PWAD\0\0\0\0\x0c\0\0\0").unwrap();
        file.set_len(256 * 1024 * 1024).unwrap();
    }
    fs::write(root.join("wrong.wad"), b"not a wad").unwrap();
    fs::write(root.join("patch.deh"), b"Patch File for DeHackEd v3.0\n").unwrap();
    for run in 0..3 {
        let start = Instant::now();
        let count = fs::read_dir(&root)
            .unwrap()
            .filter(|e| Inspection::read(&e.as_ref().unwrap().path()).is_ok())
            .count();
        println!(
            "run={run} accepted={count} listing_ms={:.3}",
            start.elapsed().as_secs_f64() * 1000.
        );
    }
}
