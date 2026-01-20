# client-sdk-generator

YAML 스키마 파일을 기반으로 다양한 환경의 포트원 SDK 코드를 자동 생성하는 도구입니다.

### 설치

```sh
# 설치
pnpm add -D -E @portone/client-sdk-generator

# TypeScript 코드 생성
pnpm portone-client-sdk-generator generate --schema ./client-sdk.yml --generator typescript ./output

# Dart 코드 생성
pnpm portone-client-sdk-generator generate --schema ./client-sdk.yml --generator dart ./output

# Kotlin 코드 생성 (Android SDK)
pnpm portone-client-sdk-generator generate --schema ./client-sdk.yml --generator kotlin ./output
```

### 개발 워크플로우

```sh
# schema.json 재생성
cargo run -p client_sdk_schema --bin generate_schema

# 테스트 실행
cargo test --workspace

# CLI 사용
cargo run -p client_sdk_generator
```

### 지원 언어

- **TypeScript**: 브라우저용 포트원 SDK
- **Dart**: Flutter용 포트원 SDK
- **Kotlin**: Android용 포트원 SDK

이 프로젝트는 [GNU Affero General Public License v3.0] 또는 그 이후 버전에 따라 라이센스가 부여됩니다. 자세한 내용은 [COPYRIGHT] 파일을 참고하세요.

[GNU Affero General Public License v3.0]: LICENSE
[COPYRIGHT]: COPYRIGHT
