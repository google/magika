/*
	Installed from @ieedan/shadcn-svelte-extras
*/

import type { WithChildren } from "bits-ui";
import type { HTMLInputAttributes } from "svelte/elements";

export type FileRejectedReason =
  | "Maximum file size exceeded"
  | "File type not allowed"
  | "Maximum files uploaded";

export type FileDropZonePropsWithoutHTML = WithChildren<{
  ref?: HTMLInputElement | null;
  /** Called with the uploaded files when the user drops or clicks and selects their files.
   *
   * @param files
   */
  onUpload: (files: File[]) => Promise<void>;
  /** The maximum amount files allowed to be uploaded */
  maxFiles?: number;
  fileCount?: number;
  /** The maximum size of a file in bytes */
  maxFileSize?: number;
  /** Called when a file does not meet the upload criteria (size, or type) */
  onFileRejected?: (opts: { reason: FileRejectedReason; file: File }) => void;

  // just for extra documentation
  /** Takes a comma separated list of one or more file types.
   *
   *  [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/accept)
   *
   * ### Usage
   * ```svelte
   * <FileDropZone
   * 		accept=".doc,.docx,application/msword,application/vnd.openxmlformats-officedocument.wordprocessingml.document"
   * />
   * ```
   *
   * ### Common Values
   * ```svelte
   * <FileDropZone accept="audio/*"/>
   * <FileDropZone accept="image/*"/>
   * <FileDropZone accept="video/*"/>
   * ```
   */
  accept?: string;
}>;

export type FileDropZoneProps = FileDropZonePropsWithoutHTML &
  Omit<HTMLInputAttributes, "multiple" | "files">;
