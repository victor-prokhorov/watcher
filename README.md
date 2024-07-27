# watcher

super simple file watcher

## examples

```sh
watcher 'README.md src/main.rs' 'cat README.md'
watcher 'impl.mips runner.mips' 'java -jar $HOME/bin/Mars45.jar nc runner.mips impl.mips'
```

## installation

```sh
cargo build --release
cargo install --path .
```
