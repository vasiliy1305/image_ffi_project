
## плагины

### Mirror

 параметры:

```json
{
    "horizontal": true,
    "vertical": false
}
```
---

### Blur

параметры:

```json
{
    "radius": 4,
    "iterations": 3
}
```

## Сборка 

```bash
cargo build --workspace
```
## Запуск

### Зеркальное отражение

```bash
cargo run -p image_processor -- \
    --input input.png \
    --output out_mirror.png \
    --plugin mirror \
    --params params/mirror.json
```

### Размытие

```bash
cargo run -p image_processor -- \
    --input input.png \
    --output out_blur.png \
    --plugin blur \
    --params params/blur.json
```



```bash
cargo test --workspace
```
