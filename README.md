# browser-sdk-generator

YAML 스키마 파일을 기반으로 다양한 환경의 포트원 SDK 코드를 자동 생성하는 도구입니다.

## 프로젝트 구조

```
crates/
├── browser_sdk_schema/          # 핵심 스키마 정의 및 파싱 라이브러리
├── sdk_generator/               # 메인 CLI 애플리케이션
├── browser_sdk_ts_codegen/      # TypeScript 코드 생성기
├── browser_sdk_dart_codegen/    # Dart 코드 생성기
└── ...
```

## 빠른 시작

### 사전 요구사항

```sh
# nextest 설치
cargo install cargo-nextest
```

### 개발 워크플로우

```sh
# browser-sdk.schema.json 재생성
cargo run -p browser_sdk_schema --bin generate_schema

# 테스트 실행
cargo nextest run --workspace

# TypeScript 코드 생성
cargo run -p sdk_generator generate --schema ./browser-sdk.yml --generator typescript ./output

# Dart 코드 생성
cargo run -p sdk_generator generate --schema ./browser-sdk.yml --generator dart ./output
```

## 스키마 파일

- `browser-sdk.yml`: 메인 스키마 정의 파일
- `browser-sdk.schema.json`: IDE 지원을 위한 JSON 스키마 파일

## 지원 언어

- **TypeScript**: 브라우저용 포트원 SDK
- **Dart**: Flutter용 포트원 SDK

이 프로젝트는 [GNU Affero General Public License v3.0] 또는 그 이후 버전에 따라 라이센스가 부여됩니다. 자세한 내용은 [COPYRIGHT] 파일을 참고하세요.

[GNU Affero General Public License v3.0]: LICENSE
[COPYRIGHT]: COPYRIGHT
