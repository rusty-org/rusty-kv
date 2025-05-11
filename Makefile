.PHONY: run-server
run-server:
	@cargo run

.PHONY: format
format:
	@cargo fmt -- --config-path .rustfmt.toml

.PHONY: clean
clean:
	@echo "Cleaning up..." && \
		rm -rf ./target && \
		echo "Cleaned up successfully"
