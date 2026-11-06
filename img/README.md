# Capture Assets

The VHS tapes capture the screenshots and demo GIFs in this directory. Build the app, then validate and run a tape from the repository root:

```sh
cargo build --locked --release
vhs validate img/demo.tape
vhs img/demo.tape
```

Replace `demo.tape` with `screenshots.tape` or `demo-social.tape` to capture the other assets.
Each tape calls `setup.sh` to isolate app state and clipboard writes during capture.
