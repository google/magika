from collections import defaultdict
from pathlib import Path
import json
from typing import List, Union, Dict, Any, Optional, Set


CONTENT_TYPES_CONFIG_PATH = (
    Path(__file__).parent / "config" / "content_types_config.json"
)


class ContentType:
    # the tool returned unknown, '',  None, or similar
    UNKNOWN = "unknown"
    UNKNOWN_MIME_TYPE = "application/unknown"
    UNKNOWN_CONTENT_TYPE_GROUP = "unknown"
    UNKNOWN_MAGIC = "Unknown"

    # the tool returned an output that we currently do not map to our content types
    UNSUPPORTED = "unsupported"

    # the tool exited with returncode != 0
    ERROR = "error"

    # there is no result for this tool
    MISSING = "missing"

    # the file is empty (or just \x00, spaces, etc.)
    EMPTY = "empty"

    # the output of the tool is gibberish / meaningless type
    CORRUPTED = "corrupted"

    # the tool did not return in time
    TIMEOUT = "timeout"

    # the mapping functions returned a type we don't recognized, and we flag it
    # as NOT VALID
    NOT_VALID = "not_valid"

    GENERIC_TEXT = "txt"

    def __init__(
        self,
        name: str,
        extensions: List[str],
        mime_type: Optional[str],
        group: Optional[str],
        magic: Optional[str],
        vt_type: Optional[str],
        datasets: List[str],
        parent: Optional[str],
        tags: List[str],
        model_target_label: Optional[str],
        target_label: Optional[str],
        correct_labels: List[str],
        threshold: Optional[float],
        add_automatic_tags: bool = True,
    ):
        self.name = name
        self.extensions = extensions
        self.mime_type = mime_type
        self.group = group
        self.magic = magic
        self.vt_type = vt_type
        self.datasets = datasets
        self.parent = parent
        self.tags = tags
        self.model_target_label = model_target_label
        self.target_label = target_label
        self.correct_labels = correct_labels
        self.threshold = threshold if threshold is not None else 1.0

        # add automatic tags based on dataset
        if add_automatic_tags:
            if self.datasets is not None:
                for dataset in self.datasets:
                    self.tags.append(f"dataset:{dataset}")
            if self.model_target_label is not None:
                self.tags.append(f"model_target_label:{self.model_target_label}")
            if self.target_label is not None:
                self.tags.append(f"target_label:{self.target_label}")
            if self.correct_labels is not None:
                for cl in self.correct_labels:
                    self.tags.append(f"correct_label:{cl}")

    @property
    def is_text(self) -> bool:
        return "text" in self.tags

    @property
    def in_scope(self) -> bool:
        if len(self.datasets) == 0:
            return False
        if self.model_target_label is None:
            return False
        if self.target_label is None:
            return False
        if len(self.correct_labels) == 0:
            return False
        return True

    def to_dict(self) -> Dict[str, Any]:
        info: Dict[str, Any] = {
            "name": self.name,
            "extensions": self.extensions,
            "mime_type": self.mime_type,
            "group": self.group,
            "magic": self.magic,
            "vt_type": self.vt_type,
            "datasets": self.datasets,
            "parent": self.parent,
            "tags": self.tags,
            "model_target_label": self.model_target_label,
            "target_label": self.target_label,
            "correct_labels": self.correct_labels,
            "threshold": self.threshold,
            "in_scope": self.in_scope,
        }
        return info

    @staticmethod
    def from_dict(info_d, add_automatic_tags=True):
        info_d_copy = dict(info_d)
        info_d_copy.pop("in_scope")
        ct = ContentType(add_automatic_tags=add_automatic_tags, **info_d_copy)
        return ct

    def __str__(self):
        return f"<{self.name}>"

    def __repr__(self):
        return str(self)


class ContentTypesManager:
    SPECIAL_CONTENT_TYPES: List[str] = [
        ContentType.UNKNOWN,
        ContentType.UNSUPPORTED,
        ContentType.ERROR,
        ContentType.MISSING,
        ContentType.EMPTY,
        ContentType.CORRUPTED,
        ContentType.NOT_VALID,
        ContentType.GENERIC_TEXT,
    ]

    SUPPORTED_TARGET_LABELS_SPEC = [
        "content-type",
        "model-target-label",
        "target-label",
    ]

    def __init__(
        self,
        content_type_config_path: Path = CONTENT_TYPES_CONFIG_PATH,
        add_automatic_tags: bool = True,
    ):
        self.cts: Dict[str, ContentType] = {}
        # tag to content type map
        self.tag2cts: Dict[str, List[ContentType]] = defaultdict(list)
        # extension to content type map
        self.ext2ct: Dict[str, ContentType] = {}
        self.load_content_types_info(
            content_type_config_path=content_type_config_path,
            add_automatic_tags=add_automatic_tags,
        )

    def load_content_types_info(
        self, content_type_config_path: Path, add_automatic_tags: bool = True
    ) -> None:
        with open(content_type_config_path) as f:
            info = json.load(f)
        self.cts = {}
        for k, v in info.items():
            assert k == v["name"]
            ct = ContentType.from_dict(v, add_automatic_tags=add_automatic_tags)
            self.cts[k] = ct
            for tag in ct.tags:
                self.tag2cts[tag].append(ct)
            for ext in ct.extensions:
                self.ext2ct[ext] = ct

    def get(self, content_type_name: str) -> Optional[ContentType]:
        return self.cts.get(content_type_name)

    def get_or_raise(self, content_type_name: str) -> ContentType:
        ct = self.get(content_type_name)
        if ct is None:
            raise Exception(f'Could get a ContentType for "{content_type_name}')
        return ct

    def get_mime_type(
        self, content_type_name: str, default: str = ContentType.UNKNOWN_MIME_TYPE
    ) -> str:
        ct = self.get(content_type_name)
        if ct is None:
            return default
        if ct.mime_type is None:
            return default
        return ct.mime_type

    def get_group(
        self,
        content_type_name: str,
        default: str = ContentType.UNKNOWN_CONTENT_TYPE_GROUP,
    ) -> str:
        ct = self.get(content_type_name)
        if ct is None:
            return default
        if ct.group is None:
            return default
        return ct.group

    def get_magic(
        self, content_type_name: str, default: str = ContentType.UNKNOWN_MAGIC
    ) -> str:
        ct = self.get(content_type_name)
        if ct is None:
            return default
        if ct.magic is None:
            return default
        return ct.magic

    def get_ct_by_ext(self, ext: str) -> Optional[ContentType]:
        return self.ext2ct.get(ext)

    def get_ct_by_ext_or_raise(self, ext: str) -> ContentType:
        ct = self.get_ct_by_ext(ext)
        if ct is None:
            raise Exception(f'Could not find ContentType for extension "{ext}"')
        return ct

    def is_valid_ct(self, ct_name: str) -> bool:
        if ct_name in ContentTypesManager.SPECIAL_CONTENT_TYPES:
            return True
        return self.cts.get(ct_name) is not None

    def get_valid_tags(self, only_explicit: bool = True) -> List[str]:
        if only_explicit:
            all_tags = sorted(
                filter(
                    lambda x: (
                        not x.split(":")[0].endswith("_label")
                        and not x.startswith("dataset")
                    ),
                    self.tag2cts.keys(),
                )
            )
        else:
            all_tags = sorted(self.tag2cts.keys())
        return all_tags

    def is_valid_tag(self, tag: str) -> bool:
        return tag in self.tag2cts.keys()

    def select(
        self, query: Optional[str] = None, must_be_in_scope: bool = True
    ) -> List[ContentType]:
        ct_names = self.select_names(query=query, must_be_in_scope=must_be_in_scope)
        # we know these are valid content types
        return list(map(self.get, ct_names))  # type: ignore

    def select_names(
        self, query: Optional[str] = None, must_be_in_scope: bool = True
    ) -> List[str]:
        ct_names_set: Set[str] = set()
        if query is None:
            # select them all, honoring must_be_in_scope
            for ct in self.cts.values():
                if must_be_in_scope and not ct.in_scope:
                    continue
                ct_names_set.add(ct.name)
        else:
            # consider each element of the query in sequence and add/remove
            # content types as appropriate (also honorig must_be_in_scope)
            entries = query.split(",")
            for entry in entries:
                if entry in ["*", "all"]:
                    # we know we get list of strings because we set only_names=True
                    ct_names_set.update(self.select_names(must_be_in_scope=must_be_in_scope))  # type: ignore
                elif entry.startswith("tag:"):
                    entry = entry[4:]
                    assert self.is_valid_tag(entry)
                    for ct in self.tag2cts[entry]:
                        if must_be_in_scope and not ct.in_scope:
                            continue
                        ct_names_set.add(ct.name)
                elif entry.startswith("-tag:"):
                    entry = entry[5:]
                    assert self.is_valid_tag(entry)
                    for ct in self.tag2cts[entry]:
                        # no need to check for must_be_in_scope when removing
                        if ct.name in ct_names_set:
                            ct_names_set.remove(ct.name)
                elif entry[0] == "-":
                    entry = entry[1:]
                    assert self.is_valid_ct(entry)
                    # no need to check for must_be_in_scope when removing
                    if entry in ct_names_set:
                        ct_names_set.remove(entry)
                else:
                    assert self.is_valid_ct(entry)
                    # this ct was manually specified, if it does not honor
                    # must_be_in_scope, that's a problem.
                    if must_be_in_scope:
                        candidate_ct: ContentType | None = self.get(entry)
                        assert candidate_ct is not None
                        assert candidate_ct.in_scope
                    ct_names_set.add(entry)

        ct_names = sorted(ct_names_set)
        return ct_names

    def get_content_types_space(self) -> List[str]:
        """Returns the full list of possible content types, including out of
        scope and special types. Returns only the names."""

        # We know that we get content type names (str), and not a lis of
        # ContentType
        return sorted(
            set(self.select_names(must_be_in_scope=False))
            | set(self.SPECIAL_CONTENT_TYPES)  # type: ignore
        )
