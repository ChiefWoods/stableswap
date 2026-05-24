export default {
  "programs/*/src/**/*.rs": () => [
    "cargo fmt --all --",
    "cargo clippy --all --all-features --fix --allow-dirty --allow-staged --all-targets -- -D warnings"
  ]
};