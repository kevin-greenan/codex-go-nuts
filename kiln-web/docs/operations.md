# Kiln Operations Guide

This document describes the current operational contract for Kiln Web.

## Runtime Model

Kiln should currently be treated as:

1. a direct-compiler-first web stack
2. a local-first, single-process framework layer
3. best suited to one request at a time through the current server wrappers

What this means in practice:

1. the current `kiln_server_receive_once(...)` and app wrappers intentionally handle one request per process invocation
2. they are ideal for smoke coverage, local development probes, and simple embedded app loops
3. a longer-lived accept loop can be built on the same listener/runtime primitives later without changing the core HTTP and routing APIs

## Request And Response Contract

Kiln currently supports:

1. request-line parsing
2. repeated headers
3. query-string lookup
4. urlencoded form parsing
5. repeated query and form values
6. cookie lookup
7. text, HTML, CSS, JavaScript, redirect, and header-rich responses

Flash-style UX messages currently use a cookie convention:

1. `kiln_reply_with_flash(...)` sets `kiln_flash`
2. `kiln_flash_level(...)` and `kiln_flash_message(...)` read it on the next request
3. `kiln_reply_clear_flash(...)` clears it after rendering

## Static Assets

Kiln currently supports file-backed asset serving for local development through the app layer:

1. declare an asset route with `kiln_route(...)`
2. map it to a file with `kiln_app_route_asset(...)`
3. serve it through `kiln_static_asset_app_reply(...)`

Current expectations:

1. asset paths should point at files that exist in the current working tree
2. content types can be passed explicitly or inferred with `kiln_asset_content_type(...)`
3. this is meant for embedded local apps, not as a replacement for a dedicated reverse proxy or CDN

## Verification

Recommended verification flow:

1. rebuild the direct compiler if compiler/runtime work changed
2. run `zsh kiln-web/tools/run_direct_smokes.sh core`
3. run `zsh kiln-web/tools/run_direct_smokes.sh apps`
4. run a localhost example when changing server flow, headers, or asset behavior

If compiler/runtime work changed, re-run a focused AshDB direct suite too so platform changes do not silently regress the database layer.

## Stability Notes

What is stable enough for application work:

1. HTTP parsing and response building
2. routing and param dispatch
3. HTML page rendering helpers
4. cookie and flash helpers
5. file-backed local asset serving

What still needs caution:

1. richer request bodies beyond urlencoded forms
2. long-running server ergonomics beyond the current one-shot wrappers
3. assuming the current helper set is already the final app DSL
