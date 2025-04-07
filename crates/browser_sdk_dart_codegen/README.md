# Dart

## 기본 타입

| In Schema | In Dart   |
| --------- | --------- |
| integer   | int       |
| number    | double    |
| boolean   | bool      |
| array     | List\<T\> |
| json      | dynamic   |

각 타입은 Extension Method `(int|double|bool|List<dynamic>) _toJson()`을
갖습니다.

## Object

```dart
/// 주소 정보
class Address {
    final Country? country;
    /// **일반주소**
    final String addressLine1;
    /// **상세주소**
    final String addressLine2;
    /// **도시**
    final String? city;
    /// **주, 도, 시**
    final String? province;

    Address({
        this.country,
        required this.addressLine1,
        required this.addressLine2,
        this.city,
        this.province
    });

    Map<String, dynamic> _toJson() => {
        if (country != null) 'country': country._toJson(),
        'addressLine1': addressLine1._toJson(),
        'addressLine2': addressLine2._toJson(),
        if (city != null) 'city': city._toJson(),
        if (province != null) 'province': province._toJson(),
    };
}
```

## Empty Object

```dart
class IssueBillingKeyRequestUnionPaypal {
    Map<String, dynamic> _toJson() => {};
}
```

## Enum

Dart에서는 non-ASCII 문자열을 identifier로 인정하지 않으므로, Enhaned Enum을
사용합니다.

```dart
/// 계좌이체, 가상계좌 발급시 사용되는 은행 코드
enum Bank {
    /// 한국은행
    BANK_OF_KOREA('BANK_OF_KOREA'),
    /// 산업은행
    KOREA_DEVELOPMENT_BANK('KOREA_DEVELOPMENT_BANK'),
    // ...
    /// 케이프투자증권
    CAPE_INVESTMENT_CERTIFICATE('CAPE_INVESTMENT_CERTIFICATE');

    final String _value;

    const Bank(String value): _value = value;

    String _toJson() => this._value;
}
```

## OneOf

```dart
/// **할부 개월 수 설정**
class MonthOption {
    /// **구매자가 선택할 수 없도록 고정된 할부 개월수**
    final int? fixedMonth;
    /// **구매자가 선택할 수 있는 할부 개월수 리스트**
    final List<int>? availableMonthList;

    MonthOption._internal({
        this.fixedMonth = null,
        this.availableMonthList = null,
    });

    MonthOption.fixedMonth(int fixedMonth) = this._internal(fixedMonth: fixedMonth);

    MonthOption.availableMonthList(List<int> availableMonthList) : this.internal(availableMonthList: availableMonthList);

    Map<String, dynamic> _toJson() => {
        if (fixedMonth != null) "fixedMonth": fixedMonth._toJson(),
        if (availableMonthList != null) "availableMonthList": availableMonthList._toJson(),
    };
}
```

## Union

Dart enum은 다른 class를 extends할 수 없어 일반적인 union의 구현이 어렵습니다.
variant에서 toUnionTypeName()을 호출할 수 있도록 합니다.

```dart
class LoadableUIType {
    final PaymentUIType? paymentUIType;
    final IssueBillingKeyUIType? issueBillingKeyUIType;

    LoadableUIType._internal({
        this.paymentUIType = null,
        this.issueBillingKeyUIType = null,
    });

    dynamic _toJson() => paymentUIType?._toJson() ?? issueBillingKeyUIType?._toJson();
}

enum PaymentUIType {
    PAYPAL_SPB('PAYPAL_SPB');

    final String _value;

    const PaymentUIType(String value): _value = value;

    String _toJson() => this._value;

    LoadableUIType toLoadableUIType() => LoadableUIType._internal(paymentUIType: this);
}

enum IssueBillingKeyUIType {
    PAYPAL_RT('PAYPAL_RT');

    final String _value;

    const IssueBillingKeyUIType(String value): _value = value;

    String _toJson() => this._value;

    LoadableUIType toLoadableUIType() => LoadableUIType._internal(issueBillingKeyUIType: this);
}
```

## Discriminated Union

현재는 각 variant가 한 union에서만 쓰이고 있지만, 여러 union에서 공유하는
variant가 생길 수 있으므로 discriminator을 union에 귀속시킴

```dart
class PaymentRequestUnion {
    // discriminator
    final String payMethod;
    final PaymentRequestUnionCard? CARD;
    // ...

    PaymentRequestUnion._internal(
        this.payMethod,
        {
            this.CARD = null,
            // ...
        }
    );

    Map<String, dynamic> _toJson() => {
        "payMethod": payMethod,
        ...?CARD?._toJson(),
    };    
}

/// **카드 정보**
class PaymentRequestUnionCard {
    // ...

    PaymentRequestUnion toPaymentRequestUnion() => PaymentRequestUnion._internal("CARD", CARD: this);
}
```

## Intersection

```dart
class PaymentRequest {
    final PaymentRequestBase paymentRequestBase;
    final PaymentRequestUnion paymentRequestUnion;

    PaymentRequest(this.paymentRequestBase, this.paymentRequestUnion);

    Map<String, dynamic> _toJson() => {
        ...paymentRequestBase,
        ...paymentRequestUnion,
    };
}
```
