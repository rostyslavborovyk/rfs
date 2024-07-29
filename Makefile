test:
	RUST_MIN_STACK=10485760 cargo test

lint:
	cargo fix --allow-dirty --allow-staged

generate_meta_file:
	cargo run --bin generate_meta_file -- --path files/image.HEIC


setup_1:
	cargo run --bin serve_peer -- --address 127.0.0.1:8000 & \
	cargo run --bin serve_peer -- --address 127.0.0.1:8001 & \
	cargo run --bin run_ui
