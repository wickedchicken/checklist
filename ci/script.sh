# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    if [ "$TRAVIS_OS_NAME" == "linux" ]; then
        yamllint -s .checklist.yml .travis.yml
    fi
    cargo fmt --all -- --check
    cross build --target $TARGET
    # Re-enable for all targets once https://github.com/rust-lang/rust/issues/62558 is
    # fixed.
    if [ "$TARGET" == "x86_64-unknown-linux-gnu" ]; then
        cross clippy -- --target $TARGET -D warnings
    fi
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
