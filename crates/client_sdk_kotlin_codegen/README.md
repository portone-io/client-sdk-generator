# Kotlin

## 기본 타입

| In Schema | In Kotlin                      |
| --------- | ------------------------------ |
| integer   | Long                           |
| number    | Double                         |
| boolean   | Boolean                        |
| array     | List<T>                        |
| json      | @RawValue Map<String, Any?>    |

## Object

```kotlin
/**
 * 주소 정보
 */
@Parcelize
data class Address(
    val country: Country? = null,
    /**
     * **일반주소**
     */
    val addressLine1: String,
    /**
     * **상세주소**
     */
    val addressLine2: String,
    /**
     * **도시**
     */
    val city: String? = null,
    /**
     * **주, 도, 시**
     */
    val province: String? = null
) : Parcelable {
    fun toJson(): Map<String, Any> = buildMap {
        country?.let { put("country", country.toJson()) }
        put("addressLine1", addressLine1)
        put("addressLine2", addressLine2)
        city?.let { put("city", city) }
        province?.let { put("province", province) }
    }
}
```

### JSON 필드를 포함한 Object

JSON 타입 필드에는 `@RawValue` annotation이 자동으로 추가됩니다.

```kotlin
/**
 * 커스텀 데이터
 */
@Parcelize
data class CustomData(
    val id: String,
    /**
     * **추가 메타데이터**
     */
    val metadata: @RawValue Map<String, Any?>,
    val tags: List<String>? = null
) : Parcelable {
    fun toJson(): Map<String, Any> = buildMap {
        put("id", id)
        put("metadata", metadata)
        tags?.let { put("tags", tags) }
    }
}
```

## Empty Object

```kotlin
@Parcelize
class IssueBillingKeyRequestUnionPaypal : Parcelable {
    fun toJson(): Map<String, Any> = emptyMap()
}
```

## Enum

Kotlin에서는 `EnumClass.valueOf(value: String)` 함수를 통해 string identifier로 enum type을 도출할 수 있습니다.

```kotlin
/**
 * 계좌이체, 가상계좌 발급시 사용되는 은행 코드
 */
enum class Bank {
    /**
     * 한국은행
     */
    BANK_OF_KOREA,
    /**
     * 산업은행
     */
    KOREA_DEVELOPMENT_BANK,
    // ...
    /**
     * 케이프투자증권
     */
    CAPE_INVESTMENT_CERTIFICATE;

    fun toJson(): String = name
}

Bank.valueOf("BANK_OF_KOREA") // Bank.BANK_OF_KOREA
```

### 숫자로 시작하는 variant 처리

숫자로 시작하는 식별자는 `_` prefix가 붙으며, `toJson()`에서 원래 값으로 매핑됩니다.

```kotlin
/**
 * 결제 수단
 */
enum class PaymentMethod {
    /**
     * 2Checkout 결제
     */
    _2checkout,
    /**
     * 3D Secure 인증
     */
    _3ds,
    /**
     * 카드 결제
     */
    card;

    fun toJson(): String = when (this) {
        _2checkout -> "2checkout"
        _3ds -> "3ds"
        card -> "card"
    }
}
```

## OneOf

Kotlin에서는 `sealed interface`를 사용하여 OneOf 타입을 구현합니다.

```kotlin
/**
 * **할부 개월 수 설정**
 */
@Parcelize
sealed interface MonthOption : Parcelable {
    /**
     * **구매자가 선택할 수 없도록 고정된 할부 개월수**
     */
    @Parcelize
    data class FixedMonth(val value: Long) : MonthOption
    /**
     * **구매자가 선택할 수 있는 할부 개월수 리스트**
     */
    @Parcelize
    data class AvailableMonthList(val value: List<Long>) : MonthOption

    fun toJson(): Map<String, Any> = when (this) {
        is FixedMonth -> mapOf("fixedMonth" to value)
        is AvailableMonthList -> mapOf("availableMonthList" to value)
    }
}
```

## Union

여러 타입 중 하나를 선택하는 Union 타입은 `sealed class`로 구현됩니다. 이름 충돌을 방지하기 위해 `private typealias`를 사용합니다.

```kotlin
private typealias _PaymentUIType = PaymentUIType
private typealias _IssueBillingKeyUIType = IssueBillingKeyUIType

@Parcelize
sealed class LoadableUIType : Parcelable {
    @Parcelize
    data class PaymentUiType(val value: _PaymentUIType) : LoadableUIType()
    @Parcelize
    data class IssueBillingKeyUiType(val value: _IssueBillingKeyUIType) : LoadableUIType()

    fun toJson(): Any = when (this) {
        is PaymentUiType -> value.toJson()
        is IssueBillingKeyUiType -> value.toJson()
    }
}
```

## Intersection

여러 타입의 필드를 평탄화(flatten)하여 하나의 `data class`로 합칩니다.

```kotlin
/**
 * 결제 요청 정보
 */
@Parcelize
data class PaymentRequest(
    /**
     * 결제 금액
     */
    val amount: Long,
    /**
     * 통화 코드
     */
    val currency: String,
    /**
     * 결제 수단
     */
    val method: PaymentMethod,
    /**
     * 카드 정보
     */
    val cardInfo: CardInfo? = null
) : Parcelable {
    fun toJson(): Map<String, Any> = buildMap {
        put("amount", amount)
        put("currency", currency)
        put("method", method.toJson())
        cardInfo?.let { put("cardInfo", cardInfo.toJson()) }
    }
}
```
