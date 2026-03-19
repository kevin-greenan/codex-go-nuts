# Kiln Status

This page keeps the fuller milestone inventory so the main README can stay focused on current status and verification.

Working now in Kiln Web:

1. inbound listener support in both compiler paths
2. direct-compiled localhost `hello` server
3. HTTP request parsing and response serialization
4. repeated-header access and cookie lookup
5. query-string and urlencoded-form parsing, including repeated values
6. router dispatch with `404`, `405`, and `:param` segments
7. `KilnReply` helpers for text, HTML, redirects, custom headers, cookies, CSS, and JavaScript
8. flash-cookie conventions for redirect-style UX messages
9. one-shot server wrapper and app wrapper
10. static reply apps and file-backed asset apps
11. HTML escaping, tags, forms, page shells, and stylesheet helpers
12. direct-compiled localhost examples for routes, forms, cookies, HTML, and CSS assets

Still later-expansion work:

1. richer body formats beyond urlencoded forms
2. more dynamic app conventions above the current wrappers
3. longer-lived accept-loop ergonomics
4. more operational polish if Kiln grows beyond embedded local apps
