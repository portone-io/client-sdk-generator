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

이 프로젝트는 [GNU Affero General Public License v3.0] 또는 그 이후 버전에 따라 라이센스가 부여됩니다. 자세한 내용은 [COPYRIGHT] 파일을 참고하세요.

[GNU Affero General Public License v3.0]: LICENSE
[COPYRIGHT]: COPYRIGHT