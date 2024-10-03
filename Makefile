run:
	pidof -s systemd | cargo run -- -

run-debug:
	pidof -s systemd | cargo run -- --debug -
