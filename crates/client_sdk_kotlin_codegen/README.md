# Kotlin

## 기본 타입

| In Schema | In Kotlin                                                   |
| --------- | ----------------------------------------------------------- |
| integer   | Int                                                         |
| number    | Double                                                      |
| boolean   | Boolean                                                     |
| array     | List<T>                                                     |
| json      | Map<String, Any?> // 정확히 1:1로 대응되는 타입은 별도 없음 |

## Object

```kotlin
/**
 * 주소 정보
 */
data class Address(
    val country: Country?,
    /** **일반주소** */
    val addressLine1: String,
    /** **상세주소** */
    val addressLine2: String,
    /** **도시** */
    val city: String?,
    /** **주, 도, 시** */
    val province: String?
) {
    fun toJson(): Map<String, Any?> = mapOf(
        "country" to country?.toJson(),
        "addressLine1" to addressLine1,
        "addressLine2" to addressLine2,
        "city" to city,
        "province" to province
    )
}
```

## Empty Object

```kotlin
class IssueBillingKeyRequestUnionPaypal {
    fun toJson(): Map<String, Any> = emptyMap()
}
```

## Enum

Kotlin에서는 EnumClass.valueOf(value: String) 함수를 통해 string identifier로 enum type을 도출할 수 있습니다.

```kotlin
/**
 * 계좌이체, 가상계좌 발급시 사용되는 은행 코드
 */
enum class Bank {
    /** 한국은행 */
    BANK_OF_KOREA,
    /** 산업은행 */
    KOREA_DEVELOPMENT_BANK,
    // ...
    /** 케이프투자증권 */
    CAPE_INVESTMENT_CERTIFICATE;

    fun toJson(): String = name
}

Bank.valueOf("BANK_OF_KOREA") //  Bank.BANK_OF_KOREA
```

## OneOf

Kotlin에서는 sealed class, interface를 사용하여 OneOf 타입을 구현합니다.

```kotlin
/**
 * **할부 개월 수 설정**
 */
sealed interface MonthOption {
    /** **구매자가 선택할 수 없도록 고정된 할부 개월수** */
    data class FixedMonth(val value: Int) : MonthOption

    /** **구매자가 선택할 수 있는 할부 개월수 리스트** */
    data class AvailableMonthList(val value: List<Int>) : MonthOption

    fun toJson(): Map<String, Any> = when (this) {
        is FixedMonth -> mapOf("fixedMonth" to value)
        is AvailableMonthList -> mapOf("availableMonthList" to value)
    }
}
```
