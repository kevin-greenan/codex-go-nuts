#!/bin/zsh
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p build/test-direct
rm -f build/test-direct/*(N)

examples=(
    examples/hello.noe
    examples/exit_status.noe
    examples/control_flow.noe
    examples/records_and_lists.noe
    examples/text_pipeline.noe
    examples/compiler_frontend.noe
)

for src in "${examples[@]}"; do
    name="${src:t:r}"
    ./build/noema_compiler.direct "$src" "build/test-direct/$name" native-arm64
done

./build/test-direct/hello
./build/test-direct/exit_status || test $? -eq 5
./build/test-direct/control_flow
./build/test-direct/records_and_lists
rm -f build/test-direct/text_pipeline.out
./build/test-direct/text_pipeline examples/hello.noe build/test-direct/text_pipeline.out
test -s build/test-direct/text_pipeline.out
./build/test-direct/compiler_frontend examples/hello.noe build/test-direct/compiler_frontend.out
test -s build/test-direct/compiler_frontend.out
