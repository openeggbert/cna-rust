#!/usr/bin/env python3
"""Answers which bound CNA routes a safe Rust caller can actually reach.

``BOUND`` means a route is declared in ``cna-sys`` and resolved when the library
loads.  That is not the same as a consumer being able to call it.  This module
measures the difference by walking the crate's call graph, rather than by
guessing that a Rust identifier is spelled like the C route.

Three measured facts make an exact answer possible.

* **Every route is acquired at exactly one place.**  ``field: symbol!(cna_x,
  ...)`` inside a table's constructor ties the C route ``cna_x`` to the Rust
  field that holds its pointer.  The field is *not* named after the route --
  ``cna_audio_category_pause`` is held in ``AudioApi::category_pause`` -- which
  is why matching route names against Rust identifiers, as the first version of
  this check did, could not work.
* **The 3,244 field names are unique across the whole crate.**  So an
  occurrence of ``.category_pause`` names one route and no other, wherever it
  appears, and no scope analysis is needed to say which.
* **The safe layer is everything outside ``native/``.**  A route is reachable
  when some code outside ``native/`` can, through any number of hops, arrive
  somewhere that reads the field.

The walk starts from every file outside ``native/``, follows the names those
files mention into ``native/`` functions, follows what *those* functions
mention, and keeps going until nothing new is reached.  Nothing about the
number of hops is baked in: a route behind four wrappers is found exactly the
way a route behind one is.

The acquisition itself is not a use.  ``field: symbol!(...)`` and the struct
declaration ``field: sys::cna_x_fn`` are blanked before any name is collected,
so the loader -- which necessarily names every field in the crate -- cannot
make every route look reachable.
"""

from __future__ import annotations

from collections import defaultdict
from pathlib import Path
import re

# Any identifier, whether or not it is reached through a dot: a wrapper is
# called as `native.audio_category_action(..)` and a field is read as
# `self.audio.category_pause`, and both have to be followed.
IDENTIFIER = re.compile(r"\b([a-z_][a-z0-9_]{2,})\b")
ACQUISITION = re.compile(r"\b([a-z_0-9]+)(\s*:\s*symbol!\(\s*)(cna_[A-Za-z0-9_]+)\s*,")
LET_ACQUISITION = re.compile(r"\blet\s+([a-z_0-9]+)(\s*=\s*symbol!\(\s*)(cna_[A-Za-z0-9_]+)\s*,")
DECLARATION = re.compile(r"\b([a-z_0-9]+)(\s*:\s*sys::cna_[A-Za-z0-9_]+_fn\b)")
FN = re.compile(r"\bfn\s+([a-z_][A-Za-z0-9_]*)")


def strip_noise(text: str) -> str:
    """Blanks comments and string literals, keeping every byte offset.

    Brace matching has to be exact, so a ``{`` inside a doc comment or a string
    must not count.  Replacing them with spaces rather than deleting them keeps
    the offsets of everything else unchanged.
    """
    out = list(text)
    index, length = 0, len(text)
    while index < length:
        two = text[index : index + 2]
        if two == "//":
            end = text.find("\n", index)
            end = length if end < 0 else end
            for position in range(index, end):
                out[position] = " "
            index = end
            continue
        if two == "/*":
            depth, position = 1, index + 2
            while position < length and depth:
                if text[position : position + 2] == "/*":
                    depth, position = depth + 1, position + 2
                elif text[position : position + 2] == "*/":
                    depth, position = depth - 1, position + 2
                else:
                    position += 1
            for cursor in range(index, min(position, length)):
                if text[cursor] != "\n":
                    out[cursor] = " "
            index = position
            continue
        if text[index] == '"' or (
            text[index] == "r"
            and index + 1 < length
            and text[index + 1] in '"#'
            and (index == 0 or not re.match(r"[A-Za-z0-9_]", text[index - 1]))
        ):
            start = index
            if text[index] == "r":
                hashes = 0
                cursor = index + 1
                while cursor < length and text[cursor] == "#":
                    hashes, cursor = hashes + 1, cursor + 1
                if cursor >= length or text[cursor] != '"':
                    index += 1
                    continue
                terminator = '"' + "#" * hashes
                end = text.find(terminator, cursor + 1)
                end = length if end < 0 else end + len(terminator)
            else:
                cursor = index + 1
                while cursor < length:
                    if text[cursor] == "\\":
                        cursor += 2
                        continue
                    if text[cursor] == '"':
                        cursor += 1
                        break
                    cursor += 1
                end = cursor
            for position in range(start, min(end, length)):
                if text[position] != "\n":
                    out[position] = " "
            index = end
            continue
        if text[index] == "'":
            match = re.match(r"'(\\.|[^\\'])'", text[index:])
            if match:
                for cursor in range(index, index + match.end()):
                    out[cursor] = " "
                index += match.end()
                continue
        index += 1
    return "".join(out)


def blank_acquisitions(text: str) -> tuple[str, dict[str, str]]:
    """Removes every field *definition* from the text, keeping byte offsets.

    ``load`` names all 3,244 fields, so counting a definition as a use would
    make every route reachable from the one function that must mention them
    all.  Returns the blanked text and the field-to-route map it collected.
    """
    fields: dict[str, str] = {}
    out = list(text)

    def blank(match: re.Match, group: int) -> None:
        for position in range(match.start(group), match.end(group)):
            out[position] = " "

    for pattern in (ACQUISITION, LET_ACQUISITION):
        for match in pattern.finditer(text):
            fields[match.group(1)] = match.group(3)
            blank(match, 1)
    for match in DECLARATION.finditer(text):
        blank(match, 1)
    return "".join(out), fields


def body_of(text: str, start: int) -> tuple[int, int] | None:
    """The braces of the function whose ``fn`` keyword ends at ``start``."""
    index, length, depth = start, len(text), 0
    while index < length:
        char = text[index]
        if char in "([":
            depth += 1
        elif char in ")]":
            depth -= 1
        elif char == ";" and depth == 0:
            return None  # a trait method with no body
        elif char == "{" and depth == 0:
            close, nesting = index, 0
            while close < length:
                if text[close] == "{":
                    nesting += 1
                elif text[close] == "}":
                    nesting -= 1
                    if nesting == 0:
                        return index + 1, close
                close += 1
            return index + 1, length
        index += 1
    return None


def analyse(root: Path) -> dict:
    """Walks ``crates/cna/src`` and reports which acquired routes are reachable."""
    units: list[set[str]] = []
    names: list[str] = []
    by_name: dict[str, list[int]] = defaultdict(list)
    field_to_route: dict[str, str] = {}
    seed: set[str] = set()

    for path in sorted(root.rglob("*.rs")):
        relative = path.relative_to(root)
        clean = strip_noise(path.read_text(encoding="utf-8"))
        clean, fields = blank_acquisitions(clean)
        field_to_route.update(fields)
        if relative.parts[0] != "native":
            seed |= set(IDENTIFIER.findall(clean))
            continue
        spans = []
        for match in FN.finditer(clean):
            span = body_of(clean, match.end())
            if span is None:
                continue
            index = len(units)
            units.append(set(IDENTIFIER.findall(clean[span[0] : span[1]])))
            names.append(match.group(1))
            by_name[match.group(1)].append(index)
            # From the `fn` keyword, not from the body: a function's own name
            # lives in its signature, and treating that as module-level text
            # would make every function in `native/` its own entry point.
            spans.append((match.start(), span[1]))
        # What is left is genuinely module level -- a `const`, a `static`, a
        # `use` -- which is not behind a call and is therefore always live.
        outside = list(clean)
        for start, end in spans:
            for position in range(start, end):
                outside[position] = " "
        seed |= set(IDENTIFIER.findall("".join(outside)))

    visited: set[int] = set()
    reached = set(seed)
    queue = list(seed)
    while queue:
        name = queue.pop()
        for index in by_name.get(name, ()):
            if index in visited:
                continue
            visited.add(index)
            for called in units[index]:
                if called not in reached:
                    reached.add(called)
                    queue.append(called)
            queue.extend(units[index] & set(by_name))
    reachable, unreachable = set(), set()
    for field, route in field_to_route.items():
        (reachable if field in reached else unreachable).add(route)
    return {
        "fieldToRoute": field_to_route,
        "reachable": reachable,
        "unreachable": unreachable,
        "unreachableFields": {
            field: route for field, route in field_to_route.items() if route in unreachable
        },
        "visitedNativeFunctions": len(visited),
        "nativeFunctions": len(units),
    }


if __name__ == "__main__":
    import json
    import sys

    result = analyse(Path(__file__).resolve().parents[2] / "crates/cna/src")
    print(
        json.dumps(
            {
                "acquired": len(result["fieldToRoute"]),
                "reachable": len(result["reachable"]),
                "unreachable": len(result["unreachable"]),
                "visitedNativeFunctions": result["visitedNativeFunctions"],
                "nativeFunctions": result["nativeFunctions"],
                "sample": sorted(result["unreachable"])[:40],
            },
            indent=2,
        )
    )
    sys.exit(0)
