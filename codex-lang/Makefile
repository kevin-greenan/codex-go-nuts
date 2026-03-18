IMAGE := codex-lang-rust

.PHONY: image compiler examples selfhost-example clean

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

native-example:
	NOEMA_BACKEND=native-arm64 ./bin/codexc examples/hello.noe build/hello.native

clean:
	rm -rf build compiler/target
