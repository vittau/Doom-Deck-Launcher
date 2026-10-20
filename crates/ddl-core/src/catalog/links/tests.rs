//! Spec 044 A1: link checks against the local test server, no network.

use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::*;
use crate::catalog::recipe_fixture::recipe;
use crate::downloads::test_http::{Request, Response, Server};

const SIZE: usize = 100;

fn checker() -> LinkChecker {
    checker_with_timeout(Duration::from_secs(3))
}

fn checker_with_timeout(timeout: Duration) -> LinkChecker {
    LinkChecker::with_fetcher(Fetcher::headers_only(timeout, timeout).allow_http_for_tests())
        .with_retry_delays(vec![Duration::ZERO; 3])
}

fn allowed() -> HostAllowlist {
    HostAllowlist::new(["127.0.0.1"])
}

/// One recipe with one package of `SIZE` bytes and these mirrors.
fn single(id: &str, urls: Vec<String>) -> Recipe {
    recipe(id, &[("free", urls, SIZE as u64)], &[])
}

fn check_one(server_urls: Vec<String>) -> Report {
    let recipe = single("r", server_urls);
    checker().check(&[&recipe], &allowed())
}

fn only(report: &Report) -> &LinkResult {
    assert_eq!(report.results.len(), 1, "{report:?}");
    &report.results[0]
}

/// A server that serves a `SIZE`-byte file at `/ok` and a few failure modes.
fn catalog_server() -> Server {
    Server::start(|request: &Request| match request.target.as_str() {
        "/ok" => Response::ok(vec![0u8; SIZE]),
        "/bigger" => Response::ok(vec![0u8; SIZE + 20]),
        "/gone" => Response::status(404, "Not Found"),
        "/gone-410" => Response::status(410, "Gone"),
        "/boom" => Response::status(500, "Internal Server Error"),
        "/to-ok" => Response::redirect("/ok"),
        "/out" => {
            // Same server, a name that is not on the allowlist.
            let host = request
                .header("host")
                .unwrap_or_default()
                .replace("127.0.0.1", "localhost");
            Response::redirect(&format!("http://{host}/ok"))
        }
        "/loop" => Response::redirect("/loop"),
        _ => Response::status(404, "Not Found"),
    })
}

#[test]
fn each_outcome_is_reported() {
    let server = catalog_server();
    let cases = [
        ("/ok", Outcome::Ok),
        ("/to-ok", Outcome::Ok),
        ("/bigger", Outcome::SizeChanged),
        ("/gone", Outcome::Gone),
        ("/gone-410", Outcome::Gone),
        ("/boom", Outcome::Error),
        ("/out", Outcome::HostRefused),
        ("/loop", Outcome::Error),
    ];
    for (path, want) in cases {
        let report = check_one(vec![server.url(path)]);
        let result = only(&report);
        assert_eq!(result.outcome, want, "{path}: {result:?}");
        assert_eq!(result.expected_size, SIZE as u64);
    }
}

#[test]
fn details_name_the_size_status_and_host() {
    let server = catalog_server();
    let size = check_one(vec![server.url("/bigger")]);
    assert_eq!(only(&size).observed_size, Some(SIZE as u64 + 20));
    assert!(only(&size).detail.contains("pinned 100 bytes"), "{size:?}");
    assert!(only(&size).detail.contains("120"), "{size:?}");

    let boom = check_one(vec![server.url("/boom")]);
    assert_eq!(only(&boom).detail, "HTTP 500");

    let out = check_one(vec![server.url("/out")]);
    assert!(only(&out).detail.contains("`localhost`"), "{out:?}");

    let ok = check_one(vec![server.url("/ok")]);
    assert_eq!(only(&ok).observed_size, Some(SIZE as u64));
    assert!(only(&ok).detail.contains("HEAD"), "{ok:?}");
}

#[test]
fn a_first_url_outside_the_allowlist_is_host_refused() {
    let server = catalog_server();
    let recipe = single("r", vec![server.url("/ok")]);
    let report = checker().check(&[&recipe], &HostAllowlist::new(["example.com"]));
    assert_eq!(only(&report).outcome, Outcome::HostRefused);
}

#[test]
fn a_plain_http_url_is_an_error_in_production() {
    // The production checker is HTTPS-only: no `allow_http_for_tests`.
    let recipe = single("r", vec!["http://127.0.0.1:9/ok".into()]);
    let report = LinkChecker::new().check(&[&recipe], &allowed());
    assert_eq!(only(&report).outcome, Outcome::Error);
    assert!(only(&report).detail.contains("plain HTTP"), "{report:?}");
}

#[test]
fn a_silent_server_times_out_as_an_error() {
    let server = Server::start(|_| {
        std::thread::sleep(Duration::from_millis(1500));
        Response::ok(vec![0u8; SIZE])
    });
    let recipe = single("r", vec![server.url("/slow")]);
    let report = checker_with_timeout(Duration::from_millis(300)).check(&[&recipe], &allowed());
    assert_eq!(only(&report).outcome, Outcome::Error, "{report:?}");
    assert!(only(&report).detail.contains("timed out"), "{report:?}");
}

/// The requests a server saw, as `METHOD target range`.
type Log = Arc<StdMutex<Vec<String>>>;

fn logged(log: &Log, request: &Request) {
    log.lock().unwrap().push(format!(
        "{} {} {}",
        request.method,
        request.target,
        request.header("range").unwrap_or("-")
    ));
}

#[test]
fn a_host_that_refuses_head_gets_a_one_byte_range_request() {
    for refusal in [405, 501, 403] {
        let log: Log = Arc::default();
        let seen = Arc::clone(&log);
        let server = Server::start(move |request: &Request| {
            logged(&seen, request);
            if request.method == "HEAD" {
                return Response::status(refusal, "Refused");
            }
            Response::status(206, "Partial Content")
                .header("Content-Range", &format!("bytes 0-0/{SIZE}"))
                .with_body(vec![0u8])
        });
        let report = check_one(vec![server.url("/file")]);
        let result = only(&report);
        assert_eq!(result.outcome, Outcome::Ok, "{refusal}: {result:?}");
        assert_eq!(result.observed_size, Some(SIZE as u64));
        assert!(result.detail.contains("GET Range"), "{result:?}");
        let log = log.lock().unwrap();
        assert_eq!(*log, ["HEAD /file -", "GET /file bytes=0-0"], "{refusal}");
    }
}

#[test]
fn the_range_fallback_reads_the_total_not_the_one_byte_body() {
    let server = Server::start(|request: &Request| {
        if request.method == "HEAD" {
            return Response::status(405, "Method Not Allowed");
        }
        // The size changed: the pinned 100 bytes is now 5,000.
        Response::status(206, "Partial Content")
            .header("Content-Range", "bytes 0-0/5000")
            .with_body(vec![0u8])
    });
    let report = check_one(vec![server.url("/file")]);
    assert_eq!(only(&report).outcome, Outcome::SizeChanged);
    assert_eq!(only(&report).observed_size, Some(5000));
}

#[test]
fn a_range_request_answered_in_full_uses_content_length() {
    let server = Server::start(|request: &Request| {
        if request.method == "HEAD" {
            Response::status(405, "Method Not Allowed")
        } else {
            // The server ignored `Range` and answers `200` with the file.
            Response::ok(vec![0u8; SIZE])
        }
    });
    let report = check_one(vec![server.url("/file")]);
    assert_eq!(only(&report).outcome, Outcome::Ok, "{report:?}");
}

#[test]
fn an_empty_file_answers_the_range_with_416_and_a_star_total() {
    let server = Server::start(|request: &Request| {
        if request.method == "HEAD" {
            Response::status(405, "Method Not Allowed")
        } else {
            Response::status(416, "Range Not Satisfiable").header("Content-Range", "bytes */0")
        }
    });
    let recipe = recipe("r", &[("free", vec![server.url("/empty")], 0)], &[]);
    let report = checker().check(&[&recipe], &allowed());
    assert_eq!(only(&report).outcome, Outcome::Ok, "{report:?}");
    assert_eq!(only(&report).observed_size, Some(0));
}

#[test]
fn a_404_on_head_is_gone_without_a_second_request() {
    let log: Log = Arc::default();
    let seen = Arc::clone(&log);
    let server = Server::start(move |request: &Request| {
        logged(&seen, request);
        Response::status(404, "Not Found")
    });
    let report = check_one(vec![server.url("/file")]);
    assert_eq!(only(&report).outcome, Outcome::Gone);
    assert_eq!(*log.lock().unwrap(), ["HEAD /file -"]);
}

#[test]
fn a_transient_failure_is_asked_again() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&hits);
    // Every other request is a bare 503, like raw.githubusercontent.com.
    let server = Server::start(move |_: &Request| {
        if seen.fetch_add(1, Ordering::SeqCst).is_multiple_of(2) {
            Response::status(503, "Service Unavailable")
        } else {
            Response::ok(vec![0u8; SIZE])
        }
    });
    let report = check_one(vec![server.url("/flaky")]);
    assert_eq!(only(&report).outcome, Outcome::Ok, "{report:?}");
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}

#[test]
fn a_persistent_server_error_gives_up_after_four_attempts() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&hits);
    let server = Server::start(move |_: &Request| {
        seen.fetch_add(1, Ordering::SeqCst);
        Response::status(503, "Service Unavailable")
    });
    let report = check_one(vec![server.url("/down")]);
    assert_eq!(only(&report).outcome, Outcome::Error);
    assert_eq!(only(&report).detail, "HTTP 503");
    assert_eq!(hits.load(Ordering::SeqCst), 4);
}

#[test]
fn a_definite_answer_is_not_retried() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&hits);
    let server = Server::start(move |_: &Request| {
        seen.fetch_add(1, Ordering::SeqCst);
        Response::status(404, "Not Found")
    });
    check_one(vec![server.url("/gone")]);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn a_url_listed_by_several_recipes_is_probed_once_and_judged_for_each() {
    let hits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&hits);
    let server = Server::start(move |_: &Request| {
        seen.fetch_add(1, Ordering::SeqCst);
        Response::ok(vec![0u8; SIZE])
    });
    let url = server.url("/shared.wad");
    let exact = recipe("exact", &[("p", vec![url.clone()], SIZE as u64)], &[]);
    let stale = recipe("stale", &[("p", vec![url], 7)], &[]);
    let report = checker().check(&[&exact, &stale], &allowed());

    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "one probe for the shared URL"
    );
    assert_eq!(report.results[0].outcome, Outcome::Ok);
    assert_eq!(report.results[1].outcome, Outcome::SizeChanged);
    assert_eq!(report.failed_recipes(), ["stale"]);
}

#[test]
fn a_403_that_repeats_on_the_range_request_is_an_error() {
    let server = Server::start(|_| Response::status(403, "Forbidden"));
    let report = check_one(vec![server.url("/file")]);
    assert_eq!(only(&report).outcome, Outcome::Error);
    assert_eq!(only(&report).detail, "HTTP 403");
}

#[test]
fn one_dead_mirror_and_one_good_one_is_a_warning() {
    let server = catalog_server();
    let recipe = single("mixed", vec![server.url("/gone"), server.url("/ok")]);
    let report = checker().check(&[&recipe], &allowed());

    assert_eq!(report.results.len(), 2);
    assert_eq!(report.results[0].outcome, Outcome::Gone);
    assert_eq!(report.results[1].outcome, Outcome::Ok);
    assert!(!report.has_failures(), "{report:?}");
    assert!(report.failed_recipes().is_empty());
    assert_eq!(
        report.warnings,
        [PackageFinding {
            recipe: "mixed".into(),
            package: "free".into(),
            ok: 1,
            total: 2,
        }]
    );
}

#[test]
fn a_recipe_with_no_good_source_fails() {
    let server = catalog_server();
    let broken = single(
        "broken",
        vec![
            server.url("/gone"),
            server.url("/bigger"),
            server.url("/out"),
        ],
    );
    let fine = single("fine", vec![server.url("/ok")]);
    let report = checker().check(&[&broken, &fine], &allowed());

    assert!(report.has_failures());
    assert_eq!(report.failed_recipes(), ["broken"]);
    assert!(report.warnings.is_empty());
    assert_eq!(report.results.len(), 4);
}

#[test]
fn a_recipe_fails_when_any_of_its_packages_has_no_source_left() {
    let server = catalog_server();
    let two = recipe(
        "two",
        &[
            ("a", vec![server.url("/ok")], SIZE as u64),
            ("b", vec![server.url("/gone")], SIZE as u64),
        ],
        &[],
    );
    let report = checker().check(&[&two], &allowed());
    assert!(report.has_failures());
    assert_eq!(report.failed_recipes(), ["two"]);
    assert_eq!(report.failed[0].package, "b");
}

#[test]
fn results_keep_recipe_package_and_mirror_order() {
    let server = catalog_server();
    let urls = ["/ok", "/gone", "/bigger", "/boom", "/to-ok"];
    let first = single("first", urls.iter().map(|p| server.url(p)).collect());
    let second = single("second", urls.iter().rev().map(|p| server.url(p)).collect());
    let report = checker().check(&[&first, &second], &allowed());

    let seen: Vec<(&str, &str)> = report
        .results
        .iter()
        .map(|r| (r.recipe.as_str(), r.url.rsplit('/').next().unwrap()))
        .collect();
    let want: Vec<(&str, &str)> = ["first", "second"]
        .iter()
        .flat_map(|id| {
            let order: Vec<&str> = if *id == "first" {
                urls.to_vec()
            } else {
                urls.iter().rev().copied().collect()
            };
            order
                .into_iter()
                .map(move |p| (*id, p.trim_start_matches('/')))
        })
        .collect();
    assert_eq!(seen, want);
}

/// Tracks how many requests a server has open at once.
#[derive(Clone, Default)]
struct Gauge {
    now: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
}

impl Gauge {
    fn enter(&self) {
        let now = self.now.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(now, Ordering::SeqCst);
    }
    fn leave(&self) {
        self.now.fetch_sub(1, Ordering::SeqCst);
    }
    fn peak(&self) -> usize {
        self.peak.load(Ordering::SeqCst)
    }
}

fn slow_server(overall: Gauge, per_host: Gauge) -> Server {
    Server::start(move |_: &Request| {
        overall.enter();
        per_host.enter();
        std::thread::sleep(Duration::from_millis(120));
        per_host.leave();
        overall.leave();
        Response::ok(vec![0u8; SIZE])
    })
}

#[test]
fn at_most_four_requests_are_in_flight_across_hosts() {
    let overall = Gauge::default();
    // Twelve servers = twelve `host:port` keys, so only the global bound holds
    // the checker back.
    let servers: Vec<(Server, Gauge)> = (0..12)
        .map(|_| {
            let own = Gauge::default();
            (slow_server(overall.clone(), own.clone()), own)
        })
        .collect();
    let recipe = single(
        "many",
        servers.iter().map(|(s, _)| s.url("/file")).collect(),
    );
    let report = checker().check(&[&recipe], &allowed());

    assert!(
        report.results.iter().all(|r| r.outcome == Outcome::Ok),
        "{report:?}"
    );
    assert!(overall.peak() <= MAX_IN_FLIGHT, "peak {}", overall.peak());
    assert!(
        overall.peak() >= 2,
        "no parallelism at all: peak {}",
        overall.peak()
    );
}

#[test]
fn at_most_one_request_is_in_flight_per_host() {
    let overall = Gauge::default();
    let own = Gauge::default();
    let server = slow_server(overall.clone(), own.clone());
    // Six URLs on one `host:port`, plus a second host that may run beside them.
    let other = slow_server(overall.clone(), Gauge::default());
    let mut urls: Vec<String> = (0..6).map(|n| server.url(&format!("/file-{n}"))).collect();
    urls.push(other.url("/file"));
    let recipe = single("one-host", urls);
    let report = checker().check(&[&recipe], &allowed());

    assert!(
        report.results.iter().all(|r| r.outcome == Outcome::Ok),
        "{report:?}"
    );
    assert_eq!(own.peak(), 1, "two requests hit one host at once");
    assert!(overall.peak() >= 2, "the second host should overlap");
}

#[test]
fn the_markdown_table_lists_recipe_url_outcome_and_detail() {
    let server = catalog_server();
    let recipe = single("table", vec![server.url("/ok"), server.url("/gone")]);
    let report = checker().check(&[&recipe], &allowed());

    let full = report.to_markdown(false);
    assert!(
        full.contains("| Recipe | URL | Outcome | Detail |"),
        "{full}"
    );
    assert!(full.contains("| table |"), "{full}");
    assert!(full.contains("| ok |"), "{full}");
    assert!(full.contains("| gone | HTTP 404 |"), "{full}");
    assert!(full.contains("**2 URLs checked:** 1 ok"), "{full}");

    let problems = report.to_markdown(true);
    assert!(!problems.contains("| ok |"), "{problems}");
    assert!(problems.contains("| gone |"), "{problems}");
}

#[test]
fn markdown_cells_escape_pipes_and_newlines() {
    let result = LinkResult {
        recipe: "r".into(),
        package: "p".into(),
        url: "https://example.com/a|b".into(),
        outcome: Outcome::Error,
        detail: "line one\nline two | three".into(),
        expected_size: 1,
        observed_size: None,
    };
    let markdown = Report::from_results(vec![result]).to_markdown(false);
    assert!(markdown.contains("a\\|b"), "{markdown}");
    assert!(
        markdown.contains("line one line two \\| three"),
        "{markdown}"
    );
}

#[test]
fn the_json_report_carries_outcomes_and_the_verdict() {
    let server = catalog_server();
    let broken = single("broken", vec![server.url("/gone")]);
    let report = checker().check(&[&broken], &allowed());
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();

    assert_eq!(json["results"][0]["outcome"], "gone");
    assert_eq!(json["results"][0]["recipe"], "broken");
    assert_eq!(json["results"][0]["expected_size"], SIZE);
    assert_eq!(json["failed"][0]["recipe"], "broken");
    assert_eq!(json["warnings"], serde_json::json!([]));
    assert_eq!(Outcome::SizeChanged.label(), "size-changed");
    assert_eq!(Outcome::HostRefused.label(), "host-refused");
    assert_eq!(
        serde_json::to_value(Outcome::HostRefused).unwrap(),
        "host-refused"
    );
}

#[test]
fn an_empty_catalog_checks_cleanly() {
    let report = checker().check(&[], &allowed());
    assert!(report.results.is_empty());
    assert!(!report.has_failures());
}
