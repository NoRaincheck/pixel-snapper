"""Pixel Snapper - Snap pixels to a perfect grid."""

from .pixel_snapper import (
    PixelSnapperConfig,
    process_image,
    process_file_cli,
    process_batch,
)

__all__ = [
    "PixelSnapperConfig",
    "process_image",
    "process_file_cli",
    "process_batch",
]

__version__ = "0.1.0"
