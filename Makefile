.PHONY: build-cli
build-cli:
	@echo "Building CLI..." && \
		mkdir -p ./cli/tmp && \
		g++ -Wall -o ./cli/tmp/cli \
			$(shell find . -name "*.cpp") \
			-licuuc \
			-licuio && \
		echo "CLI built successfully, binary -> ./cli/tmp/cli"

.PHONY: run-cli
run-cli: build-cli
	./cli/tmp/cli

.PHONY: run-server
run-server:
	@cargo run

.PHONY: run
run:
	@$(MAKE) -j2 run-cli run-server

.PHONY: format
format:
	@cargo fmt -- --config-path .rustfmt.toml
	@clang-format -i -style=file \
		$(shell find . -name "*.cpp") \
		$(shell find . -name "*.hpp")

.PHONY: clean
clean:
	@echo "Cleaning up..." && \
		rm -rf ./cli/tmp && \
		rm -rf ./target && \
		echo "Cleaned up successfully"
