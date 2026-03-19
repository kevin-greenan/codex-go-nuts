#!/bin/zsh
set -euo pipefail

ROOT_DIR="${0:A:h:h}"
REPO_DIR="${ROOT_DIR:h}"
TMP_DIR="${TMPDIR:-/tmp}/hearthlight-smoke"
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
        hearthlight/examples/dashboard_smoke.noe \
        hearthlight/examples/action_smoke.noe \
        hearthlight/examples/planner_smoke.noe \
        hearthlight/examples/pantry_smoke.noe \
        hearthlight/examples/chores_smoke.noe \
        hearthlight/examples/grocery_generation_smoke.noe
      ;;
    all)
      print -l \
        hearthlight/examples/dashboard_smoke.noe \
        hearthlight/examples/action_smoke.noe \
        hearthlight/examples/planner_smoke.noe \
        hearthlight/examples/pantry_smoke.noe \
        hearthlight/examples/chores_smoke.noe \
        hearthlight/examples/grocery_generation_smoke.noe
      ;;
    *)
      print -u2 "unknown suite: ${1}"
      print -u2 "expected one of: core all"
      return 1
      ;;
  esac
}

run_one() {
  local example_path="$1"
  local stem="${example_path:t:r}"
  local binary_path="${TMP_DIR}/${stem}.suite.native"
  local db_path="${TMP_DIR}/${stem}.suite.db"

  rm -f "${binary_path}" "${db_path}" "${db_path}.journal" "${db_path}.wal"
  "${COMPILER_STAGE2}" "${example_path}" "${binary_path}" native-arm64
  "${binary_path}" "${db_path}"
}

for example_path in ${(f)"$(examples_for_suite "${SUITE}")"}; do
  print "==> ${example_path:t}"
  run_one "${example_path}"
done
