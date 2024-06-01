#!/bin/sh
set -e

echo "======================= test ======================="
cargo -q test
echo "===================== tarpaulin ===================="
cargo -q tarpaulin --ignore-tests -- -q
echo "====================== clippy ======================"
cargo -q clippy -- -D warnings
echo "======================== fmt ======================="
cargo -q fmt -- --check
echo "======================= audit ======================"
# RUSTSEC-2021-0146, RUSTSEC-2023-0028, RUSTSEC-2023-0050, RUSTSEC-2023-0081: https://github.com/tomaka/rouille/issues/271
# RUSTSEC-2024-0436: https://github.com/leptos-rs/leptos/issues/3685
cargo -q audit \
	--ignore RUSTSEC-2021-0146 \
	--ignore RUSTSEC-2023-0028 \
	--ignore RUSTSEC-2023-0050 \
	--ignore RUSTSEC-2023-0081 \
	--ignore RUSTSEC-2024-0436
