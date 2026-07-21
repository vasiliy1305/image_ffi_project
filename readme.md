cargo build --workspace

# Зеркальный разворот
cargo run -p image_processor -- \
  --input input.png \
  --output out_mirror.png \
  --plugin mirror \
  --params params/mirror.json

# Размытие
cargo run -p image_processor -- \
  --input input.png \
  --output out_blur.png \
  --plugin blur \
  --params params/blur.json


