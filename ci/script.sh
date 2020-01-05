# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET #-- #--version
    cross build --target $TARGET --release #-- --version

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET #-- #--version
    cross test --target $TARGET --release #-- --version

    cross run --target $TARGET -- --version
    cross run --target $TARGET --release -- --version
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
