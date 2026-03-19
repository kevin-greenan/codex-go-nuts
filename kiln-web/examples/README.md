# Kiln Example Suites

Kiln keeps its smokes small and focused on purpose. They are easier to debug this way than one giant integration fixture.

Recommended grouping:

## Core HTTP

1. `http_parse_smoke.noe`
2. `route_smoke.noe`
3. `reply_smoke.noe`
4. `server_smoke.noe`

## HTML And Forms

1. `html_smoke.noe`
2. `app_smoke.noe`
3. `flash_smoke.noe`

## App Layer

1. `static_app_smoke.noe`
2. `asset_smoke.noe`

## Localhost Examples

1. `hello_server.noe`
2. `route_server.noe`
3. `html_server.noe`
4. `form_server.noe`
5. `cookie_server.noe`
6. `cookie_reply_server.noe`
7. `form_page_server.noe`
8. `static_site_server.noe`
9. `asset_site_server.noe`

## PR Acceptance Set

For merge confidence on this branch, the highest-signal set is:

1. `http_parse_smoke.noe`
2. `reply_smoke.noe`
3. `flash_smoke.noe`
4. `asset_smoke.noe`
5. `app_smoke.noe`

Use [tools/run_direct_smokes.sh](/Users/kevin/Documents/Projects/AI/codex-go-nuts/kiln-web/tools/run_direct_smokes.sh) to run grouped suites with the pure direct compiler path.
