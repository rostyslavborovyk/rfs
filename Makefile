test:
	RUST_MIN_STACK=10485760 cargo test

lint:
	cargo fix --allow-dirty --allow-staged

generate_meta_file:
	cargo run --bin generate_meta_file -- --path files/image.HEIC

start_local_peer:
	cargo run --bin serve_peer

setup_1:
	cargo run --bin serve_peer -- --address 127.0.0.1:8000 & \
	cargo run --bin serve_peer -- --address 127.0.0.1:8001 & \
	cargo run --bin run_ui

kill_setup_1:
	kill $(ps -a | grep serve_peer | awk '{print $1}')
