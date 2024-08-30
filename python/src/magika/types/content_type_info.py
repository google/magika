from dataclasses import dataclass
from typing import List

from magika.types.content_type_label import ContentTypeLabel


@dataclass(frozen=True)
class ContentTypeInfo:
    label: ContentTypeLabel
    mime_type: str
    group: str
    description: str
    extensions: List[str]
    is_text: bool
