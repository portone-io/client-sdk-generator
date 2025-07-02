# @portone/client-sdk-generator

## 0.1.4

### Patch Changes

- [#23](https://github.com/portone-io/client-sdk-generator/pull/23) [`6c20637`](https://github.com/portone-io/client-sdk-generator/commit/6c2063749d786661a14662cc2ef4b8c81c09c671) Thanks [@CirnoV](https://github.com/CirnoV)! - \[TypeScript\] Array 타입이 Tuple 형태로 생성되는 문제 해결

## 0.1.3

### Patch Changes

- [#21](https://github.com/portone-io/client-sdk-generator/pull/21) [`23c20c4`](https://github.com/portone-io/client-sdk-generator/commit/23c20c4af341bfd8f4f4f75810764b70cbad400b) Thanks [@CirnoV](https://github.com/CirnoV)! - Enum 타입이 prefix가 있는 값과 없는 값을 모두 허용하도록 개선

  - Enum 타입 정의가 이제 두 가지 형태의 값을 모두 받을 수 있습니다
    - prefix 없는 값: `'CARD'`
    - prefix 있는 값: `'PG_CARD'` (value_prefix가 'PG'인 경우)

## 0.1.2

### Patch Changes

- [`b6af13a`](https://github.com/portone-io/client-sdk-generator/commit/b6af13ad35a4ca486144caaaa0a2ae4fa89b324a) Thanks [@CirnoV](https://github.com/CirnoV)! - Enum 타입 파라미터를 참조하는 referenceRef 생성 시, 항상 ImportEntry의 `is\_type\_only` 를 false로 함

## 0.1.1

### Patch Changes

- [#18](https://github.com/portone-io/client-sdk-generator/pull/18) [`0ecc4bb`](https://github.com/portone-io/client-sdk-generator/commit/0ecc4bbe81d0af6d3610ecec12cd34a8f937aa4d) Thanks [@CirnoV](https://github.com/CirnoV)! - ResourceRef 타입인 Named Parameter가 enum 타입을 가리킬 때 const 선언을 같이 생성하도록 수정

- [#18](https://github.com/portone-io/client-sdk-generator/pull/18) [`0ecc4bb`](https://github.com/portone-io/client-sdk-generator/commit/0ecc4bbe81d0af6d3610ecec12cd34a8f937aa4d) Thanks [@CirnoV](https://github.com/CirnoV)! - TypeScript 코드 파싱 오류 메시지 개선

## 0.1.0

### Minor Changes

- [#17](https://github.com/portone-io/client-sdk-generator/pull/17) [`583e10f`](https://github.com/portone-io/client-sdk-generator/commit/583e10fbd11cee32a4f8f2dda28e7a1d94e6aeb0) Thanks [@CirnoV](https://github.com/CirnoV)! - - 모든 TypeScript 파일 생성 시 생성된 파일임을 알려주는 헤더 주석 추가
