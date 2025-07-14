
# TypeScript

## 기본 타입

| In Schema     | In TypeScript         |
| ----------    | --------------------- |
| string        | string                |
| stringLiteral | 'value'               |
| integer       | number                |
| boolean       | boolean               |
| array         | T[]                   |
| emptyObject   | Record<string, never> |
| json          | Record<string, any>   |

## Object

`properties`에 정의된 각 속성을 포함하는 객체 타입으로 변환됩니다.

```typescript
/**
 * 주소 정보
 */
type Address = {
  /**
   * 국가
   */
  country?: string;
  
  /**
   * 일반주소
   */
  addressLine1: string;
  
  /**
   * 상세주소
   */
  addressLine2: string;
}
```

## Empty Object

```typescript
/**
 * 빈 데이터
 */
type EmptyData = Record<string, never>;
```

## Enum

`variants` 속성에 정의된 enum 변형들이 상수 객체와 타입으로 변환됩니다.

`name` 이 지정되지 않은 경우 오류가 발생합니다.

### value_prefix가 없는 경우

```typescript
/**
 * 결제 상태
 */
const PaymentStatus = {
  /**
   * 준비됨
   */
  'READY': 'READY',
  
  /**
   * 승인됨
   */
  'APPROVED': 'APPROVED',
  
  /**
   * 취소됨
   */
  'CANCELLED': 'CANCELLED'
} as const;

/**
 * 결제 상태
 */
type PaymentStatus = (typeof PaymentStatus[keyof typeof PaymentStatus]);
```

### value_prefix가 있는 경우

```typescript
// value_prefix: "STATUS_"로 정의된 경우
/**
 * 결제 상태
 */
const PaymentStatus = {
  /**
   * 준비됨
   */
  'READY': 'STATUS_READY',
  
  /**
   * 승인됨
   */
  'APPROVED': 'STATUS_APPROVED',
  
  /**
   * 취소됨
   */
  'CANCELLED': 'STATUS_CANCELLED'
} as const;

/**
 * 결제 상태
 */
type PaymentStatus = (typeof PaymentStatus[keyof typeof PaymentStatus]);
```

## OneOf

`OneOfType` 유틸리티 타입을 이용해 `properties` 속성에 정의된 프로퍼티들로 OneOf 타입을 생성합니다.

```typescript
import type { OneOfType } from "../utils";

/**
 * 할부 개월 수 설정
 */
type MonthOption = OneOfType<{
  /**
   * 구매자가 선택할 수 없도록 고정된 할부 개월수
   */
  fixedMonth: number;
  
  /**
   * 구매자가 선택할 수 있는 할부 개월수 리스트
   */
  availableMonthList: number[];
}>;
```

## Union

`types` 배열에 정의된 여러 타입의 유니온으로 변환됩니다.

```typescript
/**
 * 파라미터 값
 */
type ParamValue = string | number | boolean;
```

## Intersection

`types` 배열에 정의된 여러 타입의 인터섹션으로 변환됩니다.

```typescript
/**
 * 사용자 프로필
 */
type UserProfile = (
  { 
    id: string;
    username: string; 
  }
) & (
  {
    address: string;
    phoneNumber: string;
  }
);
```

## ResourceRef

다른 리소스를 참조하는 타입으로, 해당 리소스의 타입을 가져오고 필요한 import 구문을 생성합니다.

```typescript
// $ref: "#/resources/common/Address"로 정의된 경우
import type { Address } from "../common/Address";

/**
 * 사용자 정보
 */
type User = {
  /**
   * 주소 정보
   */
  address: Address;
};
```

## Error

`transaction_type`과 `properties` 속성을 사용하여 에러 클래스와 타입가드 함수를 생성합니다.

### transaction_type이 있는 경우

```typescript
import { PortOneError, isPortOneError } from "#/resources/exception/index";

function isPaymentError(
  error: unknown
): error is PaymentError {
  return (
    isPortOneError(error) &&
    error.__portOneErrorType === 'PaymentError'
  );
}

class PaymentError extends Error implements PortOneError {
  static [Symbol.hasInstance](instance: unknown): boolean {
    return isPaymentError(instance);
  }
  __portOneErrorType = 'PaymentError';
  // transaction_type 설정
  transactionType = 'PAYMENT';
  
  /**
   * 에러 코드
   */
  code: string;
  
  /**
   * 에러 메시지
   */
  message: string;

  constructor({
    code,
    message
  }: {
    /**
     * 에러 코드
     */
    code: string;
    
    /**
     * 에러 메시지
     */
    message: string;
  }) {
    super(message);

    this.code = code;
    this.message = message;
  }
}
```

### transaction_type이 없는 경우

```typescript
import { PortOneError, isPortOneError } from "#/resources/exception/index";

function isGenericError(
  error: unknown
): error is GenericError {
  return (
    isPortOneError(error) &&
    error.__portOneErrorType === 'GenericError'
  );
}

class GenericError extends Error implements PortOneError {
  static [Symbol.hasInstance](instance: unknown): boolean {
    return isGenericError(instance);
  }
  __portOneErrorType = 'GenericError';
  // transaction_type 필드가 생성되지 않음
  
  /**
   * 에러 코드
   */
  code: string;

  constructor({
    code,
    message
  }: {
    /**
     * 에러 코드
     */
    code: string;
    message: string;
  }) {
    super(message);

    this.code = code;
  }
}
```

## Json

임의의 JSON 값을 위한 타입입니다.

```typescript
/**
 * 추가 데이터
 */
type AdditionalData = Record<string, any>;
```

## Parameter 공통 속성

### optional: true

모든 타입에 적용 가능하며, 선택적 필드로 표시됩니다.

```typescript
/**
 * 사용자 정보
 */
type User = {
  id: string;
  
  /**
   * optional: true로 설정된 필드
   */
  nickname?: string;
};
```

### description

필드 설명이 JSDoc 주석으로 추가됩니다.

```typescript
/**
 * 여기에 파라미터 설명이 들어갑니다.
 */
type PaymentInfo = {
  /**
   * 여기에 필드 설명이 들어갑니다.
   */
  amount: number;
};
```

### deprecated: true

deprecated 표시가 JSDoc 주석에 추가됩니다.

```typescript
/**
 * @deprecated
 */
type OldPaymentMethod = string;
```

### name

이름이 있는 파라미터는 별도의 타입으로 선언됩니다.

`parameterType` 이 `enum` 인 경우, 필수로 입력해야 합니다.

```typescript
/**
 * 이름이 있는 파라미터는 모듈 최상단에 별도 타입으로 정의됩니다.
 */
type CardNumber = string;

type Payment = {
  /**
   * 카드 번호
   */
  cardNumber: CardNumber;
};
```
