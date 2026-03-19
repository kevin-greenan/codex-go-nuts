IMAGE := codex-lang-rust

.PHONY: image compiler examples native-examples selfhost-example selfhost-check parity-check clean

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
	./bin/codexc selfhost/mini_compiler.noe build/mini_compiler

selfhost-example:
	./bin/codexc selfhost/mini_compiler.noe build/mini_compiler
	./build/mini_compiler examples/series.noe build/series.selfhost.generated.c
	cc -O3 build/series.selfhost.generated.c -o build/series.selfhost

selfhost-check:
	./bin/codexc selfhost/mini_compiler.noe build/mini_compiler
	./build/mini_compiler examples/mini_source.noe build/mini_source.generated.c
	cc -O3 build/mini_source.generated.c -o build/mini_source.generated
	./build/mini_source.generated || test $$? -eq 5
	./build/mini_compiler examples/series.noe build/series.selfhost.generated.c
	cc -O3 build/series.selfhost.generated.c -o build/series.selfhost
	./build/series.selfhost
	./build/mini_compiler examples/selfhost_text.noe build/selfhost_text.generated.c
	cc -O3 build/selfhost_text.generated.c -o build/selfhost_text.generated
	./build/selfhost_text.generated examples/hello.noe build/selfhost_text.out
	rg -n "selfhost" build/selfhost_text.out
	./build/mini_compiler examples/frontend_demo.noe build/frontend_demo.selfhost.generated.c
	cc -O3 build/frontend_demo.selfhost.generated.c -o build/frontend_demo.selfhost
	./build/frontend_demo.selfhost examples/hello.noe

parity-check: selfhost-check native-examples

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
