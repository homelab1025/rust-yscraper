# Task Completion Steps

Before considering a task finished:
1. Run `cargo fmt --all`.
2. Run `cargo clippy --all-targets --all-features -D warnings`.
3. Run `cargo test -p web_server`.
4. (Optional) Run `cargo llvm-cov --html` for coverage check.
5. List changed entities, flows, and design patterns.
6. For large changes, create diagrams in the `docs/` folder.
