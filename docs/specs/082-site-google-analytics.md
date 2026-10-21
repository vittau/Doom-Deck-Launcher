# 082 — Measure website visits with GA4

**Status:** Complete — implementation and focused verification passed. **Origin:** owner supplied measurement ID `G-BTYE5G8WCM` after creating the
website property and requests its configuration.

Add the standard asynchronous Google tag for this measurement ID to the generated website
landing page and credits page. Initialize it once per page using the documented gtag.js
integration. This measures website traffic only; no launcher telemetry or new application
behavior is in scope. Do not attach custom user identifiers, URL data or custom events.
Keep main content functional without JavaScript and retain existing metadata/branding and
publication guards. No secrets are needed: a GA measurement ID is public configuration.

Own site HTML templates, narrowly scoped renderer tests and docs/reports/082-site-google-analytics.md.
Verify generated pages each load the correct script once and initialize the correct ID once;
run existing site tests. Coordinator publishes through the independent website workflow after
the signed new-ID channel is available, then checks live source. Real event reception remains
for owner confirmation in Analytics Realtime/DebugView. No visible windows, screen captures,
new dependencies, application edits, push, tag or release. Merge main before final checks.
