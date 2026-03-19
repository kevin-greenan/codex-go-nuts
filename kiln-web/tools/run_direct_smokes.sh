#!/bin/zsh
set -euo pipefail

ROOT_DIR="${0:A:h:h}"
REPO_DIR="${ROOT_DIR:h}"
TMP_DIR="${TMPDIR:-/tmp}/kiln-web-smoke"
COMPILER_STAGE1="${TMP_DIR}/noema_compiler.suite1"
COMPILER_STAGE2="${TMP_DIR}/noema_compiler.suite2"
SUITE="${1:-core}"

mkdir -p "${TMP_DIR}"

cd "${REPO_DIR}"

./codex-lang/build/noema_compiler.pure4 codex-lang/selfhost/compiler.noe "${COMPILER_STAGE1}" native-arm64
"${COMPILER_STAGE1}" codex-lang/selfhost/compiler.noe "${COMPILER_STAGE2}" native-arm64

examples_for_suite() {
  case "${1}" in
    core)
      print -l \
        kiln-web/examples/http_parse_smoke.noe \
        kiln-web/examples/route_smoke.noe \
        kiln-web/examples/reply_smoke.noe \
        kiln-web/examples/server_smoke.noe
      ;;
    apps)
      print -l \
        kiln-web/examples/html_smoke.noe \
        kiln-web/examples/app_smoke.noe \
        kiln-web/examples/static_app_smoke.noe \
        kiln-web/examples/asset_smoke.noe \
        kiln-web/examples/flash_smoke.noe
      ;;
    all)
      print -l \
        kiln-web/examples/http_parse_smoke.noe \
        kiln-web/examples/route_smoke.noe \
        kiln-web/examples/reply_smoke.noe \
        kiln-web/examples/server_smoke.noe \
        kiln-web/examples/html_smoke.noe \
        kiln-web/examples/app_smoke.noe \
        kiln-web/examples/static_app_smoke.noe \
        kiln-web/examples/asset_smoke.noe \
        kiln-web/examples/flash_smoke.noe
      ;;
    *)
      print -u2 "unknown suite: ${1}"
      print -u2 "expected one of: core apps all"
      return 1
      ;;
  esac
}

run_one() {
  local example_path="$1"
  local stem="${example_path:t:r}"
  local binary_path="${TMP_DIR}/${stem}.suite.native"

  rm -f "${binary_path}"
  "${COMPILER_STAGE2}" "${example_path}" "${binary_path}" native-arm64
  "${binary_path}"
}

for example_path in ${(f)"$(examples_for_suite "${SUITE}")"}; do
  print "==> ${example_path:t}"
  run_one "${example_path}"
done
