IMAGE := codex-lang-rust

.PHONY: image compiler examples native-examples selfhost-example selfhost-check parity-check bootstrap-compiler bootstrap-check direct-check clean

image:
	docker build -t $(IMAGE) -f container/Dockerfile .

compiler:
	docker run --rm -v "$$(pwd)":/workspace -w /workspace $(IMAGE) cargo build --manifest-path compiler/Cargo.toml --release

examples:
	./bin/codexc examples/hello.noe build/hello
	./bin/codexc examples/series.noe build/series
	./bin/codexc examples/frontend_demo.noe build/frontend_demo
	./bin/codexc examples/socket_probe.noe build/socket_probe
	./bin/codexc examples/http_get.noe build/http_get
	./bin/codexc selfhost/compiler_1.noe build/compiler_1

selfhost-example:
	./bin/codexc selfhost/compiler_1.noe build/compiler_1
	./build/compiler_1 examples/series.noe build/series.selfhost

bootstrap-compiler:
	./bin/codexc selfhost/compiler_1.noe build/compiler_1
	./build/compiler_1 selfhost/compiler_1.noe build/noema_compiler

selfhost-check: bootstrap-compiler
	./build/compiler_1 examples/mini_source.noe build/mini_source.generated
	./build/mini_source.generated || test $$? -eq 5
	./build/compiler_1 examples/series.noe build/series.selfhost
	./build/series.selfhost
	./build/compiler_1 examples/selfhost_text.noe build/selfhost_text.generated
	./build/selfhost_text.generated examples/hello.noe build/selfhost_text.out
	rg -n "selfhost" build/selfhost_text.out
	./build/compiler_1 examples/frontend_demo.noe build/frontend_demo.selfhost
	./build/frontend_demo.selfhost examples/hello.noe

parity-check: selfhost-check native-examples

bootstrap-check: bootstrap-compiler
	./build/noema_compiler examples/frontend_demo.noe build/frontend_demo.bootstrap
	./build/frontend_demo.bootstrap examples/hello.noe

direct-check:
	./bin/codexc selfhost/compiler_1.noe build/compiler_1
	./build/compiler_1 examples/hello.noe build/hello.direct native-arm64
	./build/hello.direct
	./build/compiler_1 examples/mini_source.noe build/mini_source.direct native-arm64
	./build/mini_source.direct || test $$? -eq 5
	./build/compiler_1 examples/series.noe build/series.direct native-arm64
	./build/series.direct

native-example:
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/hello.noe build/hello.native

native-examples:
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/hello.noe build/hello.native
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/mini_source.noe build/mini_source.native
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/series.noe build/series.native
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/frontend_demo.noe build/frontend_demo.native
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/socket_probe.noe build/socket_probe.native
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/http_get.noe build/http_get.native

clean:
	rm -rf build compiler/target
