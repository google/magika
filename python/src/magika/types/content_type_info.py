# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Module defining the ContentTypeInfo dataclass."""

import warnings
from dataclasses import dataclass
from typing import List

from magika.logger import get_logger
from magika.types.content_type_label import ContentTypeLabel


@dataclass(frozen=True)
class ContentTypeInfo:
    """Dataclass holding information about a content type.

    Attributes:
        label: The ContentTypeLabel enum value.
        mime_type: The mime type associated to the content type.
        group: A high-level category for the content type (e.g., "document",
            "image").
        description: A human-readable description.
        extensions: A list of common file extensions.
        is_text: A boolean indicating if the content type is text-based.
    """

    label: ContentTypeLabel
    mime_type: str
    group: str
    description: str
    extensions: List[str]
    is_text: bool

    @property
    def ct_label(self) -> str:
        """DEPRECATED: Returns the string value of the content type label.

        Warns:
            DeprecationWarning: This property is deprecated. Use `.label`
                instead.
        """
        warnings.warn(
            "`.ct_label` is deprecated and will be removed in a future version. Use `.label` instead. Consult the documentation for more information.",
            category=DeprecationWarning,
            stacklevel=2,
        )
        return str(self.label)

    @property
    def score(self) -> float:
        """UNSUPPORTED: This property is no longer supported and raises an error.

        Raises:
            AttributeError: This property is unsupported. The score is now on
                the MagikaResult object.
        """
        error_msg = "Unsupported field error: `.score.` is not stored anymore in the `dl` or `output` objects; it is now stored in `MagikaResult`. Consult the documentation for more information."
        log = get_logger()
        log.error(error_msg)
        raise AttributeError(error_msg)

    @property
    def magic(self) -> str:
        """DEPRECATED: Returns the description of the content type.

        Warns:
            DeprecationWarning: This property is deprecated. Use
                `.description` instead.
        """
        warnings.warn(
            "`.magic` is deprecated and will be removed in a future version. Use `.description` instead. Consult the documentation for more information.",
            category=DeprecationWarning,
            stacklevel=2,
        )
        return self.description
