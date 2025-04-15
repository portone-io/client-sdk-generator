# browser-sdk-generator

```sh
# nextest 설치
cargo install cargo-nextest
```

```sh
# schema.json 생성
cargo create-schema

# 테스트 실행
cargo nextest run --workspace

# ./output 디렉토리에 typescript 코드 생성
cargo generator-cli generate --schema ./browser-sdk.yml --generator typescript ./output
```
