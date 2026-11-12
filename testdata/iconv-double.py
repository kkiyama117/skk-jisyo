#!/usr/bin/env python3
"""Synthetic iconv for the gen-euc-jp.py CLI tests."""

import os
import sys

FORWARD = {
    b"\xA4\xA2": "\u3042",
    b"\xFE\xFE": "\U00020089",
    b"\x8F\xA2\xB7": "\uFF5E",
    b"\xA4\xA3": "xy",
    b"\x8E\xB1": "\uFF71",
}
REVERSE = {text: code for code, text in FORWARD.items()}
REVERSE["\uFF71"] = b"\x8E\xB2"


def direction(argv):
    source = target = None
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "-c":
            index += 1
        elif argument == "-f" and index + 1 < len(argv):
            source = argv[index + 1]
            index += 2
        elif argument == "-t" and index + 1 < len(argv):
            target = argv[index + 1]
            index += 2
        else:
            return None
    return source, target


def main():
    pair = direction(sys.argv[1:])
    forward = pair == ("EUC-JP", "UTF-8")
    if not forward and pair != ("UTF-8", "EUC-JP"):
        sys.stderr.write("iconv double: unsupported options\n")
        return 2

    records = sys.stdin.buffer.read().split(b"\n")
    if records[-1] == b"":
        records.pop()

    output = bytearray()
    skipped = False
    for record in records:
        if forward:
            text = FORWARD.get(record)
            piece = text.encode("utf-8") if text is not None else b""
            skipped = skipped or text is None
        else:
            try:
                text = record.decode("utf-8")
            except UnicodeDecodeError:
                text = None
            piece = REVERSE.get(text, b"") if text is not None else b""
            skipped = skipped or piece == b""
        output += piece + b"\n"

    if forward:
        fault = os.environ.get("JISYO_TEST_ICONV_FAULT")
        if fault == "stderr":
            sys.stderr.write("synthetic failure\n")
        elif fault == "missing-lf" and output.endswith(b"\n"):
            output = output[:-1]
        elif fault == "extra-line":
            output += b"\n"
        elif fault == "exit2":
            sys.stdout.buffer.write(output)
            return 2

    sys.stdout.buffer.write(output)
    return 1 if skipped else 0


if __name__ == "__main__":
    sys.exit(main())
