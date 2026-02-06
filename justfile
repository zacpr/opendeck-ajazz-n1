id := "net.ashurtech.plugins.opendeck-ajazz-n1.sdPlugin"

release: bump package tag

package: build-linux collect zip

bump next=`git cliff --bumped-version | tr -d "v"`:
    git diff --cached --exit-code

    echo "We will bump version to {{next}}, press any key"
    read ans

    sed -i 's/"Version": ".*"/"Version": "{{next}}"/g' manifest.json
    sed -i 's/^version = ".*"$/version = "{{next}}"/g' Cargo.toml

tag next=`git cliff --bumped-version`:
    echo "Generating changelog"
    git cliff -o CHANGELOG.md --tag {{next}}

    echo "We will now commit the changes, please review before pressing any key"
    read ans

    git add .
    git commit -m "chore(release): {{next}}"
    git tag "{{next}}"

build-linux:
    cargo build --release --target x86_64-unknown-linux-gnu --target-dir target/plugin-linux

build-mac:
    docker run --rm -it -v $(pwd):/io -w /io ghcr.io/rust-cross/cargo-zigbuild:sha-eba2d7e cargo zigbuild --release --target universal2-apple-darwin --target-dir target/plugin-mac

build-win:
    cargo build --release --target x86_64-pc-windows-gnu --target-dir target/plugin-win

clean:
    sudo rm -rf target/ build/

collect:
    rm -rf build
    mkdir -p build/{{id}}/scripts
    cp assets/icon.png build/{{id}}/icon.png
    cp manifest.json build/{{id}}
    cp -r scripts/* build/{{id}}/scripts/
    cp target/plugin-linux/x86_64-unknown-linux-gnu/release/opendeck-ajazz-n1 build/{{id}}/opendeck-ajazz-n1-linux
    @echo ""
    @echo "✓ Plugin files collected to: $(pwd)/build/{{id}}/"
    @echo ""

[working-directory: "build"]
zip:
    zip -r opendeck-ajazz-n1.sdPlugin {{id}}/
    @echo ""
    @echo "Build output dir: $(pwd)"
    @echo "✓ Plugin package created: $(pwd)/opendeck-ajazz-n1.sdPlugin"
    @echo ""
