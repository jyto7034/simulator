// Interpret the stored number's canonical decimal value, not a rounded product
// of binary floats: e.g. 1.001 * 1000 is not exactly 1001 in JavaScript.
export function secondsToMilliseconds(seconds: number): number | null {
  if (!Number.isFinite(seconds) || seconds < 0) return null;
  const [coefficient, exponent = "0"] = String(seconds).split("e");
  const [whole, fraction = ""] = coefficient.split(".");
  const shift = Number(exponent) + 3 - fraction.length;
  if (shift < 0) return null;
  const milliseconds = BigInt(whole + fraction) * 10n ** BigInt(shift);
  return milliseconds <= BigInt(Number.MAX_SAFE_INTEGER)
    ? Number(milliseconds)
    : null;
}

export const timeValueMessage =
  "0 이상의 시간을 0.001초 단위로 입력하세요. 밀리초 값은 안전한 정수 범위여야 합니다.";
