from dataclasses import dataclass

from magika.types.content_type_label import ContentTypeLabel


@dataclass(kw_only=True)
class ContentTypeInfo:
    name: ContentTypeLabel
    mime_type: str
    group: str
    description: str
    is_text: bool
