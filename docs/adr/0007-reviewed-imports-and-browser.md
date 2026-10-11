# ADR 0007: Isolated browsing and reviewed recipe imports

Date: 2026-10-11. Status: Accepted.

## Evidence before implementation

Tauri 2.12.3 provides `WebviewWindowBuilder::data_directory`, navigation,
popup/download/permission callbacks, `eval_with_callback`, and
`clear_all_browsing_data`. Checked the installed pinned source and
[official API](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html),
[capability documentation](https://v2.tauri.app/security/capabilities/) and
[Microsoft profile guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/user-data-folder).
The isolated `browser_spike` example uses an owned loopback fixture, not household
storage. Separate Windows processes verified an HttpOnly login cookie and local
storage survive restart; native evaluation reads owned Recipe metadata while
remote `get_app_info` and `get_catalog` calls are denied. Loopback HTTP is an
example-only exception and is never accepted by production browsing.

## Decision and boundary

Ship a dedicated, unprivileged `recipe-browser` window with a stable backend-owned
app-data profile. Only local `main` has capabilities. Main navigation stays on the
initial HTTPS origin: cross-origin redirects, popups, downloads, device permissions
and native clipboard permissions are denied. Login/MFA that requires another origin
can be completed in the user's regular browser followed by manual import; no
authentication tokens are copied. Session retention is best-effort.

The supported product extraction mechanism is deliberate user paste of authorized
JSON-LD, or manual recipe text. We do not harvest arbitrary site DOMs. There is no
Rust HTTP fetch, proxy, HTML execution, remote-to-main event channel, arbitrary
script IPC or filesystem-path IPC. HTTPS URLs reject credentials, query/fragment
secrets, IP literals, local names and nonstandard ports. Browser network activity
is ordinary WebView2 activity; this is not an SSRF-proof HTTP client or a sandbox
for the site's own subresources. No backend fetch exists. Payloads are bounded;
JSON recursion is bounded by serde and extractor traversal. Errors omit input.

Drafts preserve original payload, yield and ingredient strings. Parsing makes
conservative suggestions; cups/spoons without a region stay unresolved, ranges
and package sizes are retained verbatim. Exact alias matches are reviewable;
uncertain matches remain unresolved. User confirmation records original provenance
and normalized meal in one transaction. Durable action receipts make retries safe.
Duplicate source URLs require an explicit save-another choice, including retired
recipes; existing meals are never overwritten. Editing retains the import record.
Reset uses WebView2's supported clearing API; no live profile deletion.
The Tauri convenience clear method drops WebView2's asynchronous completion
result. Microsoft's [clear-data contract](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2profile2)
requires waiting before closing. A small Windows-only native adapter waits for
`ClearBrowsingDataAll` completion with a 30-second timeout. It uses the existing
Tauri dependency versions of `webview2-com` and `windows-core`, with no new package
versions resolved. Active recipe content closes first; a hidden, unprivileged
`recipe-browser-reset` window keeps the profile alive until completion. Failures
are reported, rather than claiming that an unconfirmed reset succeeded.

Native compatibility visits to all five supplied URLs subsequently loaded the
expected recipe page titles, including Spend With Pennies and NYT Cooking. That
does not establish full recipe visibility, authenticated access or extraction
permission. The earlier research-browser retrieval limitations above are specific
to that tool. No publisher recipe content was harvested by the native probe.

## Requested sites and conditional adapters

- Budget Bytes: [terms section 4.3.3](https://www.budgetbytes.com/terms-conditions/)
  prohibits scraping. Manual entry only; no automated adapter.
- Allrecipes: supplied recipe was readable through the research browser. A right
  to automated collection was not established; manual entry is the supported path.
- Spend With Pennies: research browser could not retrieve the supplied page.
  [Publisher policy](https://www.spendwithpennies.com/disclosure-and-privacy-policy/)
  does not establish an automated integration permission; manual entry.
- RecipeTin Eats: supplied recipe readable; [recipe/image policy](https://www.recipetineats.com/policy-use-of-recipes-images/)
  restricts redistribution and commercial use. No automated integration permission
  established; manual entry, retaining personal source attribution.
- NYT Cooking: research browser could not retrieve supplied page. No authenticated
  account was supplied. No hidden metadata extraction or bypass; manual entry from
  content the user can access and is permitted to retain.
- Cook Smarts: [terms](https://www.cooksmarts.com/terms-of-use/) restrict downloading
  beyond normal caching and collection absent permission. No sanctioned recipe
  API/export integration was established. Automated integration is unsupported;
  use manual entry or an authorized export. No login credentials were requested.

The plan's site-specific adapters are conditional. None is justified by the current
permission evidence. This is an intentional browser-assisted/manual fallback,
not a claim that the five requested sites support automated imports. Original
fixtures cover their URL/policy handling and representative JSON-LD shapes without
redistributing publisher recipe content. An owned fixture supplies the allowed-site
review/save gate. Future approved adapters require new permission evidence.
