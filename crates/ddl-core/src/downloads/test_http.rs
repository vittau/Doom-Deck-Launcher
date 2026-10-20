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
    /// Request headers, lowercased names.
    pub headers: Vec<(String, String)>,
}

impl Request {
    /// The value of `name` (case-insensitive), if the request carried it.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// A response the handler wants to send.
pub struct Response {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    /// When set, the body is written as `(chunk_size, delay)` pieces, sleeping
    /// `delay` before each piece, instead of in one write. Lets a test model a
    /// slow-but-steady link or a server that stalls after the headers.
    pub trickle: Option<(usize, Duration)>,
    /// When set (with `trickle`), the first piece is written immediately and
    /// only the later pieces wait `delay`. Models a connection that delivers
    /// some bytes and then goes quiet (spec 024 idle tests).
    pub first_now: bool,
    /// When set, only this many body bytes are written even though
    /// `Content-Length` describes the whole body: a transfer cut mid-body
    /// (spec 040 resume tests).
    pub send_only: Option<usize>,
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
            trickle: None,
            first_now: false,
            send_only: None,
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

    /// Sends the body in `chunk`-byte pieces, waiting `delay` before each one.
    #[must_use]
    pub fn trickle(mut self, chunk: usize, delay: Duration) -> Self {
        self.trickle = Some((chunk, delay));
        self
    }

    /// Sends the first `chunk` bytes immediately, then waits `delay` before
    /// each remaining piece: a connection that delivers some data and stalls.
    #[must_use]
    pub fn stall_after(mut self, chunk: usize, delay: Duration) -> Self {
        self.trickle = Some((chunk, delay));
        self.first_now = true;
        self
    }

    /// Advertises the full body in `Content-Length` but writes only `bytes`
    /// of it before closing: a connection cut mid-transfer.
    #[must_use]
    pub fn truncated(mut self, bytes: usize) -> Self {
        self.send_only = Some(bytes);
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
    let mut headers = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.trim_end().split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }

    let mut parts = request_line.split_whitespace();
    let request = Request {
        method: parts.next().unwrap_or_default().to_owned(),
        target: parts.next().unwrap_or_default().to_owned(),
        headers,
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

    let mut stream = stream;
    stream.write_all(&out)?;
    stream.flush()?;

    // A `HEAD` answer carries the headers (and the `Content-Length` of the body
    // a `GET` would return) but never the body itself (spec 044).
    if request.method.eq_ignore_ascii_case("HEAD") {
        thread::sleep(Duration::from_millis(25));
        return Ok(());
    }

    // Trickled bodies model a slow link (or a stall when the first delay is
    // long): the headers land immediately, then each piece is sent after a
    // pause. Everything else goes out in one write.
    match response.trickle {
        Some((chunk, delay)) => {
            for (index, piece) in response.body.chunks(chunk.max(1)).enumerate() {
                if !(response.first_now && index == 0) {
                    thread::sleep(delay);
                }
                stream.write_all(piece)?;
                stream.flush()?;
            }
        }
        None => {
            let sent = response
                .send_only
                .map_or(response.body.len(), |limit| limit.min(response.body.len()));
            stream.write_all(&response.body[..sent])?;
            stream.flush()?;
        }
    }

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
