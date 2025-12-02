local-install:
  cargo build --release
  sudo cp ./target/release/historic /usr/bin/
