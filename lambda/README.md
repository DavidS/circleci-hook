# Development

Run in emulator:

```shell
cargo lambda watch
```

execute test invocation (garbage data):
```shell
cargo lambda invoke --data-ascii '{"command": "hi"}' circleci-hook-lambda
```

(use `cargo watch` to invoke on every code-change)
