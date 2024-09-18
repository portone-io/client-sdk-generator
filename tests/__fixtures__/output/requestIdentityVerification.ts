import type * as Parameters from "./parameters/index.js";

export type requestIdentityVerification = (identityVerificationRequest: {
  /**
   * ### 상점 ID
   * - 포트원에서 채번하는 상점 ID입니다.
   * - 관리자콘솔의 결제 연동 페이지에서 확인하실 수 있습니다.
   */
  storeId: Parameters.storeId;
  /**
   * ### 채널 키
   * - 콘솔에서 표시되는 채널 키입니다.
   * - `pgProvider` 파라미터가 없는 경우에 필수로 존재해야 합니다. 두 파라미터가 모두 존재하는 경우 `channelKey`를 적용하니 둘 중 하나만 제공해주세요.
   */
  channelKey: Parameters.channelKey;
  /**
   * ### 본인인증건 고유 번호
   * - 고객사가 채번하는 본인인증 건에 대한 고유 번호입니다.
   * - 이미 본인인증이 완료된 `identityVerificationId`로 다시 본인인증을 시도하는 경우 에러가 발생합니다.
   */
  identityVerificationId: string;
}) => Promise<void>;
