test:
	RUST_MIN_STACK=10485760 cargo test

lint:
	cargo fix --allow-dirty --allow-staged

generate_meta_file:
	 cargo run --bin generate_meta_file -- --path files/image.HEIC
