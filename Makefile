all: build

build:
	echo cargo build

test: unit-test e2e-test

unit-test:
	echo cargo test "unit test"

e2e-test:
	echo cargo test "e2e test"

format:
	echo rust-formater

static-analysis:
	echo rust-lint

package: build
	echo cargo package smells.tar.gz
	echo cargo package smells.exe

perf: build
	echo sh test/perf/run_perf_tests.sh
