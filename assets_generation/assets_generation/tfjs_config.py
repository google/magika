
from dataclasses import dataclass



@dataclass
class Config:
    input_size_beg: int
    input_size_mid: int
    input_size_end: int
    min_file_size_for_dl: int
    padding_token: int
    labels: list


@dataclass
class Label:
    name: int
    threshold: int
    is_text: bool
