# Dart

## 기본 타입

| In Schema | In Dart   |
| --------- | --------- |
| integer   | int       |
| number    | double    |
| boolean   | bool      |
| array     | List\<T\> |
| json      | Object    |

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

    Map<String, dynamic> toJson() => {
        if (country != null) 'country': country.toJson(),
        'addressLine1': addressLine1,
        'addressLine2': addressLine2,
        if (city != null) 'city': city!,
        if (province != null) 'province': province!,
    };
}
```

## Empty Object

```dart
class IssueBillingKeyRequestUnionPaypal {
    Map<String, dynamic> toJson() => {};
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

    String toJson() => this._value;
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

    MonthOption.internal({
        this.fixedMonth,
        this.availableMonthList,
    });

    MonthOption.fixedMonth(int fixedMonth) = this.internal(fixedMonth: fixedMonth);

    MonthOption.availableMonthList(List<int> availableMonthList) : this.internal(availableMonthList: availableMonthList);

    Map<String, dynamic> toJson() => {
        if (fixedMonth != null) "fixedMonth": fixedMonth.toJson(),
        if (availableMonthList != null) "availableMonthList": availableMonthList.toJson(),
    };
}
```

## Union

Dart enum은 다른 class를 extends할 수 없어 일반적인 union의 구현이 어렵습니다.
variant에서 toUnionTypeName()을 호출할 수 있도록 합니다.

```dart
// Usage
LoadableUIType unionValue = PaymentUIType.PAYPAL_SPB.toLooadableUIType();

class LoadableUIType {
    final PaymentUIType? paymentUIType;
    final IssueBillingKeyUIType? issueBillingKeyUIType;

    LoadableUIType.internal({
        this.paymentUIType,
        this.issueBillingKeyUIType,
    });

    dynamic toJson() => paymentUIType?.toJson() ?? issueBillingKeyUIType?.toJson();
}

enum PaymentUIType {
    PAYPAL_SPB('PAYPAL_SPB');

    final String _value;

    const PaymentUIType(String value): _value = value;

    String toJson() => this._value;

    LoadableUIType toLoadableUIType() => LoadableUIType.internal(paymentUIType: this);
}

enum IssueBillingKeyUIType {
    PAYPAL_RT('PAYPAL_RT');

    final String _value;

    const IssueBillingKeyUIType(String value): _value = value;

    String toJson() => this._value;

    LoadableUIType toLoadableUIType() => LoadableUIType.internal(issueBillingKeyUIType: this);
}
```

## Intersection

```dart
class PaymentRequest {
    final PaymentRequestBase paymentRequestBase;
    final PaymentRequestUnion paymentRequestUnion;

    PaymentRequest(this.paymentRequestBase, this.paymentRequestUnion);

    Map<String, dynamic> toJson() => {
        ...paymentRequestBase.toJson(),
        ...paymentRequestUnion.toJson(),
    };
}
```
