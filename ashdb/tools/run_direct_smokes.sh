#!/bin/zsh
set -euo pipefail

ROOT_DIR="${0:A:h:h}"
REPO_DIR="${ROOT_DIR:h}"
TMP_DIR="${TMPDIR:-/tmp}/ashdb-smoke"
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
        ashdb/examples/pager_smoke.noe \
        ashdb/examples/table_smoke.noe \
        ashdb/examples/db_smoke.noe
      ;;
    recovery)
      print -l \
        ashdb/examples/repair_database_smoke.noe \
        ashdb/examples/reopen_regress_smoke.noe \
        ashdb/examples/lock_smoke.noe \
        ashdb/examples/validate_smoke.noe
      ;;
    schema)
      print -l \
        ashdb/examples/schema_smoke.noe \
        ashdb/examples/defaults_smoke.noe \
        ashdb/examples/unique_smoke.noe \
        ashdb/examples/foreign_key_smoke.noe \
        ashdb/examples/migrate_smoke.noe \
        ashdb/examples/patch_smoke.noe
      ;;
    all)
      print -l \
        ashdb/examples/pager_smoke.noe \
        ashdb/examples/table_smoke.noe \
        ashdb/examples/db_smoke.noe \
        ashdb/examples/schema_smoke.noe \
        ashdb/examples/defaults_smoke.noe \
        ashdb/examples/unique_smoke.noe \
        ashdb/examples/foreign_key_smoke.noe \
        ashdb/examples/migrate_smoke.noe \
        ashdb/examples/patch_smoke.noe \
        ashdb/examples/repair_database_smoke.noe \
        ashdb/examples/reopen_regress_smoke.noe \
        ashdb/examples/lock_smoke.noe \
        ashdb/examples/validate_smoke.noe
      ;;
    *)
      print -u2 "unknown suite: ${1}"
      print -u2 "expected one of: core recovery schema all"
      return 1
      ;;
  esac
}

run_one() {
  local example_path="$1"
  local stem="${example_path:t:r}"
  local binary_path="${TMP_DIR}/${stem}.suite.native"
  local db_path="${TMP_DIR}/${stem}.suite.db"

  rm -f "${db_path}" "${db_path}.journal" "${db_path}.wal" "${binary_path}"
  "${COMPILER_STAGE2}" "${example_path}" "${binary_path}" native-arm64
  "${binary_path}" "${db_path}"
}

for example_path in ${(f)"$(examples_for_suite "${SUITE}")"}; do
  print "==> ${example_path:t}"
  run_one "${example_path}"
done
