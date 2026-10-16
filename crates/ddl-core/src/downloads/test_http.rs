//! A tiny blocking HTTP/1.1 server and temporary-directory guard, used only by
//! the download tests (spec 006, A1/A2).
//!
//! Tests must not touch the network (`AGENTS.md`, spec 006 A5), so every
//! download test runs against `Server`, a real `TcpListener` on `127.0.0.1`
//! that answers from a closure. It speaks just enough HTTP for the fetcher:
//! one request per connection, `Content-Length`, `Location` and a status line.
#![cfg(any(test, feature = "test-support"))]

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// The part of a request the tests care about.
#[allow(dead_code)] // `method` is recorded for completeness though tests use `target`.
pub struct Request {
    pub method: String,
    pub target: String,
}

/// A response the handler wants to send.
pub struct Response {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    /// `200 OK` with a body.
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self::status(200, "OK").with_body(body)
    }

    /// A response with the given status and no body.
    pub fn status(status: u16, reason: &str) -> Self {
        Self {
            status,
            reason: reason.to_owned(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// `302 Found` pointing at `location`.
    pub fn redirect(location: &str) -> Self {
        Self::status(302, "Found").header("Location", location)
    }

    /// Adds a header.
    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// Sets the body.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
}

/// A running test server. Dropping it stops the accept loop.
pub struct Server {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    /// Starts a server whose responses come from `handler`.
    pub fn start<F>(handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking accept");
        let addr = listener.local_addr().expect("local addr");
        let stop = Arc::new(AtomicBool::new(false));
        let handler = Arc::new(handler);
        let accept_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let handler = Arc::clone(&handler);
                        thread::spawn(move || {
                            let _ = serve_connection(stream, handler.as_ref());
                        });
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if accept_stop.load(Ordering::SeqCst) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            addr,
            stop,
            handle: Some(handle),
        }
    }

    /// An `http://host:port<path>` URL for this server.
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wake the non-blocking accept loop so it observes the stop flag.
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn serve_connection<F>(stream: TcpStream, handler: &F) -> std::io::Result<()>
where
    F: Fn(&Request) -> Response + ?Sized,
{
    // Accepted sockets can inherit the listener's non-blocking mode (notably
    // on macOS); force blocking reads so the request is read reliably.
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }

    let mut parts = request_line.split_whitespace();
    let request = Request {
        method: parts.next().unwrap_or_default().to_owned(),
        target: parts.next().unwrap_or_default().to_owned(),
    };

    let response = handler(&request);
    let mut out = Vec::new();
    write!(out, "HTTP/1.1 {} {}\r\n", response.status, response.reason)?;
    write!(out, "Content-Length: {}\r\n", response.body.len())?;
    out.extend_from_slice(b"Connection: close\r\n");
    for (name, value) in &response.headers {
        write!(out, "{name}: {value}\r\n")?;
    }
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(&response.body);

    let mut stream = stream;
    stream.write_all(&out)?;
    stream.flush()?;

    // Give the client a moment to read before the socket closes. Closing while
    // response bytes are still in flight lets the kernel deliver a clean FIN;
    // an immediate close can race an early-aborting client (the size-limit
    // tests) into a reset that truncates the response.
    thread::sleep(Duration::from_millis(25));
    Ok(())
}

/// A unique temporary directory removed when the guard is dropped.
pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// Creates `ddl-test-<label>-<pid>-<n>` under the system temp directory.
    pub fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ddl-test-{label}-{}-{unique}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create test dir");
        Self { path }
    }

    /// The directory path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
