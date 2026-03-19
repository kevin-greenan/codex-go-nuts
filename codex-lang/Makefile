IMAGE := codex-lang-rust

.PHONY: image compiler bootstrap-compiler bootstrap-direct test-direct-examples direct-bootstrap-check clean

image:
	docker build -t $(IMAGE) -f container/Dockerfile .

compiler:
	docker run --rm -v "$$(pwd)":/workspace -w /workspace $(IMAGE) cargo build --manifest-path compiler/Cargo.toml --release

bootstrap-compiler:
	./bin/codexc selfhost/compiler.noe build/compiler

bootstrap-direct: bootstrap-compiler
	./build/compiler selfhost/compiler.noe build/noema_compiler.direct native-arm64

test-direct-examples: bootstrap-direct
	./tests/direct_examples.sh

direct-bootstrap-check: bootstrap-direct
	./build/noema_compiler.direct selfhost/compiler.noe build/noema_compiler.stage2 native-arm64
	shasum build/noema_compiler.direct.generated.s build/noema_compiler.stage2.generated.s
	./build/noema_compiler.stage2 examples/hello.noe build/hello.from_stage2 native-arm64
	./build/hello.from_stage2

clean:
	rm -rf build compiler/target
