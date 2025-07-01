---
"@portone/client-sdk-generator": patch
---

Enum 타입이 prefix가 있는 값과 없는 값을 모두 허용하도록 개선

- Enum 타입 정의가 이제 두 가지 형태의 값을 모두 받을 수 있습니다
  - prefix 없는 값: `'CARD'`
  - prefix 있는 값: `'PG_CARD'` (value_prefix가 'PG'인 경우)
