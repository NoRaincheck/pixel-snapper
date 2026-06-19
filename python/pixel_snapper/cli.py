"""CLI entry point for pixel-snapper."""

import sys
from pathlib import Path


def main():
    """Run pixel-snapper CLI."""
    from . import PixelSnapperConfig

    args = sys.argv[1:]
    if len(args) < 2:
        print("Usage: pixel-snapper <input> <output> [k_colors] [--pixel-size <size>]")
        sys.exit(1)

    input_path = Path(args[0])
    output_path = Path(args[1])

    k_colors = 16
    pixel_size_override = None

    i = 2
    while i < len(args):
        arg = args[i]
        if arg == "--pixel-size":
            if i + 1 < len(args):
                try:
                    pixel_size_override = float(args[i + 1])
                except ValueError:
                    print(f"Warning: invalid --pixel-size '{args[i + 1]}', ignoring")
                i += 2
                continue
            else:
                print("Warning: --pixel-size requires a value")
                break
        elif arg.startswith("--"):
            print(f"Warning: unknown argument '{arg}', ignoring")
        else:
            try:
                k = int(arg)
                if k > 0:
                    k_colors = k
                else:
                    print(
                        f"Warning: invalid k_colors '{arg}', falling back to default ({k_colors})"
                    )
            except ValueError:
                print(
                    f"Warning: invalid k_colors '{arg}', falling back to default ({k_colors})"
                )
        i += 1

    config = PixelSnapperConfig(
        k_colors=k_colors, pixel_size_override=pixel_size_override
    )

    if input_path.is_dir():
        _process_batch_dir(input_path, output_path, config)
    else:
        _process_single(input_path, output_path, config)


def _process_single(input_path: Path, output_path: Path, config):
    """Process a single image file."""
    from . import process_file_cli

    process_file_cli(str(input_path), str(output_path), config)
    print(f"Processing: {input_path}")
    print(f"Saved to: {output_path}")


def _process_batch_dir(input_dir: Path, output_dir: Path, config):
    """Process a batch of images from a directory."""
    from . import process_batch

    def on_event(event):
        etype = event.get("type")
        if etype == "batch_started":
            total = event["total"]
            print(
                f"Batch processing {total} image{'s' if total != 1 else ''} from: {event['input_dir']}"
            )
        elif etype == "started":
            print(f"Processing {event['index'] + 1}/{event['total']}: {event['input']}")
        elif etype == "finished":
            print(
                f"Done {event['index'] + 1}/{event['total']}: {event['input']} -> {event['output']}"
            )
        elif etype == "failed":
            print(
                f"Failed {event['index'] + 1}/{event['total']}: {event['input']} -> {event['output']} ({event['error']})",
                file=sys.stderr,
            )
        elif etype == "batch_finished":
            total = event["total"]
            print(
                f"Processed {total} image{'s' if total != 1 else ''} in: {event['input_dir']}"
            )

    process_batch(str(input_dir), str(output_dir), config, on_event)
