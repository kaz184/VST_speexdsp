#!/usr/bin/env fish

cargo build --target=x86_64-pc-windows-gnu
and cp ./target/x86_64-pc-windows-gnu/debug/rust_vst.dll ~/Sync/Repo/speex_vst
