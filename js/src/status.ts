export enum Status {
  // No errors.
  OK = "ok",

  // Used when a file path does not exist.
  FILE_NOT_FOUND_ERROR = "file-not-found-error",

  //Used when a file path exists, but there are permission issues, e.g., can't  read file.
  PERMISSION_ERROR = "permission-error",

  // Represents a generic error-like unknown status.
  UNKNOWN = "unknown",
}
