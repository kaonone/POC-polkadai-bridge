const floatRegExp = /^\d+?([.]|[.]\d+)?$/;

function makeFloatDecimalsRegExp(decimals: number) {
  return new RegExp(`^\\d+?([.]|[.]\\d{1,${decimals}})?$`);
}

export function validateFloat(value: string, decimals: number): string | undefined {
  return (
    !floatRegExp.test(value) && 'Enter a valid number' ||
    !makeFloatDecimalsRegExp(decimals).test(value) && `Enter a valid number with decimals less than ${decimals} digits` ||
    undefined
  );
}
