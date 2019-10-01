/**
 * @summary
 * Checks error, caught in try/catch block and returns correct error representation of that
 */
function getErrorMsg(error: any): string {
  return error
    ? (error.message || String(error))
    : 'Unknown error';
}

export default getErrorMsg;
