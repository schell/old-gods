#! /bin/bash

section() {
    echo "--- $(TZ=UTC date +%Y%m%d-%H:%M:%S) - $1"
}

section "Rust Setup"

if [ -z ${GITHUB_REF+x} ]; then
    export GITHUB_REF=`git rev-parse --symbolic-full-name HEAD`
fi

export PATH=$PATH:$HOME/.cargo/bin

if hash rustup 2>/dev/null; then
    echo "Have rustup, skipping installation..."
else
    echo "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    echo "  done installing the rust toolchain."
fi

rustup update
rustup toolchain install nightly

if hash wasm-pack 2>/dev/null; then
    echo "Have wasm-pack, skipping installation..."
else
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    echo "  done installing wasm-pack."
fi

#if hash wasm-opt 2>/dev/null; then
#    echo "Have wasm-opt and other wasm tools, skipping installation..."
#else
#    echo "Installing wasm cli tools..."
#    git clone https://github.com/WebAssembly/binaryen.git
#    cd binaryen
#    cmake .
#    make
#    cp bin/* $HOME/.cargo/bin
#    echo "  done installing wasm tools."
#    cd ..
#fi

echo "Building w/ cargo..."
cargo build || exit 1

echo "Building w/ wasm-pack..."
wasm-pack build --release --target web examples/loading-maps || exit 1

#echo "Optimizing..."
#wasm-opt -Os ad-to-bag/pkg/ad_to_bag_client_bg.wasm -o ad-to-bag/pkg/ad_to_bag_client_bg.wasm || exit 1
#wasm2js -Oz ad-to-bag/pkg/ad_to_bag_client_bg.wasm -o ad-to-bag/pkg/ad_to_bag_client_bg.js || exit 1
#bash scripts/js_min.sh ad-to-bag/pkg/ad_to_bag_client_bg.js ad-to-bag/pkg/ad_to_bag_client_bg.min.js || exit 1
echo "Done building on ${GITHUB_REF}"
